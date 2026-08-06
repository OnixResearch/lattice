//! Workspace-shell pilot for shared contextual action routing.

const CONTEXT_APPLICATION: &str = "workspace-context.application";
const FACT_CONTENT_KIND: &str = "workspace.content-kind";
const FACT_SELECTION_AVAILABLE: &str = "workspace.selection-available";
const SOURCE_WORKSPACE_CONTEXT: &str = "workspace-contextual-routing";
const ACTION_CLOSE_PANE: &str = "workspace.pane.close";
const ACTION_FIND: &str = "workspace.find";
const ACTION_SELECT_ALL: &str = "workspace.select-all";
const ACTION_REFRESH: &str = "workspace.refresh";
const ACTION_OPEN_PALETTE: &str = "workspace.command-palette.open";
const ACTION_DISMISS_MODAL: &str = "workspace.modal.dismiss";
const HANDLER_APPLICATION_FIND: &str = "workspace-handler.application.find";
const HANDLER_APPLICATION_REFRESH: &str = "workspace-handler.application.refresh";
const HANDLER_APPLICATION_PALETTE: &str = "workspace-handler.application.palette";
const HANDLER_WORKSPACE_REFRESH: &str = "workspace-handler.workspace.refresh";
const HANDLER_PANE_CLOSE: &str = "workspace-handler.pane.close";
const HANDLER_CONTENT_FIND: &str = "workspace-handler.content.find";
const HANDLER_CONTENT_SELECT_ALL: &str = "workspace-handler.content.select-all";
const HANDLER_MODAL_DISMISS: &str = "workspace-handler.modal.dismiss";

/// Representative action vocabulary covered by the workspace routing pilot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextualAction {
    /// Close the focused pane.
    ClosePane,
    /// Find in the nearest eligible owner.
    Find,
    /// Select all in focused content.
    SelectAll,
    /// Refresh the active workspace with application fallback.
    Refresh,
    /// Open the application command palette.
    OpenCommandPalette,
    /// Dismiss the active modal.
    DismissModal,
}

impl ContextualAction {
    /// Returns the stable shared action identity.
    #[must_use]
    pub fn action_id(self) -> crate::action_registry::ActionId {
        crate::action_registry::ActionId::new(match self {
            Self::ClosePane => ACTION_CLOSE_PANE,
            Self::Find => ACTION_FIND,
            Self::SelectAll => ACTION_SELECT_ALL,
            Self::Refresh => ACTION_REFRESH,
            Self::OpenCommandPalette => ACTION_OPEN_PALETTE,
            Self::DismissModal => ACTION_DISMISS_MODAL,
        })
    }
}

/// Focused content kind represented by the workspace pilot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextualContentKind {
    /// Text-editor content.
    Editor,
    /// Hierarchical tree content.
    Tree,
}

impl ContextualContentKind {
    fn context_kind(self) -> crate::action_registry::ActionContextKind {
        match self {
            Self::Editor => crate::action_registry::ActionContextKind::Editor,
            Self::Tree => crate::action_registry::ActionContextKind::Tree,
        }
    }

    const fn fact_value(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Tree => "tree",
        }
    }
}

/// Active modal state supplied by the application shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualModalState {
    /// Stable modal context suffix.
    pub id: String,
    /// Whether dismiss is currently enabled.
    pub dismiss_enabled: bool,
    /// Whether the boundary allows application-level escape routes.
    pub allows_application_escape: bool,
}

impl ContextualModalState {
    /// Creates a blocking, dismissible modal state.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            dismiss_enabled: true,
            allows_application_escape: false,
        }
    }

    /// Marks dismiss disabled.
    #[must_use]
    pub const fn dismiss_disabled(mut self) -> Self {
        self.dismiss_enabled = false;
        self
    }

    /// Allows application-level escape routes through the modal boundary.
    #[must_use]
    pub const fn allowing_application_escape(mut self) -> Self {
        self.allows_application_escape = true;
        self
    }
}

/// Closed availability state for one workspace contextual capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextualAvailability {
    /// Capability is available.
    Enabled,
    /// Capability is unavailable and must fail closed.
    Disabled,
}

impl ContextualAvailability {
    const fn from_enabled(enabled: bool) -> Self {
        if enabled { Self::Enabled } else { Self::Disabled }
    }

    const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// Application-owned state inputs used to build one routing generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Monotonic generation.
    pub generation: u64,
    /// Focused content kind.
    pub content_kind: ContextualContentKind,
    /// Focused-content handler availability.
    pub content: ContextualAvailability,
    /// Select-all applicability.
    pub selection: ContextualAvailability,
    /// Focused-pane close availability.
    pub close: ContextualAvailability,
    /// Workspace-refresh availability before application fallback.
    pub workspace_refresh: ContextualAvailability,
    /// Optional active modal state.
    pub modal: Option<ContextualModalState>,
}

