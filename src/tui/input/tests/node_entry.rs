use super::*;

#[test]
fn add_node_keys_create_shell_node_and_cancel_flow() {
    let (_temp_dir, mut app) = test_app();

    app.open_add_node_flow();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('s'))), None,);
    for _ in 0.."Shell 1".chars().count() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Backspace)), None,);
    }
    for ch in "One".chars() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);
    for ch in "echo one".chars() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

    assert_eq!(app.workflow.graph.node_count(), 1);
    assert!(!app.is_add_node_flow_open());

    app.open_add_node_flow();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);
    assert_eq!(app.workflow.graph.node_count(), 1);
    assert!(!app.is_add_node_flow_open());
}

#[test]
fn add_node_keys_create_image_input_node() {
    let (_temp_dir, mut app) = test_app();

    app.open_add_node_flow();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('i'))), None,);
    for _ in 0.."Image 1".chars().count() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Backspace)), None,);
    }
    for ch in "Selfie".chars() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);
    for ch in "/tmp/selfie.png".chars() {
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
    }
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

    assert_eq!(app.workflow.graph.node_count(), 1);
    assert!(!app.is_add_node_flow_open());
}

#[test]
fn add_node_keys_create_http_and_file_nodes() {
    let (_temp_dir, mut app) = test_app();

    app.open_add_node_flow();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('h'))), None);
    app.add_node_flow.label = "Fetch".to_string();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None);
    app.add_node_flow.command = "POST https://example.com/items".to_string();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None);

    app.open_add_node_flow();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('f'))), None);
    app.add_node_flow.label = "Read".to_string();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None);
    app.add_node_flow.command = "read ./input.txt".to_string();
    assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None);

    let fetch = app.workflow.graph.nodes().find(|node| node.label == "Fetch").expect("HTTP node");
    let read = app.workflow.graph.nodes().find(|node| node.label == "Read").expect("file node");
    assert!(matches!(
        app.workflow.node_config(fetch.id),
        Some(NodeConfig::Http { method, url, .. })
            if method == "POST" && url == "https://example.com/items"
    ));
    assert!(matches!(
        app.workflow.node_config(read.id),
        Some(NodeConfig::File {
            operation: crate::file_node::FileOperationKind::Read,
            path,
            ..
        }) if path == "./input.txt"
    ));
    assert!(!app.is_add_node_flow_open());
}

#[test]
fn invalid_http_and_file_drafts_do_not_create_nodes() {
    let (_temp_dir, mut app) = test_app();

    app.open_add_node_flow();
    app.start_http_node_flow();
    app.add_node_flow.step = AddNodeStep::Command;
    app.add_node_flow.command = "INVALID not-a-url".to_string();
    app.confirm_add_node_flow();
    assert_eq!(app.workflow.graph.node_count(), 0);
    assert!(app.is_add_node_flow_open());

    app.start_file_node_flow();
    app.add_node_flow.step = AddNodeStep::Command;
    app.add_node_flow.command = "erase ./input.txt".to_string();
    app.confirm_add_node_flow();
    assert_eq!(app.workflow.graph.node_count(), 0);
    assert!(app.is_add_node_flow_open());
}
