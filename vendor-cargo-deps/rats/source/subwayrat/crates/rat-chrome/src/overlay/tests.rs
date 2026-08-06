use ratatui::widgets::Widget;

const MIN_TERMINAL_AXIS_CELLS: u16 = 1;
const MAX_BACKGROUND_COLUMNS_CELLS: usize = 256;
const MAX_BACKGROUND_ROWS_CELLS: usize = 128;

#[test]
fn centers_fixed_size() {
    let area_rect = ratatui::layout::Rect::new(0, 0, 80, 24);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(40))
        .with_height(super::OverlaySize::Fixed(10));

    let layout = super::layout(area_rect, &model);

    assert_eq!(layout.outer, ratatui::layout::Rect::new(20, 7, 40, 10));
}

#[test]
fn anchors_percentage_to_right_edge() {
    let area_rect = ratatui::layout::Rect::new(0, 0, 100, 40);
    let model = super::OverlayModel::default()
        .with_anchor(super::OverlayAnchor::Right)
        .with_width(super::OverlaySize::Percent(30))
        .with_height(super::OverlaySize::Percent(100));

    let layout = super::layout(area_rect, &model);

    assert_eq!(layout.outer, ratatui::layout::Rect::new(70, 0, 30, 40));
}

#[test]
fn anchors_to_bottom_edge() {
    let area_rect = ratatui::layout::Rect::new(4, 2, 80, 24);
    let model = super::OverlayModel::default()
        .with_anchor(super::OverlayAnchor::Bottom)
        .with_width(super::OverlaySize::Fixed(20))
        .with_height(super::OverlaySize::Fixed(5));

    let layout = super::layout(area_rect, &model);

    assert_eq!(layout.outer, ratatui::layout::Rect::new(34, 21, 20, 5));
}

#[test]
fn clamps_oversized_requests_to_bounds() {
    let area_rect = ratatui::layout::Rect::new(3, 5, 80, 24);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(120))
        .with_height(super::OverlaySize::Fixed(40))
        .with_offsets(50, 50);

    let layout = super::layout(area_rect, &model);

    assert_eq!(layout.outer, area_rect);
}

#[test]
fn border_chrome_reduces_inner_rect() {
    let area_rect = ratatui::layout::Rect::new(0, 0, 80, 24);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(40))
        .with_height(super::OverlaySize::Fixed(10))
        .with_border(true);

    let layout = super::layout(area_rect, &model);

    assert_eq!(layout.inner, ratatui::layout::Rect::new(21, 8, 38, 8));
}

#[test]
fn borderless_uses_full_area_as_inner_rect() {
    let area_rect = ratatui::layout::Rect::new(0, 0, 80, 24);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(40))
        .with_height(super::OverlaySize::Fixed(10))
        .with_border(false);

    let layout = super::layout(area_rect, &model);

    assert_eq!(layout.inner, layout.outer);
}

#[test]
fn clear_resets_cells_before_chrome() -> Result<(), core::convert::Infallible> {
    let area_rect = ratatui::layout::Rect::new(0, 0, 20, 6);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(8))
        .with_height(super::OverlaySize::Fixed(3))
        .with_border(false)
        .with_clear(true);

    let buffer = render_sample(area_rect, &model, &super::OverlayStyle::default())?;
    let layout = super::layout(area_rect, &model);

    assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), " ");
    assert_eq!(buffer[(0, 0)].symbol(), "x");
    Ok(())
}

#[test]
fn backdrop_styles_only_outside_cells() -> Result<(), core::convert::Infallible> {
    let area_rect = ratatui::layout::Rect::new(0, 0, 20, 6);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(8))
        .with_height(super::OverlaySize::Fixed(3))
        .with_border(false)
        .with_backdrop(true);
    let style =
        super::OverlayStyle::default().with_backdrop(ratatui::style::Style::new().bg(ratatui::style::Color::Blue));

    let buffer = render_sample(area_rect, &model, &style)?;
    let layout = super::layout(area_rect, &model);

    assert_eq!(buffer[(0, 0)].bg, ratatui::style::Color::Blue);
    assert_eq!(buffer[(layout.inner.x, layout.inner.y)].bg, ratatui::style::Color::Black);
    assert_eq!(buffer[(0, 0)].symbol(), "x");
    Ok(())
}

#[test]
fn buffer_renderer_draws_chrome_without_frame() {
    let area_rect = ratatui::layout::Rect::new(0, 0, 20, 6);
    let model = super::OverlayModel::default()
        .with_width(super::OverlaySize::Fixed(10))
        .with_height(super::OverlaySize::Fixed(4))
        .with_clear(true)
        .with_title(" Buffer ");
    let style = super::OverlayStyle::default()
        .with_border(ratatui::style::Style::new().fg(ratatui::style::Color::Red))
        .with_fill(ratatui::style::Style::new().bg(ratatui::style::Color::Green));
    let mut buffer = ratatui::buffer::Buffer::empty(area_rect);
    paint_test_background(&mut buffer, area_rect);

    let layout = super::render_frame(&mut buffer, area_rect, &model, &style);

    assert_eq!(layout, super::layout(area_rect, &model));
    assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, ratatui::style::Color::Red);
    assert_eq!(buffer[(layout.inner.x, layout.inner.y)].bg, ratatui::style::Color::Green);
    assert_eq!(buffer[(0, 0)].symbol(), "x");
}

#[test]
fn buffer_renderer_zero_area_is_noop() {
    let zero_area_rect = ratatui::layout::Rect::new(0, 0, 0, 0);
    let model = super::OverlayModel::default().with_clear(true);
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 1, 1));

    let layout = super::render_frame(&mut buffer, zero_area_rect, &model, &super::OverlayStyle::default());

    assert_eq!(layout.outer, ratatui::layout::Rect::new(0, 0, 0, 0));
    assert_eq!(buffer[(0, 0)].symbol(), " ");
}

fn render_sample(
    area_rect: ratatui::layout::Rect,
    model: &super::OverlayModel,
    style: &super::OverlayStyle,
) -> Result<ratatui::buffer::Buffer, core::convert::Infallible> {
    let backend = ratatui::backend::TestBackend::new(
        area_rect.width.max(MIN_TERMINAL_AXIS_CELLS),
        area_rect.height.max(MIN_TERMINAL_AXIS_CELLS),
    );
    let mut terminal = ratatui::Terminal::new(backend)?;
    terminal.draw(|frame| {
        let frame_area_rect = frame.area();
        let background = test_background(frame_area_rect);
        frame.render_widget(ratatui::widgets::Paragraph::new(background), frame_area_rect);
        super::frame(frame, area_rect, model, style);
    })?;
    Ok(terminal.backend().buffer().clone())
}

fn paint_test_background(buffer: &mut ratatui::buffer::Buffer, area_rect: ratatui::layout::Rect) {
    ratatui::widgets::Paragraph::new(test_background(area_rect)).render(area_rect, buffer);
}

fn test_background(area_rect: ratatui::layout::Rect) -> Vec<ratatui::text::Line<'static>> {
    let columns_cells = usize::from(area_rect.width).min(MAX_BACKGROUND_COLUMNS_CELLS);
    let rows_cells = usize::from(area_rect.height).min(MAX_BACKGROUND_ROWS_CELLS);
    let row_text = "x".repeat(columns_cells);
    std::iter::repeat_with(|| ratatui::text::Line::from(row_text.clone())).take(rows_cells).collect()
}
