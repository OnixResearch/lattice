const EMPTY_LABEL: &str = "";
const DEFAULT_DIAGNOSTIC_MESSAGE: &str = "pointer route rejected";
const DEFAULT_UNSUPPORTED_KIND: &str = "unknown";

pub(super) fn unsupported_event_kind(event: &super::input::PointerEvent) -> Option<&str> {
    unsupported_source(&event.source)
        .or_else(|| unsupported_phase(&event.phase))
        .or_else(|| unsupported_button(&event.button))
        .or_else(|| unsupported_space(&event.coordinate_space))
        .or_else(|| unsupported_click_count(&event.click_count))
        .or_else(|| first_unsupported_button(&event.buttons))
        .or_else(|| unsupported_wheel_unit(event.wheel_delta.as_ref()))
}

pub(super) fn unsupported_diagnostic(kind: &str) -> super::result::PointerDiagnostic {
    let message = if kind == EMPTY_LABEL {
        DEFAULT_DIAGNOSTIC_MESSAGE.to_string()
    } else {
        format!("unsupported pointer input: {kind}")
    };
    super::result::PointerDiagnostic::new(
        super::result::PointerDiagnosticCode::UnsupportedKind,
        None,
        message,
    )
}

pub(super) fn disabled_diagnostic(
    region: &super::area::HitRegion,
) -> super::result::PointerRouteOutcome {
    super::result::PointerRouteOutcome::Diagnostic(super::result::PointerDiagnostic::new(
        super::result::PointerDiagnosticCode::DisabledTarget,
        Some(region.target_id.clone()),
        region
            .diagnostic
            .clone()
            .unwrap_or_else(|| "target is disabled".to_string()),
    ))
}

pub(super) fn hidden_diagnostic(
    region: &super::area::HitRegion,
) -> super::result::PointerRouteOutcome {
    super::result::PointerRouteOutcome::Diagnostic(super::result::PointerDiagnostic::new(
        super::result::PointerDiagnosticCode::HiddenTarget,
        Some(region.target_id.clone()),
        region
            .diagnostic
            .clone()
            .unwrap_or_else(|| "target is hidden".to_string()),
    ))
}

pub(super) fn default_kind_label() -> String {
    DEFAULT_UNSUPPORTED_KIND.to_string()
}

fn unsupported_source(source: &super::input::PointerSource) -> Option<&str> {
    if let super::input::PointerSource::Unsupported(kind) = source {
        Some(kind)
    } else {
        None
    }
}

fn unsupported_phase(phase: &super::input::PointerPhase) -> Option<&str> {
    if let super::input::PointerPhase::Unsupported(kind) = phase {
        Some(kind)
    } else {
        None
    }
}

fn unsupported_button(button: &super::input::PointerButton) -> Option<&str> {
    if let super::input::PointerButton::Unsupported(kind) = button {
        Some(kind)
    } else {
        None
    }
}

fn unsupported_space(space: &super::shape::PointerCoordinateSpace) -> Option<&str> {
    if let super::shape::PointerCoordinateSpace::Unsupported(kind) = space {
        Some(kind)
    } else {
        None
    }
}

fn unsupported_click_count(click_count: &super::input::PointerClickCount) -> Option<&str> {
    if let super::input::PointerClickCount::Unsupported(kind) = click_count {
        Some(kind)
    } else {
        None
    }
}

fn first_unsupported_button(buttons: &super::input::PointerButtonSet) -> Option<&str> {
    buttons.unsupported.first().map(std::string::String::as_str)
}

fn unsupported_wheel_unit(wheel_delta: Option<&super::input::PointerWheelDelta>) -> Option<&str> {
    let unit = &wheel_delta?.unit;
    if let super::input::PointerWheelUnit::Unsupported(kind) = unit {
        Some(kind)
    } else {
        None
    }
}
