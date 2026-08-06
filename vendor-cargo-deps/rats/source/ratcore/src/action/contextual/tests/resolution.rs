use super::*;

#[test]
fn nearest_focused_owner_overrides_application_fallback() {
    let snapshot = accepted_snapshot();

    let probe = probe_action(&snapshot, &action_id(ACTION_FIND), None);
    let route = probe.outcome.route().expect("find is routed");

    assert_eq!(route.registration_id, HANDLER_EDITOR_FIND);
    assert_eq!(route.owner_context, CONTEXT_EDITOR);
    assert_eq!(route.owner_generation, 1);
    assert_eq!(probe.provenance.considerations.len(), 1);
}

#[test]
fn bubble_falls_back_but_stop_rejects_ineligible_local_handler() {
    let mut fallback_candidate = candidate(1);
    fallback_candidate.registrations[1].predicate = ActionContextPredicate::FactEquals {
        fact_id: ActionContextFactId::new(FACT_KIND),
        value: ActionContextFactValue::Text("tree".to_string()),
    };
    let fallback_snapshot = accept_action_snapshot(fallback_candidate.clone(), &registry(), None, None)
        .snapshot
        .expect("bubble candidate is valid");
    let fallback = probe_action(&fallback_snapshot, &action_id(ACTION_FIND), None);

    fallback_candidate.registrations[1].propagation = ActionPropagation::Stop;
    let stop_snapshot = accept_action_snapshot(fallback_candidate, &registry(), None, None)
        .snapshot
        .expect("stop candidate is valid");
    let stopped = probe_action(&stop_snapshot, &action_id(ACTION_FIND), None);

    assert_eq!(fallback.outcome.route().map(|route| route.registration_id.as_str()), Some(HANDLER_APP_FIND));
    assert_eq!(diagnostic_code(&stopped), Some(RoutingDiagnosticCode::Unsupported));
}

#[test]
fn modal_boundary_blocks_outer_routes_and_explicit_escape_survives() {
    let mut modal_candidate = candidate(1);
    modal_candidate.contexts.push(
        ActionContext::child("context.modal", CONTEXT_EDITOR, ActionContextKind::Modal, 1)
            .with_modal(ActionContextModalBoundary::blocking()),
    );
    modal_candidate.focused_context = ActionContextId::new("context.modal");
    let snapshot = accept_action_snapshot(modal_candidate, &registry(), None, None)
        .snapshot
        .expect("modal candidate is valid");

    let blocked = probe_action(&snapshot, &action_id(ACTION_FIND), None);
    let escape = probe_action(&snapshot, &action_id(ACTION_PALETTE), None);

    assert_eq!(diagnostic_code(&blocked), Some(RoutingDiagnosticCode::Blocked));
    assert_eq!(
        escape.outcome.route().map(|route| route.registration_id.as_str()),
        Some("handler.application.palette")
    );
}

#[test]
fn disabled_owner_unhandled_action_and_invalid_chain_fail_closed() {
    let mut disabled_candidate = candidate(1);
    disabled_candidate.contexts[3] = disabled_candidate.contexts[3].clone().disabled("editor is read only");
    let snapshot = accept_action_snapshot(disabled_candidate, &registry(), None, None)
        .snapshot
        .expect("disabled owner is a valid visible state");

    let disabled = probe_action(&snapshot, &action_id(ACTION_SAVE), None);
    let unhandled = probe_action(&snapshot, &action_id(ACTION_UNHANDLED), None);
    let invalid = probe_action(
        &snapshot,
        &action_id(ACTION_FIND),
        Some(&[ActionContextId::new(CONTEXT_EDITOR), ActionContextId::new(CONTEXT_APP)]),
    );

    assert_eq!(diagnostic_code(&disabled), Some(RoutingDiagnosticCode::Disabled));
    assert_eq!(diagnostic_code(&unhandled), Some(RoutingDiagnosticCode::Unhandled));
    assert_eq!(diagnostic_code(&invalid), Some(RoutingDiagnosticCode::InvalidActiveChain));
}

