//! Renderer-neutral focus graph and accessibility metadata.
//!
//! The graph stores focus scopes, focusable nodes, semantic roles, labels,
//! disabled/hidden state, shortcut hints, modal traps, and announcements.
//! Renderer adapters can map the same transition into web ARIA/live-region
//! output or terminal status/focus styling without changing widget logic.
//!
//! # Example
//!
//! ```
//! use ratcore::focus::{FocusGraph, FocusNode, FocusRole, FocusScope};
//!
//! let graph = FocusGraph::new()
//!     .with_scope(FocusScope::root("root", "Application"))
//!     .with_node(FocusNode::new("save", "root", FocusRole::Button, "Save"))
//!     .activate_first().unwrap();
//! assert_eq!(graph.active_id().map(|id| id.as_str()), Some("save"));
//! assert_eq!(graph.tui_projection().unwrap().status_text, "Button: Save");
//! ```

const ROOT_SCOPE_ID: &str = "root";
const STYLE_FOCUSED: &str = "focused";
const STATUS_SEPARATOR: &str = ": ";

semantic_string_id! {
    /// Identifier for a focus scope.
    pub struct FocusScopeId;
}

semantic_string_id! {
    /// Identifier for a focusable node.
    ///
    /// ```compile_fail
    /// use ratcore::focus::{FocusNode, FocusRole, FocusScopeId};
    /// let scope = FocusScopeId::new("root");
    /// let _ = FocusNode::new(scope, "root", FocusRole::Button, "Save");
    /// ```
    pub struct FocusNodeId;
}

/// Focus scope such as an app root, panel, menu, or modal dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusScope {
    /// Stable scope ID.
    pub id: FocusScopeId,
    /// Optional parent scope ID.
    pub parent: Option<FocusScopeId>,
    /// Scope label for announcements.
    pub label: String,
    /// Whether focus is trapped inside this scope while active.
    pub modal: bool,
    /// Whether descendants use roving focus semantics.
    pub roving: bool,
}

impl FocusScope {
    /// Creates a root scope.
    #[must_use]
    pub fn root(id: impl Into<FocusScopeId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            parent: None,
            label: label.into(),
            modal: false,
            roving: false,
        }
    }

    /// Creates a child scope.
    #[must_use]
    pub fn child(id: impl Into<FocusScopeId>, parent: impl Into<FocusScopeId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            parent: Some(parent.into()),
            label: label.into(),
            modal: false,
            roving: false,
        }
    }

    /// Marks the scope as modal.
    #[must_use]
    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    /// Marks the scope as a roving-focus scope.
    #[must_use]
    pub fn roving(mut self) -> Self {
        self.roving = true;
        self
    }
}

/// Focusable node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusNode {
    /// Stable node ID.
    pub id: FocusNodeId,
    /// Owning scope ID.
    pub scope: FocusScopeId,
    /// Semantic role.
    pub role: FocusRole,
    /// Accessible label.
    pub label: String,
    /// Optional shortcut hints.
    pub shortcuts: Vec<String>,
    /// Whether the node cannot be activated.
    pub disabled: bool,
    /// Whether the node is hidden from traversal.
    pub hidden: bool,
}

impl FocusNode {
    /// Creates a focus node.
    #[must_use]
    pub fn new(
        id: impl Into<FocusNodeId>,
        scope: impl Into<FocusScopeId>,
        role: FocusRole,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            scope: scope.into(),
            role,
            label: label.into(),
            shortcuts: Vec::new(),
            disabled: false,
            hidden: false,
        }
    }

    /// Creates a node for a known widget family.
    #[must_use]
    pub fn for_widget(
        kind: FocusWidgetKind,
        id: impl Into<FocusNodeId>,
        scope: impl Into<FocusScopeId>,
        label: impl Into<String>,
    ) -> Self {
        Self::new(id, scope, kind.default_role(), label)
    }

    /// Adds shortcut hints.
    #[must_use]
    pub fn with_shortcuts(mut self, shortcuts: Vec<String>) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    /// Marks the node disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Marks the node hidden.
    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    fn eligible(&self) -> bool {
        !self.disabled && !self.hidden
    }
}

/// Focus graph validation or traversal error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusErrorCode {
    /// A referenced scope does not exist.
    UnknownScope,
    /// A referenced node does not exist.
    UnknownNode,
    /// A node has an empty label.
    MissingLabel,
    /// The active node is disabled.
    ActiveDisabled,
    /// The active node is hidden.
    ActiveHidden,
    /// No eligible focus target exists.
    NoEligibleTarget,
    /// A requested focus target is outside the active modal scope.
    ModalEscape,
}

/// Typed identity associated with a focus diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusErrorId {
    /// Focus scope identity.
    Scope(FocusScopeId),
    /// Focus node identity.
    Node(FocusNodeId),
}

impl FocusErrorId {
    /// Returns the raw identity text for display adapters.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Scope(id) => id.as_str(),
            Self::Node(id) => id.as_str(),
        }
    }
}

/// Deterministic focus error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusError {
    /// Stable error code.
    pub code: FocusErrorCode,
    /// Related typed ID when available.
    pub id: Option<FocusErrorId>,
    /// Human-readable diagnostic.
    pub message: String,
}

impl FocusError {
    fn new(code: FocusErrorCode, id: Option<FocusErrorId>, message: impl Into<String>) -> Self {
        Self {
            code,
            id,
            message: message.into(),
        }
    }
}

/// Result of a focus transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusTransition {
    /// Previously active node ID.
    pub previous: Option<FocusNodeId>,
    /// Newly active node ID.
    pub active: Option<FocusNodeId>,
    /// Announcement text for status/live-region adapters.
    pub announcement: Option<String>,
}

mod accessibility;
mod checks;
mod model;
mod motion;
mod semantics;
mod views;
pub use accessibility::*;
pub use model::FocusGraph;
pub use semantics::FocusRole;
pub use semantics::FocusWidgetKind;
pub use views::AriaProjection;
pub use views::TuiFocusProjection;

/// Returns the default root scope ID used by examples and simple applications.
#[must_use]
pub fn default_root_scope_id() -> FocusScopeId {
    FocusScopeId::new(ROOT_SCOPE_ID)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
