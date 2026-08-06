//! Breadcrumb path model, truncation, and renderer.
//!
//! Breadcrumbs display path-like segments with clamped navigation and left-side
//! truncation when the path is wider than the available terminal cells.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

const DEFAULT_SEPARATOR: char = '/';
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: u16 = 1;
const MIN_SEGMENTS_FOR_ELLIPSIS: usize = 2;

/// Pure breadcrumb state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbModel {
    /// Path segments.
    pub segments: Vec<String>,
    /// Active segment index.
    pub active: usize,
    /// Separator rendered between segments.
    pub separator: char,
}

impl Default for BreadcrumbModel {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            active: 0,
            separator: DEFAULT_SEPARATOR,
        }
    }
}

impl BreadcrumbModel {
    /// Create a breadcrumb model from path segments.
    pub fn new<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            segments: segments.into_iter().map(Into::into).collect(),
            active: 0,
            separator: DEFAULT_SEPARATOR,
        }
    }

    /// Set active index, clamping to the last segment.
    pub fn with_active(mut self, active: usize) -> Self {
        self.set_active(active);
        self
    }

    /// Set separator character.
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Set active index, clamping to the last segment.
    pub fn set_active(&mut self, active: usize) -> Option<usize> {
        let last_index = self.segments.len().checked_sub(1)?;
        self.active = active.min(last_index);
        Some(self.active)
    }

    /// Move active segment left, clamping at the first segment.
    pub fn move_left(&mut self) -> Option<usize> {
        if self.segments.is_empty() {
            return None;
        }
        self.active = self.active.saturating_sub(1);
        Some(self.active)
    }

    /// Move active segment right, clamping at the last segment.
    pub fn move_right(&mut self) -> Option<usize> {
        let last_index = self.segments.len().checked_sub(1)?;
        self.active = self.active.saturating_add(1).min(last_index);
        Some(self.active)
    }

    /// Return the active segment index and label.
    pub fn select(&self) -> Option<(usize, String)> {
        self.segments.get(self.active).map(|label| (self.active, label.clone()))
    }
}

/// Visual style for breadcrumbs.
#[derive(Debug, Clone)]
pub struct BreadcrumbStyle {
    /// Background fill style.
    pub background: Style,
    /// Inactive segment style.
    pub segment: Style,
    /// Active segment style.
    pub active_segment: Style,
    /// Separator style.
    pub separator: Style,
    /// Ellipsis style.
    pub ellipsis: Style,
}

impl Default for BreadcrumbStyle {
    fn default() -> Self {
        Self {
            background: Style::default().bg(Color::Black),
            segment: Style::default().fg(Color::Gray),
            active_segment: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            separator: Style::default().fg(Color::DarkGray),
            ellipsis: Style::default().fg(Color::DarkGray),
        }
    }
}

impl BreadcrumbStyle {
    /// Set background style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }

    /// Set inactive segment style.
    pub fn with_segment(mut self, segment: Style) -> Self {
        self.segment = segment;
        self
    }

    /// Set active segment style.
    pub fn with_active_segment(mut self, active_segment: Style) -> Self {
        self.active_segment = active_segment;
        self
    }

    /// Set separator style.
    pub fn with_separator(mut self, separator: Style) -> Self {
        self.separator = separator;
        self
    }

    /// Set ellipsis style.
    pub fn with_ellipsis(mut self, ellipsis: Style) -> Self {
        self.ellipsis = ellipsis;
        self
    }
}

/// Layout for one rendered segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreadcrumbSegmentLayout {
    /// Index into `BreadcrumbModel::segments`.
    pub index: usize,
    /// Segment rectangle.
    pub rect: Rect,
}

/// Computed breadcrumb layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BreadcrumbLayout {
    /// First visible segment index.
    pub visible_start: usize,
    /// True when an ellipsis prefix is rendered.
    pub has_ellipsis: bool,
    /// Rendered segment rectangles.
    pub segments: Vec<BreadcrumbSegmentLayout>,
}

