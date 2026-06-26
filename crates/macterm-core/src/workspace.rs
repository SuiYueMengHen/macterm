use crate::tab::Tab;
use std::fmt;

/// Unique identifier for a workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(pub uuid::Uuid);

impl WorkspaceId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A workspace holds a set of tabs
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl Workspace {
    /// Create a new workspace with one default tab
    pub fn new(name: impl Into<String>) -> Self {
        let tab = Tab::new("terminal");
        Self {
            id: WorkspaceId::new(),
            name: name.into(),
            tabs: vec![tab],
            active_tab: 0,
        }
    }

    /// Get the active tab
    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    /// Get the active tab (mutable)
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    /// Add a new tab
    pub fn add_tab(&mut self, title: impl Into<String>) {
        self.tabs.push(Tab::new(title));
        self.active_tab = self.tabs.len() - 1;
    }

    /// Remove the active tab. Returns false if there's only one tab.
    pub fn remove_active_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        true
    }

    /// Switch to a specific tab index
    pub fn switch_to_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab = index;
            true
        } else {
            false
        }
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    /// Tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}
