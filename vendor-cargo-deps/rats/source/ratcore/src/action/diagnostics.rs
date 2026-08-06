use super::ActionId;

/// Stable issue code for registry validation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionRegistryIssueCode {
    /// An action ID is empty after trimming whitespace.
    EmptyId,
    /// An action label is empty after trimming whitespace.
    EmptyLabel,
    /// An action category is empty after trimming whitespace.
    EmptyCategory,
    /// An action source name is empty after trimming whitespace.
    EmptySource,
    /// A shortcut scope is empty after trimming whitespace.
    EmptyShortcutScope,
    /// A shortcut key is empty after trimming whitespace.
    EmptyShortcutKey,
    /// A shortcut modifier is empty after trimming whitespace.
    EmptyShortcutModifier,
    /// More than one descriptor declared the same action ID.
    DuplicateId,
    /// More than one enabled action declared the same shortcut in the same scope.
    ShortcutConflict,
}

/// Deterministic registry validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRegistryIssue {
    /// Stable issue code.
    pub code: ActionRegistryIssueCode,
    /// Affected descriptor IDs when known.
    pub action_ids: Vec<ActionId>,
    /// Affected field when the issue is field-local.
    pub field: Option<String>,
    /// Human-readable diagnostic.
    pub message: String,
}

impl ActionRegistryIssue {
    pub(super) fn field_issue(code: ActionRegistryIssueCode, action_id: ActionId, field: &str, message: &str) -> Self {
        Self {
            code,
            action_ids: vec![action_id],
            field: Some(field.to_string()),
            message: message.to_string(),
        }
    }

    pub(super) fn multi_action_issue(
        code: ActionRegistryIssueCode,
        mut action_ids: Vec<ActionId>,
        message: String,
    ) -> Self {
        action_ids.sort();
        action_ids.dedup();
        Self {
            code,
            action_ids,
            field: None,
            message,
        }
    }
}
