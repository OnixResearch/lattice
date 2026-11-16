use super::*;

// r[verify n8nclone.tui_ux.add_node.guided]
// r[verify n8nclone.tui_ux.add_node.shell_fields]
// r[verify n8nclone.tui_ux.workspace]
#[test]
fn add_node_flow_creates_shell_node_from_entered_fields() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_add_node_flow();
    app.start_manual_shell_node_flow();
    app.add_node_flow.label = "Custom".to_string();
    app.confirm_add_node_flow();
    app.add_node_flow.command = "echo custom".to_string();
    app.confirm_add_node_flow();

    assert_eq!(app.workflow.graph.node_count(), 1);
    let node_id = app.graph_state.focused.expect("new node focused");
    assert_eq!(app.workflow.graph.node(node_id).expect("node").label, "Custom");
    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "echo custom"),
        _ => panic!("expected shell config"),
    }
    assert!(!app.is_add_node_flow_open());
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.agent_nodes.add_flow]
// r[verify n8nclone.tui_ux.agent_nodes.add_flow.input]
#[test]
fn add_node_flow_creates_image_input_node() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_add_node_flow();
    app.start_image_input_node_flow();
    app.add_node_flow.label = "Selfie".to_string();
    app.confirm_add_node_flow();
    app.add_node_flow.command = "/tmp/selfie.png".to_string();
    app.confirm_add_node_flow();

    let node_id = app.graph_state.focused.expect("new node focused");
    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::ImageInput { path, .. } => assert_eq!(path, "/tmp/selfie.png"),
        _ => panic!("expected image input config"),
    }
    assert!(!app.is_add_node_flow_open());
    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.agent_nodes.add_flow]
// r[verify n8nclone.tui_ux.agent_nodes.add_flow.clankers]
// r[verify n8nclone.tui_ux.agent_nodes.add_flow.pi]
#[test]
fn add_node_flow_creates_pi_and_clankers_agent_nodes() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_add_node_flow();
    app.start_pi_agent_node_flow();
    app.add_node_flow.label = "Pi image agent".to_string();
    app.confirm_add_node_flow();
    app.add_node_flow.command = "make art".to_string();
    app.confirm_add_node_flow();
    let pi_node_id = app.graph_state.focused.expect("pi node focused");
    match app.workflow.node_config(pi_node_id).expect("pi node config") {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.target, crate::agent_node::Target::Pi);
            assert_eq!(settings.prompt, "make art");
        }
        _ => panic!("expected agent config"),
    }

    app.open_add_node_flow();
    app.start_clankers_agent_node_flow();
    app.add_node_flow.label = "Clankers image agent".to_string();
    app.confirm_add_node_flow();
    app.add_node_flow.command = "make ink art".to_string();
    app.confirm_add_node_flow();
    let clankers_node_id = app.graph_state.focused.expect("clankers node focused");
    match app.workflow.node_config(clankers_node_id).expect("clankers node config") {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.target, crate::agent_node::Target::Clankers);
            assert_eq!(settings.output_mode, crate::agent_node::OutputMode::Image);
        }
        _ => panic!("expected agent config"),
    }
}

// r[verify n8nclone.tui_ux.add_node.template]
#[test]
fn add_node_flow_creates_node_from_template() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.set_node_templates(vec![NodeTemplate {
        name: "grep".to_string(),
        description: "Filter text".to_string(),
        kind: "shell".to_string(),
        command: Some("grep needle".to_string()),
        parameters: Vec::new(),
        input_ports: vec![PortTemplate {
            label: "stdin".to_string(),
            type_tag: "string".to_string(),
        }],
        output_ports: vec![PortTemplate {
            label: "stdout".to_string(),
            type_tag: "string".to_string(),
        }],
    }]);

    app.open_add_node_flow();
    app.start_template_node_flow();
    app.confirm_add_node_flow();

    assert_eq!(app.workflow.graph.node_count(), 1);
    let node_id = app.graph_state.focused.expect("new node focused");
    let node = app.workflow.graph.node(node_id).expect("node");
    assert_eq!(node.label, "grep");
    assert_eq!(node.input_ports.len(), 1);
    assert_eq!(node.output_ports.len(), 1);
    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "grep needle"),
        _ => panic!("expected shell config"),
    }
}

