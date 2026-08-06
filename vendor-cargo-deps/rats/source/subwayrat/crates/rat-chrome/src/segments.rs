//! Status bar layout and renderer.
//!
//! A status bar is non-interactive chrome: it fills its area, renders left and
//! right segments against the edges, and centers the middle segment in the
//! remaining non-overlapping space.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

const STATUS_BAR_HEIGHT: u16 = 1;

/// Non-interactive three-segment status bar.
#[derive(Debug, Clone, Default)]
pub struct StatusBar {
    /// Left-aligned spans.
    pub left: Vec<Span<'static>>,
    /// Centered spans.
    pub center: Vec<Span<'static>>,
    /// Right-aligned spans.
    pub right: Vec<Span<'static>>,
    /// Background style used to fill the full area.
    pub background: Style,
}

impl StatusBar {
    /// Create an empty status bar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set left-aligned spans.
    pub fn with_left(mut self, left: Vec<Span<'static>>) -> Self {
        self.left = left;
        self
    }

    /// Set centered spans.
    pub fn with_center(mut self, center: Vec<Span<'static>>) -> Self {
        self.center = center;
        self
    }

    /// Set right-aligned spans.
    pub fn with_right(mut self, right: Vec<Span<'static>>) -> Self {
        self.right = right;
        self
    }

    /// Set the background fill style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }
}

/// Computed status bar segment rectangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusBarLayout {
    /// Left segment rect.
    pub left: Rect,
    /// Center segment rect.
    pub center: Rect,
    /// Right segment rect.
    pub right: Rect,
}

/// Compute status bar segment layout without rendering.
pub fn compute_status_bar_layout(area: Rect, status: &StatusBar) -> StatusBarLayout {
    if area.width == 0 || area.height == 0 {
        return StatusBarLayout::default();
    }

    let left_full = line_width(&status.left);
    let right_full = line_width(&status.right);
    let center_full = line_width(&status.center);

    let right_width = right_full.min(area.width);
    let left_width = left_full.min(area.width.saturating_sub(right_width));
    let left_end = area.x.saturating_add(left_width);
    let right_x = rect_right(area).saturating_sub(right_width);
    let center_available = right_x.saturating_sub(left_end);
    let center_width = center_full.min(center_available);
    let preferred_center_x = area.x.saturating_add(area.width.saturating_sub(center_width) / 2);
    let max_center_x = right_x.saturating_sub(center_width);
    let center_x = preferred_center_x.clamp(left_end, max_center_x.max(left_end));

    StatusBarLayout {
        left: Rect::new(area.x, area.y, left_width, STATUS_BAR_HEIGHT),
        center: Rect::new(center_x, area.y, center_width, STATUS_BAR_HEIGHT),
        right: Rect::new(right_x, area.y, right_width, STATUS_BAR_HEIGHT),
    }
}

/// Render a status bar and return its computed segment layout.
pub fn render_status_bar(frame: &mut Frame, area: Rect, status: &StatusBar) -> StatusBarLayout {
    let layout = compute_status_bar_layout(area, status);

    if area.width == 0 || area.height == 0 {
        return layout;
    }

    let buffer = frame.buffer_mut();
    fill_area(buffer, area, status.background);
    render_segment(buffer, layout.left, &status.left, status.background);
    render_segment(buffer, layout.center, &status.center, status.background);
    render_segment(buffer, layout.right, &status.right, status.background);

    layout
}

