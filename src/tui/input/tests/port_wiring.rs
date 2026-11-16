use super::*;

// r[verify n8nclone.tui_ux.mouse_wiring.drag_release]
// r[verify n8nclone.tui_ux.mouse_wiring.drag_release.compatible]
#[test]
fn mouse_drag_release_creates_compatible_edge() {
    let (_temp_dir, mut app) = app_with_two_shell_nodes();
    let (source_output, target_input, _) = shell_port_positions(&app);

    assert_eq!(app.workflow.graph.edge_count(), 0);
    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),),
        None,
    );
    assert!(matches!(app.graph_state.mode, InteractionMode::Wiring { .. }));

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Drag(MouseButton::Left), target_input.0, target_input.1),),
        None,
    );
    assert_eq!(app.workflow.graph.edge_count(), 0);

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), target_input.0, target_input.1),),
        None,
    );

    assert_eq!(app.workflow.graph.edge_count(), 1);
    assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.mouse_wiring.snap_near_ports]
// r[verify n8nclone.tui_ux.mouse_wiring.snap_near_ports.release]
#[test]
fn mouse_drag_release_near_visible_input_snaps_to_edge() {
    let (_temp_dir, mut app) = app_with_two_shell_nodes();
    let (source_output, target_input, _) = shell_port_positions(&app);
    let snap_offset = u16::try_from(PORT_SNAP_RADIUS_CELLS).expect("snap radius fits in terminal coordinates");
    let near_target = (target_input.0.saturating_add(snap_offset), target_input.1);

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),),
        None,
    );
    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Drag(MouseButton::Left), near_target.0, near_target.1),),
        None,
    );
    let target_canvas = screen_to_canvas(&app, target_input.0, target_input.1);
    assert!(
        matches!(
            &app.graph_state.mode,
            InteractionMode::Wiring { cursor_x, cursor_y, .. }
                if cursor_x.get() == target_canvas.x && cursor_y.get() == target_canvas.y
        ),
        "wiring preview should snap to the visible input dot",
    );
    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), near_target.0, near_target.1),),
        None,
    );

    assert_eq!(app.workflow.graph.edge_count(), 1);
    assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.mouse_wiring.visible_port_clicks]
// r[verify n8nclone.tui_ux.mouse_wiring.visible_port_clicks.complete]
// r[verify n8nclone.tui_ux.mouse_wiring.visible_port_clicks.start]
#[test]
fn mouse_click_release_keeps_click_to_click_wiring_available() {
    let (_temp_dir, mut app) = app_with_two_shell_nodes();
    let (source_output, target_input, _) = shell_port_positions(&app);

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),),
        None,
    );
    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), source_output.0, source_output.1),),
        None,
    );
    assert!(matches!(app.graph_state.mode, InteractionMode::Wiring { .. }));
    assert_eq!(app.workflow.graph.edge_count(), 0);
    assert!(!app.has_unsaved_changes());

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Down(MouseButton::Left), target_input.0, target_input.1),),
        None,
    );

    assert_eq!(app.workflow.graph.edge_count(), 1);
    assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.mouse_wiring.invalid_release]
// r[verify n8nclone.tui_ux.mouse_wiring.snap_near_ports.far_release]
#[test]
fn mouse_drag_release_on_non_port_cancels_wiring_without_edge() {
    let (_temp_dir, mut app) = app_with_two_shell_nodes();
    let (source_output, _, target_body) = shell_port_positions(&app);

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),),
        None,
    );
    assert!(matches!(app.graph_state.mode, InteractionMode::Wiring { .. }));

    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Drag(MouseButton::Left), target_body.0, target_body.1),),
        None,
    );
    assert_eq!(
        handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), target_body.0, target_body.1),),
        None,
    );

    assert_eq!(app.workflow.graph.edge_count(), 0);
    assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
    assert!(!app.has_unsaved_changes());
}
