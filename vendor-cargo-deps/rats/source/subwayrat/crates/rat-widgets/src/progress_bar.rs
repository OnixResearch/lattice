//! Horizontal progress bar widget.
//!
//! Renders a text-based progress bar with optional elapsed/total labels
//! and percentage display. Suitable for media playback position and
//! download progress.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct ProgressBar {
    /// Progress as fraction 0.0..=1.0
    progress: f64,
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

fn clamp_progress(p: f64) -> f64 {
    p.clamp(0.0, 1.0)
}

fn format_time(secs: u64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m:02}:{s:02}")
}

impl ProgressBar {
    pub fn new(progress: f64) -> Self {
        Self {
            progress: clamp_progress(progress),
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

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = clamp_progress(progress);
        self
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

    /// Convenience: formats both labels as "MM:SS".
    pub fn with_time_labels(mut self, elapsed_secs: u64, total_secs: u64) -> Self {
        self.left_label = Some(format_time(elapsed_secs));
        self.right_label = Some(format_time(total_secs));
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

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = clamp_progress(progress);
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
            ((self.progress * bar_width as f64).round() as usize).min(bar_width)
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
            let pct = (self.progress * 100.0).round() as u8;
            spans.push(Span::styled(format!(" {pct:>3}%"), self.percentage_style));
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

    // -- clamping --

    #[test]
    fn clamp_negative() {
        let bar = ProgressBar::new(-0.5);
        assert_eq!(bar.progress, 0.0);
    }

    #[test]
    fn clamp_over_one() {
        let bar = ProgressBar::new(3.0);
        assert_eq!(bar.progress, 1.0);
    }

    #[test]
    fn clamp_normal() {
        let bar = ProgressBar::new(0.42);
        assert!((bar.progress - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn set_progress_clamps() {
        let mut bar = ProgressBar::new(0.5);
        bar.set_progress(-1.0);
        assert_eq!(bar.progress, 0.0);
        bar.set_progress(99.0);
        assert_eq!(bar.progress, 1.0);
    }

    // -- time labels --

    #[test]
    fn time_label_formatting() {
        let bar = ProgressBar::new(0.5).with_time_labels(150, 300);
        assert_eq!(bar.left_label.as_deref(), Some("02:30"));
        assert_eq!(bar.right_label.as_deref(), Some("05:00"));
    }

    #[test]
    fn time_label_zero() {
        let bar = ProgressBar::new(0.0).with_time_labels(0, 0);
        assert_eq!(bar.left_label.as_deref(), Some("00:00"));
        assert_eq!(bar.right_label.as_deref(), Some("00:00"));
    }

    #[test]
    fn time_label_large() {
        // 90 minutes 5 seconds = 5405s
        let bar = ProgressBar::new(0.0).with_time_labels(5405, 5405);
        assert_eq!(bar.left_label.as_deref(), Some("90:05"));
    }

    // -- render: no panic on tiny areas --

    fn render_in_area(bar: &ProgressBar, width: u16, height: u16) {
        let backend = TestBackend::new(width.max(1), height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, width, height);
                bar.render(f, area);
            })
            .unwrap();
    }

    #[test]
    fn render_zero_width() {
        let bar = ProgressBar::new(0.5)
            .with_time_labels(60, 120)
            .with_percentage(true);
        render_in_area(&bar, 0, 1);
    }

    #[test]
    fn render_zero_height() {
        let bar = ProgressBar::new(0.5);
        render_in_area(&bar, 40, 0);
    }

    #[test]
    fn render_width_one() {
        let bar = ProgressBar::new(1.0);
        render_in_area(&bar, 1, 1);
    }

    #[test]
    fn render_narrow_with_labels() {
        // Labels alone exceed the width — bar portion collapses to 0.
        let bar = ProgressBar::new(0.5)
            .with_left_label("00:00")
            .with_right_label("99:99")
            .with_percentage(true);
        render_in_area(&bar, 10, 1);
    }

    #[test]
    fn render_normal() {
        let bar = ProgressBar::new(0.5)
            .with_time_labels(30, 60)
            .with_percentage(true);
        render_in_area(&bar, 40, 1);
    }

    // -- builder chaining --

    #[test]
    fn builder_chain() {
        let bar = ProgressBar::new(0.0)
            .with_progress(0.75)
            .with_chars('=', '-')
            .with_left_label("A")
            .with_right_label("B")
            .with_percentage(true)
            .with_filled_style(Style::default().fg(Color::Green))
            .with_empty_style(Style::default().fg(Color::Red))
            .with_label_style(Style::default().fg(Color::White));

        assert!((bar.progress - 0.75).abs() < f64::EPSILON);
        assert_eq!(bar.filled_char, '=');
        assert_eq!(bar.empty_char, '-');
        assert_eq!(bar.left_label.as_deref(), Some("A"));
        assert_eq!(bar.right_label.as_deref(), Some("B"));
        assert!(bar.show_percentage);
    }
}
