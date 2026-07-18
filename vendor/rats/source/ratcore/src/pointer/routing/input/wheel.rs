/// Wheel delta unit after renderer normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerWheelUnit {
    /// Logical line or row units.
    Lines,
    /// Page units.
    Pages,
    /// Pixel units.
    Pixels,
    /// Terminal cell units.
    Cells,
    /// Renderer reported a wheel unit the shared router does not understand.
    Unsupported(String),
}

/// Renderer-neutral wheel delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerWheelDelta {
    /// Horizontal wheel delta.
    pub horizontal: i32,
    /// Vertical wheel delta.
    pub vertical: i32,
    /// Delta unit.
    pub unit: PointerWheelUnit,
}

impl PointerWheelDelta {
    /// Creates a wheel delta.
    #[must_use]
    pub const fn new(horizontal: i32, vertical: i32, unit: PointerWheelUnit) -> Self {
        Self {
            horizontal,
            vertical,
            unit,
        }
    }
}