#[test]
fn equal_priority_conflict_is_ambiguous_with_stable_ids() {
    let mut ambiguous_candidate = candidate(1);
    ambiguous_candidate.registrations.push(
        ActionHandlerRegistration::new("handler.editor.find.alternate", ACTION_FIND, CONTEXT_EDITOR, 1)
            .with_priority(10),
    );
    let snapshot = accept_action_snapshot(ambiguous_candidate, &registry(), None, None)
        .snapshot
        .expect("equal priority is a resolution-time ambiguity");

    let probe = probe_action(&snapshot, &action_id(ACTION_FIND), None);
    let diagnostic = probe.outcome.diagnostic().expect("ambiguous route is rejected");

    assert_eq!(diagnostic.code, RoutingDiagnosticCode::Ambiguous);
    assert_eq!(
        diagnostic.registration_ids.iter().map(ActionHandlerRegistrationId::as_str).collect::<Vec<_>>(),
        vec!["handler.editor.find", "handler.editor.find.alternate"]
    );
}

#[test]
fn route_probe_and_every_activation_surface_choose_one_target() {
    let snapshot = accepted_snapshot();
    let surfaces = [
        ActionActivationSurface::Keybinding,
        ActionActivationSurface::Menu,
        ActionActivationSurface::CommandPalette,
        ActionActivationSurface::Toolbar,
        ActionActivationSurface::Pointer,
        ActionActivationSurface::Semantic,
    ];
    let renderers = [
        ActionRendererAdapter::Subwayrat,
        ActionRendererAdapter::Meteorite,
        ActionRendererAdapter::IcedPilot,
    ];

    let targets = surfaces
        .iter()
        .flat_map(|surface| {
            renderers.iter().map(|renderer| {
                let receipt = invoke_action(&snapshot, &ActionInvocation::new(ACTION_FIND, 1, *surface, *renderer));
                receipt.probe.outcome.route().map(|route| route.registration_id.clone())
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(targets.len(), 18);
    assert!(targets.iter().all(|target| { target.as_ref().is_some_and(|id| id == HANDLER_EDITOR_FIND) }));
}

#[test]
fn stale_invocation_rejects_focus_or_catalog_generation_change() {
    let previous = accepted_snapshot();
    let current = accept_action_snapshot(candidate(2), &registry(), None, Some(&previous))
        .snapshot
        .expect("new generation is accepted");

    let receipt = invoke_action(
        &current,
        &ActionInvocation::new(ACTION_FIND, 1, ActionActivationSurface::Menu, ActionRendererAdapter::Meteorite),
    );

    assert_eq!(receipt.observed_generation, 1);
    assert_eq!(receipt.current_generation, 2);
    assert_eq!(diagnostic_code(&receipt.probe), Some(RoutingDiagnosticCode::StaleRouteGeneration));
    assert!(receipt.probe.outcome.route().is_none());
}

#[test]
fn projections_and_replay_delegate_to_production_router() {
    let snapshot = accepted_snapshot();
    let surfaces = build_command_views(&snapshot);
    let replay = replay_action_routes(&snapshot, &[
        ActionRouteReplayStep::new(
            ACTION_FIND,
            ActionActivationSurface::Keybinding,
            ActionRendererAdapter::Subwayrat,
            1,
        ),
        ActionRouteReplayStep::new(ACTION_FIND, ActionActivationSurface::Pointer, ActionRendererAdapter::Meteorite, 1),
        ActionRouteReplayStep::new(
            ACTION_UNHANDLED,
            ActionActivationSurface::Semantic,
            ActionRendererAdapter::Meteorite,
            1,
        ),
    ]);

    assert!(surfaces.keymap.iter().any(|item| item.action_id == ACTION_FIND));
    assert!(surfaces.menu.iter().any(|item| { item.action_id == ACTION_UNHANDLED && !item.enabled }));
    assert!(surfaces.palette.iter().any(|item| { item.action_id == ACTION_FIND && item.enabled }));
    assert!(surfaces.toolbar.iter().any(|item| { item.action_id == ACTION_FIND && item.enabled }));
    assert_eq!(replay.receipts.len(), 3);
    assert_eq!(
        replay.receipts[0].invocation.probe.outcome.route().map(|route| route.registration_id.as_str()),
        Some(HANDLER_EDITOR_FIND)
    );
    assert_eq!(diagnostic_code(&replay.receipts[2].invocation.probe), Some(RoutingDiagnosticCode::Unhandled));
}
