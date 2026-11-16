use super::*;

#[test]
fn property_edit_updates_shell_command() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo old");
    app.clear_workflow_dirty();
    let node_id = app.selected_node_id().expect("selected shell node");

    assert!(app.start_property_edit(PropertyEditField::Command));
    app.property_edit.as_mut().expect("property edit").draft = "echo new".to_string();
    assert!(app.save_property_edit());

    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "echo new"),
        _ => panic!("expected shell config"),
    }
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.properties.validate_timeout]
#[test]
fn property_edit_rejects_invalid_timeout_without_mutating() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");
    app.clear_workflow_dirty();
    let node_id = app.selected_node_id().expect("selected shell node");

    assert!(app.start_property_edit(PropertyEditField::TimeoutSecs));
    app.property_edit.as_mut().expect("property edit").draft = "not seconds".to_string();
    assert!(!app.save_property_edit());

    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { timeout_secs, .. } => {
            assert_eq!(*timeout_secs, DEFAULT_SHELL_TIMEOUT_SECS)
        }
        _ => panic!("expected shell config"),
    }
    assert!(app.property_edit.is_some());
    assert!(!app.has_unsaved_changes());
}

#[test]
fn property_edit_cancel_preserves_original_value() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Original", "echo example");
    app.clear_workflow_dirty();
    let node_id = app.selected_node_id().expect("selected shell node");

    assert!(app.start_property_edit(PropertyEditField::Label));
    app.property_edit.as_mut().expect("property edit").draft = "Changed".to_string();
    app.cancel_property_edit();

    let label = app.workflow.graph.node(node_id).expect("node").label.as_str();
    assert_eq!(label, "Original");
    assert!(app.property_edit.is_none());
    assert!(!app.has_unsaved_changes());
}

#[test]
fn timeout_validation_accepts_whole_seconds_only() {
    assert_eq!(validate_timeout_secs("42"), Ok(42));
    assert_eq!(validate_timeout_secs("nope"), Err(PropertyEditError::InvalidTimeout));
}

#[test]
fn condition_property_edit_reconciles_ports_and_edges_by_label() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_condition_node("Route", ConditionNodeConfig {
        branches: vec![
            ConditionBranch {
                label: "remove".to_string(),
                predicate: control_flow::ConditionPredicate::Always,
            },
            ConditionBranch {
                label: "keep".to_string(),
                predicate: control_flow::ConditionPredicate::Always,
            },
        ],
    });
    let condition_id = app.graph_state.focused.expect("condition selected");
    app.add_shell_node("Remove target", "cat");
    let remove_target = app.graph_state.focused.expect("remove target selected");
    app.add_shell_node("Keep target", "cat");
    let keep_target = app.graph_state.focused.expect("keep target selected");
    let condition = app.workflow.graph.node(condition_id).expect("condition node");
    let removed_port = condition.output_ports[0].id;
    let kept_port = condition.output_ports[1].id;
    let remove_input = app.workflow.graph.node(remove_target).expect("remove target").input_ports[0].id;
    let keep_input = app.workflow.graph.node(keep_target).expect("keep target").input_ports[0].id;
    app.workflow.graph.add_edge(removed_port, remove_input).expect("remove edge");
    app.workflow.graph.add_edge(kept_port, keep_input).expect("keep edge");
    app.sync_graph_state();
    app.select_node(condition_id);

    assert!(app.start_property_edit(PropertyEditField::ConditionBranches));
    app.property_edit.as_mut().expect("branch edit").draft = "added=always, keep=always".to_string();
    assert!(app.save_property_edit());

    let reconciled = app.workflow.graph.node(condition_id).expect("reconciled condition");
    assert_eq!(reconciled.output_ports[0].label, "added");
    assert_eq!(reconciled.output_ports[1].id, kept_port);
    assert!(app.workflow.graph.port(removed_port).is_none());
    assert_eq!(app.workflow.graph.edges().len(), 1);
    assert_eq!(app.workflow.graph.edges()[0].source, kept_port);
    assert_eq!(app.workflow.graph.edges()[0].target, keep_input);
    assert_eq!(app.graph_state.focused, Some(condition_id));
    assert!(app.graph_state.selected.contains(&condition_id));
}

