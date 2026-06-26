use crate::layout::SplitNode;
use crate::pane::{Pane, PaneId};
use std::fmt;

/// Unique identifier for a tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub uuid::Uuid);

impl TabId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for TabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A tab contains a tree of split panes
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub root: SplitNode,
    pub active_pane: PaneId,
    #[allow(dead_code)]
    pane_counter: u64,
}

impl Tab {
    /// Create a new tab with a single terminal pane
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        let pane = Pane::terminal(format!("{}-1", &title));
        let pane_id = pane.id;
        let root = SplitNode::leaf(pane_id);

        Self {
            id: TabId::new(),
            title,
            root,
            active_pane: pane_id,
            pane_counter: 1,
        }
    }

    /// Get all pane IDs in the tab
    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.root.pane_ids()
    }

    /// Get the number of panes
    pub fn pane_count(&self) -> usize {
        self.root.count()
    }

    /// Set the active pane
    pub fn set_active_pane(&mut self, id: PaneId) {
        self.active_pane = id;
    }

    pub fn active_pane(&self) -> PaneId {
        self.active_pane
    }

    /// Rename the tab
    pub fn rename(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }
}
