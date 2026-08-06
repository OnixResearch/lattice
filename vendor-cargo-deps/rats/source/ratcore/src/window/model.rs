use super::*;

/// Input parts for rebuilding viewport state from an owning widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportParts {
    /// Number of logical items.
    pub item_count: u32,
    /// Logical viewport extent supplied by renderer-owned measurement.
    pub viewport_extent: u32,
    /// Cursor index supplied by widget state.
    pub cursor: u32,
    /// Scroll offset supplied by widget state.
    pub scroll_offset: u32,
    /// Optional selection anchor supplied by widget state.
    pub anchor: Option<u32>,
    /// Optional selection supplied by widget state.
    pub selection: Option<SelectionRange>,
}

/// Pure viewport state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportState {
    item_count: u32,
    viewport_extent: u32,
    cursor: u32,
    scroll_offset: u32,
    anchor: Option<u32>,
    selection: Option<SelectionRange>,
}

impl ViewportState {
    /// Creates a viewport state over `item_count` items and a non-zero extent.
    ///
    /// # Errors
    ///
    /// Returns [`ViewportError::ZeroViewportExtent`] when `viewport_extent` is zero.
    pub fn new(item_count: u32, viewport_extent: u32) -> Result<Self, ViewportError> {
        if viewport_extent == 0 {
            return Err(ViewportError::ZeroViewportExtent);
        }
        Ok(Self {
            item_count,
            viewport_extent,
            cursor: 0,
            scroll_offset: 0,
            anchor: None,
            selection: None,
        })
    }

    /// Creates a viewport state from external widget state.
    ///
    /// # Errors
    ///
    /// Returns a [`ViewportError`] when the viewport extent is zero or when
    /// cursor, scroll, anchor, or selection indices are outside the collection.
    pub fn from_parts(parts: ViewportParts) -> Result<Self, ViewportError> {
        if parts.viewport_extent == 0 {
            return Err(ViewportError::ZeroViewportExtent);
        }
        if parts.item_count == 0 {
            return Ok(Self {
                item_count: parts.item_count,
                viewport_extent: parts.viewport_extent,
                cursor: 0,
                scroll_offset: 0,
                anchor: None,
                selection: None,
            });
        }
        if parts.cursor >= parts.item_count {
            return Err(ViewportError::CursorOutOfBounds { index: parts.cursor });
        }
        if parts.scroll_offset >= parts.item_count {
            return Err(ViewportError::ScrollOutOfBounds {
                offset: parts.scroll_offset,
            });
        }
        if let Some(anchor_index) = parts.anchor
            && anchor_index >= parts.item_count
        {
            return Err(ViewportError::AnchorOutOfBounds { index: anchor_index });
        }
        if let Some(selection_range) = parts.selection {
            SelectionRange::new(selection_range.anchor, selection_range.focus, parts.item_count)?;
        }
        Ok(Self {
            item_count: parts.item_count,
            viewport_extent: parts.viewport_extent,
            cursor: parts.cursor,
            scroll_offset: parts.scroll_offset,
            anchor: parts.anchor,
            selection: parts.selection,
        }
        .with_cursor_visible())
    }

    /// Returns the item count.
    #[must_use]
    pub fn item_count(&self) -> u32 {
        self.item_count
    }

    /// Returns the viewport extent in logical rows/items.
    #[must_use]
    pub fn viewport_extent(&self) -> u32 {
        self.viewport_extent
    }

    /// Returns the cursor index.
    #[must_use]
    pub fn cursor(&self) -> u32 {
        self.cursor
    }

    /// Returns the scroll offset.
    #[must_use]
    pub fn scroll_offset(&self) -> u32 {
        self.scroll_offset
    }

    /// Returns the optional anchor index.
    #[must_use]
    pub fn anchor(&self) -> Option<u32> {
        self.anchor
    }

    /// Returns the optional selection range.
    #[must_use]
    pub fn selection(&self) -> Option<SelectionRange> {
        self.selection
    }

