//! Tests for condition, merge, and subworkflow control-flow nodes.

use std::collections::HashMap;

use n8nclone::control_flow::ConditionBranch;
use n8nclone::control_flow::ConditionNodeConfig;
use n8nclone::control_flow::ConditionPredicate;
use n8nclone::control_flow::ControlFlowError;
use n8nclone::control_flow::MergeInputValue;
use n8nclone::control_flow::MergeNodeConfig;
use n8nclone::control_flow::MergePolicy;
use n8nclone::control_flow::SubworkflowInputMapping;
use n8nclone::control_flow::SubworkflowNodeConfig;
use n8nclone::control_flow::SubworkflowOutputMapping;
use n8nclone::control_flow::evaluate_condition;
use n8nclone::control_flow::merge_decision;
use n8nclone::control_flow::parse_condition_predicate;
use n8nclone::control_flow::validate_merge_config;
use n8nclone::control_flow::validate_subworkflow_reference;
use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::node_kind_provider::property_lines_for_config;
use n8nclone::tui::App;
use n8nclone::tui::app::PropertyEditField;
use n8nclone::workflow_format::portable_to_workflow;
use n8nclone::workflow_format::workflow_to_portable;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::has_fatal_diagnostics;
use n8nclone::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;

const READY_TEXT: &str = "ready";
const READY_BRANCH: &str = "ready_branch";
const FALLBACK_BRANCH: &str = "fallback";
const LEFT_INPUT: &str = "left";
const RIGHT_INPUT: &str = "right";
const CHILD_WORKFLOW: &str = "child-workflow";
const PARENT_WORKFLOW: &str = "parent-workflow";
const MISSING_WORKFLOW: &str = "missing-workflow";
const DECISION_LABEL: &str = "Decision";
const MERGE_LABEL: &str = "Join";
const SUBWORKFLOW_LABEL: &str = "Child";
const HTTP_HEADERS_OUTPUT_INDEX: usize = 1;
const MERGE_OUTPUT_INDEX: usize = 0;
const MERGE_RIGHT_INPUT_INDEX: usize = 1;
const SINGLE_OUTPUT_COUNT: usize = 1;
const NO_OUTPUT_COUNT: usize = 0;
const CONTROL_FLOW_NODE_COUNT: usize = 2;
const FIRST_SEQUENCE: u64 = 1;
const SECOND_SEQUENCE: u64 = 2;

fn ready_condition_config() -> ConditionNodeConfig {
    ConditionNodeConfig {
        branches: vec![
            ConditionBranch {
                label: READY_BRANCH.to_string(),
                predicate: ConditionPredicate::StringEquals(READY_TEXT.to_string()),
            },
            ConditionBranch {
                label: FALLBACK_BRANCH.to_string(),
                predicate: ConditionPredicate::BoolEquals(true),
            },
        ],
    }
}

fn merge_config(policy: MergePolicy) -> MergeNodeConfig {
    MergeNodeConfig {
        policy,
        required_inputs: vec![LEFT_INPUT.to_string(), RIGHT_INPUT.to_string()],
    }
}

fn test_app() -> App {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    App::new(n8nclone::config::AppConfig {
        data_dir: temp_dir.path().join("data"),
        ..n8nclone::config::AppConfig::default()
    })
}

fn subworkflow_config(workflow_name: &str) -> SubworkflowNodeConfig {
    SubworkflowNodeConfig {
        workflow_name: workflow_name.to_string(),
        input_mappings: vec![SubworkflowInputMapping {
            parent_input: LEFT_INPUT.to_string(),
            child_input: "stdin".to_string(),
        }],
        output_mappings: vec![SubworkflowOutputMapping {
            child_output: "stdout".to_string(),
            parent_output: "result".to_string(),
        }],
    }
}

// r[verify n8nclone.workflow_execution.control_flow.condition]
#[test]
fn condition_predicates_match_typed_values_and_reject_malformed_input() {
    let config = ConditionNodeConfig {
        branches: vec![
            ConditionBranch {
                label: "high".to_string(),
                predicate: parse_condition_predicate("gt:10").expect("numeric predicate parses"),
            },
            ConditionBranch {
                label: "has_name".to_string(),
                predicate: parse_condition_predicate("json_exists:/name").expect("JSON pointer predicate parses"),
            },
        ],
    };

    let numeric_matches = evaluate_condition(&config, Some(&Value::from_string("12".to_string())));
    let json_matches = evaluate_condition(&config, Some(&Value::from_json(serde_json::json!({ "name": "pi" }))));
    let invalid_predicate = parse_condition_predicate("gt:not-a-number");

    assert_eq!(numeric_matches, vec!["high".to_string()]);
    assert_eq!(json_matches, vec!["has_name".to_string()]);
    assert!(matches!(invalid_predicate, Err(ControlFlowError::InvalidPredicate(_))));
}

