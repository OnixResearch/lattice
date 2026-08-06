use super::super::*;

const POINT_X: i32 = 4;
const POINT_Y: i32 = 4;
const ORIGIN: i32 = 0;
const WHEEL_VERTICAL_DELTA: i32 = -3;

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

#[test]
fn full_event_taxonomy_normalizes_supported_mouse_variants() {
    let modifiers = PointerModifiers {
        ctrl: true,
        alt: false,
        shift: true,
        meta: false,
    };
    let wheel = event_with_phase(PointerPhase::Wheel)
        .with_coordinate_space(PointerCoordinateSpace::Document)
        .with_wheel_delta(PointerWheelDelta::new(ORIGIN, WHEEL_VERTICAL_DELTA, PointerWheelUnit::Pixels))
        .with_modifiers(modifiers);
    let double_click = click_event()
        .with_coordinate_space(PointerCoordinateSpace::WidgetLocal)
        .with_click_count(PointerClickCount::Double);
    let drag = PointerEvent::new(
        PointerPoint::new(POINT_X, POINT_Y),
        PointerButton::Primary,
        PointerPhase::DragUpdate,
        PointerSource::Terminal,
    )
    .with_buttons(PointerButtonSet::from_button(&PointerButton::Primary));

    let normalized_wheel = normalize_pointer_event(wheel).expect("wheel event is supported");
    let normalized_double = normalize_pointer_event(double_click).expect("double click is supported");
    let normalized_drag = normalize_pointer_event(drag).expect("drag event is supported");

    assert_eq!(normalized_wheel.coordinate_space, PointerCoordinateSpace::Document);
    assert_eq!(normalized_wheel.wheel_delta.expect("wheel delta").unit, PointerWheelUnit::Pixels);
    assert_eq!(normalized_wheel.modifiers, modifiers);
    assert_eq!(normalized_double.click_count, PointerClickCount::Double);
    assert!(normalized_drag.buttons.primary);
}

#[test]
fn unsupported_pointer_details_fail_closed() {
    let unsupported_events = [
        click_event().with_click_count(PointerClickCount::Unsupported("negative".to_string())),
        event_with_phase(PointerPhase::Unsupported("native-hover".to_string())),
        PointerEvent::new(
            PointerPoint::new(POINT_X, POINT_Y),
            PointerButton::Unsupported("tilt-wheel".to_string()),
            PointerPhase::Down,
            PointerSource::Mouse,
        ),
        PointerEvent::new(
            PointerPoint::new(POINT_X, POINT_Y),
            PointerButton::None,
            PointerPhase::Move,
            PointerSource::Unsupported("backend-pointer".to_string()),
        ),
        event_with_phase(PointerPhase::Move)
            .with_coordinate_space(PointerCoordinateSpace::Unsupported("polar".to_string())),
        event_with_phase(PointerPhase::Wheel).with_wheel_delta(PointerWheelDelta::new(
            ORIGIN,
            WHEEL_VERTICAL_DELTA,
            PointerWheelUnit::Unsupported("degrees".to_string()),
        )),
        event_with_phase(PointerPhase::Move).with_buttons(PointerButtonSet::none().with_unsupported("extra-button")),
    ];

    for event in unsupported_events {
        let outcome = route_pointer_event(&event, &HitMap::default());

        assert!(matches!(
            outcome,
            PointerRouteOutcome::Diagnostic(PointerDiagnostic {
                code: PointerDiagnosticCode::UnsupportedKind,
                ..
            })
        ));
    }
}
