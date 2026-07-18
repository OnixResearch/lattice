use super::*;

#[test]
fn position_construction() {
    let pos = Position::new(-5, 10);
    assert_eq!(pos.x, -5);
    assert_eq!(pos.y, 10);
}

#[test]
fn position_equality() {
    let pos1 = Position::new(5, 10);
    let pos2 = Position::new(5, 10);
    let pos3 = Position::new(6, 10);

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn viewport_construction() {
    let viewport = Viewport::new(80, 24);
    assert_eq!(viewport.offset_x, 0);
    assert_eq!(viewport.offset_y, 0);
    assert_eq!(viewport.width, 80);
    assert_eq!(viewport.height, 24);
    assert_eq!(viewport.zoom, 1.0);
}

#[test]
fn screen_to_canvas_default_zoom() {
    let viewport = Viewport::new(80, 24);
    let pos = viewport.screen_to_canvas(10, 5);
    assert_eq!(pos, Position::new(10, 5));
}

#[test]
fn screen_to_canvas_with_offset() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = 100;
    viewport.offset_y = 50;

    let pos = viewport.screen_to_canvas(10, 5);
    assert_eq!(pos, Position::new(110, 55));
}

#[test]
fn screen_to_canvas_with_zoom() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = 2.0;

    let pos = viewport.screen_to_canvas(10, 5);
    assert_eq!(pos, Position::new(5, 3)); // rounded from 5.0, 2.5
}

#[test]
fn canvas_to_screen_visible() {
    let viewport = Viewport::new(80, 24);
    let result = viewport.canvas_to_screen(Position::new(5, 3));
    assert_eq!(result, Some((5, 3)));
}

#[test]
fn canvas_to_screen_outside_viewport() {
    let viewport = Viewport::new(80, 24);
    let result = viewport.canvas_to_screen(Position::new(100, 3));
    assert_eq!(result, None);

    let result = viewport.canvas_to_screen(Position::new(5, 30));
    assert_eq!(result, None);
}

#[test]
fn canvas_to_screen_negative_with_offset() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = -20;
    viewport.offset_y = 0;

    let result = viewport.canvas_to_screen(Position::new(-10, 0));
    assert_eq!(result, Some((10, 0)));
}

#[test]
fn pan_shifts_offset() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = 10;
    viewport.offset_y = 20;

    viewport.pan(5, -3);
    assert_eq!(viewport.offset_x, 15);
    assert_eq!(viewport.offset_y, 17);
}

#[test]
fn zoom_in() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = 1.0;

    viewport.zoom_in();
    assert_eq!(viewport.zoom, 1.25);
}

#[test]
fn zoom_clamped_to_max() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = MAX_ZOOM;

    viewport.zoom_in();
    assert_eq!(viewport.zoom, MAX_ZOOM);
}

#[test]
fn zoom_out() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = 1.25;

    viewport.zoom_out();
    assert_eq!(viewport.zoom, 1.0);
}

#[test]
fn zoom_clamped_to_min() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = MIN_ZOOM;

    viewport.zoom_out();
    assert_eq!(viewport.zoom, MIN_ZOOM);
}

#[test]
fn reset_zoom() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = 2.5;

    viewport.reset_zoom();
    assert_eq!(viewport.zoom, 1.0);
}

#[test]
fn resize_updates_dimensions() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = 10;
    viewport.offset_y = 20;
    viewport.zoom = 1.5;

    viewport.resize(120, 40);
    assert_eq!(viewport.width, 120);
    assert_eq!(viewport.height, 40);
    // Preserve offset and zoom
    assert_eq!(viewport.offset_x, 10);
    assert_eq!(viewport.offset_y, 20);
    assert_eq!(viewport.zoom, 1.5);
}

#[test]
fn visible_canvas_size_at_zoom_1() {
    let viewport = Viewport::new(80, 24);
    let (width, height) = viewport.visible_canvas_size();
    assert_eq!(width, 80);
    assert_eq!(height, 24);
}

#[test]
fn visible_canvas_size_at_zoom_2() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = 2.0;

    let (width, height) = viewport.visible_canvas_size();
    assert_eq!(width, 40);
    assert_eq!(height, 12);
}

#[test]
fn visible_canvas_size_at_zoom_half() {
    let mut viewport = Viewport::new(80, 24);
    viewport.zoom = 0.5;

    let (width, height) = viewport.visible_canvas_size();
    assert_eq!(width, 160);
    assert_eq!(height, 48);
}

#[test]
fn screen_to_canvas_with_offset_and_zoom() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = 100;
    viewport.offset_y = 50;
    viewport.zoom = 2.0;

    let pos = viewport.screen_to_canvas(10, 8);
    // 10 / 2.0 = 5, 8 / 2.0 = 4, then add offset
    assert_eq!(pos, Position::new(105, 54));
}

#[test]
fn canvas_to_screen_with_offset_and_zoom() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = 10;
    viewport.offset_y = 5;
    viewport.zoom = 2.0;

    let result = viewport.canvas_to_screen(Position::new(15, 8));
    // (15-10)*2 = 10, (8-5)*2 = 6
    assert_eq!(result, Some((10, 6)));
}

#[test]
fn pan_saturating() {
    let mut viewport = Viewport::new(80, 24);
    viewport.offset_x = i32::MAX - 5;

    viewport.pan(10, 0); // Would overflow
    assert_eq!(viewport.offset_x, i32::MAX);
}
