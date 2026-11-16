//! Canvas coordinates, port snapping, and graph synchronization.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::input) struct GraphCounts {
    pub(in crate::tui::input) nodes: usize,
    pub(in crate::tui::input) edges: usize,
}

pub(in crate::tui::input) fn graph_counts(app: &crate::tui::app::State) -> GraphCounts {
    GraphCounts {
        nodes: app.graph_state.graph.node_count(),
        edges: app.graph_state.graph.edge_count(),
    }
}

pub(in crate::tui::input) fn graph_counts_changed(before: GraphCounts, after: GraphCounts) -> bool {
    before != after
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::input) struct CanvasPosition {
    pub(in crate::tui::input) x: i32,
    pub(in crate::tui::input) y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::input) struct MouseCoordinates {
    pub(in crate::tui::input) column: u16,
    pub(in crate::tui::input) row: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::input) struct PortSnapTarget {
    pub(in crate::tui::input) marker: CanvasPosition,
    pub(in crate::tui::input) hit: CanvasPosition,
}

pub(in crate::tui::input) fn port_snap_target(graph: &Graph, cursor: CanvasPosition) -> Option<PortSnapTarget> {
    let mut nearest = None;
    for node in graph.nodes() {
        let Some(bounds) = node_bounds(graph, node.id) else {
            continue;
        };
        let origin = CanvasPosition {
            x: bounds.x.get(),
            y: bounds.y.get(),
        };
        let input_side = PortSide {
            origin,
            width: i32::from(bounds.width),
            is_output: false,
        };
        let output_side = PortSide {
            is_output: true,
            ..input_side
        };
        nearest = nearest_node_ports(nearest, cursor, input_side, node.input_ports.len());
        nearest = nearest_node_ports(nearest, cursor, output_side, node.output_ports.len());
    }
    let target = nearest.map(|(_, target)| target);
    if let Some(target) = target {
        // Snapping must never capture a pointer outside the configured cell radius.
        debug_assert!((i64::from(cursor.x) - i64::from(target.marker.x)).abs() <= i64::from(PORT_SNAP_RADIUS_CELLS));
        debug_assert!((i64::from(cursor.y) - i64::from(target.marker.y)).abs() <= i64::from(PORT_SNAP_RADIUS_CELLS));
    }
    target
}

#[derive(Clone, Copy)]
struct PortSide {
    origin: CanvasPosition,
    width: i32,
    is_output: bool,
}

fn nearest_node_ports(
    mut nearest: Option<(i64, PortSnapTarget)>,
    cursor: CanvasPosition,
    side: PortSide,
    count: usize,
) -> Option<(i64, PortSnapTarget)> {
    for index in 0..count {
        if let Some(target) = port_target(side.origin, side.width, index, side.is_output) {
            nearest = nearest_port_snap_target(nearest, cursor, target.marker, target.hit);
        }
    }
    nearest
}

pub(in crate::tui::input) fn nearest_port_snap_target(
    current: Option<(i64, PortSnapTarget)>,
    cursor: CanvasPosition,
    marker: CanvasPosition,
    hit: CanvasPosition,
) -> Option<(i64, PortSnapTarget)> {
    let dx = (i64::from(cursor.x) - i64::from(marker.x)).abs();
    let dy = (i64::from(cursor.y) - i64::from(marker.y)).abs();
    if dx > i64::from(PORT_SNAP_RADIUS_CELLS) || dy > i64::from(PORT_SNAP_RADIUS_CELLS) {
        return current;
    }

    let distance = dx + dy;
    let target = PortSnapTarget { marker, hit };
    match current {
        Some((best_distance, best_target)) if best_distance <= distance => Some((best_distance, best_target)),
        _ => Some((distance, target)),
    }
}

pub(in crate::tui::input) fn apply_graph_actions(app: &mut crate::tui::app::State, actions: &[GraphAction]) {
    app.graph_state.apply_actions(actions);
    app.sync_workflow_graph_from_state();
}

