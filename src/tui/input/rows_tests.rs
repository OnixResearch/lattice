use crossterm::event::MouseButton;
use ratatui::layout::Rect;

use super::*;

fn click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::empty(),
    }
}

#[test]
fn hit_testing_excludes_header_footer_and_outside_columns() {
    let area = Rect::new(10, 20, 30, 6);
    assert_eq!(rows_hit_offset(area, &click(10, 21)), Some(0));
    assert_eq!(rows_hit_offset(area, &click(39, 24)), Some(3));
    for mouse in [
        click(9, 21),
        click(40, 21),
        click(10, 19),
        click(10, 20),
        click(10, 25),
        click(10, 26),
    ] {
        assert_eq!(rows_hit_offset(area, &mouse), None);
    }
    assert_eq!(rows_hit_offset(Rect::new(10, 20, 0, 6), &click(10, 21)), None);
    assert_eq!(rows_hit_offset(Rect::new(10, 20, 30, 1), &click(10, 21)), None);
}

#[test]
fn hit_testing_near_coordinate_limit_does_not_wrap() {
    let area = Rect {
        x: u16::MAX - 1,
        y: u16::MAX - 3,
        width: 10,
        height: 10,
    };
    assert_eq!(rows_hit_offset(area, &click(u16::MAX, u16::MAX)), Some(2));
    assert_eq!(rows_hit_offset(area, &click(0, 0)), None);
}

#[test]
fn overflowed_row_index_does_not_mutate_selection_or_hidden_graph() {
    let directory = tempfile::tempdir().expect("app directory");
    let mut app = crate::tui::app::State::new(crate::config::AppConfig {
        data_dir: directory.path().join("data"),
        ..crate::config::AppConfig::default()
    });
    app.add_shell_node("one", "true");
    app.add_shell_node("two", "true");
    app.toggle_canvas_view();
    let selected = app.graph_state.selected.clone();
    let focused = app.graph_state.focused;
    let node_count = app.workflow.graph.node_count();
    let edge_count = app.workflow.graph.edge_count();
    app.rows_hit_area = Some(Rect::new(0, 0, 80, 6));
    app.rows_scroll = usize::MAX;
    assert_eq!(handle_mouse(&mut app, &click(1, 2)), None);
    assert_eq!(app.graph_state.selected, selected);
    assert_eq!(app.graph_state.focused, focused);
    assert_eq!(app.workflow.graph.node_count(), node_count);
    assert_eq!(app.workflow.graph.edge_count(), edge_count);
}
