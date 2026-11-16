use super::*;

#[test]
fn rows_view_toggles_and_arrow_keys_select_rows() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("one", "true");
    app.add_shell_node("two", "true");
    let first = app.workflow.graph.nodes().next().expect("first node").id;
    let first_graph = crate::tui::graph::subwayrat_node_id(first);
    app.graph_state.focused = Some(first_graph);
    app.graph_state.selected = std::collections::HashSet::from([first_graph]);
    app.focus = FocusPanel::Canvas;

    assert_eq!(app.canvas_view, crate::tui::rows::CanvasView::Graph);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('v'))), None);
    assert_eq!(app.canvas_view, crate::tui::rows::CanvasView::Rows);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Down)), None);
    assert_ne!(app.graph_state.focused, Some(first_graph));
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Up)), None);
    assert_eq!(app.graph_state.focused, Some(first_graph));
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::End)), None);
    assert_ne!(app.graph_state.focused, Some(first_graph));
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Home)), None);
    assert_eq!(app.graph_state.focused, Some(first_graph));
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::PageDown)), None);
    assert_ne!(app.graph_state.focused, Some(first_graph));
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::PageUp)), None);
    assert_eq!(app.graph_state.focused, Some(first_graph));
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('v'))), None);
    assert_eq!(app.canvas_view, crate::tui::rows::CanvasView::Graph);
}

#[test]
fn rows_view_mouse_click_selects_the_clicked_row() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("one", "true");
    app.add_shell_node("two", "true");
    app.add_shell_node("three", "true");
    app.toggle_canvas_view();
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 120, 40));
    crate::tui::panels::render(&mut app, buffer.area, &mut buffer);
    let area = app.rows_hit_area.expect("rows area recorded");
    let second = app.workflow.graph.nodes().nth(1).expect("second node").id;
    let second_graph = crate::tui::graph::subwayrat_node_id(second);

    let event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: area.x.saturating_add(2),
        row: area.y.saturating_add(2),
        modifiers: KeyModifiers::empty(),
    });
    assert_eq!(handle_event(&mut app, &event), None);
    assert_eq!(app.graph_state.focused, Some(second_graph));
}

// r[verify lattice.tui_ux.rows.selection]
#[test]
fn rows_scrolled_hit_testing_and_gestures_do_not_mutate_hidden_graph() {
    let (_temp_dir, mut app) = test_app();
    for index in 0..30 {
        app.add_shell_node(format!("node-{index}"), "true");
    }
    app.clear_workflow_dirty();
    app.toggle_canvas_view();
    app.select_row_boundary(true);
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 120, 24));
    crate::tui::panels::render(&mut app, buffer.area, &mut buffer);
    let area = app.rows_hit_area.expect("rows hit area");
    assert!(app.rows_scroll > 0);
    let layout = crate::tui::rows::layout(&app.workflow, 1).expect("valid workflow row layout");
    let first_visible = layout.rows[app.rows_scroll].node_id;
    let event = |kind, row| {
        Event::Mouse(MouseEvent {
            kind,
            column: area.x + 1,
            row,
            modifiers: KeyModifiers::empty(),
        })
    };
    handle_event(&mut app, &event(MouseEventKind::Down(MouseButton::Left), area.y + 1));
    assert_eq!(app.graph_state.focused, Some(crate::tui::graph::subwayrat_node_id(first_visible)));
    let selected = app.graph_state.selected.clone();
    handle_event(&mut app, &event(MouseEventKind::Down(MouseButton::Left), area.y));
    handle_event(&mut app, &event(MouseEventKind::Down(MouseButton::Left), area.y + area.height - 1));
    handle_event(&mut app, &event(MouseEventKind::Drag(MouseButton::Left), area.y + 2));
    handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Left));
    handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Right));
    assert_eq!(app.graph_state.selected, selected);
    assert_eq!(app.last_drag_canvas, None);
    assert!(!app.has_unsaved_changes());
    assert_eq!(app.workflow.graph.node_count(), 30);
    assert_eq!(app.workflow.graph.edge_count(), 0);
}

#[test]
fn delete_key_removes_selected_node_and_all_owned_workflow_state() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("delete me", "true");
    let node_id = app.graph_state.focused.expect("focused shell node");
    let mut credentials = NodeCredentialRefs::default();
    credentials.env.insert("TOKEN".to_string(), SecretRef::new("env", "TOKEN", None));
    app.workflow.set_node_credentials(node_id, credentials);
    app.workflow.set_node_runner_settings(node_id, NodeRunnerSettings::new("host", "shell"));
    app.clear_workflow_dirty();
    app.focus = FocusPanel::Canvas;

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Delete)), None);

    assert!(app.workflow.graph.node(node_id).is_none());
    assert!(app.workflow.node_config(node_id).is_none());
    assert!(app.workflow.node_credentials(node_id).is_none());
    assert!(app.workflow.node_runner_settings(node_id).is_none());
    assert!(app.graph_state.selected.is_empty());
    assert!(app.graph_state.focused.is_none());
    assert!(app.has_unsaved_changes());
}

#[test]
fn delete_key_without_selection_preserves_workflow_and_dirty_state() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("keep me", "true");
    let node_id = app.graph_state.focused.expect("focused shell node");
    app.graph_state.selected.clear();
    app.graph_state.focused = None;
    app.clear_workflow_dirty();
    app.focus = FocusPanel::Canvas;

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Delete)), None);

    assert!(app.workflow.graph.node(node_id).is_some());
    assert!(app.workflow.node_config(node_id).is_some());
    assert!(!app.has_unsaved_changes());
}

#[test]
fn unrepresentable_drag_delta_preserves_workflow_and_reanchors_pointer() {
    let (_directory, mut app) = test_app();
    app.add_shell_node("one", "true");
    app.clear_workflow_dirty();
    app.graph_state.viewport.offset_x = 0;
    app.graph_state.viewport.offset_y = 0;
    app.last_drag_canvas = Some((i32::MIN, i32::MIN));
    let before = crate::workflow_format::workflow_to_portable(&app.workflow);
    let mouse = MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: 1,
        row: 1,
        modifiers: KeyModifiers::empty(),
    };
    let position = mouse_canvas_position(&app, &mouse);
    assert_eq!(handle_mouse(&mut app, &mouse), None);
    assert_eq!(crate::workflow_format::workflow_to_portable(&app.workflow), before);
    assert!(!app.has_unsaved_changes());
    assert_eq!(app.last_drag_canvas, Some((position.x, position.y)));
}