pub(in crate::tui::input) fn apply_graph_actions_and_mark_count_changes(
    app: &mut crate::tui::app::State,
    before: GraphCounts,
    actions: &[GraphAction],
) {
    apply_graph_actions(app, actions);
    if graph_counts_changed(before, graph_counts(app)) {
        app.mark_workflow_dirty();
    }
}

pub(in crate::tui::input) fn mouse_canvas_position(app: &crate::tui::app::State, mouse: &MouseEvent) -> CanvasPosition {
    let canvas_pos = app.graph_state.viewport.screen_to_canvas(
        mouse.column.saturating_sub(app.graph_state.area.x),
        mouse.row.saturating_sub(app.graph_state.area.y),
    );
    CanvasPosition {
        x: canvas_pos.x,
        y: canvas_pos.y,
    }
}

pub(in crate::tui::input) fn snapped_wiring_canvas_position(
    app: &crate::tui::app::State,
    mouse: &MouseEvent,
) -> CanvasPosition {
    let cursor = mouse_canvas_position(app, mouse);
    port_snap_target(&app.graph_state.graph, cursor).map_or(cursor, |target| target.marker)
}

pub(in crate::tui::input) fn graph_click_coordinates(
    app: &crate::tui::app::State,
    mouse: &MouseEvent,
) -> MouseCoordinates {
    let cursor = mouse_canvas_position(app, mouse);
    let hit = port_snap_target(&app.graph_state.graph, cursor).map_or(cursor, |target| target.hit);
    canvas_to_mouse_coordinates(app, hit).unwrap_or(MouseCoordinates {
        column: mouse.column,
        row: mouse.row,
    })
}

pub(in crate::tui::input) fn canvas_to_mouse_coordinates(
    app: &crate::tui::app::State,
    canvas: CanvasPosition,
) -> Option<MouseCoordinates> {
    app.graph_state
        .viewport
        .canvas_to_screen(Position::new(canvas.x, canvas.y))
        .map(|(column, row)| MouseCoordinates {
            column: column.saturating_add(app.graph_state.area.x),
            row: row.saturating_add(app.graph_state.area.y),
        })
}

pub(in crate::tui::input) fn canvas_position_tuple(position: CanvasPosition) -> (i32, i32) {
    (position.x, position.y)
}

pub(in crate::tui::input) fn is_drag_release(start: Option<(i32, i32)>, end: CanvasPosition) -> bool {
    matches!(start, Some(start) if start != canvas_position_tuple(end))
}

fn port_target(origin: CanvasPosition, width: i32, index: usize, is_output: bool) -> Option<PortSnapTarget> {
    let index = i64::try_from(index).ok()?;
    let (marker_x, hit_x) = port_columns(PortSide {
        origin,
        width,
        is_output,
    });
    let y_cells = i64::from(origin.y).checked_add(index)?;
    let marker_y = y_cells.checked_add(i64::from(PORT_MARKER_ROW_FROM_NODE_TOP))?;
    let hit_y = y_cells.checked_add(i64::from(PORT_HIT_ROW_FROM_NODE_TOP))?;
    // Ports beyond the canvas coordinate range cannot be mouse targets.
    Some(PortSnapTarget {
        marker: checked_position((marker_x, marker_y))?,
        hit: checked_position((hit_x, hit_y))?,
    })
}

fn checked_position(position: (i64, i64)) -> Option<CanvasPosition> {
    Some(CanvasPosition {
        x: i32::try_from(position.0).ok()?,
        y: i32::try_from(position.1).ok()?,
    })
}

fn port_columns(side: PortSide) -> (i64, i64) {
    if side.is_output {
        let right = i64::from(side.origin.x) + i64::from(side.width);
        (right - i64::from(OUTPUT_PORT_MARKER_RIGHT_INSET), right - i64::from(OUTPUT_PORT_HIT_RIGHT_INSET))
    } else {
        (i64::from(side.origin.x) + i64::from(INPUT_PORT_MARKER_X_FROM_NODE_LEFT), i64::from(side.origin.x))
    }
}

#[cfg(test)]
mod tests;
