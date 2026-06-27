use std::collections::HashMap;

use crate::pty::{PtyEvent, PtySession};
use macterm_core::*;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tokio::sync::mpsc;

/// Pending confirmation action (for close pane / quit confirmation dialogs)
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    None,
    ClosePane,
    Quit,
}

/// Tracks an active drag-to-resize operation on a split border
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeState {
    Idle,
    Dragging {
        /// A pane ID within the split subtree being resized (identifies which split)
        split_pane: PaneId,
        /// Direction of the split
        direction: SplitDirection,
        /// The full area of this split node (for ratio calculation)
        area: Rect,
        /// The ratio when the drag started
        start_ratio: f32,
        /// The cursor position (x for Horizontal, y for Vertical) when drag started
        start_pos: u16,
    },
}

/// Application state
pub struct App {
    /// The workspace (holds tabs)
    pub workspace: Workspace,
    /// PTY sessions keyed by pane ID
    pub sessions: HashMap<PaneId, PtySession>,
    /// Event receiver for PTY events
    pub pty_rx: mpsc::UnboundedReceiver<PtyEvent>,
    /// Whether the app is running
    pub running: bool,
    /// Terminal area for layout calculations
    pub area: Rect,
    /// Whether to show the file tree sidebar
    pub show_file_tree: bool,
    /// Whether to show the command palette
    pub show_command_palette: bool,
    /// Input buffer for command palette
    pub command_input: String,
    /// Status message text
    pub status_message: Option<String>,
    /// Frame when the status message was last set (for auto-fade)
    pub status_message_frame: u64,
    /// Color for the status message (default: amber, green=success, red=error)
    pub status_message_color: Color,
    /// Whether to toggle status bar visibility
    pub show_status_bar: bool,
    /// Whether to show the help overlay
    pub show_help: bool,
    /// Animation frame counter
    pub frame_count: u64,
    /// Current resize drag state (for split border drag-to-resize)
    pub resize_state: ResizeState,
    /// Pending confirmation action (E4/E5: close pane / quit confirmation)
    pub confirm_action: ConfirmAction,
    /// Scroll offset for the tab bar (A4: tab scrolling)
    pub tab_scroll_offset: usize,
    /// Cached file tree entries (D1): (filename, is_directory)
    pub file_tree_entries: Vec<(String, bool)>,
    /// Scrolling offset for the file tree
    pub file_tree_scroll: usize,
    /// Search overlay (E1): whether search is active
    pub show_search: bool,
    /// Search query text
    pub search_query: String,
    /// Match positions in the active pane: vec of (row, col_start, col_end)
    pub search_matches: Vec<(u16, u16, u16)>,
    /// Index into search_matches for the currently highlighted match
    pub search_match_index: usize,
}

impl App {
    pub fn new() -> (Self, mpsc::UnboundedSender<PtyEvent>) {
        let (pty_tx, pty_rx) = mpsc::unbounded_channel();

        let workspace = Workspace::new("default");

        let mut sessions = HashMap::new();

        // Spawn initial terminal pane
        let first_pane_id = workspace.active_tab().active_pane();
        match PtySession::spawn(first_pane_id, 80, 24, pty_tx.clone()) {
            Ok(session) => {
                sessions.insert(first_pane_id, session);
            }
            Err(e) => {
                log::error!("Failed to spawn initial PTY: {}", e);
            }
        }

        (
            Self {
                workspace,
                sessions,
                pty_rx,
                running: true,
                area: Rect::default(),
                show_file_tree: false,
                show_command_palette: false,
                command_input: String::new(),
                status_message: None,
                status_message_frame: 0,
                status_message_color: Color::Reset,
                show_status_bar: true,
                show_help: false,
                frame_count: 0,
                resize_state: ResizeState::Idle,
                confirm_action: ConfirmAction::None,
                tab_scroll_offset: 0,
                file_tree_entries: Vec::new(),
                file_tree_scroll: 0,
                show_search: false,
                search_query: String::new(),
                search_matches: Vec::new(),
                search_match_index: 0,
            },
            pty_tx,
        )
    }

