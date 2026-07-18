//! Pointer-to-docking session bridge.

/// Converts a shared pointer route outcome into a docking session event.
///
/// # Errors
///
/// Returns a docking diagnostic when the pointer outcome is diagnostic or is not
/// a drag/resize interaction class.
pub fn pointer_outcome_to_session_event(
    outcome: &crate::pointer_input_routing::PointerRouteOutcome,
    event: &crate::pointer_input_routing::PointerEvent,
) -> Result<crate::docking::DragEvent, crate::docking::DragDiagnostic> {
    match outcome {
        crate::pointer_input_routing::PointerRouteOutcome::Routed(route) => {
            pointer_route_to_event(route, event)
        }
        crate::pointer_input_routing::PointerRouteOutcome::Diagnostic(diagnostic) => {
            Err(pointer_diagnostic_to_drag_diagnostic(diagnostic))
        }
    }
}

fn pointer_route_to_event(
    route: &crate::pointer_input_routing::PointerRoute,
    event: &crate::pointer_input_routing::PointerEvent,
) -> Result<crate::docking::DragEvent, crate::docking::DragDiagnostic> {
    let Some(phase) = pointer_interaction_to_drag_phase(route.interaction) else {
        return Err(crate::docking::DragDiagnostic::new(
            crate::docking::DragDiagnosticCode::UnsupportedPhase,
            Some(route.target_id.clone()),
            "pointer route is not a docking drag interaction",
        ));
    };
    Ok(crate::docking::DragEvent::new(
        phase,
        route.target_id.clone(),
        crate::docking::DragPoint::new(event.position.x, event.position.y),
    ))
}

fn pointer_interaction_to_drag_phase(
    interaction: crate::pointer_input_routing::PointerInteractionClass,
) -> Option<crate::docking::DragPhase> {
    match interaction {
        crate::pointer_input_routing::PointerInteractionClass::DragStart => {
            Some(crate::docking::DragPhase::Start)
        }
        crate::pointer_input_routing::PointerInteractionClass::DragUpdate => {
            Some(crate::docking::DragPhase::Update)
        }
        crate::pointer_input_routing::PointerInteractionClass::DragCommit => {
            Some(crate::docking::DragPhase::Commit)
        }
        crate::pointer_input_routing::PointerInteractionClass::DragCancel => {
            Some(crate::docking::DragPhase::Cancel)
        }
        crate::pointer_input_routing::PointerInteractionClass::Activation
        | crate::pointer_input_routing::PointerInteractionClass::ContextMenu
        | crate::pointer_input_routing::PointerInteractionClass::WheelScroll
        | crate::pointer_input_routing::PointerInteractionClass::Hover
        | crate::pointer_input_routing::PointerInteractionClass::PassThrough
        | crate::pointer_input_routing::PointerInteractionClass::ModalCapture
        | crate::pointer_input_routing::PointerInteractionClass::NoOpDiagnostic => None,
    }
}

fn pointer_diagnostic_to_drag_diagnostic(
    diagnostic: &crate::pointer_input_routing::PointerDiagnostic,
) -> crate::docking::DragDiagnostic {
    crate::docking::DragDiagnostic::new(
        pointer_diagnostic_code_to_drag_code(diagnostic.code),
        diagnostic.target_id.clone(),
        diagnostic.message.clone(),
    )
}

fn pointer_diagnostic_code_to_drag_code(
    code: crate::pointer_input_routing::PointerDiagnosticCode,
) -> crate::docking::DragDiagnosticCode {
    match code {
        crate::pointer_input_routing::PointerDiagnosticCode::NoTarget
        | crate::pointer_input_routing::PointerDiagnosticCode::StaleTarget => {
            crate::docking::DragDiagnosticCode::StaleTarget
        }
        crate::pointer_input_routing::PointerDiagnosticCode::DisabledTarget => {
            crate::docking::DragDiagnosticCode::DisabledAffordance
        }
        crate::pointer_input_routing::PointerDiagnosticCode::HiddenTarget
        | crate::pointer_input_routing::PointerDiagnosticCode::UnsupportedKind => {
            crate::docking::DragDiagnosticCode::UnsupportedPhase
        }
    }
}

#[cfg(test)]
mod tests {
    const POINT_X: i32 = 3;
    const POINT_Y: i32 = 4;

    fn pointer_event(
        phase: crate::pointer_input_routing::PointerPhase,
    ) -> crate::pointer_input_routing::PointerEvent {
        crate::pointer_input_routing::PointerEvent::new(
            crate::pointer_input_routing::PointerPoint::new(POINT_X, POINT_Y),
            crate::pointer_input_routing::PointerButton::Primary,
            phase,
            crate::pointer_input_routing::PointerSource::Mouse,
        )
    }

    #[test]
    fn pointer_drag_route_becomes_docking_session_event() {
        let event = pointer_event(crate::pointer_input_routing::PointerPhase::DragStart);
        let outcome = crate::pointer_input_routing::PointerRouteOutcome::Routed(
            crate::pointer_input_routing::PointerRoute {
                target_id: "splitter".to_string(),
                action_id: None,
                modal_capture: false,
                interaction: crate::pointer_input_routing::PointerInteractionClass::DragStart,
                capture: crate::pointer_input_routing::PointerCaptureTransition::Start,
            },
        );

        let drag_event = super::pointer_outcome_to_session_event(&outcome, &event)
            .expect("drag route converts to docking event");

        assert_eq!(drag_event.phase, crate::docking::DragPhase::Start);
        assert_eq!(drag_event.target_id, "splitter".to_string());
        assert_eq!(
            drag_event.position,
            crate::docking::DragPoint::new(POINT_X, POINT_Y)
        );
    }

    #[test]
    fn non_drag_and_diagnostic_outcomes_fail_closed() {
        let event = pointer_event(crate::pointer_input_routing::PointerPhase::Down);
        let activation = crate::pointer_input_routing::PointerRouteOutcome::Routed(
            crate::pointer_input_routing::PointerRoute {
                target_id: "button".to_string(),
                action_id: None,
                modal_capture: false,
                interaction: crate::pointer_input_routing::PointerInteractionClass::Activation,
                capture: crate::pointer_input_routing::PointerCaptureTransition::None,
            },
        );
        let diagnostic = crate::pointer_input_routing::PointerRouteOutcome::Diagnostic(
            crate::pointer_input_routing::PointerDiagnostic::new(
                crate::pointer_input_routing::PointerDiagnosticCode::DisabledTarget,
                Some("splitter".to_string()),
                "disabled",
            ),
        );

        let rejected_activation = super::pointer_outcome_to_session_event(&activation, &event);
        let rejected_diagnostic = super::pointer_outcome_to_session_event(&diagnostic, &event);

        assert!(matches!(
            rejected_activation,
            Err(crate::docking::DragDiagnostic {
                code: crate::docking::DragDiagnosticCode::UnsupportedPhase,
                ..
            })
        ));
        assert!(matches!(
            rejected_diagnostic,
            Err(crate::docking::DragDiagnostic {
                code: crate::docking::DragDiagnosticCode::DisabledAffordance,
                ..
            })
        ));
    }
}
