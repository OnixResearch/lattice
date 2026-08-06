use super::*;

const GRID_WIDTH: i32 = 80;
const GRID_HEIGHT: i32 = 24;
const INSIDE_X: i32 = 10;
const INSIDE_Y: i32 = 5;
const OUTSIDE_X: i32 = 81;
const PRIMARY_BUTTON: i32 = 0;
const SECONDARY_BUTTON: i32 = 1;
const UNKNOWN_BUTTON: i32 = 7;
const NO_BUTTON: i32 = -1;

fn ctrl_alt_shift_modifiers() -> crate::pointer_input_routing::PointerModifiers {
    crate::pointer_input_routing::PointerModifiers {
        ctrl: true,
        alt: true,
        shift: true,
        meta: false,
    }
}

#[test]
fn key_and_mouse_adapters_map_positive_inputs() {
    let key = adapt_ratzilla_key_input(&RatzillaKeyInput::new(KEY_ENTER, Vec::new()));
    let mouse = adapt_ratzilla_mouse_input(
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, PRIMARY_BUTTON),
        WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT),
    );
    let movement = adapt_ratzilla_mouse_input(
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, NO_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::Moved),
        WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT),
    );

    assert_eq!(key.action_id, Some(ACTION_ACTIVATE.to_string()));
    assert_eq!(mouse.action_id, Some(ACTION_POINTER_ROUTE.to_string()));
    assert_eq!(movement.action_id, Some(ACTION_POINTER_ROUTE.to_string()));
    assert!(key.diagnostic.is_none());
    assert!(mouse.diagnostic.is_none());
    assert!(movement.diagnostic.is_none());
}

#[test]
fn mouse_adapter_sequence_smoke_covers_representative_event_classes() {
    let bounds = WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT);
    let scenarios = [
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, PRIMARY_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::SingleClick),
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, PRIMARY_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::DoubleClick),
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, NO_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::Moved),
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, NO_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::Entered),
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, NO_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::Exited),
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, PRIMARY_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::ButtonUp),
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, SECONDARY_BUTTON)
            .with_kind(crate::pointer_input_routing::RatzillaMouseKind::ButtonDown),
    ];

    for scenario in scenarios {
        let outcome = adapt_ratzilla_mouse_input(scenario, bounds);

        assert_eq!(outcome.action_id, Some(ACTION_POINTER_ROUTE.to_string()));
        assert!(outcome.diagnostic.is_none());
    }
}

#[test]
fn mouse_adapter_preserves_ratzilla_modifier_metadata() {
    let modifiers = ctrl_alt_shift_modifiers();
    let outcome = adapt_ratzilla_mouse_input(
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, PRIMARY_BUTTON).with_modifiers(modifiers),
        WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT),
    );

    assert_eq!(outcome.action_id, Some(ACTION_POINTER_ROUTE.to_string()));
    assert_eq!(outcome.pointer_modifiers, Some(modifiers));
    assert!(outcome.diagnostic.is_none());
}

#[test]
fn mouse_adapter_keeps_modifiers_on_fail_closed_diagnostics() {
    let modifiers = ctrl_alt_shift_modifiers();
    let outcome = adapt_ratzilla_mouse_input(
        RatzillaMouseInput::new(OUTSIDE_X, INSIDE_Y, PRIMARY_BUTTON).with_modifiers(modifiers),
        WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT),
    );

    assert_eq!(outcome.action_id, None);
    assert_eq!(outcome.pointer_modifiers, Some(modifiers));
    assert!(matches!(
        outcome.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::OutOfBoundsCoordinate,
            ..
        })
    ));
}

#[test]
fn key_and_mouse_adapters_fail_closed_for_unsupported_inputs() {
    let unsupported_key = adapt_ratzilla_key_input(&RatzillaKeyInput::new("F25", Vec::new()));
    let unsupported_modifier = adapt_ratzilla_key_input(&RatzillaKeyInput::new(KEY_ENTER, vec!["Hyper".to_string()]));
    let out_of_bounds = adapt_ratzilla_mouse_input(
        RatzillaMouseInput::new(OUTSIDE_X, INSIDE_Y, PRIMARY_BUTTON),
        WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT),
    );
    let unknown_button = adapt_ratzilla_mouse_input(
        RatzillaMouseInput::new(INSIDE_X, INSIDE_Y, UNKNOWN_BUTTON),
        WebGridBounds::new(GRID_WIDTH, GRID_HEIGHT),
    );

    assert!(matches!(
        unsupported_key.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::UnsupportedKey,
            ..
        })
    ));
    assert!(matches!(
        unsupported_modifier.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::UnsupportedModifierCombination,
            ..
        })
    ));
    assert!(matches!(
        out_of_bounds.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::OutOfBoundsCoordinate,
            ..
        })
    ));
    assert!(matches!(
        unknown_button.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::UnknownMouseButton,
            ..
        })
    ));
}

#[test]
fn ratzilla_composition_repeat_and_physical_location_remain_explicit() {
    let composing = adapt_ratzilla_key_input(&RatzillaKeyInput::new("Dead", Vec::new()).with_composing(true));
    let repeat = adapt_ratzilla_key_input(&RatzillaKeyInput::new(KEY_ENTER, Vec::new()).with_repeat(true));
    let left = normalize_ratzilla_key_input(&RatzillaKeyInput::new("k", Vec::new()).with_location("left"))
        .expect("left location is renderer-only");
    let right = normalize_ratzilla_key_input(&RatzillaKeyInput::new("k", Vec::new()).with_location("right"))
        .expect("right location is renderer-only");

    assert!(matches!(
        composing.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::CompositionExcluded,
            ..
        })
    ));
    assert!(matches!(
        repeat.diagnostic,
        Some(WebInputDiagnostic {
            code: WebInputDiagnosticCode::RepeatExcluded,
            ..
        })
    ));
    assert_eq!(left, right);
}
