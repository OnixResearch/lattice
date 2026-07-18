//! Inline widget wrapper for rat-markdown rendered content.

use crate::widget::InlineWidget;
use rat_markdown::{MarkdownStyle, PlainHighlighter, SyntaxHighlighter, render_markdown};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget, Wrap};

/// Inline markdown widget — renders markdown source into styled ratatui lines.
///
/// Wraps `rat_markdown::render_markdown` to satisfy `InlineWidget`.
pub struct InlineMarkdown {
    source: String,
    style: MarkdownStyle,
    highlighter: Box<dyn SyntaxHighlighter>,
}

impl InlineMarkdown {
    /// Create a markdown widget with default styling and no syntax highlighting.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            style: MarkdownStyle::from_base(Style::default()),
            highlighter: Box::new(PlainHighlighter),
        }
    }

    /// Set the markdown style.
    pub fn style(mut self, style: MarkdownStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the syntax highlighter for code blocks.
    pub fn highlighter(mut self, highlighter: impl SyntaxHighlighter + 'static) -> Self {
        self.highlighter = Box::new(highlighter);
        self
    }
}

impl InlineWidget for InlineMarkdown {
    fn height(&self, _width: u16) -> u16 {
        // Each markdown line produces one visual line.
        // Word wrapping within lines is not yet handled — that would
        // require width-aware measurement per line.
        let lines = render_markdown(&self.source, &self.style, self.highlighter.as_ref());
        lines.len() as u16
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let lines = render_markdown(&self.source, &self.style, self.highlighter.as_ref());
        let text = Text::from(lines);
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_counts_lines() {
        let md = InlineMarkdown::new("# Title\n\nSome **bold** text.\n\n- item 1\n- item 2");
        // 6 lines of markdown input
        assert_eq!(md.height(80), 6);
    }

    #[test]
    fn height_empty() {
        let md = InlineMarkdown::new("");
        assert_eq!(md.height(80), 0);
    }

    #[test]
    fn height_single_line() {
        let md = InlineMarkdown::new("hello world");
        assert_eq!(md.height(80), 1);
    }
}