    /// Handle a tick event (called every frame)
    pub fn tick(&mut self) {
        self.frame_count += 1;
        // Auto-dismiss status message after ~120 frames (2s at 60fps)
        if self.status_message.is_some()
            && self.frame_count.saturating_sub(self.status_message_frame) > 120
        {
            self.status_message = None;
        }
    }

    /// Set a status message with auto-fade timing (default amber color)
    pub fn set_status_message(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_message_frame = self.frame_count;
        self.status_message_color = Color::Reset;
    }

    /// Set a status message with a specific color (for success/error notifications)
    pub fn set_status_message_colored(&mut self, msg: String, color: Color) {
        self.status_message = Some(msg);
        self.status_message_frame = self.frame_count;
        self.status_message_color = color;
    }

    /// Handle PTY events from background reader threads
    pub fn handle_pty_events(&mut self) {
        while let Ok(event) = self.pty_rx.try_recv() {
            match event {
                PtyEvent::Output(pane_id, _) => {
                    // Screen content is already in the vt100 parser
                    // Just log for now
                    log::trace!("Output received for pane {}", pane_id);
                }
                PtyEvent::Resized(pane_id, cols, rows) => {
                    if let Some(session) = self.sessions.get_mut(&pane_id) {
                        let _ = session.resize(cols, rows);
                    }
                }
                PtyEvent::Exited(pane_id, code) => {
                    log::info!("Pane {} exited with code {}", pane_id, code);
                    self.set_status_message(format!("Pane {} exited ({})", pane_id, code));
                }
            }
        }
    }

    /// Resize all PTY sessions in the active tab to their actual split-tree dimensions.
    /// Must match the content area calculation in `render()` (accounting for file tree sidebar).
    pub fn resize_active_panes(&mut self) {
        let status_h = if self.show_status_bar { 1 } else { 0 };
        let head_h: u16 = 2;
        let file_tree_w: u16 = if self.show_file_tree { 20 } else { 0 };
        let content_area = Rect {
            x: file_tree_w,
            y: head_h,
            width: self.area.width.saturating_sub(file_tree_w),
            height: self.area.height.saturating_sub(head_h + status_h),
        };
        let tab = self.workspace.active_tab();
        let pane_rects = crate::ui::pane_rects_from_tree(&tab.root, content_area);
        for (pane_id, pane_rect) in &pane_rects {
            if let Some(session) = self.sessions.get_mut(pane_id) {
                // Each pane renders with a border (1 cell on each side) and a 1-row title bar.
                // The PTY content area is therefore: width-2 × height-3.
                let pty_cols = pane_rect.width.saturating_sub(2).max(20);
                let pty_rows = pane_rect.height.saturating_sub(3).max(5);
                let _ = session.resize(pty_cols, pty_rows);
            }
        }
    }

    /// Write input to the active pane
    pub fn write_to_active_pane(&mut self, data: &[u8]) {
        let active_pane = self.workspace.active_tab().active_pane();
        if let Some(session) = self.sessions.get_mut(&active_pane) {
            let _ = session.write(data);
        }
    }

    /// Split the active pane
    pub fn split_active_pane(&mut self, direction: SplitDirection) {
        let tab = self.workspace.active_tab();
        let active_pane = tab.active_pane();

        // Create new pane
        let new_pane = Pane::terminal(format!("term-{}", tab.pane_count() + 1));
        let new_pane_id = new_pane.id;

        // Replace the active pane leaf with a split
        let new_split = SplitNode::Split {
            direction,
            ratio: 0.5,
            left: Box::new(SplitNode::Leaf(active_pane)),
            right: Box::new(SplitNode::Leaf(new_pane_id)),
        };

        let tab = self.workspace.active_tab_mut();
        if let Some(new_root) = std::mem::replace(&mut tab.root, SplitNode::Leaf(active_pane))
            .replace_pane_with_split(&active_pane, new_split)
        {
            tab.root = new_root;
        }

        // Spawn PTY for the new pane (will be resized to actual dimensions below)
        let (tx, _rx) = mpsc::unbounded_channel();
        match PtySession::spawn(new_pane_id, 80, 24, tx) {
            Ok(session) => {
                self.sessions.insert(new_pane_id, session);
            }
            Err(e) => {
                log::error!("Failed to spawn PTY for new pane: {}", e);
            }
        }
        // Immediately resize all panes to their actual split-tree dimensions
        self.resize_active_panes();
    }