/// Compute visible breadcrumb segments without rendering.
pub fn compute_breadcrumb_layout(area: Rect, model: &BreadcrumbModel) -> BreadcrumbLayout {
    if area.width == 0 || area.height == 0 || model.segments.is_empty() {
        return BreadcrumbLayout::default();
    }

    let visible_start = visible_start_for_width(model, area.width);
    let has_ellipsis = visible_start > 0;
    let mut cursor = area.x;
    let right = rect_right(area);
    let mut segments = Vec::new();

    if has_ellipsis {
        cursor = cursor.saturating_add(ELLIPSIS_WIDTH.min(right.saturating_sub(cursor)));
        if cursor < right {
            cursor = cursor.saturating_add(separator_width(model).min(right.saturating_sub(cursor)));
        }
    }

    for index in visible_start..model.segments.len() {
        if index > visible_start {
            cursor = cursor.saturating_add(separator_width(model).min(right.saturating_sub(cursor)));
        }
        if cursor >= right {
            break;
        }

        let width = text_width_u16(&model.segments[index]).min(right.saturating_sub(cursor));
        if width == 0 {
            break;
        }
        segments.push(BreadcrumbSegmentLayout {
            index,
            rect: Rect::new(cursor, area.y, width, 1),
        });
        cursor = cursor.saturating_add(width);
    }

    BreadcrumbLayout {
        visible_start,
        has_ellipsis,
        segments,
    }
}

/// Render breadcrumbs and return the computed layout.
pub fn render_breadcrumb(
    frame: &mut Frame,
    area: Rect,
    model: &BreadcrumbModel,
    style: &BreadcrumbStyle,
) -> BreadcrumbLayout {
    let layout = compute_breadcrumb_layout(area, model);

    if area.width == 0 || area.height == 0 {
        return layout;
    }

    let buffer = frame.buffer_mut();
    fill_area(buffer, area, style.background);
    render_breadcrumb_content(buffer, area, model, style, &layout);

    layout
}

fn render_breadcrumb_content(
    buffer: &mut Buffer,
    area: Rect,
    model: &BreadcrumbModel,
    style: &BreadcrumbStyle,
    layout: &BreadcrumbLayout,
) {
    let mut cursor = area.x;
    let right = rect_right(area);

    if layout.has_ellipsis && cursor < right {
        buffer.set_string(cursor, area.y, ELLIPSIS, style.ellipsis.patch(style.background));
        cursor = cursor.saturating_add(ELLIPSIS_WIDTH);
        render_separator(buffer, &mut cursor, right, area.y, model, style);
    }

    for segment in &layout.segments {
        if segment.index > layout.visible_start {
            render_separator(buffer, &mut cursor, right, area.y, model, style);
        }
        if cursor >= right {
            break;
        }

        let segment_style = if segment.index == model.active {
            style.active_segment
        } else {
            style.segment
        }
        .patch(style.background);
        let text = truncate_to_width(&model.segments[segment.index], segment.rect.width);
        buffer.set_string(segment.rect.x, segment.rect.y, text, segment_style);
        cursor = segment.rect.x.saturating_add(segment.rect.width);
    }
}

fn render_separator(
    buffer: &mut Buffer,
    cursor: &mut u16,
    right: u16,
    y: u16,
    model: &BreadcrumbModel,
    style: &BreadcrumbStyle,
) {
    if *cursor >= right {
        return;
    }
    buffer.set_string(*cursor, y, model.separator.to_string(), style.separator.patch(style.background));
    *cursor = cursor.saturating_add(separator_width(model));
}

fn visible_start_for_width(model: &BreadcrumbModel, width: u16) -> usize {
    if model.segments.len() < MIN_SEGMENTS_FOR_ELLIPSIS || total_width(model, 0, false) <= width {
        return 0;
    }

    for start in 1..model.segments.len() {
        if total_width(model, start, true) <= width {
            return start;
        }
    }

    model.segments.len().saturating_sub(1)
}

fn total_width(model: &BreadcrumbModel, start: usize, ellipsis: bool) -> u16 {
    let mut width = if ellipsis { ELLIPSIS_WIDTH } else { 0 };
    if ellipsis && start < model.segments.len() {
        width = width.saturating_add(separator_width(model));
    }

    for index in start..model.segments.len() {
        if index > start {
            width = width.saturating_add(separator_width(model));
        }
        width = width.saturating_add(text_width_u16(&model.segments[index]));
    }
    width
}

fn separator_width(model: &BreadcrumbModel) -> u16 {
    UnicodeWidthChar::width(model.separator).unwrap_or_default().min(usize::from(u16::MAX)) as u16
}

