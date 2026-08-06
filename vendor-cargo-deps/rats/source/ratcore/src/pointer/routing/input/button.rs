/// Pointer button after renderer normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerButton {
    /// Primary pointer button.
    Primary,
    /// Secondary pointer button.
    Secondary,
    /// Middle pointer button.
    Middle,
    /// Browser or device back button.
    Back,
    /// Browser or device forward button.
    Forward,
    /// No button is associated with the event.
    None,
    /// Renderer reported a button the shared router does not understand.
    Unsupported(String),
}

/// Active pointer button state after renderer normalization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PointerButtonSet {
    /// Primary pointer button is active.
    pub primary: bool,
    /// Secondary pointer button is active.
    pub secondary: bool,
    /// Middle pointer button is active.
    pub middle: bool,
    /// Back pointer button is active.
    pub back: bool,
    /// Forward pointer button is active.
    pub forward: bool,
    /// Unsupported active buttons reported by a renderer.
    pub unsupported: Vec<String>,
}

impl PointerButtonSet {
    /// Creates an empty active-button set.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates active-button state from the event button.
    #[must_use]
    pub fn from_button(button: &PointerButton) -> Self {
        match button {
            PointerButton::Primary => Self {
                primary: true,
                ..Self::default()
            },
            PointerButton::Secondary => Self {
                secondary: true,
                ..Self::default()
            },
            PointerButton::Middle => Self {
                middle: true,
                ..Self::default()
            },
            PointerButton::Back => Self {
                back: true,
                ..Self::default()
            },
            PointerButton::Forward => Self {
                forward: true,
                ..Self::default()
            },
            PointerButton::None => Self::default(),
            PointerButton::Unsupported(kind) => Self {
                unsupported: vec![kind.clone()],
                ..Self::default()
            },
        }
    }

    /// Returns true when no supported or unsupported buttons are active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.primary && !self.secondary && !self.middle && !self.back && !self.forward && self.unsupported.is_empty()
    }

    /// Returns true when a context-menu style button is active.
    #[must_use]
    pub const fn has_context_button(&self) -> bool {
        self.secondary
    }

    /// Adds unsupported active-button metadata.
    #[must_use]
    pub fn with_unsupported(mut self, kind: impl Into<String>) -> Self {
        self.unsupported.push(kind.into());
        self
    }
}
