//! Rendering functions for streaming output display.
//!
//! Separated from core buffer logic to allow UI-independent testing
//! and different front-end implementations.

use crate::streaming_output::{DisplayLine, StreamingOutput};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Render lines for the inline chat view.
///
/// Returns the visible slice of lines based on scroll state and
/// `visible_height`. Lines are prefixed with the chat border and
/// tool-output style.
///
/// `visible_height` is how many lines of output to show (not counting
/// the stats footer).
pub fn render_streaming_lines<'a>(
    output: &mut StreamingOutput,
    visible_height: usize,
    border_style: Style,
) -> Vec<Line<'a>> {
    let output_style = Style::default().fg(Color::DarkGray);
    let omit_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM);

    // Build the full logical line list.
    let display_count = output.display_line_count();

    // Clamp scroll offset.
    let max_offset = display_count.saturating_sub(visible_height);
    if output.auto_follow() || output.scroll_offset > max_offset {
        output.scroll_offset = max_offset;
    }

    let start = output.scroll_offset;
    let end = (start + visible_height).min(display_count);

    let mut result = Vec::with_capacity(end - start);

    for i in start..end {
        let line = output.get_display_line(i);
        match line {
            DisplayLine::Text(text) => {
                result.push(Line::from(vec![
                    Span::styled("│ ", border_style),
                    Span::styled(format!("  │ {}", text), output_style),
                ]));
            }
            DisplayLine::Omitted(n) => {
                result.push(Line::from(vec![
                    Span::styled("│ ", border_style),
                    Span::styled(format!("  ┄ {} lines omitted ┄", n), omit_style),
                ]));
            }
        }
    }

    result
}

/// Render a compact stats footer line.
pub fn render_streaming_stats<'a>(output: &StreamingOutput, border_style: Style) -> Line<'a> {
    let stats_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM);

    let follow_indicator = if output.auto_follow() {
        "↓follow"
    } else {
        "scroll"
    };
    let truncated = if output.omitted() > 0 {
        format!(" ({} omitted)", output.omitted())
    } else {
        String::new()
    };

    Line::from(vec![
        Span::styled("│ ", border_style),
        Span::styled(
            format!(
                "  {} lines · {} · {}{}",
                output.total_lines(),
                format_bytes(output.total_bytes()),
                follow_indicator,
                truncated,
            ),
            stats_style,
        ),
    ])
}

/// Format a byte count as human-readable.
fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_output::{StreamingConfig, StreamingOutput};

    #[test]
    fn render_lines_basic() {
        let mut out = StreamingOutput::new();
        out.push_line("hello");
        out.push_line("world");

        let border = Style::default().fg(Color::DarkGray);
        let lines = render_streaming_lines(&mut out, 10, border);
        assert_eq!(lines.len(), 2);

        let text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("hello"));
    }

    #[test]
    fn render_lines_with_truncation() {
        let config = StreamingConfig {
            max_lines: 10,
            head_lines: 2,
            tail_lines: 2,
            visible_lines: 16,
        };
        let mut out = StreamingOutput::with_config(config);

        for i in 0..10 {
            out.push_line(&format!("L{}", i));
        }

        let border = Style::default().fg(Color::DarkGray);
        // Show all display lines (5 total: 2 head + omit + 2 tail).
        let lines = render_streaming_lines(&mut out, 10, border);
        assert_eq!(lines.len(), 5);

        // Check omission marker.
        let omit_text: String = lines[2].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(omit_text.contains("6 lines omitted"));
    }

    #[test]
    fn render_stats_footer() {
        let mut out = StreamingOutput::new();
        out.push_text("hello\nworld");

        let border = Style::default().fg(Color::DarkGray);
        let stats = render_streaming_stats(&out, border);
        let text: String = stats.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("2 lines"));
        assert!(text.contains("follow"));
    }

    #[test]
    fn empty_buffer_render() {
        let mut out = StreamingOutput::new();
        let border = Style::default().fg(Color::DarkGray);
        let lines = render_streaming_lines(&mut out, 10, border);
        assert!(lines.is_empty());
    }

    #[test]
    fn format_bytes_display() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(2 * 1024 * 1024 + 512 * 1024), "2.5 MB");
    }
}
