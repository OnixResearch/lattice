//! Tooltip model, wrapping, placement, and renderer.
//!
//! Tooltips render below their anchor by default and flip above the anchor when
//! the requested box would overflow the viewport bottom edge.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

const MIN_TOOLTIP_CONTENT_WIDTH: u16 = 1;
const DEFAULT_TOOLTIP_MAX_WIDTH: u16 = 40;
const BORDER_THICKNESS: u16 = 1;
const BORDER_SIZE: u16 = BORDER_THICKNESS * 2;
const ANCHOR_BELOW_OFFSET: u16 = 1;

/// Tooltip anchor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TooltipAnchor {
    /// Anchor x coordinate.
    pub x: u16,
    /// Anchor y coordinate.
    pub y: u16,
}

impl TooltipAnchor {
    /// Create a tooltip anchor.
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// Tooltip border mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipBorder {
    /// No border; content occupies the full outer rect.
    None,
    /// Draw a standard ratatui border.
    #[default]
    Plain,
}

/// Tooltip visual style.
#[derive(Debug, Clone)]
pub struct TooltipStyle {
    /// Background fill style.
    pub background: Style,
    /// Content text style.
    pub text: Style,
    /// Border style.
    pub border: Style,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            background: Style::default().bg(Color::Black),
            text: Style::default().fg(Color::White),
            border: Style::default().fg(Color::Gray),
        }
    }
}

impl TooltipStyle {
    /// Set background style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }

    /// Set text style.
    pub fn with_text(mut self, text: Style) -> Self {
        self.text = text;
        self
    }

    /// Set border style.
    pub fn with_border(mut self, border: Style) -> Self {
        self.border = border;
        self
    }
}

/// Tooltip configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tooltip {
    /// Anchor position.
    pub anchor: TooltipAnchor,
    /// Content text. Newlines are preserved before wrapping.
    pub content: String,
    /// Maximum content width before wrapping.
    pub max_width: u16,
    /// Border mode.
    pub border: TooltipBorder,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            anchor: TooltipAnchor::default(),
            content: String::new(),
            max_width: DEFAULT_TOOLTIP_MAX_WIDTH,
            border: TooltipBorder::Plain,
        }
    }
}

impl Tooltip {
    /// Create a tooltip with content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Self::default()
        }
    }

    /// Set anchor position.
    pub fn with_anchor(mut self, anchor: TooltipAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set content text.
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set maximum content width.
    pub fn with_max_width(mut self, max_width: u16) -> Self {
        self.max_width = max_width.max(MIN_TOOLTIP_CONTENT_WIDTH);
        self
    }

    /// Set border mode.
    pub fn with_border(mut self, border: TooltipBorder) -> Self {
        self.border = border;
        self
    }
}

/// Computed tooltip layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TooltipLayout {
    /// Full tooltip rect including optional border.
    pub outer: Rect,
    /// Content rect inside the optional border.
    pub inner: Rect,
    /// Wrapped content lines.
    pub lines: Vec<String>,
    /// True when the tooltip was placed above the anchor.
    pub flipped: bool,
}

/// Compute tooltip layout and wrapped content without rendering.
pub fn compute_tooltip_layout(viewport: Rect, tooltip: &Tooltip) -> TooltipLayout {
    if viewport.width == 0 || viewport.height == 0 {
        return TooltipLayout::default();
    }

    let lines = wrap_tooltip_content(&tooltip.content, tooltip.max_width.max(MIN_TOOLTIP_CONTENT_WIDTH));
    let content_width = lines
        .iter()
        .map(|line| text_width_u16(line))
        .max()
        .unwrap_or(MIN_TOOLTIP_CONTENT_WIDTH)
        .max(MIN_TOOLTIP_CONTENT_WIDTH)
        .min(viewport.width);
    let content_height = usize_to_u16(lines.len()).max(1).min(viewport.height);
    let border_size = tooltip_border_size(tooltip.border);
    let outer_width = content_width.saturating_add(border_size).min(viewport.width);
    let outer_height = content_height.saturating_add(border_size).min(viewport.height);
    let max_x = rect_right(viewport).saturating_sub(outer_width).max(viewport.x);
    let x = tooltip.anchor.x.clamp(viewport.x, max_x);
    let below_y = tooltip.anchor.y.saturating_add(ANCHOR_BELOW_OFFSET);
    let below_bottom = below_y.saturating_add(outer_height);
    let viewport_bottom = rect_bottom(viewport);
    let should_flip = below_bottom > viewport_bottom && tooltip.anchor.y >= outer_height;
    let y = if should_flip {
        tooltip.anchor.y.saturating_sub(outer_height)
    } else {
        below_y.clamp(viewport.y, viewport_bottom.saturating_sub(outer_height).max(viewport.y))
    };
    let outer = Rect::new(x, y, outer_width, outer_height);
    let inner = tooltip_inner_rect(outer, tooltip.border);

    TooltipLayout {
        outer,
        inner,
        lines,
        flipped: should_flip,
    }
}

