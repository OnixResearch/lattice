use super::super::*;

const LOW_Z: i32 = 1;
const HIGH_Z: i32 = 10;
const POINT_X: i32 = 4;
const POINT_Y: i32 = 4;
const OUTSIDE_X: i32 = 20;
const OUTSIDE_Y: i32 = 20;
const ORIGIN: i32 = 0;
const REGION_SIZE: i32 = 8;
const WHEEL_VERTICAL_DELTA: i32 = -3;
const TRIPLE_CLICK_COUNT: u8 = 3;

fn click_event() -> PointerEvent {
    PointerEvent::new(
        PointerPoint::new(POINT_X, POINT_Y),
        PointerButton::Primary,
        PointerPhase::Down,
        PointerSource::Mouse,
    )
}

fn event_with_phase(phase: PointerPhase) -> PointerEvent {
    PointerEvent::new(PointerPoint::new(POINT_X, POINT_Y), PointerButton::None, phase, PointerSource::Mouse)
}

fn region(id: &str, z_order: i32) -> HitRegion {
    HitRegion::new(id, PointerRect::new(ORIGIN, ORIGIN, REGION_SIZE, REGION_SIZE), z_order)
}

fn assert_routed_interaction(
    outcome: PointerRouteOutcome,
    expected_target: &str,
    expected_interaction: PointerInteractionClass,
) {
    let PointerRouteOutcome::Routed(route) = outcome else {
        panic!("expected routed pointer outcome");
    };
    assert_eq!(route.target_id, expected_target.to_string());
    assert_eq!(route.interaction, expected_interaction);
}

#[test]
fn route_selection_and_capture_lifecycle_are_deterministic() {
    let hit_map = HitMap::new(vec![
        region("background", LOW_Z).with_action("open.background"),
        region("toolbar", HIGH_Z).with_action("toolbar.open"),
    ]);
    let capture_map = HitMap::new(vec![region("splitter", HIGH_Z).with_action("splitter.drag")]);
    let update = PointerEvent::new(
        PointerPoint::new(OUTSIDE_X, OUTSIDE_Y),
        PointerButton::Primary,
        PointerPhase::DragUpdate,
        PointerSource::Mouse,
    )
    .with_captured_target("splitter");
    let release = PointerEvent::new(
        PointerPoint::new(OUTSIDE_X, OUTSIDE_Y),
        PointerButton::Primary,
        PointerPhase::DragCommit,
        PointerSource::Mouse,
    )
    .with_captured_target("splitter");

    let routed = route_pointer_event(&click_event(), &hit_map);
    let updated = route_pointer_event(&update, &capture_map);
    let released = route_pointer_event(&release, &capture_map);

    assert_eq!(
        routed,
        PointerRouteOutcome::Routed(PointerRoute {
            target_id: PointerTargetId::new("toolbar"),
            action_id: Some(PointerActionId::new("toolbar.open")),
            modal_capture: false,
            interaction: PointerInteractionClass::Activation,
            capture: PointerCaptureTransition::Start,
        })
    );
    assert_eq!(
        updated,
        PointerRouteOutcome::Routed(PointerRoute {
            target_id: PointerTargetId::new("splitter"),
            action_id: Some(PointerActionId::new("splitter.drag")),
            modal_capture: false,
            interaction: PointerInteractionClass::DragUpdate,
            capture: PointerCaptureTransition::Update,
        })
    );
    assert_eq!(
        released,
        PointerRouteOutcome::Routed(PointerRoute {
            target_id: PointerTargetId::new("splitter"),
            action_id: Some(PointerActionId::new("splitter.drag")),
            modal_capture: false,
            interaction: PointerInteractionClass::DragCommit,
            capture: PointerCaptureTransition::Release,
        })
    );
}

