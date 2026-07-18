//! Rendering the fuzzy finder.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use crate::state::FuzzyState;
use crate::types::FuzzySource;

#[derive(Debug, Clone)]
pub struct FuzzyStyle {
    pub prompt: Style,
    pub query: Style,
    pub match_highlight: Style,
    pub selected: Style,
    pub context: Style,
    pub count: Style,
    pub body: Style,
}

impl Default for FuzzyStyle {
    fn default() -> Self {
        Self {
            prompt: Style::default().fg(Color::Cyan),
            query: Style::default().add_modifier(Modifier::BOLD),
            match_highlight: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            selected: Style::default().bg(Color::Rgb(40, 40, 60)),
            context: Style::default().fg(Color::DarkGray),
            count: Style::default().fg(Color::DarkGray),
            body: Style::default(),
        }
    }
}

/// The fuzzy finder widget. Requires a `FuzzySource` reference for rendering candidate text.
pub struct FuzzyFinder<'a, S: FuzzySource> {
    source: &'a S,
    style: FuzzyStyle,
    block: Option<Block<'a>>,
}

impl<'a, S: FuzzySource> FuzzyFinder<'a, S> {
    pub fn new(source: &'a S, style: FuzzyStyle) -> Self {
        Self {
            source,
            style,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<S: FuzzySource> StatefulWidget for FuzzyFinder<'_, S> {
    type State = FuzzyState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };
        if inner.width < 5 || inner.height < 2 {
            return;
        }

        let candidates = self.source.candidates();

        // Input line
        let prompt_line = Line::from(vec![
            Span::styled("> ", self.style.prompt),
            Span::styled(state.query.clone(), self.style.query),
        ]);
        buf.set_line(inner.x, inner.y, &prompt_line, inner.width);

        // Count
        let total = candidates.len();
        let filtered = state.results.len();
        let count_str = format!("{}/{}", filtered, total);
        let count_x = inner.x + inner.width - count_str.len() as u16;
        buf.set_line(
            count_x,
            inner.y,
            &Line::from(Span::styled(count_str, self.style.count)),
            inner.width,
        );

        // Result list
        let list_height = (inner.height - 1) as usize;
        // Adjust scroll
        if state.selected < state.scroll_offset {
            state.scroll_offset = state.selected;
        }
        if state.selected >= state.scroll_offset + list_height {
            state.scroll_offset = state.selected.saturating_sub(list_height - 1);
        }

        let visible = &state.results
            [state.scroll_offset..state.results.len().min(state.scroll_offset + list_height)];
        for (row, scored) in visible.iter().enumerate() {
            let y = inner.y + 1 + row as u16;
            let is_selected = state.scroll_offset + row == state.selected;

            if is_selected {
                for x in inner.x..inner.x + inner.width {
                    buf[(x, y)].set_style(self.style.selected);
                }
            }

            if let Some(candidate) = candidates.get(scored.index) {
                // Icon
                let mut spans: Vec<Span<'static>> = Vec::new();
                if let Some(icon) = candidate.icon {
                    spans.push(Span::styled(format!("{} ", icon), self.style.body));
                }

                // Text with match highlights
                let text_chars: Vec<char> = candidate.text.chars().collect();
                let mut chunk = String::new();
                let mut in_match = false;
                for (ci, &ch) in text_chars.iter().enumerate() {
                    let is_match = scored.positions.contains(&ci);
                    if is_match != in_match {
                        if !chunk.is_empty() {
                            let sty = if in_match {
                                self.style.match_highlight
                            } else {
                                self.style.body
                            };
                            spans.push(Span::styled(std::mem::take(&mut chunk), sty));
                        }
                        in_match = is_match;
                    }
                    chunk.push(ch);
                }
                if !chunk.is_empty() {
                    let sty = if in_match {
                        self.style.match_highlight
                    } else {
                        self.style.body
                    };
                    spans.push(Span::styled(chunk, sty));
                }

                // Context
                if let Some(ref ctx) = candidate.context {
                    spans.push(Span::styled(format!("  {}", ctx), self.style.context));
                }

                buf.set_line(inner.x, y, &Line::from(spans), inner.width);
            }
        }
    }
}
