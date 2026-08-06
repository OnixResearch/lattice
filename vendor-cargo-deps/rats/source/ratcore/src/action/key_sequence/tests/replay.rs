use super::super::super::ActionContext;
use super::super::super::ActionContextKind;
use super::super::super::ActionHandlerRegistration;
use super::super::super::ActionPropagation;
use super::super::super::ActionRendererAdapter;
use super::super::super::ActionRoutingCandidate;
use super::super::super::RoutingDiagnosticCode;
use super::super::super::accept_action_snapshot;
use super::super::BindingResolutionOutcome;
use super::super::KeyModifier;
use super::super::KeySequenceInput;
use super::super::KeySequenceReducerOutcome;
use super::super::KeySequenceReplayStep;
use super::super::dispatch_resolved_binding;
use super::super::replay_key_sequences;
use super::super::resolve_key_sequence;
use super::ACTION_OPEN;
use super::ACTION_SAVE;
use super::LAYER_APP;
use super::accepted_catalog;
use super::active;
use super::binding;
use super::registry;
use super::sequence;
use super::stroke;

fn routing_snapshot() -> Result<super::super::super::ActionRoutingSnapshot, String> {
    let candidate = ActionRoutingCandidate::new(
        1,
        "context.application",
        vec![ActionContext::root(
            "context.application",
            ActionContextKind::Application,
            1,
        )],
        vec![
            ActionHandlerRegistration::new("handler.save", ACTION_SAVE, "context.application", 1)
                .with_propagation(ActionPropagation::RootOnly),
        ],
    );
    let build = accept_action_snapshot(candidate, &registry(), None, None);
    build.snapshot.ok_or_else(|| format!("routing diagnostics: {:?}", build.diagnostics))
}

#[test]
fn replay_uses_production_prefix_reducer_and_contextual_router() -> Result<(), String> {
    let catalog = accepted_catalog(1, vec![binding(
        "binding.save.sequence",
        ACTION_SAVE,
        sequence(vec![
            stroke('x', vec![KeyModifier::Control])?,
            stroke('s', vec![KeyModifier::Control])?,
        ])?,
        LAYER_APP,
    )])?;
    let active = active(&catalog, 1, &[LAYER_APP])?;
    let replay = replay_key_sequences(&catalog, &registry(), &active, &routing_snapshot()?, &[
        KeySequenceReplayStep::new(
            KeySequenceInput::Stroke {
                stroke: stroke('x', vec![KeyModifier::Control])?,
                observed_catalog_generation: 1,
                observed_context_generation: 1,
            },
            ActionRendererAdapter::Subwayrat,
        ),
        KeySequenceReplayStep::new(
            KeySequenceInput::Stroke {
                stroke: stroke('s', vec![KeyModifier::Control])?,
                observed_catalog_generation: 1,
                observed_context_generation: 1,
            },
            ActionRendererAdapter::Meteorite,
        ),
    ]);

    assert_eq!(replay.receipts.len(), 2);
    assert!(matches!(replay.receipts[0].transition.outcome, KeySequenceReducerOutcome::Pending(_)));
    assert!(matches!(replay.receipts[1].transition.outcome, KeySequenceReducerOutcome::Resolved(_)));
    assert!(replay.receipts[1].dispatch.as_ref().is_some_and(|dispatch| {
        dispatch
            .invocation
            .as_ref()
            .and_then(|receipt| receipt.probe.outcome.route())
            .is_some_and(|route| route.registration_id == "handler.save")
    }));
    Ok(())
}

#[test]
fn bound_but_unhandled_differs_from_unbound_and_stale_dispatch() -> Result<(), String> {
    let bound_sequence = sequence(vec![stroke('o', vec![KeyModifier::Control])?])?;
    let unbound_sequence = sequence(vec![stroke('z', Vec::new())?])?;
    let catalog = accepted_catalog(1, vec![binding("binding.open", ACTION_OPEN, bound_sequence.clone(), LAYER_APP)])?;
    let active_layers = active(&catalog, 1, &[LAYER_APP])?;
    let bound = resolve_key_sequence(&catalog, &registry(), &active_layers, &bound_sequence);
    let unbound = resolve_key_sequence(&catalog, &registry(), &active_layers, &unbound_sequence);
    let resolved = bound.outcome.resolved().ok_or_else(|| "bound sequence did not resolve".to_string())?;
    let dispatch = dispatch_resolved_binding(
        &routing_snapshot()?,
        &catalog,
        &active_layers,
        resolved,
        ActionRendererAdapter::Meteorite,
    );
    let stale_catalog = accepted_catalog(2, vec![binding("binding.open", ACTION_OPEN, bound_sequence, LAYER_APP)])?;
    let stale_active = active(&stale_catalog, 1, &[LAYER_APP])?;
    let stale = dispatch_resolved_binding(
        &routing_snapshot()?,
        &stale_catalog,
        &stale_active,
        resolved,
        ActionRendererAdapter::Subwayrat,
    );

    assert!(matches!(unbound.outcome, BindingResolutionOutcome::Unbound));
    assert_eq!(
        dispatch
            .invocation
            .as_ref()
            .and_then(|receipt| receipt.probe.outcome.diagnostic())
            .map(|diagnostic| diagnostic.code),
        Some(RoutingDiagnosticCode::Unhandled)
    );
    assert!(stale.invocation.is_none());
    assert!(stale.diagnostic.is_some());
    Ok(())
}
