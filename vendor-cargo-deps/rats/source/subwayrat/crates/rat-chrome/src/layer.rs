use ratatui::widgets::Widget;

#[path = "overlay/geometry.rs"]
mod geometry;
#[path = "overlay/paint.rs"]
mod paint;
#[path = "overlay/style.rs"]
mod style;
#[cfg(test)]
#[path = "overlay/tests.rs"]
mod tests;

const DEFAULT_OVERLAY_PERCENT: u16 = 50;

/// Anchoring positions for overlay placement within a viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchor {
    /// Center both horizontally and vertically.
    Center,
    /// Anchor to the top edge and center horizontally.
    Top,
    /// Anchor to the top-right corner.
    TopRight,
    /// Anchor to the right edge and center vertically.
    Right,
    /// Anchor to the bottom-right corner.
    BottomRight,
    /// Anchor to the bottom edge and center horizontally.
    Bottom,
    /// Anchor to the bottom-left corner.
    BottomLeft,
    /// Anchor to the left edge and center vertically.
    Left,
    /// Anchor to the top-left corner.
    TopLeft,
}

/// Width or height sizing relative to a viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlaySize {
    /// Fixed cell count.
    Fixed(u16),
    /// Percentage of the viewport, clamped to `0..=100`.
    Percent(u16),
}

/// Structural configuration for an overlay frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayModel {
    /// Overlay anchor within the viewport.
    pub anchor: OverlayAnchor,
    /// Requested overlay width.
    pub width: OverlaySize,
    /// Requested overlay height.
    pub height: OverlaySize,
    /// Horizontal offset applied after anchor positioning.
    pub offset_x: i16,
    /// Vertical offset applied after anchor positioning.
    pub offset_y: i16,
    /// Clear the overlay region before applying fill and chrome.
    pub clear: bool,
    /// Style the viewport outside the overlay rect with the backdrop style.
    pub dim_backdrop: bool,
    /// Draw border/title chrome.
    pub border: bool,
    /// Optional title rendered with border chrome.
    pub title: Option<String>,
}

impl Default for OverlayModel {
    fn default() -> Self {
        Self {
            anchor: OverlayAnchor::Center,
            width: OverlaySize::Percent(DEFAULT_OVERLAY_PERCENT),
            height: OverlaySize::Percent(DEFAULT_OVERLAY_PERCENT),
            offset_x: 0,
            offset_y: 0,
            clear: false,
            dim_backdrop: false,
            border: true,
            title: None,
        }
    }
}

impl OverlayModel {
    /// Set the overlay anchor.
    pub fn with_anchor(mut self, anchor: OverlayAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set the overlay width.
    pub fn with_width(mut self, width: OverlaySize) -> Self {
        self.width = width;
        self
    }

    /// Set the overlay height.
    pub fn with_height(mut self, height: OverlaySize) -> Self {
        self.height = height;
        self
    }

    /// Set both x/y offsets.
    pub fn with_offsets(mut self, x: i16, y: i16) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Set the horizontal offset.
    pub fn with_offset_x(mut self, x: i16) -> Self {
        self.offset_x = x;
        self
    }

    /// Set the vertical offset.
    pub fn with_offset_y(mut self, y: i16) -> Self {
        self.offset_y = y;
        self
    }

    /// Enable or disable clearing the overlay region before drawing.
    pub fn with_clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    /// Enable or disable backdrop styling outside the overlay rect.
    pub fn with_backdrop(mut self, dim_backdrop: bool) -> Self {
        self.dim_backdrop = dim_backdrop;
        self
    }

    /// Enable or disable border chrome.
    pub fn with_border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// Set the overlay title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Visual styling for an overlay frame.
#[derive(Debug, Clone)]
pub struct OverlayStyle {
    /// Style applied to the border chrome.
    pub border: ratatui::style::Style,
    /// Style applied to the title text.
    pub title: ratatui::style::Style,
    /// Style applied to viewport cells outside the overlay rect when enabled.
    pub backdrop: ratatui::style::Style,
    /// Style applied to the returned inner content rect.
    pub fill: ratatui::style::Style,
}

/// Computed layout returned by the overlay primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OverlayLayout {
    /// Full overlay rect including border/title chrome.
    pub outer: ratatui::layout::Rect,
    /// Child content rect after chrome is applied.
    pub inner: ratatui::layout::Rect,
}

/// Compute the overlay layout for a viewport without rendering it.
pub fn layout(viewport: ratatui::layout::Rect, model: &OverlayModel) -> OverlayLayout {
    let outer = geometry::compute_outer_rect(viewport, model);
    let inner = if model.border {
        ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .inner(outer)
    } else {
        outer
    };

    OverlayLayout { outer, inner }
}

/// Render an overlay frame and return its computed layout.
pub fn frame(
    frame: &mut ratatui::Frame,
    viewport: ratatui::layout::Rect,
    model: &OverlayModel,
    style: &OverlayStyle,
) -> OverlayLayout {
    render_frame(frame.buffer_mut(), viewport, model, style)
}

/// Render an overlay frame into a raw buffer and return its computed layout.
pub fn render_frame(
    buf: &mut ratatui::buffer::Buffer,
    viewport: ratatui::layout::Rect,
    model: &OverlayModel,
    style: &OverlayStyle,
) -> OverlayLayout {
    let layout = layout(viewport, model);
    debug_assert!(layout.outer.width <= viewport.width);
    debug_assert!(layout.outer.height <= viewport.height);

    if viewport.width == 0 || viewport.height == 0 {
        return layout;
    }

    paint_overlay(buf, viewport, model, style, layout);
    layout
}

fn paint_overlay(
    buf: &mut ratatui::buffer::Buffer,
    viewport: ratatui::layout::Rect,
    model: &OverlayModel,
    style: &OverlayStyle,
    layout: OverlayLayout,
) {
    if model.dim_backdrop {
        paint::backdrop(buf, viewport, layout.outer, style.backdrop);
    }
    if model.clear && layout.outer.width > 0 && layout.outer.height > 0 {
        ratatui::widgets::Clear.render(layout.outer, buf);
    }
    paint_inner(buf, layout.inner, style.fill);
    paint_border(buf, model, style, layout.outer);
}

fn paint_inner(
    buf: &mut ratatui::buffer::Buffer,
    inner: ratatui::layout::Rect,
    style: ratatui::style::Style,
) {
    if inner.width > 0 && inner.height > 0 {
        paint::rect(buf, inner, style);
    }
}

fn paint_border(
    buf: &mut ratatui::buffer::Buffer,
    model: &OverlayModel,
    style: &OverlayStyle,
    outer: ratatui::layout::Rect,
) {
    if !model.border {
        return;
    }

    let mut block = ratatui::widgets::Block::new()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(style.border);
    if let Some(title) = &model.title {
        block = block.title(ratatui::text::Line::from(ratatui::text::Span::styled(
            title.clone(),
            style.title,
        )));
    }
    block.render(outer, buf);
}