#[test]
fn invalid_targets_and_empty_maps_fail_closed() {
    let hidden_map = HitMap::new(vec![region("hidden", HIGH_Z).with_visible(false)]);
    let disabled_map = HitMap::new(vec![region("disabled", HIGH_Z).with_enabled(false)]);
    let stale_event = click_event().with_captured_target("stale");
    let cases = [
        (route_pointer_event(&click_event(), &hidden_map), PointerDiagnosticCode::HiddenTarget),
        (route_pointer_event(&click_event(), &disabled_map), PointerDiagnosticCode::DisabledTarget),
        (route_pointer_event(&stale_event, &HitMap::default()), PointerDiagnosticCode::StaleTarget),
        (route_pointer_event(&click_event(), &HitMap::default()), PointerDiagnosticCode::NoTarget),
    ];

    for (outcome, expected_code) in cases {
        assert!(matches!(
            outcome,
            PointerRouteOutcome::Diagnostic(PointerDiagnostic { code, .. }) if code == expected_code
        ));
    }
}

#[test]
fn route_action_classification_covers_mouse_intents() {
    let hit_map = HitMap::new(vec![region("target", HIGH_Z).with_action("target.action")]);
    let cases = [
        (click_event(), PointerInteractionClass::Activation),
        (click_event().with_click_count(PointerClickCount::Double), PointerInteractionClass::Activation),
        (
            PointerEvent::new(
                PointerPoint::new(POINT_X, POINT_Y),
                PointerButton::Secondary,
                PointerPhase::Down,
                PointerSource::Mouse,
            ),
            PointerInteractionClass::ContextMenu,
        ),
        (event_with_phase(PointerPhase::Move), PointerInteractionClass::Hover),
        (event_with_phase(PointerPhase::Enter), PointerInteractionClass::Hover),
        (event_with_phase(PointerPhase::Leave), PointerInteractionClass::Hover),
        (
            event_with_phase(PointerPhase::Wheel).with_wheel_delta(PointerWheelDelta::new(
                ORIGIN,
                WHEEL_VERTICAL_DELTA,
                PointerWheelUnit::Lines,
            )),
            PointerInteractionClass::WheelScroll,
        ),
        (event_with_phase(PointerPhase::DragStart), PointerInteractionClass::DragStart),
        (event_with_phase(PointerPhase::DragUpdate), PointerInteractionClass::DragUpdate),
        (event_with_phase(PointerPhase::DragCommit), PointerInteractionClass::DragCommit),
        (event_with_phase(PointerPhase::DragCancel), PointerInteractionClass::DragCancel),
        (
            event_with_phase(PointerPhase::Down).with_click_count(PointerClickCount::Multiple(TRIPLE_CLICK_COUNT)),
            PointerInteractionClass::Activation,
        ),
    ];

    for (event, expected_interaction) in cases {
        assert_routed_interaction(route_pointer_event(&event, &hit_map), "target", expected_interaction);
    }
}

#[test]
fn modal_pass_through_and_renderer_projection_are_stable() {
    let modal_map = HitMap::new(vec![
        region("panel", LOW_Z).with_action("panel.focus"),
        region("glass", HIGH_Z).with_capture(HitCapturePolicy::PassThrough),
        region("modal", HIGH_Z + LOW_Z)
            .with_capture(HitCapturePolicy::ModalCapture)
            .with_action("modal.dismiss"),
    ]);
    let pass_through_map = HitMap::new(vec![region("glass", HIGH_Z).with_capture(HitCapturePolicy::PassThrough)]);
    let event = click_event();

    let modal = route_pointer_event(&event, &modal_map);
    let pass_through = route_pointer_event(&event, &pass_through_map);
    let web = project_renderer_pointer_route(PointerRenderer::Meteorite, &event, &modal_map);
    let tui = project_renderer_pointer_route(PointerRenderer::Subwayrat, &event, &modal_map);
    let ratzilla = project_renderer_pointer_route(PointerRenderer::Ratzilla, &event, &modal_map);

    assert_routed_interaction(modal, "modal", PointerInteractionClass::ModalCapture);
    assert_routed_interaction(pass_through, "glass", PointerInteractionClass::PassThrough);
    assert_eq!(web.outcome, tui.outcome);
    assert_eq!(web.outcome, ratzilla.outcome);
}
