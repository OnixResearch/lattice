//! Infinite canvas viewport with pan/zoom and coordinate mapping.
//!
//! This crate provides coordinate math for infinite canvas widgets - mapping between
//! screen coordinates (visible terminal area) and canvas coordinates (infinite space).
//! It has no dependency on ratatui; rendering is the caller's responsibility.
//! Its integer positions, terminal dimensions, zoom rounding, and viewport state
//! remain canvas-domain/terminal projection behavior rather than a general scene
//! transform contract; see `docs/scene-intent-evaluation.md`.
//!
//! ## Types
//!
//! - [`Position`]: A coordinate on an infinite canvas (can be negative)
//! - [`Viewport`]: Pan/zoom camera for viewing the canvas
//!
//! ## Example
//!
//! ```rust
//! use rat_canvas::{Position, Viewport};
//!
//! let mut viewport = Viewport::new(80, 24);
//!
//! // Convert screen click to canvas position
//! let canvas_pos = viewport.screen_to_canvas(10, 5);
//! assert_eq!(canvas_pos, Position::new(10, 5));
//!
//! // Pan the viewport
//! viewport.pan(50, 25);
//! let canvas_pos = viewport.screen_to_canvas(10, 5);
//! assert_eq!(canvas_pos, Position::new(60, 30));
//! ```

/// A position on the canvas (can be negative for infinite canvas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new position.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Minimum zoom level (zoomed out).
pub const MIN_ZOOM: f32 = 0.25;

/// Maximum zoom level (zoomed in).
pub const MAX_ZOOM: f32 = 4.0;

/// Zoom step for zoom in/out operations.
pub const ZOOM_STEP: f32 = 0.25;

// Compile-time assertions for zoom constants
const _: () = assert!(MIN_ZOOM > 0.0, "MIN_ZOOM must be positive");
const _: () = assert!(MAX_ZOOM > MIN_ZOOM, "MAX_ZOOM must be greater than MIN_ZOOM");
const _: () = assert!(ZOOM_STEP > 0.0, "ZOOM_STEP must be positive");

/// Viewport - what part of the canvas is visible.
///
/// Maps between screen coordinates (u16, visible terminal area) and canvas
/// coordinates (i32, infinite space). Tracks pan position (offset), dimensions
/// (terminal size), and zoom level.
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Canvas X offset (pan position)
    pub offset_x: i32,
    /// Canvas Y offset (pan position)
    pub offset_y: i32,
    /// Screen width in columns
    pub width: u16,
    /// Screen height in rows
    pub height: u16,
    /// Zoom level: 1.0 = normal, 2.0 = zoomed in, 0.5 = zoomed out
    pub zoom: f32,
}

impl Viewport {
    /// Create a new viewport with the given dimensions.
    ///
    /// The viewport starts at offset (0, 0) with zoom 1.0.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            width,
            height,
            zoom: 1.0,
        }
    }

    /// Convert screen coordinates to canvas coordinates.
    ///
    /// At zoom > 1.0, screen coordinates map to smaller canvas areas.
    /// At zoom < 1.0, screen coordinates map to larger canvas areas.
    pub fn screen_to_canvas(&self, screen_x: u16, screen_y: u16) -> Position {
        debug_assert!(self.zoom > 0.0, "zoom must be positive");
        debug_assert!(self.width > 0, "width must be positive");
        debug_assert!(self.height > 0, "height must be positive");

        let canvas_x = (f32::from(screen_x) / self.zoom).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
        let canvas_y = (f32::from(screen_y) / self.zoom).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32;

        Position::new(canvas_x.saturating_add(self.offset_x), canvas_y.saturating_add(self.offset_y))
    }

    /// Convert canvas coordinates to screen coordinates (if visible).
    ///
    /// Returns `None` if the canvas position is outside the visible viewport.
    /// At zoom > 1.0, canvas positions map to larger screen areas.
    pub fn canvas_to_screen(&self, pos: Position) -> Option<(u16, u16)> {
        debug_assert!(self.zoom > 0.0, "zoom must be positive");
        debug_assert!(self.width > 0, "width must be positive");
        debug_assert!(self.height > 0, "height must be positive");

        let canvas_x_delta_cells = pos.x.saturating_sub(self.offset_x);
        let canvas_y_delta_cells = pos.y.saturating_sub(self.offset_y);

        let screen_x = (canvas_x_delta_cells as f32 * self.zoom).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
        let screen_y = (canvas_y_delta_cells as f32 * self.zoom).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32;

        if screen_x < 0 {
            return None;
        }
        if screen_y < 0 {
            return None;
        }
        if screen_x >= i32::from(self.width) {
            return None;
        }
        if screen_y >= i32::from(self.height) {
            return None;
        }

        let screen_x_columns = u16::try_from(screen_x).ok()?;
        let screen_y_rows = u16::try_from(screen_y).ok()?;

        Some((screen_x_columns, screen_y_rows))
    }

    /// Pan the viewport by the given delta in canvas units.
    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.offset_x = self.offset_x.saturating_add(dx);
        self.offset_y = self.offset_y.saturating_add(dy);
    }

    /// Zoom in by one step, clamped to MAX_ZOOM.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM);
    }

    /// Zoom out by one step, clamped to MIN_ZOOM.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - ZOOM_STEP).max(MIN_ZOOM);
    }

    /// Reset zoom to 1.0 (100%).
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }

    /// Resize the viewport when terminal dimensions change.
    ///
    /// Preserves offset and zoom level.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    /// Get the visible canvas area (number of canvas cells visible).
    ///
    /// At zoom > 1.0, fewer canvas cells are visible.
    /// At zoom < 1.0, more canvas cells are visible.
    pub fn visible_canvas_size(&self) -> (u16, u16) {
        debug_assert!(self.zoom > 0.0, "zoom must be positive");
        debug_assert!(self.width > 0, "width must be positive");
        debug_assert!(self.height > 0, "height must be positive");

        let visible_width = (f32::from(self.width) / self.zoom).round().clamp(0.0, u16::MAX as f32) as u16;
        let visible_height = (f32::from(self.height) / self.zoom).round().clamp(0.0, u16::MAX as f32) as u16;

        (visible_width, visible_height)
    }
}

#[cfg(test)]
mod tests;
