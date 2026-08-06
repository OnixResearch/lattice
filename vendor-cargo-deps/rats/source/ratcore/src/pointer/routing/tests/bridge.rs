use super::super::*;

const POINT_X: i32 = 4;
const POINT_Y: i32 = 4;
const ORIGIN: i32 = 0;
const REGION_SIZE: i32 = 8;
const HIGH_Z: i32 = 10;
const LOW_Z: i32 = 1;
const PRIMARY_BUTTON_NUMBER: i32 = 0;
const NO_BUTTON_NUMBER: i32 = -1;
const UNKNOWN_BUTTON_NUMBER: i32 = 9;
const WHEEL_DELTA: i32 = -2;

fn ctrl_shift_modifiers() -> PointerModifiers {
    PointerModifiers {
        ctrl: true,
        alt: false,
        shift: true,
        meta: false,
    }
}

fn point() -> PointerPoint {
    PointerPoint::new(POINT_X, POINT_Y)
}

fn rect() -> PointerRect {
    PointerRect::new(ORIGIN, ORIGIN, REGION_SIZE, REGION_SIZE)
}

#[test]
fn terminal_adapter_normalizes_supported_and_unsupported_events() {
    let drag = normalize_subwayrat_mouse_input(
        SubwayratMouseInput::new(point(), SubwayratMouseKind::Drag, SubwayratMouseButton::Left)
            .with_captured_target("splitter"),
    )
    .expect("terminal drag is supported");
    let scroll = normalize_subwayrat_mouse_input(SubwayratMouseInput::new(
        point(),
        SubwayratMouseKind::ScrollDown,
        SubwayratMouseButton::None,
    ))
    .expect("terminal wheel is supported");
    let unsupported = normalize_subwayrat_mouse_input(SubwayratMouseInput::new(
        point(),
        SubwayratMouseKind::Unsupported("sgr-pixel".to_string()),
        SubwayratMouseButton::Left,
    ));

    assert_eq!(drag.phase, PointerPhase::DragUpdate);
    assert_eq!(drag.captured_target, Some(PointerTargetId::new("splitter")));
    assert_eq!(scroll.phase, PointerPhase::Wheel);
    assert!(scroll.wheel_delta.is_some());
    assert!(matches!(
        unsupported,
        Err(PointerDiagnostic {
            code: PointerDiagnosticCode::UnsupportedKind,
            ..
        })
    ));
}

#[test]
fn ratzilla_adapter_keeps_movement_and_enter_exit_supported() {
    let moved =
        normalize_ratzilla_mouse_input(RatzillaPointerInput::new(point(), RatzillaMouseKind::Moved, NO_BUTTON_NUMBER))
            .expect("movement is supported");
    let entered = normalize_ratzilla_mouse_input(RatzillaPointerInput::new(
        point(),
        RatzillaMouseKind::Entered,
        NO_BUTTON_NUMBER,
    ))
    .expect("enter is supported");
    let double_click = normalize_ratzilla_mouse_input(RatzillaPointerInput::new(
        point(),
        RatzillaMouseKind::DoubleClick,
        PRIMARY_BUTTON_NUMBER,
    ))
    .expect("double click is supported");
    let unknown_button = normalize_ratzilla_mouse_input(RatzillaPointerInput::new(
        point(),
        RatzillaMouseKind::ButtonDown,
        UNKNOWN_BUTTON_NUMBER,
    ));

    assert_eq!(moved.phase, PointerPhase::Move);
    assert_eq!(entered.phase, PointerPhase::Enter);
    assert_eq!(double_click.click_count, PointerClickCount::Double);
    assert!(matches!(
        unknown_button,
        Err(PointerDiagnostic {
            code: PointerDiagnosticCode::UnsupportedKind,
            ..
        })
    ));
}

#[test]
fn ratzilla_adapter_preserves_modifier_metadata() {
    let modifiers = ctrl_shift_modifiers();
    let event = normalize_ratzilla_mouse_input(
        RatzillaPointerInput::new(point(), RatzillaMouseKind::SingleClick, PRIMARY_BUTTON_NUMBER)
            .with_modifiers(modifiers),
    )
    .expect("modified Ratzilla click is supported");

    assert_eq!(event.modifiers, modifiers);
    assert_eq!(event.source, PointerSource::Ratzilla);
}

#[test]
fn dom_adapter_normalizes_context_wheel_drag_and_unsupported_paths() {
    let context =
        normalize_dom_pointer_input(DomPointerInput::new(point(), DomPointerKind::ContextMenu, PointerButton::Primary))
            .expect("context menu is supported");
    let wheel = normalize_dom_pointer_input(
        DomPointerInput::new(point(), DomPointerKind::Wheel, PointerButton::None)
            .with_wheel_delta(PointerWheelDelta::new(ORIGIN, WHEEL_DELTA, PointerWheelUnit::Pixels)),
    )
    .expect("wheel is supported");
    let drag =
        normalize_dom_pointer_input(DomPointerInput::new(point(), DomPointerKind::DragEnd, PointerButton::Primary))
            .expect("drag end is supported");
    let unsupported = normalize_dom_pointer_input(DomPointerInput::new(
        point(),
        DomPointerKind::Unsupported("browser-gesture".to_string()),
        PointerButton::None,
    ));

    assert_eq!(context.button, PointerButton::Secondary);
    assert_eq!(wheel.phase, PointerPhase::Wheel);
    assert_eq!(drag.phase, PointerPhase::DragCommit);
    assert!(matches!(
        unsupported,
        Err(PointerDiagnostic {
            code: PointerDiagnosticCode::UnsupportedKind,
            ..
        })
    ));
}

#[test]
fn rendered_regions_derive_hit_maps_without_geometry_drift() {
    let hit_map = derive_pointer_hit_map(vec![
        RenderedPointerRegion::new("background", rect(), LOW_Z).with_action("background.focus"),
        RenderedPointerRegion::new("modal", rect(), HIGH_Z)
            .with_capture(HitCapturePolicy::ModalCapture)
            .with_action("modal.dismiss"),
    ]);
    let disabled_map = derive_pointer_hit_map(vec![
        RenderedPointerRegion::new("disabled", rect(), HIGH_Z)
            .with_enabled(false)
            .with_diagnostic("disabled by renderer"),
    ]);
    let disabled = route_pointer_event(
        &PointerEvent::new(point(), PointerButton::Primary, PointerPhase::Down, PointerSource::Mouse),
        &disabled_map,
    );

    assert_eq!(hit_map.regions.len(), 2);
    assert!(matches!(
        disabled,
        PointerRouteOutcome::Diagnostic(PointerDiagnostic {
            code: PointerDiagnosticCode::DisabledTarget,
            ..
        })
    ));
}
