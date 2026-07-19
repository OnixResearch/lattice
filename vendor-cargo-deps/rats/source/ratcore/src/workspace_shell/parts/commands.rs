//! Workspace shell commands, effects, action metadata, and typed UI intents.

/// Focus behavior for commands that create panes or tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPolicy {
    /// Preserve the previously focused pane.
    Preserve,
    /// Transfer focus to the newly created or targeted pane.
    Transfer,
}

/// Split axis requested by a workspace shell command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    /// Horizontal split creates side-by-side regions.
    Horizontal,
    /// Vertical split creates stacked regions.
    Vertical,
}

/// Pure workspace shell command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Create a tab in an existing workspace.
    CreateTab {
        /// Target workspace live ID.
        workspace: super::types::LiveId,
        /// Tab to insert.
        tab: super::state::Tab,
        /// Focus behavior after insertion.
        focus_policy: FocusPolicy,
    },
    /// Split a pane by inserting a sibling pane.
    SplitPane {
        /// Target workspace live ID.
        workspace: super::types::LiveId,
        /// Target tab live ID.
        tab: super::types::LiveId,
        /// Pane live ID being split.
        target_pane: super::types::LiveId,
        /// Pane to insert.
        pane: super::state::Pane,
        /// Requested split axis.
        axis: SplitAxis,
        /// Focus behavior after insertion.
        focus_policy: FocusPolicy,
    },
    /// Move focus to a pane.
    FocusPane {
        /// Target workspace live ID.
        workspace: super::types::LiveId,
        /// Target tab live ID.
        tab: super::types::LiveId,
        /// Target pane live ID.
        pane: super::types::LiveId,
    },
    /// Close a pane and retire its live ID.
    ClosePane {
        /// Target workspace live ID.
        workspace: super::types::LiveId,
        /// Target tab live ID.
        tab: super::types::LiveId,
        /// Target pane live ID.
        pane: super::types::LiveId,
    },
    /// Restore a validated shell model.
    RestoreLayout {
        /// Candidate restored shell model.
        model: super::state::Model,
    },
    /// Update a pane task status.
    UpdateTaskStatus {
        /// Target pane live ID.
        pane: super::types::LiveId,
        /// New task status.
        status: super::types::TaskStatus,
    },
    /// Represent a disabled command activation attempt.
    DisabledAction {
        /// Stable action ID.
        action_id: String,
        /// Disabled reason.
        reason: String,
    },
}

/// Declarative effect request returned by the pure reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectRequest {
    /// Application may create resources associated with a tab.
    CreateTab {
        /// Inserted tab live ID.
        tab: super::types::LiveId,
    },
    /// Application may arrange renderer-owned split visuals.
    SplitPane {
        /// Inserted pane live ID.
        pane: super::types::LiveId,
        /// Requested split axis.
        axis: SplitAxis,
    },
    /// Application may update external focus affordances.
    FocusPane {
        /// Focused pane live ID.
        pane: super::types::LiveId,
    },
    /// Application may dispose resources associated with a pane.
    ClosePane {
        /// Closed pane live ID.
        pane: super::types::LiveId,
    },
    /// Application may publish status changes.
    UpdateTaskStatus {
        /// Updated pane live ID.
        pane: super::types::LiveId,
        /// New task status.
        status: super::types::TaskStatus,
    },
    /// Application may rebuild renderer state after layout restore.
    RestoreLayout,
}

/// Result of reducing one workspace command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Accepted shell model after the command, or the previous model on failure.
    pub model: super::state::Model,
    /// Declarative effects for the imperative shell to execute.
    pub effects: Vec<EffectRequest>,
    /// Deterministic command diagnostics.
    pub diagnostics: Vec<super::types::Diagnostic>,
}

/// Workspace action metadata before registry projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDescriptor {
    /// Stable action ID.
    pub id: String,
    /// User-facing action label.
    pub label: String,
    /// Optional action description.
    pub description: Option<String>,
    /// Optional shortcut.
    pub shortcut: Option<crate::action_registry::ActionShortcut>,
    /// Focus behavior associated with the action.
    pub focus_policy: FocusPolicy,
    /// Whether the command can currently run.
    pub enabled: bool,
    /// Disabled reason when the command cannot run.
    pub disabled_reason: Option<String>,
}

impl ActionDescriptor {
    /// Creates enabled workspace action metadata.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, focus_policy: FocusPolicy) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            shortcut: None,
            focus_policy,
            enabled: true,
            disabled_reason: None,
        }
    }

    /// Adds a description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a shortcut.
    #[must_use]
    pub fn with_shortcut(mut self, shortcut: crate::action_registry::ActionShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Marks the action disabled with a deterministic reason.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// Workspace action projection with focus-policy metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProjection {
    /// Stable action ID.
    pub action_id: String,
    /// User-facing action label.
    pub label: String,
    /// Workspace action category.
    pub category: String,
    /// Optional user-facing shortcut label.
    pub shortcut_label: Option<String>,
    /// Whether the action can be activated.
    pub enabled: bool,
    /// Disabled reason when the action cannot be activated.
    pub disabled_reason: Option<String>,
    /// Focus behavior associated with the action.
    pub focus_policy: FocusPolicy,
    /// Disabled diagnostic shown by renderers when present.
    pub diagnostic: Option<super::types::Diagnostic>,
}

/// Workspace command surfaces projected through the shared action registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSurfaces {
    /// Registry descriptors used to build shared projections.
    pub descriptors: Vec<crate::action_registry::ActionDescriptor>,
    /// Workspace-specific projections including focus policy metadata.
    pub workspace: Vec<ActionProjection>,
    /// Keymap projection.
    pub keymap: Vec<crate::action_registry::KeymapBindingProjection>,
    /// Menu projection.
    pub menu: Vec<crate::action_registry::MenuItemProjection>,
    /// Command palette projection.
    pub palette: Vec<crate::action_registry::CommandPaletteItemProjection>,
    /// Shortcut help labels by action ID.
    pub help: std::collections::BTreeMap<String, String>,
}

/// Typed UI intent emitted by workspace shell components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Focus a pane row.
    FocusPane {
        /// Workspace live ID.
        workspace: super::types::LiveId,
        /// Tab live ID.
        tab: super::types::LiveId,
        /// Pane live ID.
        pane: super::types::LiveId,
    },
    /// Select a tab row.
    SelectTab {
        /// Workspace live ID.
        workspace: super::types::LiveId,
        /// Tab live ID.
        tab: super::types::LiveId,
    },
    /// Open a pane context menu.
    OpenContextMenu {
        /// Pane live ID.
        pane: super::types::LiveId,
    },
    /// Request a split gesture.
    SplitPane {
        /// Pane live ID.
        pane: super::types::LiveId,
        /// Requested axis.
        axis: SplitAxis,
    },
    /// Invoke an action registry command.
    InvokeAction {
        /// Stable action ID.
        action_id: String,
    },
}
