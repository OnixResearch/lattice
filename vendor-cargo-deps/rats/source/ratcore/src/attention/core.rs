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
//! assert_eq!(graph.active_id(), Some("save"));
//! assert_eq!(graph.tui_projection().unwrap().status_text, "Button: Save");
//! ```

const ROOT_SCOPE_ID: &str = "root";
const STYLE_FOCUSED: &str = "focused";
const STATUS_SEPARATOR: &str = ": ";

/// Focus scope such as an app root, panel, menu, or modal dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusScope {
    /// Stable scope ID.
    pub id: String,
    /// Optional parent scope ID.
    pub parent: Option<String>,
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
    pub fn root(id: impl Into<String>, label: impl Into<String>) -> Self {
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
    pub fn child(
        id: impl Into<String>,
        parent: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
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
    pub id: String,
    /// Owning scope ID.
    pub scope: String,
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
        id: impl Into<String>,
        scope: impl Into<String>,
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
        id: impl Into<String>,
        scope: impl Into<String>,
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

/// Deterministic focus error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusError {
    /// Stable error code.
    pub code: FocusErrorCode,
    /// Related ID when available.
    pub id: Option<String>,
    /// Human-readable diagnostic.
    pub message: String,
}

impl FocusError {
    fn new(code: FocusErrorCode, id: Option<String>, message: impl Into<String>) -> Self {
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
    pub previous: Option<String>,
    /// Newly active node ID.
    pub active: Option<String>,
    /// Announcement text for status/live-region adapters.
    pub announcement: Option<String>,
}

mod checks;
mod model;
mod motion;
mod semantics;
mod views;
pub use model::FocusGraph;
pub use semantics::{FocusRole, FocusWidgetKind};
pub use views::{AriaProjection, TuiFocusProjection};

/// Returns the default root scope ID used by examples and simple applications.
#[must_use]
pub fn default_root_scope_id() -> &'static str {
    ROOT_SCOPE_ID
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
