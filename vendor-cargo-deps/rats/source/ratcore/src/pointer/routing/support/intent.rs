pub(super) fn classify_interaction(
    event: &super::input::PointerEvent,
    capture_policy: super::area::HitCapturePolicy,
) -> super::result::PointerInteractionClass {
    assert_event_shape(event);
    if let Some(class) = policy_interaction(capture_policy) {
        return class;
    }
    if event.wheel_delta.is_some() || event.phase == super::input::PointerPhase::Wheel {
        return super::result::PointerInteractionClass::WheelScroll;
    }
    if event.button == super::input::PointerButton::Secondary || event.buttons.has_context_button()
    {
        return super::result::PointerInteractionClass::ContextMenu;
    }
    phase_interaction(event)
}

pub(super) fn capture_transition(
    event: &super::input::PointerEvent,
    capture_policy: super::area::HitCapturePolicy,
    interaction: super::result::PointerInteractionClass,
) -> super::result::PointerCaptureTransition {
    assert_capture_shape(event, interaction);
    if let Some(transition) = policy_capture_transition(capture_policy) {
        return transition;
    }
    interaction_capture_transition(event, interaction)
}

fn phase_interaction(event: &super::input::PointerEvent) -> super::result::PointerInteractionClass {
    match event.phase {
        super::input::PointerPhase::Down => super::result::PointerInteractionClass::Activation,
        super::input::PointerPhase::Move
        | super::input::PointerPhase::Enter
        | super::input::PointerPhase::Leave => movement_interaction(event),
        super::input::PointerPhase::Up => release_interaction(event),
        super::input::PointerPhase::DragStart => super::result::PointerInteractionClass::DragStart,
        super::input::PointerPhase::DragUpdate => {
            super::result::PointerInteractionClass::DragUpdate
        }
        super::input::PointerPhase::DragCommit => {
            super::result::PointerInteractionClass::DragCommit
        }
        super::input::PointerPhase::Cancel | super::input::PointerPhase::DragCancel => {
            super::result::PointerInteractionClass::DragCancel
        }
        super::input::PointerPhase::Wheel => super::result::PointerInteractionClass::WheelScroll,
        super::input::PointerPhase::Unsupported(_) => {
            super::result::PointerInteractionClass::NoOpDiagnostic
        }
    }
}

fn movement_interaction(
    event: &super::input::PointerEvent,
) -> super::result::PointerInteractionClass {
    if event.captured_target.is_some() || !event.buttons.is_empty() {
        super::result::PointerInteractionClass::DragUpdate
    } else {
        super::result::PointerInteractionClass::Hover
    }
}

fn release_interaction(
    event: &super::input::PointerEvent,
) -> super::result::PointerInteractionClass {
    if event.captured_target.is_some() {
        super::result::PointerInteractionClass::DragCommit
    } else {
        super::result::PointerInteractionClass::Activation
    }
}

fn policy_interaction(
    capture_policy: super::area::HitCapturePolicy,
) -> Option<super::result::PointerInteractionClass> {
    match capture_policy {
        super::area::HitCapturePolicy::ModalCapture => {
            Some(super::result::PointerInteractionClass::ModalCapture)
        }
        super::area::HitCapturePolicy::PassThrough => {
            Some(super::result::PointerInteractionClass::PassThrough)
        }
        super::area::HitCapturePolicy::Capture => None,
    }
}

fn policy_capture_transition(
    capture_policy: super::area::HitCapturePolicy,
) -> Option<super::result::PointerCaptureTransition> {
    match capture_policy {
        super::area::HitCapturePolicy::ModalCapture => {
            Some(super::result::PointerCaptureTransition::Modal)
        }
        super::area::HitCapturePolicy::PassThrough => {
            Some(super::result::PointerCaptureTransition::None)
        }
        super::area::HitCapturePolicy::Capture => None,
    }
}

fn interaction_capture_transition(
    event: &super::input::PointerEvent,
    interaction: super::result::PointerInteractionClass,
) -> super::result::PointerCaptureTransition {
    match interaction {
        super::result::PointerInteractionClass::DragStart => {
            super::result::PointerCaptureTransition::Start
        }
        super::result::PointerInteractionClass::DragUpdate => {
            super::result::PointerCaptureTransition::Update
        }
        super::result::PointerInteractionClass::DragCommit => {
            super::result::PointerCaptureTransition::Release
        }
        super::result::PointerInteractionClass::DragCancel => {
            super::result::PointerCaptureTransition::Cancel
        }
        super::result::PointerInteractionClass::Activation => activation_capture_transition(event),
        super::result::PointerInteractionClass::ContextMenu
        | super::result::PointerInteractionClass::WheelScroll
        | super::result::PointerInteractionClass::Hover
        | super::result::PointerInteractionClass::PassThrough
        | super::result::PointerInteractionClass::ModalCapture
        | super::result::PointerInteractionClass::NoOpDiagnostic => {
            super::result::PointerCaptureTransition::None
        }
    }
}

fn activation_capture_transition(
    event: &super::input::PointerEvent,
) -> super::result::PointerCaptureTransition {
    match event.phase {
        super::input::PointerPhase::Down => super::result::PointerCaptureTransition::Start,
        super::input::PointerPhase::Up if event.captured_target.is_some() => {
            super::result::PointerCaptureTransition::Release
        }
        super::input::PointerPhase::Up
        | super::input::PointerPhase::Move
        | super::input::PointerPhase::Enter
        | super::input::PointerPhase::Leave
        | super::input::PointerPhase::Wheel
        | super::input::PointerPhase::DragStart
        | super::input::PointerPhase::DragUpdate
        | super::input::PointerPhase::DragCommit
        | super::input::PointerPhase::Cancel
        | super::input::PointerPhase::DragCancel
        | super::input::PointerPhase::Unsupported(_) => {
            super::result::PointerCaptureTransition::None
        }
    }
}

fn assert_event_shape(event: &super::input::PointerEvent) {
    debug_assert!(!matches!(
        event.phase,
        super::input::PointerPhase::Unsupported(_)
    ));
    debug_assert!(
        event.wheel_delta.as_ref().is_none_or(|delta| !matches!(
            delta.unit,
            super::input::PointerWheelUnit::Unsupported(_)
        ))
    );
}

fn assert_capture_shape(
    event: &super::input::PointerEvent,
    interaction: super::result::PointerInteractionClass,
) {
    debug_assert!(
        event
            .captured_target
            .as_ref()
            .is_none_or(|target| !target.is_empty())
    );
    debug_assert!(interaction != super::result::PointerInteractionClass::NoOpDiagnostic);
}
