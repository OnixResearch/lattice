semantic_string_id! {
    /// Stable target identifier for a hit region.
    pub struct PointerTargetId;
}

/// Canonical registry action identifier associated with a pointer route.
pub type PointerActionId = crate::action_registry::ActionId;

/// Integer pointer coordinate in renderer-neutral viewport space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointerPoint {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

impl PointerPoint {
    /// Creates a point.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Integer rectangle used for hit regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerRect {
    /// Left coordinate.
    pub x: i32,
    /// Top coordinate.
    pub y: i32,
    /// Width in logical cells or pixels.
    pub width: i32,
    /// Height in logical cells or pixels.
    pub height: i32,
}

impl PointerRect {
    /// Creates a rectangle.
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// Returns true when the point is inside the half-open rectangle.
    #[must_use]
    pub const fn contains(self, point: PointerPoint) -> bool {
        if self.width <= 0 || self.height <= 0 {
            return false;
        }
        let right = self.x.saturating_add(self.width);
        let bottom = self.y.saturating_add(self.height);
        self.x <= point.x && point.x < right && self.y <= point.y && point.y < bottom
    }
}

/// Coordinate space used by a normalized pointer event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerCoordinateSpace {
    /// Coordinates are relative to the logical viewport.
    Viewport,
    /// Coordinates are relative to a widget or component local origin.
    WidgetLocal,
    /// Coordinates are in document or page space.
    Document,
    /// Coordinates are in global screen space.
    Screen,
    /// Coordinates are terminal cells.
    TerminalCell,
    /// Renderer reported a coordinate space the shared router does not understand.
    Unsupported(String),
}
