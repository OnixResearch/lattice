/// Browser DOM pointer, mouse, wheel, or context event kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomPointerKind {
    /// Pointer or mouse down.
    Down,
    /// Pointer or mouse up.
    Up,
    /// Click.
    Click,
    /// Double click.
    DoubleClick,
    /// Context menu.
    ContextMenu,
    /// Pointer movement.
    Move,
    /// Pointer enter.
    Enter,
    /// Pointer leave.
    Leave,
    /// Wheel event.
    Wheel,
    /// Drag start.
    DragStart,
    /// Drag update.
    DragUpdate,
    /// Drag end.
    DragEnd,
    /// Cancellation.
    Cancel,
    /// Unsupported browser event.
    Unsupported(String),
}

/// Browser DOM pointer input extracted before the shared core sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomPointerInput {
    /// Document or component coordinate.
    pub position: super::super::shape::PointerPoint,
    /// Coordinate space.
    pub coordinate_space: super::super::shape::PointerCoordinateSpace,
    /// Event kind.
    pub kind: DomPointerKind,
    /// Event button.
    pub button: super::super::input::PointerButton,
    /// Active buttons.
    pub buttons: super::super::input::PointerButtonSet,
    /// Optional wheel delta.
    pub wheel_delta: Option<super::super::input::PointerWheelDelta>,
    /// Active modifiers.
    pub modifiers: super::super::input::PointerModifiers,
}

impl DomPointerInput {
    /// Creates browser pointer input.
    #[must_use]
    pub fn new(
        position: super::super::shape::PointerPoint,
        kind: DomPointerKind,
        button: super::super::input::PointerButton,
    ) -> Self {
        let buttons = super::super::input::PointerButtonSet::from_button(&button);
        Self {
            position,
            coordinate_space: super::super::shape::PointerCoordinateSpace::Document,
            kind,
            button,
            buttons,
            wheel_delta: None,
            modifiers: super::super::input::PointerModifiers::default(),
        }
    }

    /// Adds active button-state metadata.
    #[must_use]
    pub fn with_buttons(mut self, buttons: super::super::input::PointerButtonSet) -> Self {
        self.buttons = buttons;
        self
    }

    /// Adds wheel delta metadata.
    #[must_use]
    pub fn with_wheel_delta(mut self, wheel_delta: super::super::input::PointerWheelDelta) -> Self {
        self.wheel_delta = Some(wheel_delta);
        self
    }

    /// Adds modifiers.
    #[must_use]
    pub const fn with_modifiers(
        mut self,
        modifiers: super::super::input::PointerModifiers,
    ) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Normalizes browser DOM pointer input into the shared pointer model.
///
/// # Errors
///
/// Returns a deterministic pointer diagnostic for unsupported browser input.
pub fn normalize_dom_pointer_input(
    input: DomPointerInput,
) -> Result<super::super::input::PointerEvent, super::super::result::PointerDiagnostic> {
    let mut event = super::super::input::PointerEvent::new(
        input.position,
        dom_button(&input),
        dom_phase(&input.kind),
        super::super::input::PointerSource::Dom,
    )
    .with_coordinate_space(input.coordinate_space)
    .with_buttons(input.buttons)
    .with_click_count(dom_click_count(&input.kind))
    .with_modifiers(input.modifiers);
    if let Some(wheel_delta) = input.wheel_delta {
        event = event.with_wheel_delta(wheel_delta);
    }
    super::super::normalize_pointer_event(event)
}

fn dom_button(input: &DomPointerInput) -> super::super::input::PointerButton {
    if input.kind == DomPointerKind::ContextMenu {
        super::super::input::PointerButton::Secondary
    } else {
        input.button.clone()
    }
}

fn dom_phase(kind: &DomPointerKind) -> super::super::input::PointerPhase {
    match kind {
        DomPointerKind::Down | DomPointerKind::Click | DomPointerKind::ContextMenu => {
            super::super::input::PointerPhase::Down
        }
        DomPointerKind::Up => super::super::input::PointerPhase::Up,
        DomPointerKind::DoubleClick => super::super::input::PointerPhase::Down,
        DomPointerKind::Move => super::super::input::PointerPhase::Move,
        DomPointerKind::Enter => super::super::input::PointerPhase::Enter,
        DomPointerKind::Leave => super::super::input::PointerPhase::Leave,
        DomPointerKind::Wheel => super::super::input::PointerPhase::Wheel,
        DomPointerKind::DragStart => super::super::input::PointerPhase::DragStart,
        DomPointerKind::DragUpdate => super::super::input::PointerPhase::DragUpdate,
        DomPointerKind::DragEnd => super::super::input::PointerPhase::DragCommit,
        DomPointerKind::Cancel => super::super::input::PointerPhase::DragCancel,
        DomPointerKind::Unsupported(kind) => {
            super::super::input::PointerPhase::Unsupported(kind.clone())
        }
    }
}

fn dom_click_count(kind: &DomPointerKind) -> super::super::input::PointerClickCount {
    match kind {
        DomPointerKind::Click => super::super::input::PointerClickCount::Single,
        DomPointerKind::DoubleClick => super::super::input::PointerClickCount::Double,
        DomPointerKind::Down
        | DomPointerKind::Up
        | DomPointerKind::ContextMenu
        | DomPointerKind::Move
        | DomPointerKind::Enter
        | DomPointerKind::Leave
        | DomPointerKind::Wheel
        | DomPointerKind::DragStart
        | DomPointerKind::DragUpdate
        | DomPointerKind::DragEnd
        | DomPointerKind::Cancel
        | DomPointerKind::Unsupported(_) => super::super::input::PointerClickCount::None,
    }
}
