use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseButton;
use rat_nodegraph::view::node_bounds;
use tempfile::TempDir;

use super::*;
use crate::config::AppConfig;
use crate::credentials::NodeCredentialRefs;
use crate::credentials::SecretRef;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::NodeRunnerSettings;
use crate::tui::app::PropertyEditState;
use crate::tui::app::WorkflowBrowserState;

fn test_app() -> (TempDir, crate::tui::app::State) {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let config = AppConfig {
        data_dir: temp_dir.path().join("data"),
        ..AppConfig::default()
    };
    (temp_dir, crate::tui::app::State::new(config))
}

const NODE_BODY_INSET: i32 = 1;

fn key_event(modifiers: KeyModifiers, code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::empty(),
    })
}

fn canvas_to_screen(app: &crate::tui::app::State, canvas_x: i32, canvas_y: i32) -> (u16, u16) {
    let viewport = &app.graph_state.viewport;
    let graph_area = app.graph_state.area;
    let screen_x = canvas_x - viewport.offset_x + i32::from(graph_area.x);
    let screen_y = canvas_y - viewport.offset_y + i32::from(graph_area.y);
    (
        u16::try_from(screen_x).expect("canvas x is visible in test viewport"),
        u16::try_from(screen_y).expect("canvas y is visible in test viewport"),
    )
}

fn screen_to_canvas(app: &crate::tui::app::State, column: u16, row: u16) -> CanvasPosition {
    let canvas_pos = app
        .graph_state
        .viewport
        .screen_to_canvas(column.saturating_sub(app.graph_state.area.x), row.saturating_sub(app.graph_state.area.y));
    CanvasPosition {
        x: canvas_pos.x,
        y: canvas_pos.y,
    }
}

fn shell_port_positions(app: &crate::tui::app::State) -> ((u16, u16), (u16, u16), (u16, u16)) {
    let node_ids = app.graph_state.graph.node_ids();
    let source_id = node_ids.first().copied().expect("source node exists");
    let target_id = node_ids.get(1).copied().expect("target node exists");
    let source_bounds = node_bounds(&app.graph_state.graph, source_id).expect("source bounds");
    let target_bounds = node_bounds(&app.graph_state.graph, target_id).expect("target bounds");
    let source_output = canvas_to_screen(
        app,
        source_bounds.x.get() + i32::from(source_bounds.width) - OUTPUT_PORT_MARKER_RIGHT_INSET,
        source_bounds.y.get() + PORT_MARKER_ROW_FROM_NODE_TOP,
    );
    let target_input = canvas_to_screen(
        app,
        target_bounds.x.get() + INPUT_PORT_MARKER_X_FROM_NODE_LEFT,
        target_bounds.y.get() + PORT_MARKER_ROW_FROM_NODE_TOP,
    );
    let target_body =
        canvas_to_screen(app, target_bounds.x.get() + NODE_BODY_INSET, target_bounds.y.get() + NODE_BODY_INSET);
    (source_output, target_input, target_body)
}

fn app_with_two_shell_nodes() -> (TempDir, crate::tui::app::State) {
    let (temp_dir, mut app) = test_app();
    app.add_shell_node("Source", "printf source");
    app.add_shell_node("Target", "cat");
    app.clear_workflow_dirty();
    (temp_dir, app)
}

mod monitoring;

mod command_routes;

mod port_wiring;

mod property_routes;

mod node_entry;
