//! Dialog frame model, layout, and renderer.
//!
//! Dialogs render a centered bordered frame, optional viewport dimming, a title
//! row, a button row, and return the content area for caller-owned body widgets.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear};

const DEFAULT_DIALOG_PERCENT: u16 = 50;
const MAX_PERCENT: u16 = 100;
const TITLE_ROW_HEIGHT: u16 = 1;
const BUTTON_ROW_HEIGHT: u16 = 1;
const BUTTON_PADDING_TEXT: &str = " ";
const BUTTON_PREFIX: &str = "[";
const BUTTON_SUFFIX: &str = "]";
const BUTTON_GAP: &str = "  ";

/// Dialog size request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogSize {
    /// Fixed terminal-cell dimensions.
    Fixed { width: u16, height: u16 },
    /// Percentage of viewport dimensions, clamped to `0..=100`.
    Percent { width: u16, height: u16 },
}

impl Default for DialogSize {
    fn default() -> Self {
        Self::Percent {
            width: DEFAULT_DIALOG_PERCENT,
            height: DEFAULT_DIALOG_PERCENT,
        }
    }
}

/// Pure dialog frame state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogModel {
    /// Dialog title.
    pub title: String,
    /// Button labels.
    pub buttons: Vec<String>,
    /// Focused button index.
    pub focused_button: usize,
    /// Requested dialog size.
    pub size: DialogSize,
    /// Whether to dim the viewport behind the dialog.
    pub dim: bool,
}

impl Default for DialogModel {
    fn default() -> Self {
        Self {
            title: String::new(),
            buttons: Vec::new(),
            focused_button: 0,
            size: DialogSize::default(),
            dim: false,
        }
    }
}

impl DialogModel {
    /// Create a dialog model.
    pub fn new<I, S>(title: impl Into<String>, buttons: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            title: title.into(),
            buttons: buttons.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set buttons.
    pub fn with_buttons<I, S>(mut self, buttons: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.buttons = buttons.into_iter().map(Into::into).collect();
        self.clamp_focus();
        self
    }

    /// Set focused button, clamping to the last button.
    pub fn with_focused_button(mut self, focused_button: usize) -> Self {
        self.focus_button(focused_button);
        self
    }

    /// Set size.
    pub fn with_size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    /// Set dimming.
    pub fn with_dim(mut self, dim: bool) -> Self {
        self.dim = dim;
        self
    }

    /// Focus a button, clamping to the last button.
    pub fn focus_button(&mut self, index: usize) -> Option<usize> {
        let last_index = self.buttons.len().checked_sub(1)?;
        self.focused_button = index.min(last_index);
        Some(self.focused_button)
    }

    /// Move button focus left, clamping at the first button.
    pub fn move_left(&mut self) -> Option<usize> {
        if self.buttons.is_empty() {
            return None;
        }
        self.focused_button = self.focused_button.saturating_sub(1);
        Some(self.focused_button)
    }

    /// Move button focus right, clamping at the last button.
    pub fn move_right(&mut self) -> Option<usize> {
        let last_index = self.buttons.len().checked_sub(1)?;
        self.focused_button = self.focused_button.saturating_add(1).min(last_index);
        Some(self.focused_button)
    }

    /// Activate the focused button, returning its index and label.
    pub fn activate(&self) -> Option<(usize, String)> {
        self.buttons
            .get(self.focused_button)
            .map(|label| (self.focused_button, label.clone()))
    }

    fn clamp_focus(&mut self) {
        if let Some(last_index) = self.buttons.len().checked_sub(1) {
            self.focused_button = self.focused_button.min(last_index);
        } else {
            self.focused_button = 0;
        }
    }
}

/// Visual style for dialog frames.
#[derive(Debug, Clone)]
pub struct DialogStyle {
    /// Outer/content background style.
    pub background: Style,
    /// Border style.
    pub border: Style,
    /// Title row style.
    pub title: Style,
    /// Non-focused button style.
    pub button: Style,
    /// Focused button style.
    pub focused_button: Style,
    /// Viewport dim style.
    pub dim: Style,
}

impl Default for DialogStyle {
    fn default() -> Self {
        Self {
            background: Style::default().bg(Color::Black),
            border: Style::default().fg(Color::Gray),
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            button: Style::default().fg(Color::White),
            focused_button: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            dim: Style::default().bg(Color::DarkGray),
        }
    }
}

impl DialogStyle {
    /// Set background style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }

    /// Set border style.
    pub fn with_border(mut self, border: Style) -> Self {
        self.border = border;
        self
    }

    /// Set title style.
    pub fn with_title(mut self, title: Style) -> Self {
        self.title = title;
        self
    }

    /// Set button style.
    pub fn with_button(mut self, button: Style) -> Self {
        self.button = button;
        self
    }

    /// Set focused button style.
    pub fn with_focused_button(mut self, focused_button: Style) -> Self {
        self.focused_button = focused_button;
        self
    }

