//! Loading/spinner indicator

use rat_spinner::SpinnerSpec;
use rat_spinner::SpinnerState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::theme::WidgetTheme;

#[derive(Debug, Clone)]
pub struct Loader<'a> {
    message: Option<Line<'a>>,
    spinner: SpinnerSpec<'a>,
    style: Option<LoaderStyle>,
}

#[derive(Debug, Clone)]
pub struct LoaderStyle {
    pub spinner_style: Style,
    pub message_style: Style,
    pub separator: &'static str,
}

impl<'a> Loader<'a> {
    pub fn new(message: impl Into<Line<'a>>) -> Self {
        Self {
            message: Some(message.into()),
            spinner: SpinnerSpec::dots(),
            style: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            message: None,
            spinner: SpinnerSpec::dots(),
            style: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<Line<'a>>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_spinner(mut self, spinner: SpinnerSpec<'a>) -> Self {
        self.spinner = spinner;
        self
    }

    pub fn with_style(mut self, style: LoaderStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, state: &SpinnerState) {
        let style = self.style.clone().unwrap_or_else(|| LoaderStyle::themed(&WidgetTheme::default()));
        self.render_with_style(frame, area, state, style);
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, state: &SpinnerState, theme: &WidgetTheme) {
        let style = self.style.clone().unwrap_or_else(|| LoaderStyle::themed(theme));
        self.render_with_style(frame, area, state, style);
    }

    fn render_with_style(&self, frame: &mut Frame, area: Rect, state: &SpinnerState, style: LoaderStyle) {
        let line = self.line_for_state(state, style);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn line_for_state(&self, state: &SpinnerState, style: LoaderStyle) -> Line<'a> {
        let spinner = state.current_frame(&self.spinner);
        self.render_line(spinner, style)
    }

    fn render_line(&self, spinner: &str, style: LoaderStyle) -> Line<'a> {
        let spinner_span = Span::styled(spinner.to_string(), style.spinner_style);

        match &self.message {
            Some(message) => {
                let mut spans = vec![spinner_span, Span::styled(style.separator, style.message_style)];
                spans.extend(message.spans.clone());

                let mut line = Line::from(spans);
                line.style = style.message_style;
                line.alignment = message.alignment;
                line
            }
            None => Line::from(spinner_span),
        }
    }
}

impl ratatui::widgets::StatefulWidget for &Loader<'_> {
    type State = SpinnerState;

    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
        let style = self.style.clone().unwrap_or_else(|| LoaderStyle::themed(&WidgetTheme::default()));
        let line = self.line_for_state(state, style);
        ratatui::widgets::Widget::render(Paragraph::new(line), area, buf);
    }
}

impl Default for LoaderStyle {
    fn default() -> Self {
        Self {
            spinner_style: Style::default(),
            message_style: Style::default(),
            separator: " ",
        }
    }
}

impl LoaderStyle {
    pub fn themed(theme: &WidgetTheme) -> Self {
        Self {
            spinner_style: Style::default().fg(theme.primary),
            message_style: Style::default().fg(theme.text_muted),
            separator: " ",
        }
    }
}

#[cfg(test)]
mod tests {
    use rat_spinner::SpinnerState;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Position;
    use ratatui::style::Color;

    use super::*;

    #[test]
    fn themed_style_uses_widget_theme() {
        let style = LoaderStyle::themed(&WidgetTheme::default());
        assert_eq!(style.spinner_style.fg, Some(WidgetTheme::default().primary));
        assert_eq!(style.message_style.fg, Some(WidgetTheme::default().text_muted));
    }

    #[test]
    fn loader_render_preserves_default_theme_colors() {
        let backend = TestBackend::new(12, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let loader = Loader::new("Load");
        let state = SpinnerState::new();

        terminal.draw(|frame| loader.render(frame, Rect::new(0, 0, 12, 1), &state)).unwrap();

        let buffer = terminal.backend().buffer();
        let spinner = buffer.cell(Position::new(0, 0)).unwrap();
        let message = buffer.cell(Position::new(2, 0)).unwrap();

        assert_eq!(spinner.symbol(), "⠋");
        assert_eq!(spinner.fg, Color::Blue);
        assert_eq!(message.symbol(), "L");
        assert_eq!(message.fg, Color::DarkGray);
    }

    #[test]
    fn loader_renders_custom_spinner_frames() {
        let backend = TestBackend::new(12, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let custom = SpinnerSpec::custom(&["[ ]", "[=]"]);
        let loader = Loader::new("Load").with_spinner(custom.clone());
        let mut state = SpinnerState::new();
        state.tick(&custom);

        terminal.draw(|frame| loader.render(frame, Rect::new(0, 0, 12, 1), &state)).unwrap();

        let buffer = terminal.backend().buffer();
        let spinner = buffer.cell(Position::new(0, 0)).unwrap();
        assert_eq!(spinner.symbol(), "[");
    }

    #[test]
    fn reference_stateful_widget_renders_repeatedly() {
        let loader = Loader::new("Load");
        let area = Rect::new(BUFFER_X, BUFFER_Y, BUFFER_WIDTH, BUFFER_HEIGHT);
        let mut first = ratatui::buffer::Buffer::empty(area);
        let mut second = ratatui::buffer::Buffer::empty(area);
        let mut state = SpinnerState::new();

        ratatui::widgets::StatefulWidget::render(&loader, area, &mut first, &mut state);
        ratatui::widgets::StatefulWidget::render(&loader, area, &mut second, &mut state);

        assert_eq!(first, second);
        assert_eq!(first.cell(Position::new(BUFFER_X, BUFFER_Y)).unwrap().symbol(), "⠋");
    }

    #[test]
    fn reference_stateful_widget_noops_on_zero_area() {
        let loader = Loader::new("Loading").with_spinner(SpinnerSpec::bounce());
        let area = Rect::new(BUFFER_X, BUFFER_Y, ZERO_WIDTH, ZERO_HEIGHT);
        let mut buffer = ratatui::buffer::Buffer::empty(area);
        let mut state = SpinnerState::new();

        ratatui::widgets::StatefulWidget::render(&loader, area, &mut buffer, &mut state);

        assert!(buffer.content().is_empty());
    }

    #[test]
    fn render_does_not_panic_on_tiny_area() {
        let backend = TestBackend::new(TINY_WIDTH, TINY_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        let loader = Loader::new("Loading").with_spinner(SpinnerSpec::bounce());
        let state = SpinnerState::new();

        terminal
            .draw(|frame| {
                loader.render(frame, Rect::new(BUFFER_X, BUFFER_Y, TINY_WIDTH, TINY_HEIGHT), &state);
            })
            .unwrap();
    }

    const BUFFER_X: u16 = 0;
    const BUFFER_Y: u16 = 0;
    const BUFFER_WIDTH: u16 = 12;
    const BUFFER_HEIGHT: u16 = 1;
    const TINY_WIDTH: u16 = 1;
    const TINY_HEIGHT: u16 = 1;
    const ZERO_WIDTH: u16 = 0;
    const ZERO_HEIGHT: u16 = 0;
}
