pub(super) fn select_eligible(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    action_id: &crate::nexus_core::ActionId,
    chain: Vec<super::super::ActionContextId>,
    mut considerations: Vec<super::super::model::ActionRouteConsideration>,
    eligible: Vec<&super::super::model::ActionHandlerRegistration>,
) -> super::super::model::ActionRouteProbe {
    debug_assert!(!eligible.is_empty());
    debug_assert!(!chain.is_empty());
    let winning_priority = eligible[0].priority;
    let winners = eligible
        .iter()
        .copied()
        .take_while(|registration| registration.priority == winning_priority)
        .collect::<Vec<_>>();
    if winners.len() > 1 {
        let mut diagnostic = super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::Ambiguous,
            "resolution.priority",
            "equal-priority registrations at the nearest eligible context are ambiguous",
        )
        .with_action(action_id.clone())
        .with_context(winners[0].owner_context.clone());
        for registration in winners {
            diagnostic = diagnostic.with_registration(registration.id.clone());
        }
        return rejected_probe(snapshot, action_id.clone(), chain, diagnostic, considerations);
    }
    let winner = eligible[0];
    for consideration in &mut considerations {
        if consideration.decision == super::super::model::ActionRouteDecision::Eligible
            && consideration.registration_id != winner.id
        {
            consideration.decision = super::super::model::ActionRouteDecision::Shadowed;
        }
    }
    super::super::model::ActionRouteProbe {
        action_id: action_id.clone(),
        routing_generation: snapshot.generation,
        outcome: super::super::model::ActionRouteOutcome::Accepted(super::super::model::ActionRoute {
            action_id: action_id.clone(),
            registration_id: winner.id.clone(),
            owner_context: winner.owner_context.clone(),
            routing_generation: snapshot.generation,
            owner_generation: winner.owner_generation,
            priority: winner.priority,
            propagation: winner.propagation,
        }),
        provenance: super::super::model::ActionRouteProvenance {
            active_chain: chain,
            considerations,
        },
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RejectionEvidence {
    pub(super) has_disabled: bool,
    pub(super) has_modal_block: bool,
    pub(super) has_predicate_mismatch: bool,
    pub(super) has_registration: bool,
}

pub(super) fn rejection_diagnostic(
    action_id: &crate::nexus_core::ActionId,
    context_id: &super::super::ActionContextId,
    registration_ids: &[super::super::ActionHandlerRegistrationId],
    evidence: RejectionEvidence,
) -> super::super::model::RoutingDiagnostic {
    debug_assert!(!action_id.as_str().is_empty());
    debug_assert!(evidence.has_registration || registration_ids.is_empty());
    let (code, message) = if evidence.has_modal_block {
        (
            super::super::model::RoutingDiagnosticCode::Blocked,
            "active modal boundary blocks the contextual route",
        )
    } else if evidence.has_disabled {
        (
            super::super::model::RoutingDiagnosticCode::Disabled,
            "contextual action owner or registration is disabled",
        )
    } else if evidence.has_predicate_mismatch || evidence.has_registration {
        (
            super::super::model::RoutingDiagnosticCode::Unsupported,
            "no contextual handler supports the current facts and chain",
        )
    } else {
        (
            super::super::model::RoutingDiagnosticCode::Unhandled,
            "no contextual handler is registered on the active chain",
        )
    };
    let mut diagnostic = super::super::model::RoutingDiagnostic::new(code, "resolution", message)
        .with_action(action_id.clone())
        .with_context(context_id.clone());
    for registration_id in registration_ids {
        diagnostic = diagnostic.with_registration(registration_id.clone());
    }
    diagnostic
}

pub(super) fn rejected_probe(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    action_id: crate::nexus_core::ActionId,
    chain: Vec<super::super::ActionContextId>,
    diagnostic: super::super::model::RoutingDiagnostic,
    considerations: Vec<super::super::model::ActionRouteConsideration>,
) -> super::super::model::ActionRouteProbe {
    super::super::model::ActionRouteProbe {
        action_id,
        routing_generation: snapshot.generation,
        outcome: super::super::model::ActionRouteOutcome::Rejected(diagnostic),
        provenance: super::super::model::ActionRouteProvenance {
            active_chain: chain,
            considerations,
        },
    }
}
