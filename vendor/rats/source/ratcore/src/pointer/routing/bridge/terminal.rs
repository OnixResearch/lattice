const WHEEL_CELL_DELTA: i32 = 1;

/// Subwayrat terminal mouse button after crossterm extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubwayratMouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle button.
    Middle,
    /// No concrete button.
    None,
    /// Unsupported crossterm button variant.
    Unsupported(String),
}

/// Subwayrat terminal mouse event kind after crossterm extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubwayratMouseKind {
    /// Button down.
    Down,
    /// Button up.
    Up,
    /// Mouse drag.
    Drag,
    /// Hover or movement.
    Move,
    /// Scroll up.
    ScrollUp,
    /// Scroll down.
    ScrollDown,
    /// Scroll left.
    ScrollLeft,
    /// Scroll right.
    ScrollRight,
    /// Unsupported crossterm mouse kind.
    Unsupported(String),
}

/// Crossterm-like terminal mouse input without depending on crossterm types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubwayratMouseInput {
    /// Cell coordinate.
    pub position: super::super::shape::PointerPoint,
    /// Event kind.
    pub kind: SubwayratMouseKind,
    /// Event button.
    pub button: SubwayratMouseButton,
    /// Active modifiers.
    pub modifiers: super::super::input::PointerModifiers,
    /// Optional captured target supplied by the shell.
    pub captured_target: Option<super::super::shape::PointerTargetId>,
}

impl SubwayratMouseInput {
    /// Creates terminal mouse input.
    #[must_use]
    pub fn new(
        position: super::super::shape::PointerPoint,
        kind: SubwayratMouseKind,
        button: SubwayratMouseButton,
    ) -> Self {
        Self {
            position,
            kind,
            button,
            modifiers: super::super::input::PointerModifiers::default(),
            captured_target: None,
        }
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

    /// Adds a capture target.
    #[must_use]
    pub fn with_captured_target(
        mut self,
        target: impl Into<super::super::shape::PointerTargetId>,
    ) -> Self {
        self.captured_target = Some(target.into());
        self
    }
}

/// Normalizes Subwayrat/crossterm mouse input into the shared pointer model.
///
/// # Errors
///
/// Returns a deterministic pointer diagnostic for unsupported terminal variants.
pub fn normalize_subwayrat_mouse_input(
    input: SubwayratMouseInput,
) -> Result<super::super::input::PointerEvent, super::super::result::PointerDiagnostic> {
    let mut event = super::super::input::PointerEvent::new(
        input.position,
        terminal_button(input.button),
        terminal_phase(&input.kind),
        super::super::input::PointerSource::Terminal,
    )
    .with_coordinate_space(super::super::shape::PointerCoordinateSpace::TerminalCell)
    .with_modifiers(input.modifiers);
    if let Some(wheel_delta) = terminal_wheel_delta(&input.kind) {
        event = event.with_wheel_delta(wheel_delta);
    }
    if let Some(target) = input.captured_target {
        event = event.with_captured_target(target);
    }
    super::super::normalize_pointer_event(event)
}

fn terminal_button(button: SubwayratMouseButton) -> super::super::input::PointerButton {
    match button {
        SubwayratMouseButton::Left => super::super::input::PointerButton::Primary,
        SubwayratMouseButton::Right => super::super::input::PointerButton::Secondary,
        SubwayratMouseButton::Middle => super::super::input::PointerButton::Middle,
        SubwayratMouseButton::None => super::super::input::PointerButton::None,
        SubwayratMouseButton::Unsupported(kind) => {
            super::super::input::PointerButton::Unsupported(kind)
        }
    }
}

fn terminal_phase(kind: &SubwayratMouseKind) -> super::super::input::PointerPhase {
    match kind {
        SubwayratMouseKind::Down => super::super::input::PointerPhase::Down,
        SubwayratMouseKind::Up => super::super::input::PointerPhase::Up,
        SubwayratMouseKind::Drag => super::super::input::PointerPhase::DragUpdate,
        SubwayratMouseKind::Move => super::super::input::PointerPhase::Move,
        SubwayratMouseKind::ScrollUp
        | SubwayratMouseKind::ScrollDown
        | SubwayratMouseKind::ScrollLeft
        | SubwayratMouseKind::ScrollRight => super::super::input::PointerPhase::Wheel,
        SubwayratMouseKind::Unsupported(kind) => {
            super::super::input::PointerPhase::Unsupported(kind.clone())
        }
    }
}

fn terminal_wheel_delta(
    kind: &SubwayratMouseKind,
) -> Option<super::super::input::PointerWheelDelta> {
    match kind {
        SubwayratMouseKind::ScrollUp => Some(super::super::input::PointerWheelDelta::new(
            0,
            WHEEL_CELL_DELTA,
            super::super::input::PointerWheelUnit::Cells,
        )),
        SubwayratMouseKind::ScrollDown => Some(super::super::input::PointerWheelDelta::new(
            0,
            -WHEEL_CELL_DELTA,
            super::super::input::PointerWheelUnit::Cells,
        )),
        SubwayratMouseKind::ScrollLeft => Some(super::super::input::PointerWheelDelta::new(
            -WHEEL_CELL_DELTA,
            0,
            super::super::input::PointerWheelUnit::Cells,
        )),
        SubwayratMouseKind::ScrollRight => Some(super::super::input::PointerWheelDelta::new(
            WHEEL_CELL_DELTA,
            0,
            super::super::input::PointerWheelUnit::Cells,
        )),
        SubwayratMouseKind::Down
        | SubwayratMouseKind::Up
        | SubwayratMouseKind::Drag
        | SubwayratMouseKind::Move
        | SubwayratMouseKind::Unsupported(_) => None,
    }
}
