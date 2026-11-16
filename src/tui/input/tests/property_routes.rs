use super::*;

// r[verify n8nclone.tui_ux.properties.discoverable_entrypoints]
// r[verify n8nclone.tui_ux.properties.discoverable_entrypoints.canvas_command]
// r[verify n8nclone.tui_ux.properties.edit_command]
#[test]
fn canvas_key_edits_shell_command_without_focus_change() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("Example", "");
    app.clear_workflow_dirty();
    app.focus = FocusPanel::Canvas;
    let node_id = app.graph_state.focused.expect("focused node");

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);
    assert!(matches!(
        app.property_edit,
        Some(PropertyEditState {
            field: PropertyEditField::Command,
            ..
        })
    ));
    for ch in ['e', 'c', 'h', 'o'] {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "echo"),
        _ => panic!("expected shell config"),
    }
    assert!(app.property_edit.is_none());
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.properties.cancel]
// r[verify n8nclone.tui_ux.properties.modal_text_entry.cancel]
#[test]
fn canvas_key_cancels_property_edit_without_mutating_command() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("Example", "echo old");
    app.clear_workflow_dirty();
    app.focus = FocusPanel::Canvas;
    let node_id = app.graph_state.focused.expect("focused node");

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('x'))), None,);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);

    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "echo old"),
        _ => panic!("expected shell config"),
    }
    assert!(app.property_edit.is_none());
    assert!(!app.has_unsaved_changes());
}

#[test]
fn ctrl_e_requests_selected_command_edit_action() {
    let (_temp_dir, mut app) = test_app();
    app.focus = FocusPanel::Output;

    assert_eq!(
        handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('e'))),
        Some(crate::tui::app::Action::EditNodeCommand),
    );
}

// r[verify n8nclone.tui_ux.properties.discoverable_entrypoints.no_selection]
#[test]
fn edit_shortcut_without_selection_does_not_mutate_workflow() {
    let (_temp_dir, mut app) = test_app();
    app.clear_workflow_dirty();

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);

    assert!(app.property_edit.is_none());
    assert_eq!(app.workflow.graph.node_count(), 0);
    assert!(!app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.properties.edit_shell]
// r[verify n8nclone.tui_ux.properties.modal_text_entry]
// r[verify n8nclone.tui_ux.properties.modal_text_entry.save_command]
#[test]
fn property_keys_edit_shell_command() {
    let (_temp_dir, mut app) = test_app();
    app.add_shell_node("Example", "");
    app.clear_workflow_dirty();
    app.focus = FocusPanel::Properties;
    let node_id = app.graph_state.focused.expect("focused node");

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);
    for ch in ['e', 'c', 'h', 'o'] {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "echo"),
        _ => panic!("expected shell config"),
    }
    assert!(app.property_edit.is_none());
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.credentials.providers.edit]
#[test]
fn property_key_edits_agent_credentials() {
    let (_temp_dir, mut app) = test_app();
    app.add_agent_node("Agent", crate::agent_node::Target::Pi, "prompt", crate::agent_node::OutputMode::Text);
    app.clear_workflow_dirty();
    app.focus = FocusPanel::Properties;
    let node_id = app.graph_state.focused.expect("focused node");

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('k'))), None,);
    for ch in "TOKEN=env:key".chars() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

    let credentials = app.workflow.node_credentials(node_id).expect("credentials");
    assert_eq!(credentials.env.get("TOKEN").expect("token ref").identity(), "env:key");
    assert!(app.property_edit.is_none());
    assert!(app.has_unsaved_changes());
}

#[test]
fn property_edit_supports_credentials_for_non_agent_node() {
    let (_temp_dir, mut app) = test_app();
    app.add_http_node("HTTP", "GET", "https://example.com");
    app.clear_workflow_dirty();
    let node_id = app.graph_state.focused.expect("focused node");

    assert!(app.start_property_edit(PropertyEditField::Credentials));
    app.property_edit.as_mut().expect("credential edit").draft = "TOKEN=env:key".to_string();
    assert!(app.save_property_edit());

    let credentials = app.workflow.node_credentials(node_id).expect("credentials");
    assert_eq!(credentials.env.get("TOKEN").expect("token ref").identity(), "env:key");
    assert!(app.property_edit.is_none());
    assert!(app.has_unsaved_changes());
}