// r[verify n8nclone.workflow_execution.control_flow.condition.match]
// r[verify n8nclone.workflow_execution.control_flow.condition.no_match]
#[tokio::test]
async fn executor_emits_only_matching_condition_branch_and_allows_no_match() {
    let mut workflow = Workflow::new("condition-workflow");
    let node_id = workflow.add_condition_node(DECISION_LABEL, ready_condition_config());
    let node = workflow.graph.node(node_id).expect("condition node exists");
    let input_port = node.input_ports.first().expect("condition input port").id;
    let ready_port = node.output_ports.iter().find(|port| port.label == READY_BRANCH).expect("ready port").id;
    let executor = Executor::new();
    let mut inputs = HashMap::new();
    inputs.insert(input_port, Value::from_string(READY_TEXT.to_string()));

    let matched = executor.execute_node(&workflow, node_id, &inputs).await;
    let no_match = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    let NodeStatus::Completed { outputs, .. } = matched.status else {
        panic!("condition should complete");
    };
    assert_eq!(outputs.len(), SINGLE_OUTPUT_COUNT);
    assert_eq!(outputs.get(&ready_port).expect("ready output").as_display_string(), READY_TEXT);
    let NodeStatus::Completed { outputs, .. } = no_match.status else {
        panic!("no-match condition should complete");
    };
    assert_eq!(outputs.len(), NO_OUTPUT_COUNT);
}

// r[verify n8nclone.workflow_execution.control_flow.merge]
// r[verify n8nclone.workflow_execution.control_flow.merge.all]
// r[verify n8nclone.workflow_execution.control_flow.merge.first]
#[test]
fn merge_decision_waits_for_all_inputs_and_selects_first_deterministically() {
    let all_decision = merge_decision(&merge_config(MergePolicy::All), vec![MergeInputValue {
        label: LEFT_INPUT.to_string(),
        sequence: FIRST_SEQUENCE,
        value: Value::from_string("left-value".to_string()),
    }])
    .expect("all merge decision succeeds");
    let first_decision = merge_decision(&merge_config(MergePolicy::First), vec![
        MergeInputValue {
            label: RIGHT_INPUT.to_string(),
            sequence: SECOND_SEQUENCE,
            value: Value::from_string("right-value".to_string()),
        },
        MergeInputValue {
            label: LEFT_INPUT.to_string(),
            sequence: FIRST_SEQUENCE,
            value: Value::from_string("left-value".to_string()),
        },
    ])
    .expect("first merge decision succeeds");
    let invalid_config = MergeNodeConfig {
        policy: MergePolicy::All,
        required_inputs: vec![LEFT_INPUT.to_string(), LEFT_INPUT.to_string()],
    };

    assert!(!all_decision.ready);
    assert_eq!(all_decision.missing_inputs, vec![RIGHT_INPUT.to_string()]);
    assert!(first_decision.ready);
    assert_eq!(first_decision.selected[0].label, LEFT_INPUT);
    assert!(matches!(validate_merge_config(&invalid_config), Err(ControlFlowError::DuplicateMergeInput(_))));
}

// r[verify n8nclone.workflow_validation.subworkflows]
// r[verify n8nclone.workflow_execution.subworkflows.invocation.recursive]
#[test]
fn subworkflow_reference_check_accepts_present_child_and_rejects_missing_or_recursive_child() {
    let config = subworkflow_config(CHILD_WORKFLOW);
    let recursive = subworkflow_config(PARENT_WORKFLOW);
    let missing = subworkflow_config(MISSING_WORKFLOW);

    assert!(
        validate_subworkflow_reference(&config, &[CHILD_WORKFLOW.to_string()], &[PARENT_WORKFLOW.to_string()]).is_ok()
    );
    assert!(matches!(
        validate_subworkflow_reference(&missing, &[CHILD_WORKFLOW.to_string()], &[PARENT_WORKFLOW.to_string()]),
        Err(ControlFlowError::MissingSubworkflow(_))
    ));
    assert!(matches!(
        validate_subworkflow_reference(&recursive, &[PARENT_WORKFLOW.to_string()], &[PARENT_WORKFLOW.to_string()]),
        Err(ControlFlowError::RecursiveSubworkflow(_))
    ));
}

// r[verify n8nclone.workflow_validation.control_flow.invalid_predicate]
// r[verify n8nclone.workflow_validation.control_flow.missing_branch]
#[test]
fn validation_rejects_condition_branches_without_matching_ports() {
    let mut workflow = Workflow::new("broken-condition");
    let node_id = workflow.add_condition_node(DECISION_LABEL, ready_condition_config());
    workflow.graph.node_mut(node_id).expect("condition node exists").output_ports.clear();

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidConditionConfig)
    );
}

