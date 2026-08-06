//! Pure renderer-neutral contextual action routing.
//!
//! Context snapshots and handler registrations contain data only. A route probe
//! selects a typed registration target; application shells remain responsible
//! for mapping that target to local messages and effects.

semantic_string_id! {
    /// Stable identity for a contextual action owner.
    pub struct ActionContextId;
}

semantic_string_id! {
    /// Stable identity for one contextual handler registration.
    pub struct ActionHandlerRegistrationId;
}

semantic_string_id! {
    /// Stable identity for a typed context fact.
    pub struct ActionContextFactId;
}

mod model;
mod predicates;
mod receipts;
mod resolution;
mod surfaces;
mod validation;

pub use model::ActionActivationSurface;
pub use model::ActionContext;
pub use model::ActionContextFactDefinition;
pub use model::ActionContextFactValue;
pub use model::ActionContextFactValueKind;
pub use model::ActionContextKind;
pub use model::ActionContextModalBoundary;
pub use model::ActionHandlerRegistration;
pub use model::ActionInvocation;
pub use model::ActionInvocationReceipt;
pub use model::ActionPropagation;
pub use model::ActionRendererAdapter;
pub use model::ActionRoute;
pub use model::ActionRouteConsideration;
pub use model::ActionRouteDecision;
pub use model::ActionRouteOutcome;
pub use model::ActionRouteProbe;
pub use model::ActionRouteProvenance;
pub use model::ActionRoutingBuild;
pub use model::ActionRoutingCandidate;
pub use model::ActionRoutingSnapshot;
pub use model::RoutingDiagnostic;
pub use model::RoutingDiagnosticCode;
pub use predicates::ActionContextPredicate;
pub use predicates::ActionContextPredicateEvaluation;
pub use predicates::ActionContextPredicateLimits;
pub use predicates::ActionStatePredicate;
pub use predicates::evaluate_action_context_predicate;
pub use receipts::ActionRouteReplay;
pub use receipts::ActionRouteReplayReceipt;
pub use receipts::ActionRouteReplayStep;
pub use receipts::replay_action_routes;
pub use resolution::invoke_action;
pub use resolution::probe_action;
pub use surfaces::ActionRouteView;
pub use surfaces::CommandViews;
pub use surfaces::build_command_views;
pub use validation::accept_action_snapshot;

#[cfg(test)]
#[path = "contextual/tests.rs"]
mod tests;
