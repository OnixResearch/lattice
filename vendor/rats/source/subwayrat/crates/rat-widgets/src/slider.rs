//! Horizontal slider widget with a thumb indicator.
//!
//! Renders a track with a draggable-looking knob/thumb at the current
//! position. Designed for parameter editors and settings panels where
//! the user adjusts a value within a range.
//!
//! ```text
//!  ━━━━━━━●─────────        (default)
//!  ═══════◆─────────        (with_chars)
//!  Label  ━━━━●───── 0.50   (with labels + value)
//! ```

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct Slider {
    /// Current value as fraction 0.0..=1.0
    value: f64,
    /// Character for the filled (left) portion of the track
    filled_char: char,
    /// Character for the empty (right) portion of the track
    empty_char: char,
    /// Thumb/knob character drawn at the current position
    thumb_char: char,
    /// Optional label before the slider
    left_label: Option<String>,
    /// Optional label after the slider (e.g. formatted value)
    right_label: Option<String>,
    // Styling
    filled_style: Style,
    empty_style: Style,
    thumb_style: Style,
    label_style: Style,
}

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

impl Slider {
    pub fn new(value: f64) -> Self {
        Self {
            value: clamp01(value),
            filled_char: '━',
            empty_char: '─',
            thumb_char: '●',
            left_label: None,
            right_label: None,
            filled_style: Style::default().fg(Color::Cyan),
            empty_style: Style::default().fg(Color::DarkGray),
            thumb_style: Style::default().fg(Color::White),
            label_style: Style::default().fg(Color::Gray),
        }
    }

    pub fn with_value(mut self, value: f64) -> Self {
        self.value = clamp01(value);
        self
    }

    /// Set the track characters (filled, empty) and thumb.
    pub fn with_chars(mut self, filled: char, empty: char, thumb: char) -> Self {
        self.filled_char = filled;
        self.empty_char = empty;
        self.thumb_char = thumb;
        self
    }

    pub fn with_thumb(mut self, ch: char) -> Self {
        self.thumb_char = ch;
        self
    }

    pub fn with_left_label(mut self, label: impl Into<String>) -> Self {
        self.left_label = Some(label.into());
        self
    }

    pub fn with_right_label(mut self, label: impl Into<String>) -> Self {
        self.right_label = Some(label.into());
        self
    }

