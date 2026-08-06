use super::*;

#[test]
fn workspace_pilot_routes_representative_actions_to_typed_messages() {
    let model = sample_model();
    let pilot = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(1, WorkspaceContextualContentKind::Editor),
        None,
    );
    let snapshot = pilot.routing.snapshot.as_ref().expect("workspace pilot is accepted");

    let cases = [
        (WorkspaceContextualAction::ClosePane, "workspace-message.close-pane"),
        (WorkspaceContextualAction::Find, "workspace-message.find"),
        (WorkspaceContextualAction::SelectAll, "workspace-message.select-all"),
        (WorkspaceContextualAction::Refresh, "workspace-message.refresh"),
        (WorkspaceContextualAction::OpenCommandPalette, "workspace-message.open-command-palette"),
    ];
    let identities = cases
        .iter()
        .map(|(action, expected)| {
            let dispatch = dispatch_workspace_contextual_action(
                &model,
                snapshot,
                &crate::action_registry::ActionInvocation::new(
                    action.action_id(),
                    1,
                    crate::action_registry::ActionActivationSurface::Keybinding,
                    crate::action_registry::ActionRendererAdapter::Subwayrat,
                ),
            );
            (dispatch.message.map(|message| message.identity()), *expected)
        })
        .collect::<Vec<_>>();

    assert!(pilot.routing.diagnostics.is_empty());
    assert_eq!(identities.len(), 5);
    assert!(identities.iter().all(|(actual, expected)| actual == &Some(*expected)));
    assert!(pilot.surfaces.as_ref().is_some_and(|surfaces| {
        surfaces
            .routes
            .iter()
            .filter(|route| route.action_id != WorkspaceContextualAction::DismissModal.action_id())
            .all(|route| route.enabled)
    }));
}

#[test]
fn focused_content_overrides_and_disabled_content_falls_back() {
    let model = sample_model();
    let local_pilot = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(1, WorkspaceContextualContentKind::Tree),
        None,
    );
    let local_snapshot = local_pilot.routing.snapshot.as_ref().expect("local pilot");
    let local = dispatch_workspace_contextual_action(
        &model,
        local_snapshot,
        &contextual_invocation(WorkspaceContextualAction::Find, 1),
    );

    let disabled_pilot = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(2, WorkspaceContextualContentKind::Tree).with_content_enabled(false),
        Some(local_snapshot),
    );
    let disabled_snapshot = disabled_pilot.routing.snapshot.as_ref().expect("disabled content remains valid");
    let fallback = dispatch_workspace_contextual_action(
        &model,
        disabled_snapshot,
        &contextual_invocation(WorkspaceContextualAction::Find, 2),
    );

    assert!(matches!(
        local.message,
        Some(WorkspaceContextualActionMessage::Find {
            target: WorkspaceContextualActionTarget::Content(_)
        })
    ));
    assert!(matches!(
        fallback.message,
        Some(WorkspaceContextualActionMessage::Find {
            target: WorkspaceContextualActionTarget::Application
        })
    ));
}

#[test]
fn disabled_close_and_inapplicable_select_all_stay_disabled_on_surfaces() {
    let model = sample_model();
    let pilot = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(1, WorkspaceContextualContentKind::Editor)
            .with_close_enabled(false)
            .with_selection_available(false),
        None,
    );
    let snapshot = pilot.routing.snapshot.as_ref().expect("disabled pilot");
    let close = dispatch_workspace_contextual_action(
        &model,
        snapshot,
        &contextual_invocation(WorkspaceContextualAction::ClosePane, 1),
    );
    let select = dispatch_workspace_contextual_action(
        &model,
        snapshot,
        &contextual_invocation(WorkspaceContextualAction::SelectAll, 1),
    );
    let surfaces = pilot.surfaces.as_ref().expect("contextual surfaces");

    assert!(close.message.is_none());
    assert!(select.message.is_none());
    assert_eq!(
        close.receipt.probe.outcome.diagnostic().map(|item| item.code),
        Some(crate::action_registry::ContextualActionDiagnosticCode::Disabled)
    );
    assert_eq!(
        select.receipt.probe.outcome.diagnostic().map(|item| item.code),
        Some(crate::action_registry::ContextualActionDiagnosticCode::Unsupported)
    );
    assert!(
        surfaces
            .menu
            .iter()
            .any(|item| { item.action_id == WorkspaceContextualAction::ClosePane.action_id() && !item.enabled })
    );
    assert!(
        surfaces
            .palette
            .iter()
            .any(|item| { item.action_id == WorkspaceContextualAction::SelectAll.action_id() && !item.enabled })
    );
}

