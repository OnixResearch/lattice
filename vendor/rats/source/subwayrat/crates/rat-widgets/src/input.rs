//! Single-line input dialog

use rat_chrome::{OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle, overlay_frame};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::theme::WidgetTheme;

const INPUT_DIALOG_WIDTH: u16 = 50;
const INPUT_DIALOG_HEIGHT: u16 = 3;
const DIALOG_HORIZONTAL_MARGIN: u16 = 4;
const CURSOR_GLYPH: &str = "_";

/// Pure data model for an input dialog — no ratatui dependency.
pub struct InputDialogModel {
    pub value: String,
    pub visible: bool,
}

impl InputDialogModel {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            visible: true,
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn backspace(&mut self) {
        self.value.pop();
    }

    pub fn submit(&mut self) -> String {
        let val = std::mem::take(&mut self.value);
        self.visible = false;
        val
    }
}

impl Default for InputDialogModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InputDialog {
    pub model: InputDialogModel,
    pub title: String,
}

impl InputDialog {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            model: InputDialogModel::new(),
            title: title.into(),
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.model.type_char(c);
    }

    pub fn backspace(&mut self) {
        self.model.backspace();
    }

    pub fn submit(&mut self) -> String {
        self.model.submit()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        if !self.model.visible {
            return;
        }

        let model = input_overlay_model(area, &self.title);
        let style = input_overlay_style(theme);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        frame.render_widget(
            Paragraph::new(input_line(&self.model.value, theme)),
            layout.inner,
        );
    }
}

fn input_overlay_model(area: Rect, title: &str) -> OverlayModel {
    let width = dialog_axis_size(area.width, INPUT_DIALOG_WIDTH, DIALOG_HORIZONTAL_MARGIN);
    let height = dialog_axis_size(area.height, INPUT_DIALOG_HEIGHT, 0);

    OverlayModel::default()
        .with_anchor(OverlayAnchor::Center)
        .with_width(OverlaySize::Fixed(width))
        .with_height(OverlaySize::Fixed(height))
        .with_clear(true)
        .with_title(input_title(title))
}

fn input_overlay_style(theme: &WidgetTheme) -> OverlayStyle {
    OverlayStyle::default()
        .with_border(Style::default().fg(theme.primary))
        .with_title(Style::default().fg(theme.primary))
        .with_fill(Style::default().bg(theme.background))
}

fn dialog_axis_size(available: u16, preferred: u16, margin: u16) -> u16 {
    preferred.min(available.saturating_sub(margin))
}

fn input_title(title: &str) -> String {
    format!(" {title} ")
}

fn input_line<'a>(value: &'a str, theme: &WidgetTheme) -> Line<'a> {
    Line::from(vec![
        Span::styled(value, Style::default().fg(theme.text)),
        Span::styled(
            CURSOR_GLYPH,
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rat_chrome::compute_overlay_layout;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::Color;
    use ratatui::widgets::Paragraph;

    const VIEWPORT_WIDTH: u16 = 60;
    const VIEWPORT_HEIGHT: u16 = 12;
    const TINY_VIEWPORT_WIDTH: u16 = 1;
    const TINY_VIEWPORT_HEIGHT: u16 = 1;
    const BACKGROUND_CELL: &str = "x";
    const INPUT_TITLE: &str = "Search";
    const INPUT_VALUE: &str = "abc";
    const TITLE_TEXT_OFFSET: u16 = 2;

    #[test]
    fn visible_input_uses_chrome_frame_and_renders_value() {
        let mut dialog = InputDialog::new(INPUT_TITLE);
        for typed in INPUT_VALUE.chars() {
            dialog.type_char(typed);
        }

        let buffer = render_input(&dialog, &WidgetTheme::default(), viewport());
        let layout = input_layout(viewport(), INPUT_TITLE);
        let cursor_x = layout.inner.x + char_count_u16(INPUT_VALUE);

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Blue);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), "a");
        assert_eq!(buffer[(cursor_x, layout.inner.y)].symbol(), CURSOR_GLYPH);
    }

    #[test]
    fn themed_input_title_and_text_use_theme_colors() {
        let mut dialog = InputDialog::new(INPUT_TITLE);
        dialog.type_char('z');
        let theme = WidgetTheme {
            primary: Color::Magenta,
            text: Color::Cyan,
            ..Default::default()
        };

        let buffer = render_input(&dialog, &theme, viewport());
        let layout = input_layout(viewport(), INPUT_TITLE);
        let title_x = layout.outer.x + TITLE_TEXT_OFFSET;

        assert_eq!(buffer[(title_x, layout.outer.y)].fg, Color::Magenta);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].fg, Color::Cyan);
    }

    #[test]
    fn hidden_input_is_noop() {
        let mut dialog = InputDialog::new(INPUT_TITLE);
        dialog.model.visible = false;

        let expected = render_background(viewport());
        let actual = render_input(&dialog, &WidgetTheme::default(), viewport());

        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_input_value_renders_cursor() {
        let dialog = InputDialog::new(INPUT_TITLE);

        let buffer = render_input(&dialog, &WidgetTheme::default(), viewport());
        let layout = input_layout(viewport(), INPUT_TITLE);

        assert_eq!(
            buffer[(layout.inner.x, layout.inner.y)].symbol(),
            CURSOR_GLYPH
        );
        assert!(
            buffer[(layout.inner.x, layout.inner.y)]
                .modifier
                .contains(Modifier::SLOW_BLINK)
        );
    }

    #[test]
    fn tiny_input_area_does_not_panic() {
        let dialog = InputDialog::new(INPUT_TITLE);

        let buffer = render_input(&dialog, &WidgetTheme::default(), tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    fn render_input(dialog: &InputDialog, theme: &WidgetTheme, viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
                dialog.render_themed(frame, area, theme);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_background(viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_test_background(frame: &mut Frame, area: Rect) {
        let line = Line::from(BACKGROUND_CELL.repeat(usize::from(area.width)));
        let rows = vec![line; usize::from(area.height)];
        frame.render_widget(Paragraph::new(rows), area);
    }

    fn input_layout(viewport: Rect, title: &str) -> rat_chrome::OverlayLayout {
        compute_overlay_layout(viewport, &input_overlay_model(viewport, title))
    }

    fn char_count_u16(value: &str) -> u16 {
        value.chars().count().try_into().unwrap()
    }

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
