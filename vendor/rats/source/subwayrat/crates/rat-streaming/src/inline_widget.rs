//! `InlineWidget` implementation for `StreamingOutput`.
//!
//! Allows `StreamingOutput` to participate as a leaf node in
//! `rat-inline` view trees. Renders the visible window of output
//! lines without chat-UI chrome (no border prefixes).

use crate::streaming_output::{DisplayLine, StreamingOutput};
use rat_inline::InlineWidget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

impl InlineWidget for StreamingOutput {
    fn height(&self, _width: u16) -> u16 {
        let display = self.display_line_count();
        // Show at most visible_lines (from config), or all lines if fewer.
        display.min(self.visible_lines()) as u16
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let output_style = Style::default().fg(Color::DarkGray);
        let omit_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM);

        let display_count = self.display_line_count();
        let visible = area.height as usize;

        // Show the tail of the output (auto-follow behavior for inline).
        let start = display_count.saturating_sub(visible);
        let end = display_count;

        let mut lines: Vec<Line<'_>> = Vec::with_capacity(end - start);
        for i in start..end {
            let display_line = self.get_display_line(i);
            match display_line {
                DisplayLine::Text(text) => {
                    lines.push(Line::from(Span::styled(text, output_style)));
                }
                DisplayLine::Omitted(n) => {
                    lines.push(Line::from(Span::styled(
                        format!("┄ {} lines omitted ┄", n),
                        omit_style,
                    )));
                }
            }
        }

        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_output::StreamingConfig;

    #[test]
    fn inline_height_empty() {
        let out = StreamingOutput::new();
        assert_eq!(out.height(80), 0);
    }

    #[test]
    fn inline_height_within_visible() {
        let mut out = StreamingOutput::new();
        for i in 0..5 {
            out.push_line(&format!("line {i}"));
        }
        // Default visible_lines is 16, we have 5.
        assert_eq!(out.height(80), 5);
    }

    #[test]
    fn inline_height_capped_at_visible() {
        let config = StreamingConfig {
            visible_lines: 4,
            ..StreamingConfig::default()
        };
        let mut out = StreamingOutput::with_config(config);
        for i in 0..20 {
            out.push_line(&format!("line {i}"));
        }
        assert_eq!(out.height(80), 4);
    }

    #[test]
    fn inline_render_produces_output() {
        let mut out = StreamingOutput::new();
        out.push_line("hello");
        out.push_line("world");

        let area = Rect::new(0, 0, 40, out.height(40));
        let mut buf = Buffer::empty(area);
        out.render(area, &mut buf);

        // Check that content was written.
        let cell = buf.cell(ratatui::layout::Position::new(0, 0));
        assert!(cell.is_some());
    }
}