    pub fn with_filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }

    pub fn with_empty_style(mut self, style: Style) -> Self {
        self.empty_style = style;
        self
    }

    pub fn with_thumb_style(mut self, style: Style) -> Self {
        self.thumb_style = style;
        self
    }

    pub fn with_label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = clamp01(value);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let total_width = area.width as usize;

        let left_cost = match &self.left_label {
            Some(l) => l.len() + 1,
            None => 0,
        };
        let right_cost = match &self.right_label {
            Some(l) => 1 + l.len(),
            None => 0,
        };
        let chrome = left_cost + right_cost;

        // The track needs at least 1 char for the thumb
        let track_width = total_width.saturating_sub(chrome);

        let mut spans: Vec<Span> = Vec::new();

        if let Some(l) = &self.left_label {
            spans.push(Span::styled(l.clone(), self.label_style));
            spans.push(Span::raw(" "));
        }

        if track_width > 0 {
            // Thumb position: 0 means leftmost, track_width-1 means rightmost
            let thumb_pos = if track_width == 1 {
                0
            } else {
                ((self.value * (track_width - 1) as f64).round() as usize).min(track_width - 1)
            };

            let filled_count = thumb_pos;
            let empty_count = track_width - 1 - thumb_pos;

            if filled_count > 0 {
                let filled: String = std::iter::repeat_n(self.filled_char, filled_count).collect();
                spans.push(Span::styled(filled, self.filled_style));
            }

            spans.push(Span::styled(self.thumb_char.to_string(), self.thumb_style));

            if empty_count > 0 {
                let empty: String = std::iter::repeat_n(self.empty_char, empty_count).collect();
                spans.push(Span::styled(empty, self.empty_style));
            }
        }

        if let Some(l) = &self.right_label {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(l.clone(), self.label_style));
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_to_string(slider: &Slider, width: u16) -> String {
        let backend = TestBackend::new(width.max(1), 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                slider.render(f, Rect::new(0, 0, width, 1));
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut s = String::new();
        for x in 0..width {
            let cell = &buf[(x, 0)];
            s.push_str(cell.symbol());
        }
        s.truncate(s.trim_end().len());
        s
    }

    // -- clamping --

    #[test]
    fn clamp_negative() {
        let s = Slider::new(-1.0);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn clamp_over_one() {
        let s = Slider::new(5.0);
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn set_value_clamps() {
        let mut s = Slider::new(0.5);
        s.set_value(-0.1);
        assert_eq!(s.value, 0.0);
        s.set_value(1.5);
        assert_eq!(s.value, 1.0);
    }

    // -- thumb position --

    #[test]
    fn thumb_at_zero() {
        // value=0 → thumb at leftmost position, all empty after
        let rendered = render_to_string(&Slider::new(0.0).with_chars('=', '-', 'O'), 10);
        assert!(rendered.starts_with('O'), "got: {rendered}");
        assert!(
            !rendered[1..].contains('='),
            "no filled chars expected: {rendered}"
        );
    }

    #[test]
    fn thumb_at_one() {
        // value=1 → thumb at rightmost position, all filled before
        let rendered = render_to_string(&Slider::new(1.0).with_chars('=', '-', 'O'), 10);
        assert!(rendered.ends_with('O'), "got: {rendered}");
        assert!(
            !rendered[..rendered.len() - 1].contains('-'),
            "no empty chars expected: {rendered}"
        );
    }

    #[test]
    fn thumb_at_half() {
        let rendered = render_to_string(&Slider::new(0.5).with_chars('=', '-', 'O'), 11);
        // 11 chars, thumb at position 5 → "=====O-----"
        assert!(rendered.contains('O'), "got: {rendered}");
        let thumb_idx = rendered.find('O').unwrap();
        assert_eq!(thumb_idx, 5, "got: {rendered}");
    }

    // -- zero/tiny areas --

    #[test]
    fn render_zero_width() {
        let s = Slider::new(0.5);
        // should not panic
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                s.render(f, Rect::new(0, 0, 0, 1));
            })
            .unwrap();
    }

    #[test]
    fn render_zero_height() {
        let s = Slider::new(0.5);
        let backend = TestBackend::new(20, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                s.render(f, Rect::new(0, 0, 20, 0));
            })
            .unwrap();
    }

    #[test]
    fn render_width_one() {
        // Only room for the thumb
        let rendered = render_to_string(&Slider::new(0.5).with_chars('=', '-', 'O'), 1);
        assert_eq!(rendered, "O");
    }

    // -- labels --

    #[test]
    fn left_label() {
        let rendered = render_to_string(
            &Slider::new(0.0)
                .with_chars('=', '-', 'O')
                .with_left_label("L"),
            12,
        );
        // "L O---------" (label + space + thumb at 0 + 9 empty)
        assert!(rendered.starts_with("L "), "got: {rendered}");
        assert!(rendered.contains('O'), "got: {rendered}");
    }

    #[test]
    fn right_label() {
        let rendered = render_to_string(
            &Slider::new(1.0)
                .with_chars('=', '-', 'O')
                .with_right_label("R"),
            12,
        );
        // 12 total - 2 right chrome = 10 track. thumb at pos 9.
        assert!(rendered.ends_with("R"), "got: {rendered}");
        assert!(rendered.contains('O'), "got: {rendered}");
    }

    #[test]
    fn both_labels() {
        let rendered = render_to_string(
            &Slider::new(0.5)
                .with_chars('=', '-', 'O')
                .with_left_label("A")
                .with_right_label("B"),
            16,
        );
        assert!(rendered.starts_with("A "), "got: {rendered}");
        assert!(rendered.ends_with("B"), "got: {rendered}");
        assert!(rendered.contains('O'), "got: {rendered}");
    }

    #[test]
    fn labels_eat_all_space() {
        // Labels so wide there's no room for the track
        let rendered = render_to_string(
            &Slider::new(0.5)
                .with_chars('=', '-', 'O')
                .with_left_label("ABCDE")
                .with_right_label("FGHIJ"),
            12,
        );
        // 12 - 6 left - 6 right = 0 track → no thumb/track rendered
        assert!(!rendered.contains('O'), "got: {rendered}");
    }

    // -- builder chaining --

    #[test]
    fn builder_chain() {
        let s = Slider::new(0.0)
            .with_value(0.75)
            .with_chars('=', '-', '#')
            .with_thumb('◆')
            .with_left_label("lo")
            .with_right_label("hi")
            .with_filled_style(Style::default().fg(Color::Green))
            .with_empty_style(Style::default().fg(Color::Red))
            .with_thumb_style(Style::default().fg(Color::Yellow))
            .with_label_style(Style::default().fg(Color::White));

        assert!((s.value - 0.75).abs() < f64::EPSILON);
        assert_eq!(s.thumb_char, '◆'); // with_thumb overrides with_chars
        assert_eq!(s.left_label.as_deref(), Some("lo"));
        assert_eq!(s.right_label.as_deref(), Some("hi"));
    }
}
