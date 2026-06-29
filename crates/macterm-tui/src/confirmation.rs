/// Pending confirmation action (for close pane / quit confirmation dialogs)
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    None,
    ClosePane,
    Quit,
}