    /// Close the active pane
    pub fn close_active_pane(&mut self) {
        let tab = self.workspace.active_tab();
        let active_pane = tab.active_pane();

        // Remove the pane from the layout tree
        let tab = self.workspace.active_tab_mut();
        let old_root = std::mem::replace(&mut tab.root, SplitNode::Leaf(active_pane));
        if let Some(new_root) = old_root.clone().remove_pane(&active_pane) {
            tab.root = new_root;
            // Remove PTY session
            self.sessions.remove(&active_pane);
            // Update active pane to the first remaining pane
            let ids = tab.pane_ids();
            if let Some(first_id) = ids.first() {
                tab.active_pane = *first_id;
            }
        } else {
            // Can't remove the last pane
            tab.root = old_root;
        }
    }

    /// Switch focus to the next pane in order
    pub fn focus_next_pane(&mut self) {
        let tab = self.workspace.active_tab();
        let ids = tab.pane_ids();
        if ids.len() <= 1 {
            return;
        }
        let current = tab.active_pane();
        let pos = ids.iter().position(|id| *id == current).unwrap_or(0);
        let next = (pos + 1) % ids.len();
        let tab = self.workspace.active_tab_mut();
        tab.active_pane = ids[next];
    }

    /// Switch focus to the previous pane in order
    pub fn focus_prev_pane(&mut self) {
        let tab = self.workspace.active_tab();
        let ids = tab.pane_ids();
        if ids.len() <= 1 {
            return;
        }
        let current = tab.active_pane();
        let pos = ids.iter().position(|id| *id == current).unwrap_or(0);
        let prev = if pos == 0 { ids.len() - 1 } else { pos - 1 };
        let tab = self.workspace.active_tab_mut();
        tab.active_pane = ids[prev];
    }

    /// Zoom the active pane (toggle)
    pub fn toggle_zoom(&mut self) {
        let _tab = self.workspace.active_tab_mut();
        // Toggle zoom state - we handle this in the renderer
    }

    /// Begin a drag-to-resize operation on a split border
    pub fn start_resize_drag(
        &mut self,
        split_pane: PaneId,
        direction: SplitDirection,
        area: Rect,
        start_ratio: f32,
        start_pos: u16,
    ) {
        self.resize_state = ResizeState::Dragging {
            split_pane,
            direction,
            area,
            start_ratio,
            start_pos,
        };
    }

    /// Update ratio during a resize drag, returns true if ratio changed
    pub fn update_resize_drag(&mut self, mouse_x: u16, mouse_y: u16) -> bool {
        let (split_pane, direction, area, start_ratio, start_pos) = match self.resize_state {
            ResizeState::Dragging {
                split_pane,
                direction,
                area,
                start_ratio,
                start_pos,
            } => (split_pane, direction, area, start_ratio, start_pos),
            ResizeState::Idle => return false,
        };
        let mouse_pos = match direction {
            SplitDirection::Horizontal => mouse_x,
            SplitDirection::Vertical => mouse_y,
        };
        let delta = (mouse_pos as i32) - (start_pos as i32);
        let size = match direction {
            SplitDirection::Horizontal => area.width.max(1) as f32,
            SplitDirection::Vertical => area.height.max(1) as f32,
        };
        let new_ratio = (start_ratio + delta as f32 / size).clamp(0.1, 0.9);
        {
            let tab = self.workspace.active_tab_mut();
            tab.root.update_ratio_by_pane(&split_pane, new_ratio);
        }
        self.resize_active_panes();
        true
    }

