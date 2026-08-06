mod request;
mod search;
mod selection;

/// Probes one action without executing its selected registration.
#[must_use]
pub fn probe_action(
    snapshot: &super::model::ActionRoutingSnapshot,
    action_id: &crate::nexus_core::ActionId,
    active_chain: Option<&[super::ActionContextId]>,
) -> super::model::ActionRouteProbe {
    debug_assert!(snapshot.generation > 0);
    debug_assert!(snapshot.predicate_limits.valid());
    let chain = active_chain.map_or_else(|| snapshot.active_chain.clone(), <[super::ActionContextId]>::to_vec);
    if let Some(diagnostic) = request::validate_input(snapshot, action_id, &chain) {
        return selection::rejected_probe(snapshot, action_id.clone(), chain, diagnostic, Vec::new());
    }
    let Some(action) = snapshot.registry.action(action_id) else {
        let diagnostic = super::model::RoutingDiagnostic::new(
            super::model::RoutingDiagnosticCode::UnknownAction,
            "request.action_id",
            "action disappeared from the accepted registry snapshot",
        )
        .with_action(action_id.clone());
        return selection::rejected_probe(snapshot, action_id.clone(), chain, diagnostic, Vec::new());
    };
    if !snapshot.registry.is_active(action) {
        let diagnostic = super::model::RoutingDiagnostic::new(
            super::model::RoutingDiagnosticCode::Disabled,
            "action.enablement",
            snapshot.registry.inactive_reason(action).unwrap_or_else(|| "action is disabled".to_string()),
        )
        .with_action(action_id.clone());
        return selection::rejected_probe(snapshot, action_id.clone(), chain, diagnostic, Vec::new());
    }
    search::resolve(snapshot, action_id, chain)
}

/// Revalidates a generation-bound invocation before shell-owned execution.
#[must_use]
pub fn invoke_action(
    snapshot: &super::model::ActionRoutingSnapshot,
    invocation: &super::model::ActionInvocation,
) -> super::model::ActionInvocationReceipt {
    debug_assert!(snapshot.generation > 0);
    debug_assert!(snapshot.active_chain.len() <= snapshot.contexts.len());
    if invocation.observed_generation != snapshot.generation {
        let chain = invocation.active_chain.clone().unwrap_or_else(|| snapshot.active_chain.clone());
        let diagnostic = super::model::RoutingDiagnostic::new(
            super::model::RoutingDiagnosticCode::StaleRouteGeneration,
            "invocation.observed_generation",
            format!(
                "surface observed generation {}, current generation is {}",
                invocation.observed_generation, snapshot.generation
            ),
        )
        .with_action(invocation.action_id.clone());
        return super::model::ActionInvocationReceipt {
            surface: invocation.surface,
            renderer: invocation.renderer,
            observed_generation: invocation.observed_generation,
            current_generation: snapshot.generation,
            probe: selection::rejected_probe(snapshot, invocation.action_id.clone(), chain, diagnostic, Vec::new()),
        };
    }
    super::model::ActionInvocationReceipt {
        surface: invocation.surface,
        renderer: invocation.renderer,
        observed_generation: invocation.observed_generation,
        current_generation: snapshot.generation,
        probe: probe_action(snapshot, &invocation.action_id, invocation.active_chain.as_deref()),
    }
}