impl Options {
    /// Creates enabled editor/tree pilot options.
    #[must_use]
    pub const fn new(generation: u64, content_kind: ContextualContentKind) -> Self {
        Self {
            generation,
            content_kind,
            content: ContextualAvailability::Enabled,
            selection: ContextualAvailability::Enabled,
            close: ContextualAvailability::Enabled,
            workspace_refresh: ContextualAvailability::Enabled,
            modal: None,
        }
    }

    /// Sets focused-content enablement.
    #[must_use]
    pub const fn with_content_enabled(mut self, enabled: bool) -> Self {
        self.content = ContextualAvailability::from_enabled(enabled);
        self
    }

    /// Sets select-all applicability.
    #[must_use]
    pub const fn with_selection_available(mut self, available: bool) -> Self {
        self.selection = ContextualAvailability::from_enabled(available);
        self
    }

    /// Sets focused-pane close enablement.
    #[must_use]
    pub const fn with_close_enabled(mut self, enabled: bool) -> Self {
        self.close = ContextualAvailability::from_enabled(enabled);
        self
    }

    /// Sets workspace-refresh enablement.
    #[must_use]
    pub const fn with_workspace_refresh_enabled(mut self, enabled: bool) -> Self {
        self.workspace_refresh = ContextualAvailability::from_enabled(enabled);
        self
    }

    /// Sets active modal state.
    #[must_use]
    pub fn with_modal(mut self, modal: ContextualModalState) -> Self {
        self.modal = Some(modal);
        self
    }
}

/// Contextual routing build plus command surfaces for the workspace pilot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualActionPilot {
    /// Atomic routing build result.
    pub routing: crate::action_registry::ActionRoutingBuild,
    /// Surfaces derived from the accepted or preserved snapshot.
    pub surfaces: Option<crate::action_registry::ContextualActionSurfaces>,
}

/// Typed target delivered to the application shell after pure resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextualActionTarget {
    /// Application root.
    Application,
    /// Active workspace.
    Workspace(super::types::WorkspaceId),
    /// Focused pane.
    Pane(super::types::PaneId),
    /// Focused editor/tree content.
    Content(super::types::PaneId),
    /// Active modal context.
    Modal(crate::action_registry::ActionContextId),
}

/// Typed workspace runtime message emitted only after accepted routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextualActionMessage {
    /// Close the focused pane.
    ClosePane {
        /// Focused workspace.
        workspace: super::types::WorkspaceId,
        /// Focused tab.
        tab: super::types::TabId,
        /// Focused pane.
        pane: super::types::PaneId,
        /// Resolved contextual owner.
        target: ContextualActionTarget,
    },
    /// Find in the resolved owner.
    Find {
        /// Resolved contextual owner.
        target: ContextualActionTarget,
    },
    /// Select all in focused content.
    SelectAll {
        /// Resolved contextual owner.
        target: ContextualActionTarget,
    },
    /// Refresh the resolved owner.
    Refresh {
        /// Resolved contextual owner.
        target: ContextualActionTarget,
    },
    /// Open the command palette.
    OpenCommandPalette {
        /// Resolved contextual owner.
        target: ContextualActionTarget,
    },
    /// Dismiss the active modal.
    DismissModal {
        /// Resolved contextual owner.
        target: ContextualActionTarget,
    },
}

impl ContextualActionMessage {
    /// Returns a stable runtime-message identity for replay evidence.
    #[must_use]
    pub const fn identity(&self) -> &'static str {
        match self {
            Self::ClosePane { .. } => "workspace-message.close-pane",
            Self::Find { .. } => "workspace-message.find",
            Self::SelectAll { .. } => "workspace-message.select-all",
            Self::Refresh { .. } => "workspace-message.refresh",
            Self::OpenCommandPalette { .. } => "workspace-message.open-command-palette",
            Self::DismissModal { .. } => "workspace-message.dismiss-modal",
        }
    }
}

#[path = "adapter.rs"]
mod adapter;
#[path = "catalog.rs"]
mod catalog;

pub use adapter::ContextualActionDispatch;
pub use adapter::ContextualKeyDispatch;
pub use adapter::ContextualKeyDispatchRequest;
pub use adapter::build_contextual_action_pilot;
pub use adapter::dispatch_contextual_action;
pub use adapter::dispatch_contextual_key_sequence;
pub use adapter::replay_contextual_actions;
