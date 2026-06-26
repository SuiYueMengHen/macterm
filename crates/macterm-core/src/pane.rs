use std::fmt;

/// Unique identifier for a pane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub uuid::Uuid);

impl PaneId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Pane type: what content the pane shows
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneKind {
    /// A shell/PTY terminal
    Terminal,
    /// A file preview
    Preview,
    /// File tree sidebar
    FileTree,
}

/// Orientation for split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// A pane represents one terminal window in the layout
#[derive(Debug, Clone)]
pub struct Pane {
    pub id: PaneId,
    pub kind: PaneKind,
    pub title: String,
    pub cwd: Option<String>,
    pub scroll_offset: u32,
    pub is_zoomed: bool,
    pub program: Option<String>,
}

impl Pane {
    pub fn new(kind: PaneKind, title: impl Into<String>) -> Self {
        Self {
            id: PaneId::new(),
            kind,
            title: title.into(),
            cwd: None,
            scroll_offset: 0,
            is_zoomed: false,
            program: None,
        }
    }

    pub fn terminal(title: impl Into<String>) -> Self {
        Self::new(PaneKind::Terminal, title)
    }

    pub fn preview() -> Self {
        Self::new(PaneKind::Preview, "Preview".to_string())
    }

    pub fn file_tree() -> Self {
        Self::new(PaneKind::FileTree, "Files".to_string())
    }
}