fn truncate_to_width(text: &str, max_width: u16) -> String {
    let max_width = usize::from(max_width);
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    if max_width <= usize::from(ELLIPSIS_WIDTH) {
        return ELLIPSIS.to_string();
    }

    let content_width = max_width - usize::from(ELLIPSIS_WIDTH);
    let mut used_width = 0usize;
    let mut output = String::new();
    for character in text.chars() {
        let width = UnicodeWidthChar::width(character).unwrap_or_default();
        if used_width + width > content_width {
            break;
        }
        used_width += width;
        output.push(character);
    }
    output.push_str(ELLIPSIS);
    output
}

fn text_width_u16(text: &str) -> u16 {
    let width = UnicodeWidthStr::width(text);
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

    use super::*;

    const AREA_X: u16 = 0;
    const AREA_Y: u16 = 0;
    const AREA_HEIGHT: u16 = 1;
    const WIDE_WIDTH: u16 = 20;
    const TRUNCATED_WIDTH: u16 = 9;
    const SINGLE_SEGMENT_WIDTH: u16 = 4;
    const ACTIVE_INDEX: usize = 1;
    const LAST_INDEX: usize = 2;
    const PROJECT_INDEX: usize = 2;
    const EXPECTED_PROJECT_X_WITH_ELLIPSIS: u16 = 2;
    const ACTIVE_RENDER_X: u16 = 5;

    #[test]
    fn navigation_clamps_at_boundaries() {
        let mut model = sample_model().with_active(ACTIVE_INDEX);

        assert_eq!(model.move_left(), Some(0));
        assert_eq!(model.move_left(), Some(0));
        assert_eq!(model.move_right(), Some(ACTIVE_INDEX));
        assert_eq!(model.move_right(), Some(LAST_INDEX));
        assert_eq!(model.move_right(), Some(LAST_INDEX));
    }

    #[test]
    fn select_returns_active_index_and_label() {
        let model = sample_model().with_active(ACTIVE_INDEX);

        assert_eq!(model.select(), Some((ACTIVE_INDEX, "user".to_string())));
    }

    #[test]
    fn truncates_from_left_and_prepends_ellipsis() {
        let model = sample_model();
        let area = Rect::new(AREA_X, AREA_Y, TRUNCATED_WIDTH, AREA_HEIGHT);

        let layout = compute_breadcrumb_layout(area, &model);

        assert!(layout.has_ellipsis);
        assert_eq!(layout.visible_start, PROJECT_INDEX);
        assert_eq!(layout.segments[0].rect.x, EXPECTED_PROJECT_X_WITH_ELLIPSIS);
    }

    #[test]
    fn single_oversized_segment_truncates_without_ellipsis_prefix() {
        let model = BreadcrumbModel::new(["verylongsegment"]);
        let area = Rect::new(AREA_X, AREA_Y, SINGLE_SEGMENT_WIDTH, AREA_HEIGHT);

        let layout = compute_breadcrumb_layout(area, &model);

        assert!(!layout.has_ellipsis);
        assert_eq!(layout.visible_start, 0);
        assert_eq!(layout.segments[0].rect.width, SINGLE_SEGMENT_WIDTH);
    }

    #[test]
    fn render_uses_active_segment_style() {
        let model = sample_model().with_active(ACTIVE_INDEX);
        let style = BreadcrumbStyle::default().with_active_segment(Style::default().fg(Color::Red));

        let buffer = render_to_buffer(WIDE_WIDTH, &model, &style);

        assert_eq!(buffer[(ACTIVE_RENDER_X, AREA_Y)].symbol(), "u");
        assert_eq!(buffer[(ACTIVE_RENDER_X, AREA_Y)].fg, Color::Red);
    }

    #[test]
    fn empty_segments_render_background_only() {
        let model = BreadcrumbModel::default();
        let style = BreadcrumbStyle::default().with_background(Style::default().bg(Color::Blue));

        let buffer = render_to_buffer(WIDE_WIDTH, &model, &style);

        assert_eq!(buffer[(AREA_X, AREA_Y)].symbol(), " ");
        assert_eq!(buffer[(AREA_X, AREA_Y)].bg, Color::Blue);
    }

    fn sample_model() -> BreadcrumbModel {
        BreadcrumbModel::new(["home", "user", "project"])
    }

    fn render_to_buffer(width: u16, model: &BreadcrumbModel, style: &BreadcrumbStyle) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, AREA_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_breadcrumb(frame, area, model, style);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}
