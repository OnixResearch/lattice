/// Semantic kind of a contextual action owner.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionContextKind {
    /// Application root.
    Application,
    /// Workspace scope.
    Workspace,
    /// Pane scope.
    Pane,
    /// Focused editor content.
    Editor,
    /// Focused tree content.
    Tree,
    /// Active modal scope.
    Modal,
    /// Application-defined semantic context kind.
    Custom(String),
}

/// Modal policy attached to a modal context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContextModalBoundary {
    /// Whether application-root escape registrations may cross this boundary.
    pub allows_application_escape: bool,
}

impl ActionContextModalBoundary {
    /// Creates a modal boundary that blocks application-root routes.
    #[must_use]
    pub const fn blocking() -> Self {
        Self {
            allows_application_escape: false,
        }
    }

    /// Creates a modal boundary that allows explicit application escape routes.
    #[must_use]
    pub const fn allowing_application_escape() -> Self {
        Self {
            allows_application_escape: true,
        }
    }
}

/// Immutable contextual action owner record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContext {
    /// Stable context identity.
    pub id: super::super::ActionContextId,
    /// Parent context, absent only for a root.
    pub parent: Option<super::super::ActionContextId>,
    /// Semantic context kind.
    pub kind: ActionContextKind,
    /// Lifecycle generation of this context instance.
    pub generation: u64,
    /// Stable sibling ordering evidence.
    pub order: u32,
    /// Whether the context belongs to the accepted active state.
    pub active: bool,
    /// Whether the context is visible.
    pub visible: bool,
    /// Whether handlers owned by this context may run.
    pub enabled: bool,
    /// Disabled reason when the context cannot handle actions.
    pub disabled_reason: Option<String>,
    /// Modal boundary metadata.
    pub modal: Option<ActionContextModalBoundary>,
    /// Associated renderer-neutral focus scope, when available.
    pub focus_scope: Option<crate::focus::FocusScopeId>,
    /// Associated renderer-neutral focus node, when available.
    pub focus_node: Option<crate::focus::FocusNodeId>,
    /// Bounded facts supplied for predicate evaluation.
    pub facts: std::collections::BTreeMap<super::super::ActionContextFactId, super::facts::ActionContextFactValue>,
}

impl ActionContext {
    /// Creates a root context.
    #[must_use]
    pub fn root(id: impl Into<super::super::ActionContextId>, kind: ActionContextKind, generation: u64) -> Self {
        Self::new(id.into(), None, kind, generation)
    }

    /// Creates a child context.
    #[must_use]
    pub fn child(
        id: impl Into<super::super::ActionContextId>,
        parent: impl Into<super::super::ActionContextId>,
        kind: ActionContextKind,
        generation: u64,
    ) -> Self {
        Self::new(id.into(), Some(parent.into()), kind, generation)
    }

    fn new(
        id: super::super::ActionContextId,
        parent: Option<super::super::ActionContextId>,
        kind: ActionContextKind,
        generation: u64,
    ) -> Self {
        Self {
            id,
            parent,
            kind,
            generation,
            order: 0,
            active: true,
            visible: true,
            enabled: true,
            disabled_reason: None,
            modal: None,
            focus_scope: None,
            focus_node: None,
            facts: std::collections::BTreeMap::new(),
        }
    }

    /// Sets stable sibling ordering evidence.
    #[must_use]
    pub const fn with_order(mut self, order: u32) -> Self {
        self.order = order;
        self
    }

    /// Marks the context inactive.
    #[must_use]
    pub const fn inactive(mut self) -> Self {
        self.active = false;
        self
    }

    /// Marks the context hidden.
    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Marks the context disabled with a deterministic reason.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }

    /// Attaches modal boundary policy.
    #[must_use]
    pub const fn with_modal(mut self, modal: ActionContextModalBoundary) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Associates a shared focus scope.
    #[must_use]
    pub fn with_focus_scope(mut self, scope: impl Into<crate::focus::FocusScopeId>) -> Self {
        self.focus_scope = Some(scope.into());
        self
    }

    /// Associates a shared focus node.
    #[must_use]
    pub fn with_focus_node(mut self, node: impl Into<crate::focus::FocusNodeId>) -> Self {
        self.focus_node = Some(node.into());
        self
    }

    /// Adds or replaces one typed fact.
    #[must_use]
    pub fn with_fact(
        mut self,
        id: impl Into<super::super::ActionContextFactId>,
        value: impl Into<super::facts::ActionContextFactValue>,
    ) -> Self {
        self.facts.insert(id.into(), value.into());
        self
    }
}

/// Explicit handler propagation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPropagation {
    /// An ineligible registration blocks fallback to ancestors.
    Stop,
    /// An ineligible registration permits fallback to ancestors.
    Bubble,
    /// The registration is valid only at the application root.
    RootOnly,
    /// The registration participates only while its owner is on the active chain.
    FocusedChain,
}

/// Pure contextual action-handler registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionHandlerRegistration {
    /// Stable registration identity used by application shells.
    pub id: super::super::ActionHandlerRegistrationId,
    /// Shared action identity.
    pub action_id: crate::nexus_core::ActionId,
    /// Owning context identity.
    pub owner_context: super::super::ActionContextId,
    /// Expected lifecycle generation of the owner.
    pub owner_generation: u64,
    /// Bounded pure applicability predicate.
    pub predicate: super::super::predicates::ActionContextPredicate,
    /// Same-context precedence.
    pub priority: i16,
    /// Explicit propagation policy.
    pub propagation: ActionPropagation,
    /// Registration-local enablement metadata.
    pub enablement: crate::nexus_core::ActionEnablement,
    /// Whether this registration may cross an active modal boundary.
    pub allows_modal_escape: bool,
}

impl ActionHandlerRegistration {
    /// Creates an enabled, stopping registration with an `Always` predicate.
    #[must_use]
    pub fn new(
        id: impl Into<super::super::ActionHandlerRegistrationId>,
        action_id: impl Into<crate::nexus_core::ActionId>,
        owner_context: impl Into<super::super::ActionContextId>,
        owner_generation: u64,
    ) -> Self {
        Self {
            id: id.into(),
            action_id: action_id.into(),
            owner_context: owner_context.into(),
            owner_generation,
            predicate: super::super::predicates::ActionContextPredicate::Always,
            priority: 0,
            propagation: ActionPropagation::Stop,
            enablement: crate::nexus_core::ActionEnablement::Enabled,
            allows_modal_escape: false,
        }
    }

    /// Sets the pure applicability predicate.
    #[must_use]
    pub fn with_predicate(mut self, predicate: super::super::predicates::ActionContextPredicate) -> Self {
        self.predicate = predicate;
        self
    }

    /// Sets same-context priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: i16) -> Self {
        self.priority = priority;
        self
    }

    /// Sets explicit propagation policy.
    #[must_use]
    pub const fn with_propagation(mut self, propagation: ActionPropagation) -> Self {
        self.propagation = propagation;
        self
    }

    /// Marks the registration disabled.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enablement = crate::nexus_core::ActionEnablement::Disabled { reason: reason.into() };
        self
    }

    /// Allows this registration to cross an active modal boundary.
    #[must_use]
    pub const fn allowing_modal_escape(mut self) -> Self {
        self.allows_modal_escape = true;
        self
    }
}
