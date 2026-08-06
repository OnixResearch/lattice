/// Compatibility name for a contextual routing diagnostic.
pub type ContextualActionDiagnostic = super::RoutingDiagnostic;
/// Compatibility name for a contextual routing diagnostic code.
pub type ContextualActionDiagnosticCode = super::RoutingDiagnosticCode;
/// Compatibility name for one contextual action route projection.
pub type ContextualActionProjection = super::ActionRouteView;
/// Compatibility name for contextual command-surface projections.
pub type ContextualActionSurfaces = super::CommandViews;

/// Atomically validates and accepts a contextual action snapshot.
#[must_use]
pub fn accept_contextual_action_snapshot(
    candidate: super::ActionRoutingCandidate,
    registry: &super::ActionRegistrySnapshot,
    focus_graph: Option<&crate::focus::FocusGraph>,
    previous: Option<&super::ActionRoutingSnapshot>,
) -> super::ActionRoutingBuild {
    super::accept_action_snapshot(candidate, registry, focus_graph, previous)
}

/// Probes one contextual action without executing its selected registration.
#[must_use]
pub fn probe_contextual_action(
    snapshot: &super::ActionRoutingSnapshot,
    action_id: &super::ActionId,
    active_chain: Option<&[super::ActionContextId]>,
) -> super::ActionRouteProbe {
    super::probe_action(snapshot, action_id, active_chain)
}

/// Revalidates a generation-bound contextual action invocation.
#[must_use]
pub fn invoke_contextual_action(
    snapshot: &super::ActionRoutingSnapshot,
    invocation: &super::ActionInvocation,
) -> super::ActionInvocationReceipt {
    super::invoke_action(snapshot, invocation)
}

/// Projects every command surface through contextual route probes.
#[must_use]
pub fn project_contextual_action_surfaces(snapshot: &super::ActionRoutingSnapshot) -> ContextualActionSurfaces {
    super::build_command_views(snapshot)
}
