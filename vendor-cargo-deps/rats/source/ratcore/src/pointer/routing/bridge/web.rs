const NO_BUTTON_NUMBER: i32 = -1;
const PRIMARY_BUTTON_NUMBER: i32 = 0;
const SECONDARY_BUTTON_NUMBER: i32 = 1;
const MIDDLE_BUTTON_NUMBER: i32 = 2;

/// Ratzilla mouse kind after shell extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatzillaMouseKind {
    /// Button down.
    ButtonDown,
    /// Button up.
    ButtonUp,
    /// Single click.
    SingleClick,
    /// Double click.
    DoubleClick,
    /// Movement.
    Moved,
    /// Entered the surface.
    Entered,
    /// Exited the surface.
    Exited,
    /// Unidentified or unsupported Ratzilla event.
    Unidentified(String),
}

/// Ratzilla mouse input without depending on Ratzilla event lifetimes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatzillaPointerInput {
    /// Cell coordinate.
    pub position: super::super::shape::PointerPoint,
    /// Event kind.
    pub kind: RatzillaMouseKind,
    /// Ratzilla button number.
    pub button: i32,
    /// Active modifiers.
    pub modifiers: super::super::input::PointerModifiers,
}

impl RatzillaPointerInput {
    /// Creates Ratzilla mouse input.
    #[must_use]
    pub const fn new(position: super::super::shape::PointerPoint, kind: RatzillaMouseKind, button: i32) -> Self {
        Self {
            position,
            kind,
            button,
            modifiers: super::super::input::PointerModifiers {
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
            },
        }
    }

    /// Adds modifiers.
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: super::super::input::PointerModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Normalizes Ratzilla mouse input into the shared pointer model.
///
/// # Errors
///
/// Returns a deterministic pointer diagnostic for unidentified events or buttons.
pub fn normalize_ratzilla_mouse_input(
    input: RatzillaPointerInput,
) -> Result<super::super::input::PointerEvent, super::super::result::PointerDiagnostic> {
    let event = super::super::input::PointerEvent::new(
        input.position,
        ratzilla_button(input.button),
        ratzilla_phase(&input.kind),
        super::super::input::PointerSource::Ratzilla,
    )
    .with_coordinate_space(super::super::shape::PointerCoordinateSpace::TerminalCell)
    .with_click_count(ratzilla_click_count(&input.kind))
    .with_modifiers(input.modifiers);
    super::super::normalize_pointer_event(event)
}

fn ratzilla_button(button: i32) -> super::super::input::PointerButton {
    match button {
        NO_BUTTON_NUMBER => super::super::input::PointerButton::None,
        PRIMARY_BUTTON_NUMBER => super::super::input::PointerButton::Primary,
        SECONDARY_BUTTON_NUMBER => super::super::input::PointerButton::Secondary,
        MIDDLE_BUTTON_NUMBER => super::super::input::PointerButton::Middle,
        _ => super::super::input::PointerButton::Unsupported(format!("ratzilla-button-{button}")),
    }
}

fn ratzilla_phase(kind: &RatzillaMouseKind) -> super::super::input::PointerPhase {
    match kind {
        RatzillaMouseKind::ButtonDown | RatzillaMouseKind::SingleClick => super::super::input::PointerPhase::Down,
        RatzillaMouseKind::ButtonUp => super::super::input::PointerPhase::Up,
        RatzillaMouseKind::DoubleClick => super::super::input::PointerPhase::Down,
        RatzillaMouseKind::Moved => super::super::input::PointerPhase::Move,
        RatzillaMouseKind::Entered => super::super::input::PointerPhase::Enter,
        RatzillaMouseKind::Exited => super::super::input::PointerPhase::Leave,
        RatzillaMouseKind::Unidentified(kind) => super::super::input::PointerPhase::Unsupported(kind.clone()),
    }
}

fn ratzilla_click_count(kind: &RatzillaMouseKind) -> super::super::input::PointerClickCount {
    match kind {
        RatzillaMouseKind::SingleClick => super::super::input::PointerClickCount::Single,
        RatzillaMouseKind::DoubleClick => super::super::input::PointerClickCount::Double,
        RatzillaMouseKind::ButtonDown
        | RatzillaMouseKind::ButtonUp
        | RatzillaMouseKind::Moved
        | RatzillaMouseKind::Entered
        | RatzillaMouseKind::Exited
        | RatzillaMouseKind::Unidentified(_) => super::super::input::PointerClickCount::None,
    }
}
