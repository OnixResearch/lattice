//! Command palette widget presentation.

use super::*;

/// Render the fuzzy command palette overlay.
pub(in crate::tui::panels) fn render_palette(app: &mut crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(60);
    let height = area.height.min(20);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = 2;
    let palette_area = Rect::new(x, y, width, height);

    // Clear background.
    Clear.render(palette_area, buf);

    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Command Palette (Ctrl-P) ")
        .border_style(Style::new().fg(Color::Yellow));

    let inner = block.inner(palette_area);
    debug_assert!(inner.width <= palette_area.width);
    debug_assert!(inner.height <= palette_area.height);
    block.render(palette_area, buf);

    // Render fuzzy finder.
    let style = FuzzyStyle {
        prompt: Style::new().fg(Color::Cyan),
        query: Style::new().add_modifier(Modifier::BOLD),
        match_highlight: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        selected: Style::new().bg(Color::Rgb(40, 40, 60)),
        context: Style::new().fg(Color::DarkGray),
        count: Style::new().fg(Color::DarkGray),
        body: Style::new(),
    };
    let widget = FuzzyFinder::new(&app.palette_data, style);
    widget.render(inner, buf, &mut app.palette_state);
}