    /// Returns the current half-open visible range.
    #[must_use]
    pub fn visible_range(&self) -> VisibleRange {
        if self.item_count == 0 {
            return VisibleRange::new(0, 0);
        }
        let start = self.scroll_offset.min(self.item_count);
        let end = start.saturating_add(self.viewport_extent).min(self.item_count);
        VisibleRange::new(start, end)
    }

    /// Moves the cursor, clamping at collection boundaries and preserving visibility.
    #[must_use]
    pub fn move_cursor(mut self, movement: Movement) -> Self {
        self.cursor = self.moved_cursor(movement);
        self.with_cursor_visible()
    }

    /// Applies an enabled or disabled navigation request.
    ///
    /// # Errors
    ///
    /// Returns [`ViewportError::DisabledMovement`] when the owning widget has
    /// disabled navigation for the request.
    pub fn apply_navigation(self, request: NavigationRequest) -> Result<Self, ViewportError> {
        match request {
            NavigationRequest::Move(movement) => Ok(self.move_cursor(movement)),
            NavigationRequest::Disabled => Err(ViewportError::DisabledMovement),
        }
    }

    /// Starts a range selection at the current cursor.
    #[must_use]
    pub fn begin_selection(mut self) -> Self {
        if self.item_count == 0 {
            self.anchor = None;
            self.selection = None;
            return self;
        }
        self.anchor = Some(self.cursor);
        self.selection = Some(SelectionRange {
            anchor: self.cursor,
            focus: self.cursor,
        });
        self
    }

    /// Extends the selection after applying movement.
    #[must_use]
    pub fn extend_selection(mut self, movement: Movement) -> Self {
        self = self.move_cursor(movement);
        let Some(anchor) = self.anchor else {
            return self.begin_selection();
        };
        if self.item_count == 0 {
            self.selection = None;
            return self;
        }
        self.selection = Some(SelectionRange {
            anchor,
            focus: self.cursor,
        });
        self
    }

    /// Clears the anchor and active selection.
    #[must_use]
    pub fn clear_selection(mut self) -> Self {
        self.anchor = None;
        self.selection = None;
        self
    }

    /// Clamps state after the backing collection changes size.
    #[must_use]
    pub fn clamp_to_item_count(mut self, item_count: u32) -> Self {
        self.item_count = item_count;
        if item_count == 0 {
            self.cursor = 0;
            self.scroll_offset = 0;
            self.anchor = None;
            self.selection = None;
            return self;
        }
        let last = item_count.saturating_sub(1);
        self.cursor = self.cursor.min(last);
        self.scroll_offset = self.scroll_offset.min(last);
        self.anchor = self.anchor.map(|anchor| anchor.min(last));
        self.selection = self.selection.and_then(|selection| selection.clamp(item_count));
        self.with_cursor_visible()
    }

    fn moved_cursor(&self, movement: Movement) -> u32 {
        if self.item_count == 0 {
            return 0;
        }
        let last = self.item_count.saturating_sub(1);
        match movement {
            Movement::First => 0,
            Movement::Last => last,
            Movement::Previous => self.cursor.saturating_sub(1),
            Movement::Next => self.cursor.saturating_add(1).min(last),
            Movement::PageUp => self.cursor.saturating_sub(self.viewport_extent),
            Movement::PageDown => self.cursor.saturating_add(self.viewport_extent).min(last),
            Movement::Delta(delta) => apply_delta(self.cursor, delta, last),
        }
    }

    fn with_cursor_visible(mut self) -> Self {
        if self.item_count == 0 {
            self.scroll_offset = 0;
            return self;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
            return self;
        }
        let visible_end = self.scroll_offset.saturating_add(self.viewport_extent);
        if self.cursor >= visible_end {
            self.scroll_offset = self.cursor.saturating_add(1).saturating_sub(self.viewport_extent);
        }
        let max_scroll = self.item_count.saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
        self
    }
}

fn apply_delta(cursor: u32, delta: i32, last: u32) -> u32 {
    if delta.is_negative() {
        cursor.saturating_sub(delta.unsigned_abs())
    } else {
        cursor.saturating_add(delta.unsigned_abs()).min(last)
    }
}
