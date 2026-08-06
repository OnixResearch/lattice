/// Candidate contextual routing state validated atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRoutingCandidate {
    /// Monotonic routing generation.
    pub generation: u64,
    /// Focused context from which the default active chain is derived.
    pub focused_context: super::super::ActionContextId,
    /// Candidate context records.
    pub contexts: Vec<super::ActionContext>,
    /// Candidate handler registrations.
    pub registrations: Vec<super::ActionHandlerRegistration>,
    /// Candidate typed fact schema.
    pub fact_definitions: Vec<super::facts::ActionContextFactDefinition>,
    /// Predicate complexity policy.
    pub predicate_limits: super::super::predicates::ActionContextPredicateLimits,
}

impl ActionRoutingCandidate {
    /// Creates a routing candidate with default predicate limits.
    #[must_use]
    pub fn new(
        generation: u64,
        focused_context: impl Into<super::super::ActionContextId>,
        contexts: Vec<super::ActionContext>,
        registrations: Vec<super::ActionHandlerRegistration>,
    ) -> Self {
        Self {
            generation,
            focused_context: focused_context.into(),
            contexts,
            registrations,
            fact_definitions: Vec::new(),
            predicate_limits: super::super::predicates::ActionContextPredicateLimits::default(),
        }
    }

    /// Sets the typed fact schema.
    #[must_use]
    pub fn with_fact_definitions(mut self, definitions: Vec<super::facts::ActionContextFactDefinition>) -> Self {
        self.fact_definitions = definitions;
        self
    }

    /// Narrows the predicate complexity policy.
    #[must_use]
    pub const fn with_predicate_limits(
        mut self,
        limits: super::super::predicates::ActionContextPredicateLimits,
    ) -> Self {
        self.predicate_limits = limits;
        self
    }
}

/// Stable contextual routing diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoutingDiagnosticCode {
    /// An identity is empty or not normalized.
    MalformedIdentity,
    /// A context identity is duplicated.
    DuplicateContextId,
    /// A registration identity is duplicated.
    DuplicateRegistrationId,
    /// A fact schema identity is duplicated.
    DuplicateFactDefinition,
    /// The routing generation is zero or non-monotonic.
    NonMonotonicGeneration,
    /// A context parent is absent.
    MissingParent,
    /// The context graph contains a cycle.
    ContextCycle,
    /// The graph does not contain exactly one application root.
    InvalidRoot,
    /// Focus association is absent, inconsistent, or stale.
    StaleFocusLink,
    /// Active modal records contradict the accepted focus chain.
    ContradictoryModalRoot,
    /// A fact record is malformed or exceeds a bound.
    MalformedFact,
    /// A fact is not declared in the schema.
    UnknownFact,
    /// A fact value does not match its declared kind.
    FactTypeMismatch,
    /// A registration references an unknown action.
    UnknownAction,
    /// A registration or predicate references an unknown owner context.
    UnknownOwner,
    /// A registration observes a stale owner lifecycle generation.
    StaleContextGeneration,
    /// A propagation declaration is inconsistent with its owner.
    InvalidPropagation,
    /// Predicate nesting exceeds the accepted bound.
    PredicateDepthExceeded,
    /// Predicate node count exceeds the accepted bound.
    PredicateNodeLimitExceeded,
    /// A predicate has malformed logical structure.
    MalformedPredicate,
    /// A request supplies an invalid active context chain.
    InvalidActiveChain,
    /// An invocation observes an old routing generation.
    StaleRouteGeneration,
    /// Action, context, or registration state is disabled.
    Disabled,
    /// An active modal or stopping registration blocks fallback.
    Blocked,
    /// Equal-priority registrations make the target ambiguous.
    Ambiguous,
    /// Registrations exist but none supports the current facts or chain.
    Unsupported,
    /// No contextual registration handles the action.
    Unhandled,
}

/// Deterministic contextual routing diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDiagnostic {
    /// Stable diagnostic code.
    pub code: RoutingDiagnosticCode,
    /// Related action when available.
    pub action_id: Option<crate::nexus_core::ActionId>,
    /// Related contexts in stable order.
    pub context_ids: Vec<super::super::ActionContextId>,
    /// Related registrations in stable order.
    pub registration_ids: Vec<super::super::ActionHandlerRegistrationId>,
    /// Affected field or phase.
    pub field: String,
    /// Human-readable explanation.
    pub message: String,
}

impl RoutingDiagnostic {
    pub(in crate::nexus_core::contextual) fn new(
        code: RoutingDiagnosticCode,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            action_id: None,
            context_ids: Vec::new(),
            registration_ids: Vec::new(),
            field: field.into(),
            message: message.into(),
        }
    }

    pub(in crate::nexus_core::contextual) fn with_action(mut self, action_id: crate::nexus_core::ActionId) -> Self {
        self.action_id = Some(action_id);
        self
    }

    pub(in crate::nexus_core::contextual) fn with_context(mut self, context_id: super::super::ActionContextId) -> Self {
        self.context_ids.push(context_id);
        self.context_ids.sort();
        self.context_ids.dedup();
        self
    }

    pub(in crate::nexus_core::contextual) fn with_registration(
        mut self,
        registration_id: super::super::ActionHandlerRegistrationId,
    ) -> Self {
        self.registration_ids.push(registration_id);
        self.registration_ids.sort();
        self.registration_ids.dedup();
        self
    }
}

/// Accepted immutable contextual routing snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRoutingSnapshot {
    pub(in crate::nexus_core::contextual) generation: u64,
    pub(in crate::nexus_core::contextual) contexts:
        std::collections::BTreeMap<super::super::ActionContextId, super::ActionContext>,
    pub(in crate::nexus_core::contextual) registrations: Vec<super::ActionHandlerRegistration>,
    pub(in crate::nexus_core::contextual) fact_definitions:
        std::collections::BTreeMap<super::super::ActionContextFactId, super::facts::ActionContextFactDefinition>,
    pub(in crate::nexus_core::contextual) active_chain: Vec<super::super::ActionContextId>,
    pub(in crate::nexus_core::contextual) registry: crate::nexus_core::ActionRegistrySnapshot,
    pub(in crate::nexus_core::contextual) predicate_limits: super::super::predicates::ActionContextPredicateLimits,
}

impl ActionRoutingSnapshot {
    /// Returns the accepted routing generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the focused-to-root default active chain.
    #[must_use]
    pub fn active_chain(&self) -> &[super::super::ActionContextId] {
        &self.active_chain
    }

    /// Returns an accepted context by identity.
    #[must_use]
    pub fn context(&self, id: &super::super::ActionContextId) -> Option<&super::ActionContext> {
        self.contexts.get(id)
    }

    /// Returns accepted registrations in deterministic order.
    #[must_use]
    pub fn registrations(&self) -> &[super::ActionHandlerRegistration] {
        &self.registrations
    }

    /// Returns the action metadata snapshot resolved by this routing generation.
    #[must_use]
    pub const fn action_registry(&self) -> &crate::nexus_core::ActionRegistrySnapshot {
        &self.registry
    }
}

/// Result of atomically validating a routing candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRoutingBuild {
    /// New accepted snapshot, or the previous snapshot after candidate rejection.
    pub snapshot: Option<ActionRoutingSnapshot>,
    /// Deterministic validation diagnostics.
    pub diagnostics: Vec<RoutingDiagnostic>,
    /// Whether the candidate replaced the previous accepted snapshot.
    pub candidate_accepted: bool,
}
