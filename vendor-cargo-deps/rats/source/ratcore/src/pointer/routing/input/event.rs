/// Renderer-neutral pointer event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerEvent {
    /// Pointer coordinate.
    pub position: super::super::shape::PointerPoint,
    /// Coordinate space for the pointer coordinate.
    pub coordinate_space: super::super::shape::PointerCoordinateSpace,
    /// Pointer button associated with the event.
    pub button: super::press::PointerButton,
    /// Active pointer buttons for stateful move, drag, and release paths.
    pub buttons: super::press::PointerButtonSet,
    /// Pointer phase.
    pub phase: super::kind::PointerPhase,
    /// Pointer source.
    pub source: super::kind::PointerSource,
    /// Wheel delta for wheel or scroll events.
    pub wheel_delta: Option<super::scroll::PointerWheelDelta>,
    /// Click count metadata.
    pub click_count: super::kind::PointerClickCount,
    /// Active modifiers.
    pub modifiers: super::kind::PointerModifiers,
    /// Optional captured target from a prior route.
    pub captured_target: Option<super::super::shape::PointerTargetId>,
}

impl PointerEvent {
    /// Creates a pointer event.
    #[must_use]
    pub fn new(
        position: super::super::shape::PointerPoint,
        button: super::press::PointerButton,
        phase: super::kind::PointerPhase,
        source: super::kind::PointerSource,
    ) -> Self {
        let buttons = super::press::PointerButtonSet::from_button(&button);
        Self {
            position,
            coordinate_space: super::super::shape::PointerCoordinateSpace::Viewport,
            button,
            buttons,
            phase,
            source,
            wheel_delta: None,
            click_count: super::kind::PointerClickCount::None,
            modifiers: super::kind::PointerModifiers::default(),
            captured_target: None,
        }
    }

    /// Adds coordinate-space metadata.
    #[must_use]
    pub fn with_coordinate_space(
        mut self,
        coordinate_space: super::super::shape::PointerCoordinateSpace,
    ) -> Self {
        self.coordinate_space = coordinate_space;
        self
    }

    /// Adds active button-state metadata.
    #[must_use]
    pub fn with_buttons(mut self, buttons: super::press::PointerButtonSet) -> Self {
        self.buttons = buttons;
        self
    }

    /// Adds wheel delta metadata.
    #[must_use]
    pub fn with_wheel_delta(mut self, wheel_delta: super::scroll::PointerWheelDelta) -> Self {
        self.wheel_delta = Some(wheel_delta);
        self
    }

    /// Adds click-count metadata.
    #[must_use]
    pub fn with_click_count(mut self, click_count: super::kind::PointerClickCount) -> Self {
        self.click_count = click_count;
        self
    }

    /// Adds modifier metadata.
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: super::kind::PointerModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Adds an already captured target.
    #[must_use]
    pub fn with_captured_target(
        mut self,
        target: impl Into<super::super::shape::PointerTargetId>,
    ) -> Self {
        self.captured_target = Some(target.into());
        self
    }
}
