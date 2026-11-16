use super::*;

// r[verify n8nclone.tui_ux.command_palette.execute]
// r[verify n8nclone.tui_ux.properties.discoverable_entrypoints]
#[test]
fn palette_executes_filtered_action_and_closes() {
    let (_temp_dir, mut app) = test_app();

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
    assert!(app.show_palette);

    for ch in ['r', 'u', 'n'] {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }

    assert_eq!(
        handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
        Some(crate::tui::app::Action::RunWorkflow),
    );
    assert!(!app.show_palette);
}

// r[verify n8nclone.tui_ux.command_palette.interactive]
#[test]
fn palette_navigation_executes_selected_action() {
    let (_temp_dir, mut app) = test_app();

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Down)), None,);

    assert_eq!(
        handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
        Some(crate::tui::app::Action::EditNodeCommand),
    );
    assert!(!app.show_palette);
}

// r[verify n8nclone.tui_ux.properties.discoverable_entrypoints.palette_command]
#[test]
fn palette_filters_to_edit_command_action() {
    let (_temp_dir, mut app) = test_app();

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
    for ch in "command".chars() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }

    assert_eq!(
        handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
        Some(crate::tui::app::Action::EditNodeCommand),
    );
    assert!(!app.show_palette);
}

// r[verify n8nclone.tui_ux.command_palette.escape]
#[test]
fn palette_escape_closes_without_changing_workflow() {
    let (_temp_dir, mut app) = test_app();
    let node_count = app.workflow.graph.node_count();

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('a'))), None,);
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);

    assert!(!app.show_palette);
    assert!(app.palette_state.query.is_empty());
    assert_eq!(app.workflow.graph.node_count(), node_count);
}

// r[verify n8nclone.tui_ux.command_palette.no_match]
#[test]
fn palette_enter_with_no_match_keeps_recoverable_state() {
    let (_temp_dir, mut app) = test_app();
    let node_count = app.workflow.graph.node_count();

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
    for ch in ['z', 'z'] {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(app.palette_state.result_count(), 0);

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

    assert!(app.show_palette);
    assert_eq!(app.palette_state.query, "zz");
    assert_eq!(app.workflow.graph.node_count(), node_count);
}

#[test]
fn quit_confirmation_keys_confirm_or_cancel() {
    let (_temp_dir, mut app) = test_app();
    app.mark_workflow_dirty();
    assert!(!app.request_quit());

    assert_eq!(
        handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
        Some(crate::tui::app::Action::ConfirmQuit),
    );

    assert!(!app.request_quit());
    assert_eq!(
        handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)),
        Some(crate::tui::app::Action::CancelQuit),
    );
}

#[test]
fn quit_confirmation_ignores_unrelated_keys() {
    let (_temp_dir, mut app) = test_app();
    app.mark_workflow_dirty();
    assert!(!app.request_quit());

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('x'))), None,);
    assert!(app.is_confirming_quit());
}

#[test]
fn workflow_browser_keys_navigate_and_close_modal() {
    let (_temp_dir, mut app) = test_app();
    app.workflow_browser = WorkflowBrowserState::open(vec!["first".to_string(), "second".to_string()], None);

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Down)), None,);
    assert_eq!(app.workflow_browser.selected, 1);

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Up)), None,);
    assert_eq!(app.workflow_browser.selected, 0);

    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);
    assert!(!app.is_workflow_browser_open());
}