/// Render a tooltip and return its computed layout.
pub fn render_tooltip(frame: &mut Frame, viewport: Rect, tooltip: &Tooltip, style: &TooltipStyle) -> TooltipLayout {
    let layout = compute_tooltip_layout(viewport, tooltip);

    if layout.outer.width == 0 || layout.outer.height == 0 {
        return layout;
    }

    frame.render_widget(Clear, layout.outer);
    fill_area(frame.buffer_mut(), layout.outer, style.background);

    if tooltip.border == TooltipBorder::Plain {
        let block = Block::default().borders(Borders::ALL).border_style(style.border);
        frame.render_widget(block, layout.outer);
    }

    render_tooltip_lines(frame.buffer_mut(), &layout, style.text.patch(style.background));

    layout
}

fn render_tooltip_lines(buffer: &mut Buffer, layout: &TooltipLayout, style: Style) {
    for (index, line) in layout.lines.iter().enumerate() {
        let y = layout.inner.y.saturating_add(usize_to_u16(index));
        if y >= rect_bottom(layout.inner) {
            break;
        }
        let text = truncate_to_width(line, layout.inner.width);
        buffer.set_string(layout.inner.x, y, text, style);
    }
}

fn wrap_tooltip_content(content: &str, max_width: u16) -> Vec<String> {
    let max_width = max_width.max(MIN_TOOLTIP_CONTENT_WIDTH);
    let mut lines = Vec::new();

    for raw_line in content.split('\n') {
        wrap_single_line(raw_line, max_width, &mut lines);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn wrap_single_line(raw_line: &str, max_width: u16, lines: &mut Vec<String>) {
    if raw_line.is_empty() {
        lines.push(String::new());
        return;
    }

    let max_width = usize::from(max_width);
    let mut current = String::new();
    let mut current_width = 0usize;

    for character in raw_line.chars() {
        let width = UnicodeWidthChar::width(character).unwrap_or_default();
        if current_width > 0 && current_width.saturating_add(width) > max_width {
            lines.push(current);
            current = String::new();
            current_width = 0;
        }
        current.push(character);
        current_width = current_width.saturating_add(width);
    }

    if !current.is_empty() {
        lines.push(current);
    }
}

fn tooltip_border_size(border: TooltipBorder) -> u16 {
    match border {
        TooltipBorder::None => 0,
        TooltipBorder::Plain => BORDER_SIZE,
    }
}

fn tooltip_inner_rect(outer: Rect, border: TooltipBorder) -> Rect {
    match border {
        TooltipBorder::None => outer,
        TooltipBorder::Plain => Block::default().borders(Borders::ALL).inner(outer),
    }
}

fn truncate_to_width(text: &str, max_width: u16) -> String {
    let max_width = usize::from(max_width);
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    let mut output = String::new();
    let mut used_width = 0usize;
    for character in text.chars() {
        let width = UnicodeWidthChar::width(character).unwrap_or_default();
        if used_width.saturating_add(width) > max_width {
            break;
        }
        output.push(character);
        used_width = used_width.saturating_add(width);
    }
    output
}

fn text_width_u16(text: &str) -> u16 {
    let width = UnicodeWidthStr::width(text);
    width.min(usize::from(u16::MAX)) as u16
}

fn usize_to_u16(value: usize) -> u16 {
    value.min(usize::from(u16::MAX)) as u16
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

    use super::*;

    const VIEWPORT_X: u16 = 0;
    const VIEWPORT_Y: u16 = 0;
    const VIEWPORT_WIDTH: u16 = 20;
    const VIEWPORT_HEIGHT: u16 = 6;
    const ANCHOR_X: u16 = 2;
    const TOP_ANCHOR_Y: u16 = 1;
    const BOTTOM_ANCHOR_Y: u16 = 5;
    const HELLO_WIDTH: u16 = 5;
    const PLAIN_BORDER_OUTER_WIDTH: u16 = HELLO_WIDTH + BORDER_SIZE;
    const PLAIN_BORDER_OUTER_HEIGHT: u16 = 3;
    const WRAP_WIDTH: u16 = 3;
    const EXPECTED_WRAP_LINES: usize = 2;
    const EXPECTED_FLIPPED_Y: u16 = BOTTOM_ANCHOR_Y - PLAIN_BORDER_OUTER_HEIGHT;
    const NO_BORDER_TEXT_X: u16 = ANCHOR_X;
    const NO_BORDER_TEXT_Y: u16 = TOP_ANCHOR_Y + ANCHOR_BELOW_OFFSET;
    const BORDER_TEXT_X: u16 = ANCHOR_X + BORDER_THICKNESS;
    const BORDER_TEXT_Y: u16 = TOP_ANCHOR_Y + ANCHOR_BELOW_OFFSET + BORDER_THICKNESS;

    #[test]
    fn computes_width_from_content_and_border() {
        let tooltip = Tooltip::new("hello")
            .with_anchor(TooltipAnchor::new(ANCHOR_X, TOP_ANCHOR_Y))
            .with_max_width(DEFAULT_TOOLTIP_MAX_WIDTH);

        let layout = compute_tooltip_layout(test_viewport(), &tooltip);

        assert_eq!(layout.outer.width, PLAIN_BORDER_OUTER_WIDTH);
        assert_eq!(layout.outer.height, PLAIN_BORDER_OUTER_HEIGHT);
        assert_eq!(layout.inner.width, HELLO_WIDTH);
    }

    #[test]
    fn flips_above_anchor_when_bottom_would_overflow() {
        let tooltip = Tooltip::new("hello")
            .with_anchor(TooltipAnchor::new(ANCHOR_X, BOTTOM_ANCHOR_Y))
            .with_max_width(DEFAULT_TOOLTIP_MAX_WIDTH);

        let layout = compute_tooltip_layout(test_viewport(), &tooltip);

        assert!(layout.flipped);
        assert_eq!(layout.outer.y, EXPECTED_FLIPPED_Y);
    }

    #[test]
    fn wraps_content_at_max_width() {
        let tooltip = Tooltip::new("abcdef").with_border(TooltipBorder::None).with_max_width(WRAP_WIDTH);

        let layout = compute_tooltip_layout(test_viewport(), &tooltip);

        assert_eq!(layout.lines, vec!["abc".to_string(), "def".to_string()]);
        assert_eq!(layout.lines.len(), EXPECTED_WRAP_LINES);
        assert_eq!(layout.outer.width, WRAP_WIDTH);
    }

    #[test]
    fn no_border_mode_uses_outer_rect_as_inner_rect() {
        let tooltip = Tooltip::new("hello")
            .with_anchor(TooltipAnchor::new(ANCHOR_X, TOP_ANCHOR_Y))
            .with_border(TooltipBorder::None);

        let layout = compute_tooltip_layout(test_viewport(), &tooltip);

        assert_eq!(layout.inner, layout.outer);
    }

    #[test]
    fn zero_max_width_is_clamped_to_minimum() {
        let tooltip = Tooltip::new("hello").with_max_width(0);

        let layout = compute_tooltip_layout(test_viewport(), &tooltip);

        assert!(layout.inner.width >= MIN_TOOLTIP_CONTENT_WIDTH);
    }

    #[test]
    fn render_plain_border_and_content() {
        let tooltip = Tooltip::new("hello").with_anchor(TooltipAnchor::new(ANCHOR_X, TOP_ANCHOR_Y));

        let buffer = render_to_buffer(&tooltip);

        assert_eq!(buffer[(ANCHOR_X, NO_BORDER_TEXT_Y)].symbol(), "┌");
        assert_eq!(buffer[(BORDER_TEXT_X, BORDER_TEXT_Y)].symbol(), "h");
    }

    #[test]
    fn render_no_border_content_at_outer_origin() {
        let tooltip = Tooltip::new("hello")
            .with_anchor(TooltipAnchor::new(ANCHOR_X, TOP_ANCHOR_Y))
            .with_border(TooltipBorder::None);

        let buffer = render_to_buffer(&tooltip);

        assert_eq!(buffer[(NO_BORDER_TEXT_X, NO_BORDER_TEXT_Y)].symbol(), "h");
    }

    fn test_viewport() -> Rect {
        Rect::new(VIEWPORT_X, VIEWPORT_Y, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn render_to_buffer(tooltip: &Tooltip) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let viewport = frame.area();
                render_tooltip(frame, viewport, tooltip, &TooltipStyle::default());
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}
