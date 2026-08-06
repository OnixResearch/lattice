pub(super) fn validate_input(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    action_id: &crate::nexus_core::ActionId,
    chain: &[super::super::ActionContextId],
) -> Option<super::super::model::RoutingDiagnostic> {
    debug_assert!(snapshot.generation > 0);
    debug_assert!(chain.len() <= snapshot.contexts.len().saturating_add(1));
    if snapshot.registry.action(action_id).is_none() {
        return Some(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::UnknownAction,
                "request.action_id",
                format!("unknown contextual action `{action_id}`"),
            )
            .with_action(action_id.clone()),
        );
    }
    if chain.is_empty() {
        return Some(super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
            "request.active_chain",
            "active context chain must not be empty",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (distance, context_id) in chain.iter().enumerate() {
        let Some(context) = snapshot.contexts.get(context_id) else {
            return Some(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
                    "request.active_chain",
                    "active chain references an unknown context",
                )
                .with_context(context_id.clone()),
            );
        };
        if !seen.insert(context_id) || !context.active || !context.visible {
            return Some(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
                    "request.active_chain",
                    "active chain contains a duplicate, inactive, or hidden context",
                )
                .with_context(context_id.clone()),
            );
        }
        let expected_parent = chain.get(distance.saturating_add(1));
        if context.parent.as_ref() != expected_parent {
            return Some(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
                    "request.active_chain",
                    "active chain does not follow context parent links to the root",
                )
                .with_context(context_id.clone()),
            );
        }
    }
    None
}

pub(super) fn modal_blocks(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    chain: &[super::super::ActionContextId],
    modal_distance: Option<usize>,
    candidate_distance: usize,
    registration: &super::super::model::ActionHandlerRegistration,
) -> bool {
    let Some(modal_distance) = modal_distance else {
        return false;
    };
    if candidate_distance <= modal_distance {
        return false;
    }
    let can_boundary_allow_application_escape = chain
        .get(modal_distance)
        .and_then(|context_id| snapshot.contexts.get(context_id))
        .and_then(|context| context.modal.as_ref())
        .is_some_and(|modal| modal.allows_application_escape)
        && snapshot.contexts.get(&registration.owner_context).is_some_and(|owner| owner.parent.is_none());
    !registration.allows_modal_escape && !can_boundary_allow_application_escape
}
