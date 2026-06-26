use std::collections::HashMap;

use crate::pty::{PtyEvent, PtySession};
use macterm_core::*;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

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
    /// Status message
    pub status_message: Option<String>,
    /// Whether to toggle status bar visibility
    pub show_status_bar: bool,
    /// Whether to show the help overlay
    pub show_help: bool,
    /// Animation frame counter
    pub frame_count: u64,
    /// Current resize drag state (for split border drag-to-resize)
    pub resize_state: ResizeState,
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
                show_status_bar: true,
                show_help: false,
                frame_count: 0,
                resize_state: ResizeState::Idle,
            },
            pty_tx,
        )
    }

    /// Handle a tick event (called every frame)
    pub fn tick(&mut self) {
        self.frame_count += 1;
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
                    self.status_message = Some(format!("Pane {} exited ({})", pane_id, code));
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
                let _ = session.resize(pane_rect.width.max(20), pane_rect.height.max(5));
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
                log::info!("Split pane: created {}", new_pane_id);
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
                self.status_message = Some(format!("Unknown command: {}", cmd));
            }
        }
    }
}
