use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::pty::{PtyEvent, PtySession};
use macterm_core::*;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tokio::sync::mpsc;

/// Cached system statistics for the header bar
#[derive(Debug, Clone)]
pub struct SysStats {
    pub cpu_pct: f32,
    pub mem_used_gb: f32,
    pub mem_total_gb: f32,
    pub cpu_brand: String,
}

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
    /// Saved split tree when zoomed; None = normal view
    pub zoom_root: Option<Box<SplitNode>>,
    /// Application configuration
    pub config: Config,
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
    /// Set true when PTY output, user input, or resize occurs; cleared after render.
    pub dirty: bool,
    /// System clipboard access
    pub clipboard: Option<arboard::Clipboard>,
    /// Mouse selection start position (row, col) — None = not selecting
    pub mouse_select_start: Option<(u16, u16)>,
    /// Mouse selection end position (row, col)
    pub mouse_select_end: Option<(u16, u16)>,
    /// Fullscreen pane cycling mode: each pane takes full content area
    pub fullscreen_pane_mode: bool,
    /// Index into pane_ids for fullscreen cycling
    pub fullscreen_pane_index: usize,
    /// Quick pane jump overlay (display-panes style)
    pub show_pane_jump: bool,
    /// System monitor for CPU/memory stats collection
    sysmon: sysinfo::System,
    /// Cached system stats (refreshed periodically)
    pub stats: SysStats,
    /// Tick counter for stats refresh (~every 120 frames)
    stats_tick: u64,
}