fn render_segment(buffer: &mut Buffer, rect: Rect, spans: &[Span<'static>], background: Style) {
    if rect.width == 0 || rect.height == 0 || spans.is_empty() {
        return;
    }

    let patched = spans
        .iter()
        .cloned()
        .map(|mut span| {
            span.style = span.style.patch(background);
            span
        })
        .collect::<Vec<_>>();
    buffer.set_line(rect.x, rect.y, &Line::from(patched), rect.width);
}

fn line_width(spans: &[Span<'static>]) -> u16 {
    let width = Line::from(spans.to_vec()).width();
    width.min(usize::from(u16::MAX)) as u16
}

fn fill_area(buffer: &mut Buffer, area: Rect, style: Style) {
    for y in area.y..rect_bottom(area) {
        for x in area.x..rect_right(area) {
            buffer[(x, y)].set_char(' ').set_style(style);
        }
    }
}

fn rect_right(rect: Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn rect_bottom(rect: Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;

    use super::*;

    const AREA_X: u16 = 0;
    const AREA_Y: u16 = 0;
    const STANDARD_WIDTH: u16 = 20;
    const NARROW_WIDTH: u16 = 10;
    const STATUS_HEIGHT: u16 = 1;
    const CENTER_X_FOR_ONE_CELL: u16 = 9;
    const RIGHT_X_FOR_ONE_CELL: u16 = 19;
    const LEFT_WIDTH: u16 = 4;
    const RIGHT_WIDTH: u16 = 5;
    const CENTER_TRUNCATED_WIDTH: u16 = 1;
    const BACKGROUND_CHECK_X: u16 = 5;

    #[test]
    fn lays_out_three_regions_without_overlap() {
        let status = StatusBar::new()
            .with_left(vec![Span::raw("L")])
            .with_center(vec![Span::raw("C")])
            .with_right(vec![Span::raw("R")]);
        let area = Rect::new(AREA_X, AREA_Y, STANDARD_WIDTH, STATUS_HEIGHT);

        let layout = compute_status_bar_layout(area, &status);

        assert_eq!(layout.left, Rect::new(AREA_X, AREA_Y, 1, STATUS_HEIGHT));
        assert_eq!(layout.center, Rect::new(CENTER_X_FOR_ONE_CELL, AREA_Y, 1, STATUS_HEIGHT));
        assert_eq!(layout.right, Rect::new(RIGHT_X_FOR_ONE_CELL, AREA_Y, 1, STATUS_HEIGHT));
    }

    #[test]
    fn truncates_center_when_left_and_right_consume_available_space() {
        let status = StatusBar::new()
            .with_left(vec![Span::raw("left")])
            .with_center(vec![Span::raw("center")])
            .with_right(vec![Span::raw("right")]);
        let area = Rect::new(AREA_X, AREA_Y, NARROW_WIDTH, STATUS_HEIGHT);

        let layout = compute_status_bar_layout(area, &status);

        assert_eq!(layout.left.width, LEFT_WIDTH);
        assert_eq!(layout.right.width, RIGHT_WIDTH);
        assert_eq!(layout.center.width, CENTER_TRUNCATED_WIDTH);
    }

    #[test]
    fn renders_three_regions() {
        let status = StatusBar::new()
            .with_left(vec![Span::raw("L")])
            .with_center(vec![Span::raw("C")])
            .with_right(vec![Span::raw("R")]);

        let buffer = render_to_buffer(STANDARD_WIDTH, &status);

        assert_eq!(buffer[(AREA_X, AREA_Y)].symbol(), "L");
        assert_eq!(buffer[(CENTER_X_FOR_ONE_CELL, AREA_Y)].symbol(), "C");
        assert_eq!(buffer[(RIGHT_X_FOR_ONE_CELL, AREA_Y)].symbol(), "R");
    }

    #[test]
    fn empty_regions_still_fill_background() {
        let status = StatusBar::new().with_background(Style::default().bg(Color::Blue));

        let buffer = render_to_buffer(STANDARD_WIDTH, &status);

        assert_eq!(buffer[(BACKGROUND_CHECK_X, AREA_Y)].symbol(), " ");
        assert_eq!(buffer[(BACKGROUND_CHECK_X, AREA_Y)].bg, Color::Blue);
    }

    #[test]
    fn zero_width_area_returns_empty_layout() {
        let status = StatusBar::new().with_left(vec![Span::raw("L")]);
        let area = Rect::new(AREA_X, AREA_Y, 0, STATUS_HEIGHT);

        let layout = compute_status_bar_layout(area, &status);

        assert_eq!(layout, StatusBarLayout::default());
    }

    fn render_to_buffer(width: u16, status: &StatusBar) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, STATUS_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_status_bar(frame, area, status);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}