    /// End a resize drag
    pub fn end_resize_drag(&mut self) {
        self.resize_state = ResizeState::Idle;
    }

    /// Handle key input for command palette
    pub fn command_palette_input(&mut self, c: char) {
        self.command_input.push(c);
    }

    pub fn command_palette_backspace(&mut self) {
        self.command_input.pop();
    }

    /// Refresh the file tree by reading the current working directory (D1)
    pub fn refresh_file_tree(&mut self) {
        self.file_tree_entries.clear();
        if let Ok(entries) = std::fs::read_dir(".") {
            let mut list: Vec<(String, bool)> = entries
                .filter_map(|e| e.ok())
                .map(|e| (e.file_name().to_string_lossy().to_string(), e.file_type().map(|t| t.is_dir()).unwrap_or(false)))
                .collect();
            list.sort_by(|a, b| {
                // Directories first, then alphabetical
                if a.1 != b.1 { b.1.cmp(&a.1) }
                else { a.0.to_lowercase().cmp(&b.0.to_lowercase()) }
            });
            self.file_tree_entries = list;
        }
        self.file_tree_scroll = 0;
    }

    /// Auto-scroll the tab bar so the active tab is visible (A4)
    pub fn ensure_tab_visible(&mut self) {
        let tab_count = self.workspace.tab_count().max(1);
        let active_tab = self.workspace.active_tab;
        let avail_w = self.area.width.max(20) as usize;
        let tab_width = (avail_w / tab_count).max(14).min(32);
        if tab_width == 0 {
            return;
        }
        let max_visible = avail_w / tab_width;
        if max_visible == 0 {
            return;
        }
        if active_tab < self.tab_scroll_offset {
            self.tab_scroll_offset = active_tab;
        } else if active_tab >= self.tab_scroll_offset + max_visible {
            self.tab_scroll_offset = active_tab.saturating_add(1).saturating_sub(max_visible);
        }
    }

    pub fn command_palette_execute(&mut self) {
        let cmd = self.command_input.trim().to_string();
        self.command_input.clear();
        self.show_command_palette = false;

        match cmd.as_str() {
            "split-v" => self.split_active_pane(SplitDirection::Vertical),
            "split-h" => self.split_active_pane(SplitDirection::Horizontal),
            "close" => self.close_active_pane(),
            "files" => self.show_file_tree = !self.show_file_tree,
            "status" => self.show_status_bar = !self.show_status_bar,
            "quit" => self.running = false,
            _ => {
                self.set_status_message(format!("Unknown command: {}", cmd));
            }
        }
    }

    // ── Search overlay (E1) ──

    /// Perform search in the active pane's screen content
    pub fn perform_search(&mut self) {
        self.search_matches.clear();
        self.search_match_index = 0;
        let query = self.search_query.trim();
        if query.is_empty() {
            return;
        }
        let active_pane = self.workspace.active_tab().active_pane();
        if let Some(session) = self.sessions.get(&active_pane) {
            if let Ok(parser) = session.parser.try_read() {
                let screen = parser.screen();
                let (rows, cols) = screen.size();
                let query_lower = query.to_lowercase();
                for row in 0..rows {
                    let mut col = 0u16;
                    while col < cols {
                        if let Some(cell) = screen.cell(row, col) {
                            let contents = cell.contents();
                            // Simple substring search
                            if contents.to_lowercase().contains(&query_lower) {
                                let start = col;
                                let end = col + contents.len() as u16;
                                self.search_matches.push((row, start, end));
                            }
                        }
                        col += 1;
                    }
                }
            }
        }
    }

    /// Move to the next search match
    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_match_index = (self.search_match_index + 1) % self.search_matches.len();
    }

    /// Move to the previous search match
    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_match_index = if self.search_match_index == 0 {
            self.search_matches.len() - 1
        } else {
            self.search_match_index - 1
        };
    }

    /// Input a character into the search query
    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.perform_search();
    }

    /// Backspace in the search query
    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.perform_search();
    }
}