impl App {
    pub fn new(config: Config) -> (Self, mpsc::UnboundedSender<PtyEvent>) {
        let (pty_tx, pty_rx) = mpsc::unbounded_channel();

        let workspace = Workspace::new("default");

        let mut sessions = HashMap::new();

        // Spawn initial terminal pane
        let first_pane_id = workspace.active_tab().active_pane();
        match PtySession::spawn(
            first_pane_id, 80, 24,
            config.scrollback_lines, config.shell.as_deref(),
            pty_tx.clone(),
        ) {
            Ok(session) => {
                sessions.insert(first_pane_id, session);
            }
            Err(e) => {
                log::error!("Failed to spawn initial PTY: {}", e);
            }
        }

        // Initialize system monitor for stats
        let mut sysmon = sysinfo::System::new_all();
        std::thread::sleep(Duration::from_millis(200));
        sysmon.refresh_cpu_usage();
        sysmon.refresh_memory();
        let cpu_brand = sysmon.cpus().first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();
        let stats = SysStats {
            cpu_pct: sysmon.global_cpu_usage(),
            mem_used_gb: sysmon.used_memory() as f32 / 1073741824.0,
            mem_total_gb: sysmon.total_memory() as f32 / 1073741824.0,
            cpu_brand,
        };

        (
            Self {
                workspace,
                sessions,
                pty_rx,
                config,
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
                zoom_root: None,
                tab_scroll_offset: 0,
                file_tree_entries: Vec::new(),
                file_tree_scroll: 0,
                show_search: false,
                search_query: String::new(),
                search_matches: Vec::new(),
                search_match_index: 0,
                dirty: true,
                clipboard: arboard::Clipboard::new().ok(),
                mouse_select_start: None,
                mouse_select_end: None,
                fullscreen_pane_mode: false,
                fullscreen_pane_index: 0,
                show_pane_jump: false,
                sysmon,
                stats,
                stats_tick: 0,
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
        // Refresh system stats every ~120 frames (~2s at 60fps)
        self.stats_tick += 1;
        if self.stats_tick >= 120 {
            self.stats_tick = 0;
            self.refresh_stats();
        }
    }

    /// Refresh CPU/memory stats via sysinfo
    fn refresh_stats(&mut self) {
        self.sysmon.refresh_cpu_usage();
        self.sysmon.refresh_memory();
        self.stats.cpu_pct = self.sysmon.global_cpu_usage();
        self.stats.mem_used_gb = self.sysmon.used_memory() as f32 / 1073741824.0;
        self.stats.mem_total_gb = self.sysmon.total_memory() as f32 / 1073741824.0;
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
        let head_h: u16 = 3;
        let file_tree_w: u16 = if self.show_file_tree { 20 } else { 0 };
        let content_area = Rect {
            x: file_tree_w,
            y: head_h,
            width: self.area.width.saturating_sub(file_tree_w),
            height: self.area.height.saturating_sub(head_h + status_h),
        };
        let tab = self.workspace.active_tab();
        // In fullscreen mode, give the active pane the full content area
        let root = if self.fullscreen_pane_mode {
            SplitNode::Leaf(tab.active_pane())
        } else {
            tab.root.clone()
        };
        let pane_rects = crate::ui::pane_rects_from_tree(&root, content_area);
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
            // Non-blocking scrollback reset — skip if parser is locked
            if let Ok(mut p) = session.parser.try_write() {
                if p.screen().scrollback() > 0 {
                    p.set_scrollback(0);
                }
            }
            let _ = session.write(data);
            // Clear any stale mouse selection so selection highlighting
            // doesn't bleed into other panes during rendering
            self.mouse_select_start = None;
            self.mouse_select_end = None;
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
        match PtySession::spawn(
            new_pane_id, 80, 24,
            self.config.scrollback_lines, self.config.shell.as_deref(),
            tx,
        ) {
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
        let tab = self.workspace.active_tab_mut();
        if let Some(saved) = self.zoom_root.take() {
            tab.root = *saved;
        } else {
            self.zoom_root = Some(Box::new(tab.root.clone()));
            let active = tab.active_pane();
            tab.root = SplitNode::Leaf(active);
        }
        self.resize_active_panes();
    }

    /// Scroll the active pane by a delta (positive = up/backward)
    pub fn scroll_active_pane(&mut self, delta: i32) {
        let pane_id = self.workspace.active_tab().active_pane();
        if let Some(session) = self.sessions.get(&pane_id) {
            if let Ok(mut p) = session.parser.try_write() {
                let (rows, _) = p.screen().size();
                let current = p.screen().scrollback();
                let new = (current as i32 + delta * rows as i32).max(0) as usize;
                p.set_scrollback(new);
            }
        }
    }

    /// Reset scrollback to bottom (called when user types to shell)
    pub fn scroll_reset(&mut self) {
        let pane_id = self.workspace.active_tab().active_pane();
        if let Some(session) = self.sessions.get(&pane_id) {
            if let Ok(mut p) = session.parser.write() {
                p.set_scrollback(0);
            }
        }
    }

    pub fn close_active_tab(&mut self) {
        if !self.workspace.remove_active_tab() {
            return;
        }
        self.resize_active_panes();
    }

    /// Copy selected text to clipboard. Returns true if copied.
    pub fn copy_selection(&mut self) -> bool {
        let (start, end) = match (self.mouse_select_start, self.mouse_select_end) {
            (Some(s), Some(e)) => (s, e),
            _ => return false,
        };
        let row_min = start.0.min(end.0);
        let row_max = start.0.max(end.0);
        let col_min = if start.0 < end.0 || (start.0 == end.0 && start.1 < end.1) {
            start.1
        } else {
            end.1
        };
        let col_max = if start.0 > end.0 || (start.0 == end.0 && start.1 > end.1) {
            start.1
        } else {
            end.1
        };

        let pane_id = self.workspace.active_tab().active_pane();
        let text = self.sessions.get(&pane_id).and_then(|session| {
            session.parser.try_read().ok().map(|guard| {
                guard.screen().contents_between(row_min, col_min, row_max, col_max)
            })
        });
        if let Some(text) = text {
            if let Some(ref mut cb) = self.clipboard {
                if cb.set_text(text).is_ok() {
                    self.set_status_message("Copied".to_string());
                    return true;
                }
            }
        }
        false
    }

    /// Paste text from clipboard into active pane
    pub fn paste_clipboard(&mut self) {
        if let Some(ref mut cb) = self.clipboard {
            if let Ok(text) = cb.get_text() {
                self.write_to_active_pane(text.as_bytes());
            }
        }
    }

    /// Toggle fullscreen pane cycling mode
    pub fn toggle_fullscreen_mode(&mut self) {
        self.fullscreen_pane_mode = !self.fullscreen_pane_mode;
        if self.fullscreen_pane_mode {
            self.fullscreen_pane_index = 0;
            self.set_status_message("Fullscreen pane mode".to_string());
        } else {
            self.set_status_message("Split pane mode".to_string());
        }
    }

    /// Cycle to next/prev pane in fullscreen mode
    pub fn cycle_fullscreen_pane(&mut self, next: bool) {
        let ids = self.workspace.active_tab().pane_ids();
        if ids.is_empty() {
            return;
        }
        if next {
            self.fullscreen_pane_index = (self.fullscreen_pane_index + 1) % ids.len();
        } else {
            self.fullscreen_pane_index = if self.fullscreen_pane_index == 0 {
                ids.len() - 1
            } else {
                self.fullscreen_pane_index - 1
            };
        }
        self.workspace.active_tab_mut().active_pane = ids[self.fullscreen_pane_index];
    }

    /// Start quick pane jump overlay
    pub fn start_pane_jump(&mut self) {
        self.show_pane_jump = true;
    }

    /// Handle a numeric key press during pane jump. Returns true if handled.
    pub fn handle_pane_jump_key(&mut self, digit: u32) -> bool {
        let ids = self.workspace.active_tab().pane_ids();
        if digit > 0 && digit as usize <= ids.len() {
            let idx = digit as usize - 1;
            self.workspace.active_tab_mut().active_pane = ids[idx];
            self.show_pane_jump = false;
            self.set_status_message(format!("Pane {}", digit));
            true
        } else {
            false
        }
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
            if let Ok(mut parser) = session.parser.try_write() {
                let (rows, cols) = parser.screen().size();
                let query_lower = query.to_lowercase();
                let saved_offset = parser.screen().scrollback();

                parser.set_scrollback(usize::MAX);
                let top_offset = parser.screen().scrollback();
                let mut offset = top_offset;

                // Stores (absolute_row, col_start, col_end)
                let mut all_matches: Vec<(u16, u16, u16)> = Vec::new();

                loop {
                    parser.set_scrollback(offset);
                    let screen = parser.screen();
                    for rel_row in 0..rows {
                        let abs_row = (offset + rel_row as usize) as u16;
                        let mut col = 0u16;
                        while col < cols {
                            if let Some(cell) = screen.cell(rel_row, col) {
                                let contents = cell.contents();
                                if contents.to_lowercase().contains(&query_lower) {
                                    let start = col;
                                    let end = col + contents.len() as u16;
                                    all_matches.push((abs_row, start, end));
                                }
                            }
                            col += 1;
                        }
                    }
                    if offset < rows as usize {
                        break;
                    }
                    offset = offset.saturating_sub(rows as usize);
                }

                parser.set_scrollback(0);
                let screen = parser.screen();
                for rel_row in 0..rows {
                    let abs_row = rel_row as u16;
                    let mut col = 0u16;
                    while col < cols {
                        if let Some(cell) = screen.cell(rel_row, col) {
                            let contents = cell.contents();
                            if contents.to_lowercase().contains(&query_lower) {
                                let start = col;
                                let end = col + contents.len() as u16;
                                all_matches.push((abs_row, start, end));
                            }
                        }
                        col += 1;
                    }
                }

                all_matches.sort_by_key(|m| m.0);
                self.search_matches = all_matches;

                // Restore original scrollback position
                parser.set_scrollback(saved_offset);
            }
        }
    }

    /// Move to the next search match
    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_match_index = (self.search_match_index + 1) % self.search_matches.len();
        self.goto_match();
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
        self.goto_match();
    }

    /// Scroll the viewport to show the current search match
    pub fn goto_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let (row, _col, _end) = self.search_matches[self.search_match_index];
        let pane_id = self.workspace.active_tab().active_pane();
        if let Some(session) = self.sessions.get(&pane_id) {
            if let Ok(mut p) = session.parser.write() {
                let (vis_rows, _) = p.screen().size();
                let scroll_pos = (row as usize).saturating_sub(vis_rows as usize / 3);
                p.set_scrollback(scroll_pos);
            }
        }
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
