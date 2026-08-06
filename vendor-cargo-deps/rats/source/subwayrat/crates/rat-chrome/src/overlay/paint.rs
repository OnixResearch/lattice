pub fn backdrop(
    buf: &mut ratatui::buffer::Buffer,
    viewport: ratatui::layout::Rect,
    overlay: ratatui::layout::Rect,
    style: ratatui::style::Style,
) {
    for y in viewport.y..bottom(viewport) {
        for x in viewport.x..right(viewport) {
            if !contains(overlay, (x, y)) {
                buf[(x, y)].set_style(style);
            }
        }
    }
}

pub fn rect(buf: &mut ratatui::buffer::Buffer, area: ratatui::layout::Rect, style: ratatui::style::Style) {
    for y in area.y..bottom(area) {
        for x in area.x..right(area) {
            buf[(x, y)].set_style(style);
        }
    }
}

fn contains(rect: ratatui::layout::Rect, point: (u16, u16)) -> bool {
    let (x, y) = point;
    x >= rect.x && x < right(rect) && y >= rect.y && y < bottom(rect)
}

fn right(rect: ratatui::layout::Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn bottom(rect: ratatui::layout::Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}