#[test]
fn malformed_condition_property_edit_preserves_prior_topology() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_condition_node("Route", ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: "keep".to_string(),
            predicate: control_flow::ConditionPredicate::Always,
        }],
    });
    let condition_id = app.graph_state.focused.expect("condition selected");
    let prior_ports = app.workflow.graph.node(condition_id).expect("condition node").output_ports.clone();
    let prior_config = app.workflow.node_config(condition_id).cloned().expect("condition config");
    app.clear_workflow_dirty();

    assert!(app.start_property_edit(PropertyEditField::ConditionBranches));
    app.property_edit.as_mut().expect("branch edit").draft = "malformed".to_string();
    assert!(!app.save_property_edit());

    assert_eq!(app.workflow.graph.node(condition_id).expect("condition node").output_ports, prior_ports);
    match (app.workflow.node_config(condition_id), prior_config) {
        (Some(NodeConfig::Condition(actual)), NodeConfig::Condition(expected)) => assert_eq!(actual, &expected),
        _ => panic!("expected unchanged condition config"),
    }
    assert!(app.property_edit.is_some());
    assert!(!app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.agent_nodes.errors]
// r[verify n8nclone.tui_ux.agent_nodes.errors.invalid_target]
// r[verify n8nclone.tui_ux.agent_nodes.properties]
// r[verify n8nclone.tui_ux.agent_nodes.properties.image_path]
// r[verify n8nclone.tui_ux.agent_nodes.properties.invalid]
// r[verify n8nclone.tui_ux.agent_nodes.properties.prompt]
// r[verify n8nclone.tui_ux.agent_nodes.properties.target]
// r[verify n8nclone.tui_ux.properties.tests]
#[test]
fn image_and_agent_property_edits_update_config_and_reject_invalid_values() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_image_input_node("Selfie", "/tmp/old.png");
    let image_node_id = app.graph_state.focused.expect("image node focused");
    assert!(app.start_property_edit(PropertyEditField::ImagePath));
    app.property_edit.as_mut().expect("image edit").draft = "/tmp/new.png".to_string();
    assert!(app.save_property_edit());
    match app.workflow.node_config(image_node_id).expect("image config") {
        NodeConfig::ImageInput { path, .. } => assert_eq!(path, "/tmp/new.png"),
        _ => panic!("expected image input config"),
    }

    app.add_agent_node("Agent", crate::agent_node::Target::Pi, "old prompt", crate::agent_node::OutputMode::Text);
    let agent_node_id = app.graph_state.focused.expect("agent node focused");
    assert!(app.start_property_edit(PropertyEditField::AgentPrompt));
    app.property_edit.as_mut().expect("prompt edit").draft = "new prompt".to_string();
    assert!(app.save_property_edit());
    assert!(app.start_property_edit(PropertyEditField::AgentTarget));
    app.property_edit.as_mut().expect("target edit").draft = "clankers".to_string();
    assert!(app.save_property_edit());
    assert!(app.start_property_edit(PropertyEditField::AgentOutputMode));
    app.property_edit.as_mut().expect("mode edit").draft = "image".to_string();
    assert!(app.save_property_edit());
    assert!(app.start_property_edit(PropertyEditField::AgentContextRefs));
    app.property_edit.as_mut().expect("context edit").draft = "@one.png, @two.png".to_string();
    assert!(app.save_property_edit());
    assert!(app.start_property_edit(PropertyEditField::AgentSession));
    app.property_edit.as_mut().expect("session edit").draft = "session-1".to_string();
    assert!(app.save_property_edit());
    assert!(app.start_property_edit(PropertyEditField::Credentials));
    app.property_edit.as_mut().expect("credential edit").draft = "TOKEN=env:artist-token".to_string();
    assert!(app.save_property_edit());
    match app.workflow.node_config(agent_node_id).expect("agent config") {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.prompt, "new prompt");
            assert_eq!(settings.target, crate::agent_node::Target::Clankers);
            assert_eq!(settings.output_mode, crate::agent_node::OutputMode::Image);
            assert_eq!(settings.context_refs, vec!["@one.png".to_string(), "@two.png".to_string()]);
            assert_eq!(settings.session.as_deref(), Some("session-1"));
        }
        _ => panic!("expected agent config"),
    }

    let credentials = app.workflow.node_credentials(agent_node_id).expect("credentials");
    assert_eq!(credentials.env.get("TOKEN").expect("token ref").identity(), "env:artist-token");

    assert!(app.start_property_edit(PropertyEditField::AgentTarget));
    app.property_edit.as_mut().expect("bad target edit").draft = "bad-target".to_string();
    assert!(!app.save_property_edit());
    assert!(app.property_edit.is_some());
    assert!(output_text_lines(&app).iter().any(|line| line.contains("Invalid agent target")));
    app.cancel_property_edit();

    assert!(app.start_property_edit(PropertyEditField::Credentials));
    app.property_edit.as_mut().expect("bad credentials edit").draft = "not-a-ref".to_string();
    assert!(!app.save_property_edit());
    assert!(app.property_edit.is_some());
}