// r[verify n8nclone.tui_ux.add_node.cancel]
// r[verify n8nclone.tui_ux.agent_nodes.add_flow.cancel]
#[test]
fn add_node_flow_cancel_preserves_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_add_node_flow();
    app.add_node_flow.label = "Should not exist".to_string();
    app.cancel_add_node_flow();

    assert_eq!(app.workflow.graph.node_count(), 0);
    assert!(!app.is_add_node_flow_open());
    assert!(!app.has_unsaved_changes());
}

// r[verify n8nclone.node_templates.parameters.prompt]
#[test]
fn add_node_flow_prompts_and_renders_template_parameters() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.set_node_templates(vec![parameterized_grep_template()]);

    app.open_add_node_flow();
    app.start_template_node_flow();
    app.confirm_add_node_flow();

    assert_eq!(app.add_node_flow.step, AddNodeStep::TemplateParameter);
    assert_eq!(app.add_node_flow.parameter_values.len(), TEMPLATE_PARAMETER_COUNT);
    assert_eq!(app.add_node_flow.parameter_index, TEMPLATE_FIRST_PARAMETER_INDEX);
    app.add_node_flow.parameter_values[TEMPLATE_FIRST_PARAMETER_INDEX] = "needle".to_string();
    app.confirm_add_node_flow();

    let node_id = app.graph_state.focused.expect("new node focused");
    match app.workflow.node_config(node_id).expect("node config") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "grep -E 'needle'"),
        _ => panic!("expected shell config"),
    }
    assert!(!app.is_add_node_flow_open());
}

// r[verify n8nclone.node_templates.parameters.prompt.cancel]
#[test]
fn cancel_template_parameter_entry_preserves_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.set_node_templates(vec![parameterized_grep_template()]);

    app.open_add_node_flow();
    app.start_template_node_flow();
    app.confirm_add_node_flow();
    app.cancel_add_node_flow();

    assert_eq!(app.workflow.graph.node_count(), 0);
    assert!(!app.is_add_node_flow_open());
    assert!(!app.has_unsaved_changes());
}

#[test]
fn invalid_template_parameter_keeps_flow_recoverable_without_node() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    let mut template = parameterized_grep_template();
    template.command = Some("head -n ${limit}".to_string());
    template.parameters = vec![TemplateParameter {
        name: "limit".to_string(),
        type_tag: TemplateParameterType::Number,
        description: "Line limit".to_string(),
        default: Some(TemplateParameterDefault::Number(TEMPLATE_NUMBER_DEFAULT)),
        required: true,
        min: Some(TEMPLATE_NUMBER_MIN),
        max: Some(TEMPLATE_NUMBER_MAX),
    }];
    app.set_node_templates(vec![template]);

    app.open_add_node_flow();
    app.start_template_node_flow();
    app.confirm_add_node_flow();
    app.add_node_flow.parameter_values[TEMPLATE_FIRST_PARAMETER_INDEX] = "not-a-number".to_string();
    app.confirm_add_node_flow();

    assert_eq!(app.workflow.graph.node_count(), 0);
    assert_eq!(app.add_node_flow.step, AddNodeStep::TemplateParameter);
    assert!(app.is_add_node_flow_open());
    assert!(output_text_lines(&app).iter().any(|line| line.contains("expects a number")));
}

#[test]
fn empty_add_node_label_uses_default() {
    assert_eq!(normalized_node_label("", "Shell 1"), "Shell 1");
    assert_eq!(normalized_node_label("  Custom  ", "Shell 1"), "Custom");
}
