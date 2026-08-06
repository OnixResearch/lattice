//! Horizontal progress bar widget.
//!
//! Renders a text-based progress bar with optional elapsed/total labels
//! and percentage display. Suitable for media playback position and
//! download progress.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

pub struct ProgressBar {
    /// Validated progress in `0.0..=1.0`.
    progress: ratcore::validated_values::UnitInterval,
    /// Character for filled portion
    filled_char: char,
    /// Character for empty portion
    empty_char: char,
    /// Optional label before the bar (e.g., "02:30")
    left_label: Option<String>,
    /// Optional label after the bar (e.g., "05:00")
    right_label: Option<String>,
    /// Show percentage after right label
    show_percentage: bool,
    // Styling
    filled_style: Style,
    empty_style: Style,
    label_style: Style,
    percentage_style: Style,
}

fn format_time(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    let m = seconds / 60;
    let s = seconds % 60;
    format!("{m:02}:{s:02}")
}

impl ProgressBar {
    pub fn new(progress: ratcore::validated_values::UnitInterval) -> Self {
        Self {
            progress,
            filled_char: '━',
            empty_char: '─',
            left_label: None,
            right_label: None,
            show_percentage: false,
            filled_style: Style::default().fg(Color::Cyan),
            empty_style: Style::default().fg(Color::DarkGray),
            label_style: Style::default().fg(Color::Gray),
            percentage_style: Style::default().fg(Color::Gray),
        }
    }

    pub fn try_from_fraction(progress: f64) -> Result<Self, ratcore::validated_values::ValueError> {
        ratcore::validated_values::UnitInterval::try_new(progress).map(Self::new)
    }

    pub fn with_progress(mut self, progress: ratcore::validated_values::UnitInterval) -> Self {
        self.progress = progress;
        self
    }

    pub fn try_with_progress_fraction(self, progress: f64) -> Result<Self, ratcore::validated_values::ValueError> {
        ratcore::validated_values::UnitInterval::try_new(progress).map(|value| self.with_progress(value))
    }

    pub fn with_chars(mut self, filled: char, empty: char) -> Self {
        self.filled_char = filled;
        self.empty_char = empty;
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

    /// Convenience: formats both duration labels as `MM:SS`.
    pub fn with_time_labels(mut self, elapsed: std::time::Duration, total: std::time::Duration) -> Self {
        self.left_label = Some(format_time(elapsed));
        self.right_label = Some(format_time(total));
        self
    }

    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
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

    pub fn with_label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    pub fn set_progress(&mut self, progress: ratcore::validated_values::UnitInterval) {
        self.progress = progress;
    }

    pub fn try_set_progress_fraction(&mut self, progress: f64) -> Result<(), ratcore::validated_values::ValueError> {
        let progress = ratcore::validated_values::UnitInterval::try_new(progress)?;
        self.set_progress(progress);
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let total_width = area.width as usize;

        // Figure out how much space the labels and chrome eat.
        let left_cost = match &self.left_label {
            Some(l) => l.len() + 1, // label + trailing space
            None => 0,
        };
        let right_cost = match &self.right_label {
            Some(l) => 1 + l.len(), // leading space + label
            None => 0,
        };
        let pct_cost = if self.show_percentage { 5 } else { 0 }; // " XXX%" or " 100%"
        let chrome = left_cost + right_cost + pct_cost;

        let bar_width = total_width.saturating_sub(chrome);

        let filled_count = if bar_width == 0 {
            0
        } else {
            ((self.progress.get() * bar_width as f64).round() as usize).min(bar_width)
        };
        let empty_count = bar_width - filled_count;

        let mut spans: Vec<Span> = Vec::new();

        if let Some(l) = &self.left_label {
            spans.push(Span::styled(l.clone(), self.label_style));
            spans.push(Span::raw(" "));
        }

        if bar_width > 0 {
            let filled: String = std::iter::repeat_n(self.filled_char, filled_count).collect();
            let empty: String = std::iter::repeat_n(self.empty_char, empty_count).collect();
            spans.push(Span::styled(filled, self.filled_style));
            spans.push(Span::styled(empty, self.empty_style));
        }

        if let Some(l) = &self.right_label {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(l.clone(), self.label_style));
        }

        if self.show_percentage {
            let pct = self.progress.rounded_percent().get();
            spans.push(Span::styled(format!(" {pct:>3}%"), self.percentage_style));
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn interval(value: f64) -> ratcore::validated_values::UnitInterval {
        ratcore::validated_values::UnitInterval::try_new(value).unwrap()
    }

    #[test]
    fn invalid_progress_is_rejected_instead_of_clamped() {
        assert!(ProgressBar::try_from_fraction(-0.5).is_err());
        assert!(ProgressBar::try_from_fraction(3.0).is_err());
        assert!(ProgressBar::try_from_fraction(f64::NAN).is_err());
        let bar = ProgressBar::new(interval(0.42));
        assert!((bar.progress.get() - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn rejected_update_preserves_previous_progress() {
        let mut bar = ProgressBar::new(interval(0.5));
        assert!(bar.try_set_progress_fraction(-1.0).is_err());
        assert_eq!(bar.progress, interval(0.5));
        bar.set_progress(interval(1.0));
        assert_eq!(bar.progress, interval(1.0));
    }

    #[test]
    fn duration_labels_preserve_existing_format() {
        let bar = ProgressBar::new(interval(0.5))
            .with_time_labels(std::time::Duration::from_secs(150), std::time::Duration::from_secs(300));
        assert_eq!(bar.left_label.as_deref(), Some("02:30"));
        assert_eq!(bar.right_label.as_deref(), Some("05:00"));

        let large = ProgressBar::new(interval(0.0))
            .with_time_labels(std::time::Duration::from_secs(5_405), std::time::Duration::from_secs(5_405));
        assert_eq!(large.left_label.as_deref(), Some("90:05"));
    }

    fn render_in_area(bar: &ProgressBar, width: u16, height: u16) {
        let backend = TestBackend::new(width.max(1), height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| bar.render(frame, Rect::new(0, 0, width, height))).unwrap();
    }

    #[test]
    fn tiny_and_normal_areas_render_without_panicking() {
        let bar = ProgressBar::new(interval(0.5))
            .with_time_labels(std::time::Duration::from_secs(60), std::time::Duration::from_secs(120))
            .with_percentage(true);
        for (width, height) in [(0, 1), (40, 0), (1, 1), (10, 1), (40, 1)] {
            render_in_area(&bar, width, height);
        }
    }

    #[test]
    fn builder_chain_uses_validated_progress() {
        let bar = ProgressBar::new(interval(0.0))
            .with_progress(interval(0.75))
            .with_chars('=', '-')
            .with_left_label("A")
            .with_right_label("B")
            .with_percentage(true)
            .with_filled_style(Style::default().fg(Color::Green))
            .with_empty_style(Style::default().fg(Color::Red))
            .with_label_style(Style::default().fg(Color::White));

        assert_eq!(bar.progress, interval(0.75));
        assert_eq!(bar.filled_char, '=');
        assert_eq!(bar.empty_char, '-');
        assert_eq!(bar.left_label.as_deref(), Some("A"));
        assert_eq!(bar.right_label.as_deref(), Some("B"));
        assert!(bar.show_percentage);
    }
}
