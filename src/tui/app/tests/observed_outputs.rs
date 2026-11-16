use super::*;

#[test]
fn completed_output_lines_are_port_labeled_and_ordered() {
    let mut workflow = Workflow::new("display", crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node("grep", "grep world");
    let node = workflow.graph.node(node_id).expect("node exists");
    let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
    let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
    let mut outputs = HashMap::new();
    outputs.insert(exit_port, crate::engine::value::Value::from_exit_code(EXIT_CODE_SUCCESS));
    outputs.insert(stdout_port, crate::engine::value::Value::from_string("hello world\n".to_string()));

    assert_eq!(
        rendered_output_records(crate::tui::app::model::values::completed_output_records(
            RunId(SNAPSHOT_RUN_ID),
            &workflow,
            node_id,
            &outputs
        )),
        vec![
            "  │ stdout: hello world".to_string(),
            "  │ exit: exit code: 0".to_string()
        ],
    );
}

// r[verify n8nclone.tui_ux.image_nodes.preview.no_raw_bytes]
// r[verify n8nclone.tui_ux.image_nodes.preview.output_panel]
#[test]
fn completed_image_outputs_render_bounded_preview_metadata() {
    let mut workflow = Workflow::new("image-preview", crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_agent_node(
        "Image agent",
        crate::agent_node::Target::Clankers,
        "make art",
        crate::agent_node::OutputMode::Image,
    );
    let node = workflow.graph.node(node_id).expect("node exists");
    let image_port = node.output_ports[AGENT_IMAGE_OUTPUT_INDEX].id;
    let mut outputs = HashMap::new();
    outputs.insert(
        image_port,
        crate::engine::value::Value::from_image(ImageValue::new(
            IMAGE_PREVIEW_BYTES.to_vec(),
            "image/png".to_string(),
            ImageMetadata::default(),
        )),
    );

    let rendered = rendered_output_records(crate::tui::app::model::values::completed_output_records(
        RunId(SNAPSHOT_RUN_ID),
        &workflow,
        node_id,
        &outputs,
    ));

    assert!(rendered.iter().any(|line| line.contains("image: image/png image")));
    assert!(!rendered.iter().any(|line| line.contains("raw-image-bytes")));
}

#[test]
fn node_usage_tokens_reads_total_or_prompt_completion() {
    let mut workflow = Workflow::new("usage", crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_agent_node(
        "Agent",
        crate::agent_node::Target::Clankers,
        "work",
        crate::agent_node::OutputMode::Text,
    );
    let metadata_port = workflow
        .graph
        .node(node_id)
        .expect("node")
        .output_ports
        .iter()
        .find(|port| port.label == "metadata")
        .expect("metadata port")
        .id;
    let mut outputs = HashMap::new();
    outputs.insert(
        metadata_port,
        crate::engine::value::Value::from_json(serde_json::json!({"usage": {"total_tokens": 1234}})),
    );
    assert_eq!(crate::tui::app::model::values::node_usage_tokens(&workflow, node_id, &outputs), Some(1234));

    outputs.insert(
        metadata_port,
        crate::engine::value::Value::from_json(
            serde_json::json!({"usage": {"prompt_tokens": 100, "completion_tokens": 23}}),
        ),
    );
    assert_eq!(crate::tui::app::model::values::node_usage_tokens(&workflow, node_id, &outputs), Some(123));

    outputs.insert(metadata_port, crate::engine::value::Value::from_json(serde_json::json!({"usage": {}})));
    assert_eq!(crate::tui::app::model::values::node_usage_tokens(&workflow, node_id, &outputs), None);

    outputs.insert(
        metadata_port,
        crate::engine::value::Value::from_json(serde_json::json!({"usage": {"total_cost_microunits": 567}})),
    );
    assert_eq!(crate::tui::app::model::values::node_usage_cost(&workflow, node_id, &outputs), Some(567));
}

#[test]
fn completed_output_records_preserve_run_node_and_port_metadata() {
    let mut workflow = Workflow::new(SNAPSHOT_WORKFLOW_NAME, crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node(SNAPSHOT_NODE_LABEL, SNAPSHOT_COMMAND);
    let node = workflow.graph.node(node_id).expect("node exists");
    let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
    let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
    let mut outputs = HashMap::new();
    outputs.insert(stdout_port, crate::engine::value::Value::from_string(SNAPSHOT_STDOUT.to_string()));
    outputs.insert(exit_port, crate::engine::value::Value::from_exit_code(EXIT_CODE_SUCCESS));

    let records =
        crate::tui::app::model::values::completed_output_records(RunId(SNAPSHOT_RUN_ID), &workflow, node_id, &outputs);

    assert_eq!(records.len(), SNAPSHOT_OUTPUT_RECORD_COUNT);
    let stdout_record = records.iter().find(|record| record.port_id == stdout_port).expect("stdout record");
    assert_eq!(stdout_record.run_id, RunId(SNAPSHOT_RUN_ID));
    assert_eq!(stdout_record.node_id, node_id);
    assert_eq!(stdout_record.port_label, "stdout");
    assert_eq!(stdout_record.text, SNAPSHOT_STDOUT_TEXT);
}

#[test]
fn latest_node_output_records_return_latest_lines_in_port_order() {
    let mut workflow = Workflow::new(SNAPSHOT_WORKFLOW_NAME, crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node(SNAPSHOT_NODE_LABEL, SNAPSHOT_COMMAND);
    let other_node_id = workflow.add_shell_node(MUTATED_NODE_LABEL, SNAPSHOT_COMMAND);
    let node = workflow.graph.node(node_id).expect("node exists");
    let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
    let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
    let other_stdout_port =
        workflow.graph.node(other_node_id).expect("other node exists").output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
    let records = vec![
        RunOutputLine {
            run_id: RunId(SNAPSHOT_RUN_ID),
            node_id,
            port_id: stdout_port,
            port_label: "stdout".to_string(),
            text: OLDER_STDOUT_TEXT.to_string(),
        },
        RunOutputLine {
            run_id: RunId(SNAPSHOT_RUN_ID),
            node_id,
            port_id: exit_port,
            port_label: "exit".to_string(),
            text: "exit code: 0".to_string(),
        },
        RunOutputLine {
            run_id: RunId(SNAPSHOT_SECOND_RUN_ID),
            node_id,
            port_id: stdout_port,
            port_label: "stdout".to_string(),
            text: SNAPSHOT_STDOUT_TEXT.to_string(),
        },
        RunOutputLine {
            run_id: RunId(SNAPSHOT_SECOND_RUN_ID),
            node_id: other_node_id,
            port_id: other_stdout_port,
            port_label: "stdout".to_string(),
            text: OTHER_NODE_TEXT.to_string(),
        },
    ];

    let rendered = rendered_latest_node_output_lines(&records, &workflow, node_id);

    assert_eq!(rendered, vec![
        "  │ stdout: snapshot output".to_string(),
        "  │ exit: exit code: 0".to_string()
    ],);
}

#[test]
fn completed_output_lines_omit_empty_streams_for_no_match() {
    let mut workflow = Workflow::new("display", crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node("grep", "grep world");
    let node = workflow.graph.node(node_id).expect("node exists");
    let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
    let stderr_port = node.output_ports[SHELL_STDERR_OUTPUT_INDEX].id;
    let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
    let mut outputs = HashMap::new();
    outputs.insert(stdout_port, crate::engine::value::Value::from_string(String::new()));
    outputs.insert(stderr_port, crate::engine::value::Value::from_string(String::new()));
    outputs.insert(exit_port, crate::engine::value::Value::from_exit_code(EXIT_CODE_GREP_NO_MATCH));

    assert_eq!(
        rendered_output_records(crate::tui::app::model::values::completed_output_records(
            RunId(SNAPSHOT_RUN_ID),
            &workflow,
            node_id,
            &outputs
        )),
        vec!["  │ exit: exit code: 1".to_string()],
    );
}