    /// Set viewport dim style.
    pub fn with_dim(mut self, dim: Style) -> Self {
        self.dim = dim;
        self
    }
}

/// Computed dialog layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DialogLayout {
    /// Outer dialog rect including border.
    pub outer: Rect,
    /// Title row inside border.
    pub title: Rect,
    /// Caller-owned content rect.
    pub content: Rect,
    /// Button row inside border.
    pub buttons: Rect,
}

/// Compute centered dialog layout without rendering.
pub fn compute_dialog_layout(viewport: Rect, model: &DialogModel) -> DialogLayout {
    if viewport.width == 0 || viewport.height == 0 {
        return DialogLayout::default();
    }

    let width = resolve_dialog_axis(model.size, viewport.width, Axis::Width).min(viewport.width);
    let height =
        resolve_dialog_axis(model.size, viewport.height, Axis::Height).min(viewport.height);
    let x = viewport
        .x
        .saturating_add(viewport.width.saturating_sub(width) / 2);
    let y = viewport
        .y
        .saturating_add(viewport.height.saturating_sub(height) / 2);
    let outer = Rect::new(x, y, width, height);
    let inner = Block::default().borders(Borders::ALL).inner(outer);
    let title = Rect::new(
        inner.x,
        inner.y,
        inner.width,
        TITLE_ROW_HEIGHT.min(inner.height),
    );
    let buttons = if inner.height == 0 {
        Rect::new(inner.x, inner.y, inner.width, 0)
    } else {
        Rect::new(
            inner.x,
            rect_bottom(inner).saturating_sub(BUTTON_ROW_HEIGHT),
            inner.width,
            BUTTON_ROW_HEIGHT.min(inner.height),
        )
    };
    let content_y = title.y.saturating_add(title.height);
    let content_bottom = buttons.y;
    let content_height = content_bottom.saturating_sub(content_y);
    let content = Rect::new(inner.x, content_y, inner.width, content_height);

    DialogLayout {
        outer,
        title,
        content,
        buttons,
    }
}

/// Render a dialog frame and return the caller-owned content rect.
pub fn render_dialog_frame(
    frame: &mut Frame,
    viewport: Rect,
    model: &DialogModel,
    style: &DialogStyle,
) -> DialogLayout {
    let layout = compute_dialog_layout(viewport, model);

    if layout.outer.width == 0 || layout.outer.height == 0 {
        return layout;
    }

    if model.dim {
        fill_area(frame.buffer_mut(), viewport, style.dim);
    }

    frame.render_widget(Clear, layout.outer);
    fill_area(frame.buffer_mut(), layout.outer, style.background);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style.border);
    frame.render_widget(block, layout.outer);
    render_title_row(frame.buffer_mut(), layout.title, model, style);
    render_button_row(frame.buffer_mut(), layout.buttons, model, style);

    layout
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Width,
    Height,
}

fn resolve_dialog_axis(size: DialogSize, viewport_axis: u16, axis: Axis) -> u16 {
    match size {
        DialogSize::Fixed { width, height } => match axis {
            Axis::Width => width,
            Axis::Height => height,
        },
        DialogSize::Percent { width, height } => {
            let percent = match axis {
                Axis::Width => width,
                Axis::Height => height,
            }
            .min(MAX_PERCENT);
            ((u32::from(viewport_axis) * u32::from(percent)) / u32::from(MAX_PERCENT)) as u16
        }
    }
}

fn render_title_row(buffer: &mut Buffer, area: Rect, model: &DialogModel, style: &DialogStyle) {
    if area.width == 0 || area.height == 0 || model.title.is_empty() {
        return;
    }
    let line = Line::from(Span::styled(
        model.title.clone(),
        style.title.patch(style.background),
    ));
    buffer.set_line(area.x, area.y, &line, area.width);
}

fn render_button_row(buffer: &mut Buffer, area: Rect, model: &DialogModel, style: &DialogStyle) {
    if area.width == 0 || area.height == 0 || model.buttons.is_empty() {
        return;
    }

    let spans = button_spans(model, style);
    let line = Line::from(spans);
    let line_width = usize_to_u16(line.width()).min(area.width);
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(line_width) / 2);
    buffer.set_line(x, area.y, &line, line_width);
}

