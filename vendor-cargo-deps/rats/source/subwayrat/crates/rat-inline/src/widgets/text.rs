//! Basic styled text widget with word wrapping.

use crate::widget::InlineWidget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;

/// A simple styled text widget for inline rendering.
///
/// Wraps text at the given width and renders with an optional style.
pub struct InlineText {
    content: String,
    style: Style,
}

impl InlineText {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl InlineWidget for InlineText {
    fn height(&self, width: u16) -> u16 {
        if width == 0 || self.content.is_empty() {
            return if self.content.is_empty() { 0 } else { 1 };
        }
        // Compute wrapped line count.
        let mut lines = 0u16;
        for line in self.content.lines() {
            if line.is_empty() {
                lines += 1;
                continue;
            }
            let line_width = unicode_width::UnicodeWidthStr::width(line) as u16;
            lines += (line_width + width - 1) / width; // ceil division
        }
        // If content ends without a trailing newline and has at least one line.
        if lines == 0 {
            lines = 1;
        }
        lines
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(Line::styled(self.content.clone(), self.style))
            .wrap(Wrap { trim: false });
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_single_line() {
        let text = InlineText::new("hello");
        assert_eq!(text.height(80), 1);
    }

    #[test]
    fn height_wrapping() {
        let text = InlineText::new("hello world"); // 11 chars
        assert_eq!(text.height(5), 3); // "hello" / " worl" / "d"
    }

    #[test]
    fn height_empty() {
        let text = InlineText::new("");
        assert_eq!(text.height(80), 0);
    }

    #[test]
    fn height_multiline() {
        let text = InlineText::new("line1\nline2\nline3");
        assert_eq!(text.height(80), 3);
    }
}
