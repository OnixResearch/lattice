/// Maximum predicate depth accepted by the default policy.
pub const DEFAULT_CONTEXT_PREDICATE_DEPTH: u32 = 8;
/// Maximum predicate node count accepted by the default policy.
pub const DEFAULT_CONTEXT_PREDICATE_NODES: u32 = 64;
const HARD_MAX_CONTEXT_PREDICATE_DEPTH: u32 = 16;
const HARD_MAX_CONTEXT_PREDICATE_NODES: u32 = 256;

/// Bounded predicate complexity policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionContextPredicateLimits {
    /// Maximum root-inclusive predicate depth.
    pub max_depth: u32,
    /// Maximum total predicate nodes.
    pub max_nodes: u32,
}

impl ActionContextPredicateLimits {
    /// Creates an explicit complexity policy.
    #[must_use]
    pub const fn new(max_depth: u32, max_nodes: u32) -> Self {
        Self { max_depth, max_nodes }
    }

    pub(super) const fn valid(self) -> bool {
        self.max_depth > 0
            && self.max_nodes > 0
            && self.max_depth <= HARD_MAX_CONTEXT_PREDICATE_DEPTH
            && self.max_nodes <= HARD_MAX_CONTEXT_PREDICATE_NODES
    }
}

impl Default for ActionContextPredicateLimits {
    fn default() -> Self {
        Self::new(DEFAULT_CONTEXT_PREDICATE_DEPTH, DEFAULT_CONTEXT_PREDICATE_NODES)
    }
}

/// Action metadata state tested by a context predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionStatePredicate {
    /// The registry action is active.
    Enabled,
    /// The registry action is inactive.
    Disabled,
}

/// Closed pure predicate vocabulary for contextual applicability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionContextPredicate {
    /// Always matches.
    Always,
    /// Every child must match.
    All(Vec<Self>),
    /// At least one child must match.
    Any(Vec<Self>),
    /// Negates one child.
    Not(Box<Self>),
    /// A typed fact must be present in the focus-to-owner chain.
    FactPresent(super::ActionContextFactId),
    /// The nearest typed fact must equal the supplied value.
    FactEquals {
        /// Typed fact identity.
        fact_id: super::ActionContextFactId,
        /// Expected typed value.
        value: super::model::facts::ActionContextFactValue,
    },
    /// One context in the focus-to-owner chain must have this kind.
    ContextKind(super::model::ActionContextKind),
    /// The registration owner must be the focused context.
    Focused,
    /// The registration owner must be an ancestor of the focused context.
    Ancestor,
    /// Shared action metadata must have the requested state.
    ActionState(ActionStatePredicate),
}

/// Deterministic result of evaluating one predicate without side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContextPredicateEvaluation {
    /// Whether the predicate matched.
    pub matched: bool,
    /// Validation or evaluation diagnostics.
    pub diagnostics: Vec<super::model::RoutingDiagnostic>,
    /// Contexts from which matching facts were observed.
    pub inspected_fact_contexts: Vec<super::ActionContextId>,
}

#[path = "predicates/evaluation.rs"]
pub(super) mod evaluation;
pub(super) mod inspection;
pub use evaluation::evaluate_action_context_predicate;
