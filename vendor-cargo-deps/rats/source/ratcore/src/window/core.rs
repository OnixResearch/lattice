//! Renderer-neutral viewport, cursor, and selection logic.
//!
//! Renderers own measurement and drawing. This module owns the pure state
//! transitions that answer which logical rows are visible, where the cursor is,
//! how scroll offsets clamp, and how range selections survive collection size
//! changes.
//!
//! # Example
//!
//! ```
//! use ratcore::viewport::{Movement, ViewportState};
//!
//! let state = ViewportState::new(20, 5).unwrap().move_cursor(Movement::PageDown);
//! assert_eq!(state.cursor(), 5);
//! assert_eq!(state.visible_range().start, 1);
//! assert_eq!(state.visible_range().end, 6);
//! ```

/// Half-open visible item range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    /// First visible item index.
    pub start: u32,
    /// One-past-last visible item index.
    pub end: u32,
}

impl VisibleRange {
    /// Creates a visible range when bounds are ordered.
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    /// Returns true when the range contains no indices.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns true when `index` is inside the half-open range.
    #[must_use]
    pub fn contains(self, index: u32) -> bool {
        self.start <= index && index < self.end
    }
}

/// Inclusive logical selection endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRange {
    /// Fixed selection anchor.
    pub anchor: u32,
    /// Moving selection focus.
    pub focus: u32,
}

impl SelectionRange {
    /// Creates a validated selection range.
    ///
    /// # Errors
    ///
    /// Returns [`ViewportError::SelectionOutOfBounds`] when either endpoint is
    /// outside the collection.
    pub fn new(anchor: u32, focus: u32, item_count: u32) -> Result<Self, ViewportError> {
        if anchor >= item_count {
            return Err(ViewportError::SelectionOutOfBounds { index: anchor });
        }
        if focus >= item_count {
            return Err(ViewportError::SelectionOutOfBounds { index: focus });
        }
        Ok(Self { anchor, focus })
    }

    /// Returns the selection as a half-open normalized range.
    #[must_use]
    pub fn normalized(self) -> VisibleRange {
        let start = self.anchor.min(self.focus);
        let end = self.anchor.max(self.focus).saturating_add(1);
        VisibleRange::new(start, end)
    }

    fn clamp(self, item_count: u32) -> Option<Self> {
        if item_count == 0 {
            return None;
        }
        let last = item_count.saturating_sub(1);
        Some(Self {
            anchor: self.anchor.min(last),
            focus: self.focus.min(last),
        })
    }
}

/// Cursor movement request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    /// Move to the first item.
    First,
    /// Move to the last item.
    Last,
    /// Move up or left by one item.
    Previous,
    /// Move down or right by one item.
    Next,
    /// Move one page toward the start.
    PageUp,
    /// Move one page toward the end.
    PageDown,
    /// Move by an arbitrary signed delta.
    Delta(i32),
}

/// Navigation request from a renderer adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationRequest {
    /// Apply an enabled movement.
    Move(Movement),
    /// Reject movement because the owning widget is disabled.
    Disabled,
}

/// Renderer family consuming a viewport projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    /// Ratatui or other terminal renderer.
    Tui,
    /// Dioxus, DOM, or other web renderer.
    Web,
}

/// Widget family consuming a viewport projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetViewportKind {
    /// Linear list or select-list widget.
    List,
    /// Table or grid-like row viewport.
    Table,
    /// Tree or outline visible-row viewport.
    Tree,
    /// Fuzzy search result viewport.
    FuzzyResults,
    /// Command palette result viewport.
    CommandPalette,
}

/// Viewport construction and navigation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportError {
    /// A renderer reported an empty viewport extent.
    ZeroViewportExtent,
    /// Cursor index is outside the item collection.
    CursorOutOfBounds {
        /// Invalid cursor index.
        index: u32,
    },
    /// Scroll offset is outside the item collection.
    ScrollOutOfBounds {
        /// Invalid scroll offset.
        offset: u32,
    },
    /// Anchor index is outside the item collection.
    AnchorOutOfBounds {
        /// Invalid anchor index.
        index: u32,
    },
    /// Selection endpoint is outside the item collection.
    SelectionOutOfBounds {
        /// Invalid selection endpoint.
        index: u32,
    },
    /// The owning renderer widget rejected movement because it is disabled.
    DisabledMovement,
}

mod bridge;
mod model;
mod virtualization;
pub use bridge::ViewportAdapterInput;
pub use bridge::ViewportAdapterProjection;
pub use bridge::project_viewport;
pub use model::ViewportParts;
pub use model::ViewportState;
pub use virtualization::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