// r[verify n8nclone.workflow_validation.control_flow.merge_types]
#[test]
fn validation_rejects_merge_inputs_incompatible_with_typed_output() {
    let mut workflow = Workflow::new("typed-merge");
    let shell_id = workflow.add_shell_node("Text", "printf text");
    let http_id = workflow.add_http_node("HTTP", "GET", "https://example.test");
    let merge_id = workflow.add_merge_node(MERGE_LABEL, merge_config(MergePolicy::All));
    let shell_stdout = workflow.graph.node(shell_id).expect("shell node").output_ports.first().expect("stdout").id;
    let http_headers = workflow.graph.node(http_id).expect("http node").output_ports[HTTP_HEADERS_OUTPUT_INDEX].id;
    let merge_node = workflow.graph.node(merge_id).expect("merge node");
    let left_input = merge_node.input_ports.first().expect("left input").id;
    let right_input = merge_node.input_ports[MERGE_RIGHT_INPUT_INDEX].id;
    workflow.graph.add_edge(shell_stdout, left_input).expect("connect text input");
    workflow.graph.add_edge(http_headers, right_input).expect("connect JSON input");
    workflow.graph.node_mut(merge_id).expect("merge node mut").output_ports[MERGE_OUTPUT_INDEX].type_tag =
        "string".to_string();

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidMergeConfig));
}

// r[verify n8nclone.workflow_format.control_flow.export]
// r[verify n8nclone.workflow_format.control_flow.import]
// r[verify n8nclone.workflow_format.control_flow.compatibility.existing]
#[test]
fn portable_format_round_trips_control_flow_configs() {
    let mut workflow = Workflow::new("control-flow-format");
    workflow.add_condition_node(DECISION_LABEL, ready_condition_config());
    workflow.add_merge_node(MERGE_LABEL, merge_config(MergePolicy::Collect));
    workflow.add_subworkflow_node(SUBWORKFLOW_LABEL, subworkflow_config(CHILD_WORKFLOW));

    let portable = workflow_to_portable(&workflow);
    let imported = portable_to_workflow(portable).expect("portable control-flow imports");

    assert_eq!(imported.graph.node_count(), workflow.graph.node_count());
    let condition = imported.graph.nodes().find(|node| node.label == DECISION_LABEL).expect("condition node imported");
    let merge = imported.graph.nodes().find(|node| node.label == MERGE_LABEL).expect("merge node imported");
    let subworkflow = imported
        .graph
        .nodes()
        .find(|node| node.label == SUBWORKFLOW_LABEL)
        .expect("subworkflow node imported");
    assert!(matches!(imported.node_config(condition.id), Some(NodeConfig::Condition(_))));
    assert!(matches!(imported.node_config(merge.id), Some(NodeConfig::Merge(_))));
    assert!(matches!(imported.node_config(subworkflow.id), Some(NodeConfig::Subworkflow(_))));
}

// r[verify n8nclone.tui_ux.control_flow.add_flow.condition]
// r[verify n8nclone.tui_ux.control_flow.properties.invalid_predicate]
#[test]
fn tui_adds_condition_node_and_preserves_previous_predicate_on_invalid_edit() {
    let mut app = test_app();
    app.open_add_node_flow();
    app.start_condition_node_flow();
    app.add_node_flow.label = DECISION_LABEL.to_string();
    app.confirm_add_node_flow();
    app.confirm_add_node_flow();
    app.clear_workflow_dirty();

    assert_eq!(app.workflow.graph.node_count(), SINGLE_OUTPUT_COUNT);
    assert!(app.start_property_edit(PropertyEditField::ConditionBranches));
    app.property_edit.as_mut().expect("property edit active").draft = "match=invalid".to_string();
    assert!(!app.save_property_edit());

    let selected_node = app.graph_state.focused.expect("new node focused");
    let Some(NodeConfig::Condition(config)) = app.workflow.node_config(selected_node) else {
        panic!("condition config remains");
    };
    assert_eq!(config.branches[0].predicate, ConditionPredicate::Always);
    assert!(app.property_edit.is_some());
}

// r[verify n8nclone.tui_ux.control_flow.add_flow.merge]
// r[verify n8nclone.tui_ux.control_flow.add_flow.subworkflow]
// r[verify n8nclone.tui_ux.control_flow.properties.missing_subworkflow]
#[test]
fn tui_adds_merge_and_subworkflow_nodes_and_keeps_missing_reference_visible() {
    let mut app = test_app();
    app.open_add_node_flow();
    app.start_merge_node_flow();
    app.add_node_flow.label = MERGE_LABEL.to_string();
    app.confirm_add_node_flow();
    app.confirm_add_node_flow();
    app.open_add_node_flow();
    app.start_subworkflow_node_flow();
    app.add_node_flow.label = SUBWORKFLOW_LABEL.to_string();
    app.confirm_add_node_flow();
    app.add_node_flow.command = MISSING_WORKFLOW.to_string();
    app.confirm_add_node_flow();

    assert_eq!(app.workflow.graph.node_count(), CONTROL_FLOW_NODE_COUNT);
    let subworkflow = app
        .workflow
        .graph
        .nodes()
        .find(|node| node.label == SUBWORKFLOW_LABEL)
        .expect("subworkflow node exists");
    let config = app.workflow.node_config(subworkflow.id).expect("subworkflow config exists");
    let lines = property_lines_for_config(config);
    assert!(matches!(config, NodeConfig::Subworkflow(_)));
    assert!(lines.iter().any(|line| line.value == MISSING_WORKFLOW));
}
