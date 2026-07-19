//! Confirmation dialog

use rat_chrome::{OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle, overlay_frame};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::theme::WidgetTheme;

const CONFIRM_DIALOG_WIDTH: u16 = 40;
const CONFIRM_DIALOG_HEIGHT: u16 = 5;
const DIALOG_HORIZONTAL_MARGIN: u16 = 4;
const CONFIRM_TITLE: &str = " Confirm ";
const CONFIRM_YES_LABEL: &str = "  [Yes] ";
const CONFIRM_NO_LABEL: &str = "  [No] ";

pub struct ConfirmDialog {
    pub message: String,
    pub selected: bool, // true = Yes, false = No
    pub visible: bool,
}

impl ConfirmDialog {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            selected: false,
            visible: true,
        }
    }

    pub fn toggle(&mut self) {
        self.selected = !self.selected;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        if !self.visible {
            return;
        }

        let model = confirm_overlay_model(area);
        let style = confirm_overlay_style(theme);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        let paragraph = Paragraph::new(confirm_content(&self.message, self.selected, theme))
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, layout.inner);
    }
}

fn confirm_overlay_model(area: Rect) -> OverlayModel {
    let width = dialog_axis_size(area.width, CONFIRM_DIALOG_WIDTH, DIALOG_HORIZONTAL_MARGIN);
    let height = dialog_axis_size(area.height, CONFIRM_DIALOG_HEIGHT, 0);

    OverlayModel::default()
        .with_anchor(OverlayAnchor::Center)
        .with_width(OverlaySize::Fixed(width))
        .with_height(OverlaySize::Fixed(height))
        .with_clear(true)
        .with_title(CONFIRM_TITLE)
}

fn confirm_overlay_style(theme: &WidgetTheme) -> OverlayStyle {
    OverlayStyle::default()
        .with_border(Style::default().fg(theme.warning))
        .with_title(Style::default())
        .with_fill(Style::default().bg(theme.background))
}

fn dialog_axis_size(available: u16, preferred: u16, margin: u16) -> u16 {
    preferred.min(available.saturating_sub(margin))
}

fn confirm_content(message: &str, selected: bool, theme: &WidgetTheme) -> Vec<Line<'static>> {
    let (yes_style, no_style) = confirm_button_styles(selected, theme);

    vec![
        Line::from(message.to_owned()),
        Line::from(""),
        Line::from(vec![
            Span::styled(CONFIRM_YES_LABEL, yes_style),
            Span::styled(CONFIRM_NO_LABEL, no_style),
        ]),
    ]
}

fn confirm_button_styles(selected: bool, theme: &WidgetTheme) -> (Style, Style) {
    let yes_style = if selected {
        Style::default()
            .fg(theme.success)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_disabled)
    };
    let no_style = if !selected {
        Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_disabled)
    };

    (yes_style, no_style)
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
    const CONFIRM_MESSAGE: &str = "Apply changes?";
    const BUTTON_LINE_OFFSET_Y: u16 = 2;
    const YES_LABEL_COLUMN_OFFSET: u16 = 3;
    const NO_LABEL_COLUMN_OFFSET: u16 = 11;

    #[test]
    fn visible_confirm_uses_chrome_frame_and_renders_message() {
        let dialog = ConfirmDialog::new(CONFIRM_MESSAGE);

        let buffer = render_confirm(&dialog, &WidgetTheme::default(), viewport());
        let layout = confirm_layout(viewport());

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Yellow);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), "A");
    }

    #[test]
    fn themed_yes_selection_preserves_success_emphasis() {
        let mut dialog = ConfirmDialog::new(CONFIRM_MESSAGE);
        dialog.toggle();

        let buffer = render_confirm(&dialog, &WidgetTheme::default(), viewport());
        let layout = confirm_layout(viewport());
        let yes_x = layout.inner.x + YES_LABEL_COLUMN_OFFSET;
        let no_x = layout.inner.x + NO_LABEL_COLUMN_OFFSET;
        let button_y = layout.inner.y + BUTTON_LINE_OFFSET_Y;

        assert_eq!(buffer[(yes_x, button_y)].fg, Color::Green);
        assert!(buffer[(yes_x, button_y)].modifier.contains(Modifier::BOLD));
        assert_eq!(buffer[(no_x, button_y)].fg, Color::DarkGray);
    }

    #[test]
    fn hidden_confirm_is_noop() {
        let mut dialog = ConfirmDialog::new(CONFIRM_MESSAGE);
        dialog.visible = false;

        let expected = render_background(viewport());
        let actual = render_confirm(&dialog, &WidgetTheme::default(), viewport());

        assert_eq!(actual, expected);
    }

    #[test]
    fn tiny_confirm_area_does_not_panic() {
        let dialog = ConfirmDialog::new(CONFIRM_MESSAGE);

        let buffer = render_confirm(&dialog, &WidgetTheme::default(), tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    fn render_confirm(dialog: &ConfirmDialog, theme: &WidgetTheme, viewport: Rect) -> Buffer {
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

    fn confirm_layout(viewport: Rect) -> rat_chrome::OverlayLayout {
        compute_overlay_layout(viewport, &confirm_overlay_model(viewport))
    }

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