#[test]
fn modal_blocks_outer_routes_allows_palette_escape_and_routes_dismiss() {
    let model = sample_model();
    let pilot = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(1, WorkspaceContextualContentKind::Editor)
            .with_modal(WorkspaceContextualModalState::new("rename")),
        None,
    );
    let snapshot = pilot.routing.snapshot.as_ref().expect("modal pilot");
    let find = dispatch_workspace_contextual_action(
        &model,
        snapshot,
        &contextual_invocation(WorkspaceContextualAction::Find, 1),
    );
    let palette = dispatch_workspace_contextual_action(
        &model,
        snapshot,
        &contextual_invocation(WorkspaceContextualAction::OpenCommandPalette, 1),
    );
    let dismiss = dispatch_workspace_contextual_action(
        &model,
        snapshot,
        &contextual_invocation(WorkspaceContextualAction::DismissModal, 1),
    );

    assert!(find.message.is_none());
    assert_eq!(
        find.receipt.probe.outcome.diagnostic().map(|item| item.code),
        Some(crate::action_registry::ContextualActionDiagnosticCode::Blocked)
    );
    assert!(matches!(
        palette.message,
        Some(WorkspaceContextualActionMessage::OpenCommandPalette {
            target: WorkspaceContextualActionTarget::Application
        })
    ));
    assert!(matches!(
        dismiss.message,
        Some(WorkspaceContextualActionMessage::DismissModal {
            target: WorkspaceContextualActionTarget::Modal(_)
        })
    ));
}

#[test]
fn stale_workspace_surface_cannot_dispatch_after_generation_change() {
    let model = sample_model();
    let first = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(1, WorkspaceContextualContentKind::Editor),
        None,
    );
    let current = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(2, WorkspaceContextualContentKind::Tree),
        first.routing.snapshot.as_ref(),
    );
    let dispatch = dispatch_workspace_contextual_action(
        &model,
        current.routing.snapshot.as_ref().expect("current snapshot"),
        &contextual_invocation(WorkspaceContextualAction::Find, 1),
    );

    assert!(dispatch.message.is_none());
    assert_eq!(
        dispatch.receipt.probe.outcome.diagnostic().map(|item| item.code),
        Some(crate::action_registry::ContextualActionDiagnosticCode::StaleRouteGeneration)
    );
}

#[test]
fn replay_keeps_renderer_surfaces_on_one_route_and_runtime_message() {
    let model = sample_model();
    let pilot = build_workspace_contextual_action_pilot(
        &model,
        &WorkspaceContextualRoutingOptions::new(1, WorkspaceContextualContentKind::Editor),
        None,
    );
    let snapshot = pilot.routing.snapshot.as_ref().expect("replay snapshot");
    let steps = [
        crate::action_registry::ActionRouteReplayStep::new(
            WorkspaceContextualAction::Refresh.action_id(),
            crate::action_registry::ActionActivationSurface::Menu,
            crate::action_registry::ActionRendererAdapter::Subwayrat,
            1,
        ),
        crate::action_registry::ActionRouteReplayStep::new(
            WorkspaceContextualAction::Refresh.action_id(),
            crate::action_registry::ActionActivationSurface::Pointer,
            crate::action_registry::ActionRendererAdapter::Meteorite,
            1,
        ),
    ];

    let replay = replay_workspace_contextual_actions(&model, snapshot, &steps);

    assert_eq!(replay.len(), 2);
    assert!(replay.iter().all(|dispatch| {
        dispatch.message.as_ref().is_some_and(|message| message.identity() == "workspace-message.refresh")
    }));
    assert_eq!(
        replay[0].receipt.probe.outcome.route().map(|route| route.registration_id.clone()),
        replay[1].receipt.probe.outcome.route().map(|route| route.registration_id.clone())
    );
}