fn button_spans(model: &DialogModel, style: &DialogStyle) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (index, label) in model.buttons.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(BUTTON_GAP));
        }
        let button_style = if index == model.focused_button {
            style.focused_button
        } else {
            style.button
        }
        .patch(style.background);
        spans.push(Span::styled(BUTTON_PREFIX, button_style));
        spans.push(Span::styled(BUTTON_PADDING_TEXT, button_style));
        spans.push(Span::styled(label.clone(), button_style));
        spans.push(Span::styled(BUTTON_PADDING_TEXT, button_style));
        spans.push(Span::styled(BUTTON_SUFFIX, button_style));
    }
    spans
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
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const TEST_BORDER_SIZE: u16 = 2;
    const VIEWPORT_X: u16 = 0;
    const VIEWPORT_Y: u16 = 0;
    const VIEWPORT_WIDTH: u16 = 80;
    const VIEWPORT_HEIGHT: u16 = 24;
    const FIXED_WIDTH: u16 = 40;
    const FIXED_HEIGHT: u16 = 10;
    const CENTERED_X: u16 = 20;
    const CENTERED_Y: u16 = 7;
    const INNER_X: u16 = CENTERED_X + 1;
    const TITLE_Y: u16 = CENTERED_Y + 1;
    const CONTENT_Y: u16 = TITLE_Y + 1;
    const CONTENT_WIDTH: u16 = FIXED_WIDTH - TEST_BORDER_SIZE;
    const CONTENT_HEIGHT: u16 = 6;
    const BUTTON_ROW_Y: u16 = CENTERED_Y + FIXED_HEIGHT - TEST_BORDER_SIZE;
    const FIRST_BUTTON_INDEX: usize = 0;
    const SECOND_BUTTON_INDEX: usize = 1;
    const DIM_CHECK_X: u16 = 0;
    const DIM_CHECK_Y: u16 = 0;

    #[test]
    fn computes_centered_fixed_size_dialog() {
        let model = sample_model();

        let layout = compute_dialog_layout(viewport(), &model);

        assert_eq!(
            layout.outer,
            Rect::new(CENTERED_X, CENTERED_Y, FIXED_WIDTH, FIXED_HEIGHT)
        );
    }

    #[test]
    fn computes_content_rect_between_title_and_buttons() {
        let model = sample_model();

        let layout = compute_dialog_layout(viewport(), &model);

        assert_eq!(
            layout.title,
            Rect::new(INNER_X, TITLE_Y, CONTENT_WIDTH, TITLE_ROW_HEIGHT)
        );
        assert_eq!(
            layout.content,
            Rect::new(INNER_X, CONTENT_Y, CONTENT_WIDTH, CONTENT_HEIGHT)
        );
        assert_eq!(
            layout.buttons,
            Rect::new(INNER_X, BUTTON_ROW_Y, CONTENT_WIDTH, BUTTON_ROW_HEIGHT)
        );
    }

    #[test]
    fn button_navigation_clamps_and_activate_returns_label() {
        let mut model = sample_model();

        assert_eq!(model.move_left(), Some(FIRST_BUTTON_INDEX));
        assert_eq!(model.move_right(), Some(SECOND_BUTTON_INDEX));
        assert_eq!(model.move_right(), Some(SECOND_BUTTON_INDEX));
        assert_eq!(
            model.activate(),
            Some((SECOND_BUTTON_INDEX, "Cancel".to_string()))
        );
    }

    #[test]
    fn empty_buttons_have_no_activation() {
        let mut model = DialogModel::new("Empty", Vec::<String>::new());

        assert_eq!(model.move_left(), None);
        assert_eq!(model.move_right(), None);
        assert_eq!(model.activate(), None);
    }

    #[test]
    fn dimming_styles_viewport_outside_dialog() {
        let model = sample_model().with_dim(true);
        let style = DialogStyle::default().with_dim(Style::default().bg(Color::Blue));

        let buffer = render_to_buffer(&model, &style);

        assert_eq!(buffer[(DIM_CHECK_X, DIM_CHECK_Y)].bg, Color::Blue);
    }

    #[test]
    fn dimming_can_be_disabled() {
        let model = sample_model().with_dim(false);
        let style = DialogStyle::default().with_dim(Style::default().bg(Color::Blue));

        let buffer = render_to_buffer(&model, &style);

        assert_ne!(buffer[(DIM_CHECK_X, DIM_CHECK_Y)].bg, Color::Blue);
    }

    #[test]
    fn zero_sized_viewport_returns_empty_layout() {
        let model = sample_model();
        let layout = compute_dialog_layout(
            Rect::new(VIEWPORT_X, VIEWPORT_Y, 0, VIEWPORT_HEIGHT),
            &model,
        );

        assert_eq!(layout, DialogLayout::default());
    }

    #[test]
    fn percent_size_resolves_against_viewport() {
        let model = DialogModel::new("Percent", ["Ok"]).with_size(DialogSize::Percent {
            width: DEFAULT_DIALOG_PERCENT,
            height: DEFAULT_DIALOG_PERCENT,
        });

        let layout = compute_dialog_layout(viewport(), &model);

        assert_eq!(layout.outer.width, VIEWPORT_WIDTH / 2);
        assert_eq!(layout.outer.height, VIEWPORT_HEIGHT / 2);
    }

    fn sample_model() -> DialogModel {
        DialogModel::new("Title", ["Ok", "Cancel"]).with_size(DialogSize::Fixed {
            width: FIXED_WIDTH,
            height: FIXED_HEIGHT,
        })
    }

    fn viewport() -> Rect {
        Rect::new(VIEWPORT_X, VIEWPORT_Y, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn render_to_buffer(model: &DialogModel, style: &DialogStyle) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let viewport = frame.area();
                render_dialog_frame(frame, viewport, model, style);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}
