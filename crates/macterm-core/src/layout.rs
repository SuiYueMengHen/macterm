use crate::pane::{PaneId, SplitDirection};

/// A node in the binary split tree
#[derive(Debug, Clone)]
pub enum SplitNode {
    /// A leaf pane
    Leaf(PaneId),
    /// A split container with two children
    Split {
        direction: SplitDirection,
        /// Ratio of left/top child (0.0 - 1.0)
        ratio: f32,
        left: Box<SplitNode>,
        right: Box<SplitNode>,
    },
}

impl SplitNode {
    /// Create a new leaf node with the given pane id
    pub fn leaf(pane_id: PaneId) -> Self {
        Self::Leaf(pane_id)
    }

    /// Split this node horizontally at the given ratio
    pub fn split_horizontal(self, right: SplitNode, ratio: f32) -> Self {
        Self::Split {
            direction: SplitDirection::Horizontal,
            ratio: ratio.clamp(0.1, 0.9),
            left: Box::new(self),
            right: Box::new(right),
        }
    }

    /// Split this node vertically at the given ratio
    pub fn split_vertical(self, right: SplitNode, ratio: f32) -> Self {
        Self::Split {
            direction: SplitDirection::Vertical,
            ratio: ratio.clamp(0.1, 0.9),
            left: Box::new(self),
            right: Box::new(right),
        }
    }

    /// Collect all pane IDs in this tree (in order)
    pub fn pane_ids(&self) -> Vec<PaneId> {
        match self {
            SplitNode::Leaf(id) => vec![*id],
            SplitNode::Split { left, right, .. } => {
                let mut ids = left.pane_ids();
                ids.extend(right.pane_ids());
                ids
            }
        }
    }

    /// Find the parent split of a given pane, and return the path to it
    pub fn find_pane(&self, target: &PaneId) -> Option<&SplitNode> {
        match self {
            SplitNode::Leaf(id) if id == target => Some(self),
            SplitNode::Split { left, right, .. } => {
                left.find_pane(target).or_else(|| right.find_pane(target))
            }
            _ => None,
        }
    }

    /// Replace a leaf node with a new split node (for splitting an existing pane)
    pub fn replace_pane_with_split(
        self,
        target: &PaneId,
        new_split: SplitNode,
    ) -> Option<SplitNode> {
        match self {
            SplitNode::Leaf(id) if id == *target => Some(new_split),
            SplitNode::Leaf(_) => Some(self),
            SplitNode::Split {
                direction,
                ratio,
                left,
                right,
            } => {
                let new_left = left.replace_pane_with_split(target, new_split.clone());
                let new_right = right.replace_pane_with_split(target, new_split);
                match (new_left, new_right) {
                    (Some(l), Some(r)) => Some(SplitNode::Split {
                        direction,
                        ratio,
                        left: Box::new(l),
                        right: Box::new(r),
                    }),
                    _ => None,
                }
            }
        }
    }

    /// Remove a pane from the tree. Returns None if the tree would be empty.
    pub fn remove_pane(self, target: &PaneId) -> Option<SplitNode> {
        match self {
            SplitNode::Leaf(id) if id == *target => None,
            SplitNode::Leaf(_) => Some(self),
            SplitNode::Split {
                direction: _,
                ratio: _,
                left,
                right,
            } => {
                let new_left = left.remove_pane(target);
                let new_right = right.remove_pane(target);
                match (new_left, new_right) {
                    (Some(l), Some(r)) => {
                        // Both children survive, keep the split
                        Some(SplitNode::Split {
                            direction: SplitDirection::Vertical,
                            ratio: 0.5,
                            left: Box::new(l),
                            right: Box::new(r),
                        })
                    }
                    (Some(child), None) | (None, Some(child)) => {
                        // One child removed, promote the other
                        Some(child)
                    }
                    (None, None) => None,
                }
            }
        }
    }

    /// Count total panes
    pub fn count(&self) -> usize {
        match self {
            SplitNode::Leaf(_) => 1,
            SplitNode::Split { left, right, .. } => left.count() + right.count(),
        }
    }

    /// Check if any leaf in this subtree matches the given pane ID
    pub fn contains(&self, target: &PaneId) -> bool {
        match self {
            SplitNode::Leaf(id) => id == target,
            SplitNode::Split { left, right, .. } => left.contains(target) || right.contains(target),
        }
    }

    /// Update the ratio of the Split node whose subtree contains `target`.
    /// Returns true if a matching split was found and updated.
    pub fn update_ratio_by_pane(&mut self, target: &PaneId, new_ratio: f32) -> bool {
        match self {
            SplitNode::Leaf(_) => false,
            SplitNode::Split { left, right, ratio, .. } => {
                if left.contains(target) || right.contains(target) {
                    *ratio = new_ratio.clamp(0.1, 0.9);
                    true
                } else {
                    false
                }
            }
        }
    }
}
