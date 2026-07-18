//! Rendering functions for search overlay and highlighting.
//!
//! Separated from core search logic to allow UI-independent testing
//! and different front-end implementations.

use crate::output_search::{OutputSearch, SearchMode};
use rat_chrome::{OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle, overlay_frame};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, layout::Rect};

const SEARCH_OVERLAY_WIDTH: u16 = 60;
const SEARCH_OVERLAY_HEIGHT: u16 = 3;
const SEARCH_OVERLAY_MARGIN: u16 = 4;
const SEARCH_OFFSET_X: i16 = -2;
const SEARCH_OFFSET_Y: i16 = 1;
const SEARCH_TITLE: &str = " Search (Ctrl+R: mode) ";
const MODE_SUBSTRING_LABEL: &str = "find";
const MODE_FUZZY_LABEL: &str = "fuzzy";
const NO_MATCHES_TEXT: &str = " no matches";
const CURSOR_GLYPH: &str = "\u{2588}";

/// Render the search overlay bar at the top-right of the messages area
pub fn render_search_overlay(search: &OutputSearch, frame: &mut Frame, area: Rect) {
    if !search.active {
        return;
    }

    let model = search_overlay_model(area);
    let style = search_overlay_style();
    let layout = overlay_frame(frame, area, &model, &style);

    if layout.inner.width == 0 || layout.inner.height == 0 {
        return;
    }

    let mode_label = match search.mode {
        SearchMode::Substring => MODE_SUBSTRING_LABEL,
        SearchMode::Fuzzy => MODE_FUZZY_LABEL,
    };

    let match_info = if search.query.is_empty() {
        String::new()
    } else if search.matches.is_empty() {
        NO_MATCHES_TEXT.to_string()
    } else {
        format!(" {}/{}", search.current + 1, search.matches.len())
    };

    let match_color = if search.matches.is_empty() && !search.query.is_empty() {
        Color::Red
    } else {
        Color::DarkGray
    };

    let search_line = Line::from(vec![
        Span::styled(
            format!(" {} ", mode_label),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(&search.query, Style::default().fg(Color::White)),
        Span::styled(
            CURSOR_GLYPH,
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(match_info, Style::default().fg(match_color)),
    ]);

    frame.render_widget(Paragraph::new(search_line), layout.inner);
}

fn search_overlay_model(area: Rect) -> OverlayModel {
    let width = area
        .width
        .saturating_sub(SEARCH_OVERLAY_MARGIN)
        .min(SEARCH_OVERLAY_WIDTH);
    let height = SEARCH_OVERLAY_HEIGHT.min(area.height);

    OverlayModel::default()
        .with_anchor(OverlayAnchor::TopRight)
        .with_offsets(SEARCH_OFFSET_X, SEARCH_OFFSET_Y)
        .with_width(OverlaySize::Fixed(width))
        .with_height(OverlaySize::Fixed(height))
        .with_clear(true)
        .with_title(SEARCH_TITLE)
}

fn search_overlay_style() -> OverlayStyle {
    OverlayStyle::default()
        .with_border(Style::default().fg(Color::Yellow))
        .with_title(Style::default().fg(Color::Yellow))
        .with_fill(Style::default())
}

/// Apply search match highlighting to rendered lines.
///
/// Returns the original base styles for each modified row so that later
/// highlighting passes (e.g. selection) can use the correct base style.
pub fn apply_search_highlights<'a>(
    lines: &mut [Line<'a>],
    plain_lines: &'a [String],
    search: &OutputSearch,
    match_style: Style,
    current_match_style: Style,
) -> Vec<Option<Style>> {
    let mut original_styles: Vec<Option<Style>> = vec![None; lines.len()];

    if search.query.is_empty() || search.matches.is_empty() {
        return original_styles;
    }

    // Walk through matches (already sorted by row) using a two-pointer approach
    let mut mi = 0;
    for row in 0..lines.len() {
        // Collect all matches on this row
        let row_start = mi;
        while mi < search.matches.len() && search.matches[mi].row == row {
            mi += 1;
        }
        if row_start == mi {
            continue; // no matches on this row
        }

        let plain = match plain_lines.get(row) {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };

        let base_style = lines[row]
            .spans
            .first()
            .map(|s| s.style)
            .unwrap_or_default();
        original_styles[row] = Some(base_style);

        let mut spans = Vec::new();
        let mut pos = 0;

        for idx in row_start..mi {
            let m = &search.matches[idx];
            let byte_start = m.byte_start.min(plain.len());
            let byte_end = m.byte_end.min(plain.len());

            // Validate character boundaries
            if !plain.is_char_boundary(byte_start) || !plain.is_char_boundary(byte_end) {
                continue;
            }

            let style = if idx == search.current {
                current_match_style
            } else {
                match_style
            };

            // Gap before this match
            if byte_start > pos && plain.is_char_boundary(pos) {
                spans.push(Span::styled(&plain[pos..byte_start], base_style));
            }

            // The match itself
            let start = byte_start.max(pos);
            if start < byte_end && plain.is_char_boundary(start) {
                spans.push(Span::styled(&plain[start..byte_end], style));
            }
            pos = byte_end;
        }

        // Remainder after last match
        if pos < plain.len() && plain.is_char_boundary(pos) {
            spans.push(Span::styled(&plain[pos..], base_style));
        }

        if !spans.is_empty() {
            lines[row] = Line::from(spans);
        }
    }

    original_styles
}

#[cfg(test)]
mod tests {
    use super::*;
    use rat_chrome::compute_overlay_layout;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    const VIEWPORT_WIDTH: u16 = 80;
    const VIEWPORT_HEIGHT: u16 = 12;
    const TINY_VIEWPORT_WIDTH: u16 = 1;
    const TINY_VIEWPORT_HEIGHT: u16 = 1;
    const BACKGROUND_CELL: &str = "x";
    const MODE_LABEL_OFFSET: u16 = 1;

    #[test]
    fn active_search_overlay_uses_chrome_and_renders_empty_query() {
        let mut search = OutputSearch::new();
        search.activate();

        let buffer = render_search(&search, viewport());
        let layout = search_layout(viewport());
        let mode_x = layout.inner.x + MODE_LABEL_OFFSET;

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Yellow);
        assert_eq!(buffer[(mode_x, layout.inner.y)].symbol(), "f");
    }

    #[test]
    fn inactive_search_overlay_is_noop() {
        let search = OutputSearch::new();

        let expected = render_background(viewport());
        let actual = render_search(&search, viewport());

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_match_query_uses_error_match_info() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "missing".to_string();

        let buffer = render_search(&search, viewport());
        let layout = search_layout(viewport());
        let no_matches_x = layout.inner.x + no_matches_offset(&search);

        assert_eq!(buffer[(no_matches_x, layout.inner.y)].fg, Color::Red);
    }

    #[test]
    fn tiny_search_overlay_area_does_not_panic() {
        let mut search = OutputSearch::new();
        search.activate();

        let buffer = render_search(&search, tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    fn render_search(search: &OutputSearch, viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
                render_search_overlay(search, frame, area);
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

    fn search_layout(viewport: Rect) -> rat_chrome::OverlayLayout {
        compute_overlay_layout(viewport, &search_overlay_model(viewport))
    }

    fn no_matches_offset(search: &OutputSearch) -> u16 {
        let mode_segment = MODE_SUBSTRING_LABEL.len() + MODE_SEGMENT_PADDING;
        let query_segment = search.query.len() + QUERY_CURSOR_WIDTH;
        (mode_segment + MODE_SEPARATOR_WIDTH + query_segment + NO_MATCHES_LEADING_SPACE)
            .try_into()
            .unwrap()
    }

    const MODE_SEGMENT_PADDING: usize = 2;
    const MODE_SEPARATOR_WIDTH: usize = 1;
    const QUERY_CURSOR_WIDTH: usize = 1;
    const NO_MATCHES_LEADING_SPACE: usize = 1;

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
