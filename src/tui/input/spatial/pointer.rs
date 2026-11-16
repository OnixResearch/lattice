//! Pointer dispatch with isolated monitoring row selection.

use super::*;

pub(in crate::tui::input) fn handle_mouse(
    app: &mut crate::tui::app::State,
    mouse: &MouseEvent,
) -> Option<crate::tui::app::Action> {
    if app.authoring.is_some() || app.property_edit.is_some() {
        // Keep modal ownership stable instead of mutating the graph behind a draft.
        return None;
    }

    if app.canvas_view == crate::tui::rows::CanvasView::Rows {
        handle_rows_mouse(app, mouse);
        // Rows are a monitoring surface: never send clicks, drags, or wheel
        // events to the hidden spatial graph.
        return None;
    }

    handle_graph_mouse(app, mouse);
    None
}

fn handle_graph_mouse(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => begin_graph_press(app, mouse),
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => handle_graph_drag(app, mouse),
        MouseEventKind::Moved => update_wiring_cursor(app, mouse),
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => finish_graph_press(app, mouse),
        MouseEventKind::ScrollUp => app.graph_state.viewport.pan(0, -MOUSE_WHEEL_PAN_ROWS),
        MouseEventKind::ScrollDown => app.graph_state.viewport.pan(0, MOUSE_WHEEL_PAN_ROWS),
        _ => {}
    }
}

fn begin_graph_press(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    apply_graph_click(app, mouse);
    let drag_start = if matches!(app.graph_state.mode, InteractionMode::Wiring { .. }) {
        snapped_wiring_canvas_position(app, mouse)
    } else {
        mouse_canvas_position(app, mouse)
    };
    app.last_drag_canvas = Some(canvas_position_tuple(drag_start));
}

fn finish_graph_press(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    let canvas_pos = snapped_wiring_canvas_position(app, mouse);
    let should_complete = matches!(app.graph_state.mode, InteractionMode::Wiring { .. })
        && is_drag_release(app.last_drag_canvas, canvas_pos);
    app.last_drag_canvas = None;
    if should_complete {
        apply_graph_click(app, mouse);
    } else if matches!(app.graph_state.mode, InteractionMode::BoxSelect { .. }) {
        let actions = app.graph_state.finish_box_select();
        apply_graph_actions(app, &actions);
    }
}

fn apply_graph_click(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    let before = graph_counts(app);
    let is_shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
    let click = graph_click_coordinates(app, mouse);
    let actions = app.graph_state.handle_mouse_click(click.column, click.row, is_shift);
    apply_graph_actions_and_mark_count_changes(app, before, &actions);
}

fn update_wiring_cursor(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    let canvas_pos = snapped_wiring_canvas_position(app, mouse);
    if let InteractionMode::Wiring { cursor_x, cursor_y, .. } = &mut app.graph_state.mode {
        *cursor_x = rat_nodegraph::view::CanvasX::from_scaled(canvas_pos.x);
        *cursor_y = rat_nodegraph::view::CanvasY::from_scaled(canvas_pos.y);
    }
}

pub(in crate::tui::input) fn rows_hit_offset(area: ratatui::layout::Rect, mouse: &MouseEvent) -> Option<usize> {
    let column = mouse.column.checked_sub(area.x)?;
    if column >= area.width {
        return None;
    }
    let row = usize::from(mouse.row.checked_sub(area.y)?.checked_sub(1)?);
    (row < crate::tui::rows::visible_count(area.height)).then_some(row)
}

pub(in crate::tui::input) fn handle_rows_mouse(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    let Some(offset) = app.rows_hit_area.and_then(|area| rows_hit_offset(area, mouse)) else {
        return;
    };
    match mouse.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            if let Some(index) = app.rows_scroll.checked_add(offset) {
                app.focus = FocusPanel::Canvas;
                app.select_row_index(index);
            }
        }
        MouseEventKind::ScrollUp => app.select_row(-1),
        MouseEventKind::ScrollDown => app.select_row(1),
        _ => {}
    }
}

pub(in crate::tui::input) fn handle_graph_drag(app: &mut crate::tui::app::State, mouse: &MouseEvent) {
    let canvas_pos = mouse_canvas_position(app, mouse);
    // During wiring, update the preview cursor instead of dragging nodes.
    if matches!(app.graph_state.mode, InteractionMode::Wiring { .. }) {
        update_wiring_cursor(app, mouse);
    } else {
        drag_graph_nodes(app, mouse, canvas_pos);
        app.last_drag_canvas = Some(canvas_position_tuple(canvas_pos));
    }
}

fn drag_graph_nodes(app: &mut crate::tui::app::State, mouse: &MouseEvent, canvas_pos: CanvasPosition) {
    let delta = app
        .last_drag_canvas
        .and_then(|(prev_x, prev_y)| Some((canvas_pos.x.checked_sub(prev_x)?, canvas_pos.y.checked_sub(prev_y)?)));
    if let Some((dx, dy)) = delta
        && (dx != 0 || dy != 0)
    {
        let actions = app.graph_state.handle_mouse_drag(mouse.column, mouse.row, dx, dy);
        let has_mutated = !actions.is_empty();
        apply_graph_actions(app, &actions);
        if has_mutated {
            app.mark_workflow_dirty();
        }
    }
}
