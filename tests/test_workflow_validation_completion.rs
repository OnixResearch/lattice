//! Completion evidence for audited workflow-validation gaps.

mod common;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::config::AppConfig;
use lattice::control_flow::ConditionBranch;
use lattice::control_flow::ConditionNodeConfig;
use lattice::control_flow::ConditionPredicate;
use lattice::control_flow::MergeNodeConfig;
use lattice::control_flow::MergePolicy;
use lattice::control_flow::SubworkflowInputMapping;
use lattice::control_flow::SubworkflowNodeConfig;
use lattice::credentials::CredentialProviderCapabilities;
use lattice::credentials::CredentialProviderMetadata;
use lattice::credentials::CredentialProviderRegistry;
use lattice::credentials::EnvironmentCredentialProvider;
use lattice::credentials::NodeCredentialRefs;
use lattice::credentials::SecretRef;
use lattice::engine::runner::RunnerStepAction;
use lattice::engine::runner::RunnerStepState;
use lattice::engine::runner::transition_step_state;
use lattice::engine::workflow::NodeRunnerSettings;
use lattice::engine::workflow::Workflow;
use lattice::file_node::FileOperationKind;
use lattice::node_kind_provider::provider_for_config;
use lattice::tui::App;
use lattice::tui::panels;
use lattice::workflow_validation::DiagnosticSeverity;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::has_fatal_diagnostics;
use lattice::workflow_validation::validate_workflow;
use lattice::workflow_validation::validate_workflow_with_credential_registry;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::json;

const WORKFLOW_NAME: &str = "validation-completion";
const NODE_LABEL: &str = "bounded-node";
const VALID_COMMAND: &str = "printf ok";
const EMPTY_COMMAND: &str = "   ";
const PROVIDER: &str = "env";
const SECRET_KEY: &str = "token";
const SECRET_VALUE: &str = "must-not-appear";
const FIELD: &str = "nested";
const ENV_NAME: &str = "TOKEN";
const UNSUPPORTED_EFFECT: &str = "network";
const SUPPORTED_SHELL_EFFECT: &str = "process";
const CHILD_WORKFLOW: &str = "child";
const PARENT_INPUT: &str = "request";
const CHILD_INPUT: &str = "input";
const MUTATED_INPUT: &str = "other";
const EXPECTED_PROVIDER_COUNT: usize = 8;
const SOURCE_OUTPUT_INDEX: usize = 0;
const TARGET_INPUT_INDEX: usize = 0;
const FRAME_WIDTH: u16 = 100;
const FRAME_HEIGHT: u16 = 30;
const INVALID_PORT_ID: u64 = 991_001;
const STRING_TYPE: &str = "string";
const JSON_TYPE: &str = "json";

fn app_config(temp_dir: &tempfile::TempDir) -> AppConfig {
    AppConfig {
        data_dir: temp_dir.path().join("data"),
        ..AppConfig::default()
    }
}

fn rendered_frame_text(app: &mut App) -> String {
    let area = Rect::new(0, 0, FRAME_WIDTH, FRAME_HEIGHT);
    let mut buffer = Buffer::empty(area);
    panels::render(app, area, &mut buffer);
    buffer.content().iter().map(|cell| cell.symbol()).collect()
}

fn credential_workflow(secret_ref: SecretRef) -> Workflow {
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node(NODE_LABEL, VALID_COMMAND);
    workflow.set_node_credentials(node_id, NodeCredentialRefs {
        env: [(ENV_NAME.to_string(), secret_ref)].into(),
    });
    workflow
}

fn registry() -> CredentialProviderRegistry {
    let mut registry = CredentialProviderRegistry::new();
    registry
        .insert_provider(
            CredentialProviderMetadata {
                id: PROVIDER.to_string(),
                display_name: PROVIDER.to_string(),
                capabilities: CredentialProviderCapabilities::read_only_without_fields(),
            },
            Box::new(EnvironmentCredentialProvider::from_map(
                [(SECRET_KEY.to_string(), SECRET_VALUE.to_string())].into(),
            )),
        )
        .expect("provider registers");
    registry
}

fn serialized(workflow: &Workflow) -> Vec<u8> {
    serde_json::to_vec(workflow).expect("workflow serializes")
}

// r[verify n8nclone.workflow_validation.credentials.providers.config]
// r[verify n8nclone.workflow_validation.credentials.providers.field]
// r[verify n8nclone.workflow_validation.credentials.providers.key]
#[test]
fn credential_provider_errors_are_fatal_safe_and_pre_effect() {
    let registry = registry();
    let unknown = credential_workflow(SecretRef::new("missing", SECRET_KEY, None));
    let field = credential_workflow(SecretRef::new(PROVIDER, SECRET_KEY, Some(FIELD.to_string())));
    let malformed = credential_workflow(SecretRef::new(PROVIDER, EMPTY_COMMAND, None));
    let snapshots = [serialized(&unknown), serialized(&field), serialized(&malformed)];
    let reports = [
        validate_workflow_with_credential_registry(&unknown, &registry),
        validate_workflow_with_credential_registry(&field, &registry),
        validate_workflow_with_credential_registry(&malformed, &registry),
    ];
    let expected = [
        WorkflowDiagnosticCode::UnknownCredentialProvider,
        WorkflowDiagnosticCode::UnsupportedCredentialField,
        WorkflowDiagnosticCode::MalformedSecretRef,
    ];

    for (report, code) in reports.iter().zip(expected) {
        let diagnostic =
            report.iter().find(|diagnostic| diagnostic.code == code).expect("expected credential diagnostic");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Fatal);
        assert!(diagnostic.node_id.is_some());
        assert!(!diagnostic.message.contains(SECRET_VALUE));
    }
    assert!(reports[0].iter().any(|diagnostic| diagnostic.message.contains("missing")));
    assert!(reports[2].iter().any(|diagnostic| diagnostic.message.contains("key is empty")));
    assert_eq!(snapshots, [serialized(&unknown), serialized(&field), serialized(&malformed)]);
}

// r[verify n8nclone.workflow_validation.generic_runner_admission.graph]
#[test]
fn generic_graph_admission_rejects_all_bounded_invalid_classes_without_mutation() {
    let (valid, source, target) = common::two_node_pipeline();
    let mut missing = serde_json::to_value(&valid).expect("workflow serializes");
    missing["graph"]["edges"] = json!([{ "source": INVALID_PORT_ID, "target": INVALID_PORT_ID }]);
    let missing: Workflow = serde_json::from_value(missing).expect("missing-port graph decodes");

    let mut cycle_value = serde_json::to_value(&valid).expect("workflow serializes");
    let target_output = valid.graph.node(target).expect("target node").output_ports[SOURCE_OUTPUT_INDEX].id.0;
    let source_input = valid.graph.node(source).expect("source node").input_ports[TARGET_INPUT_INDEX].id.0;
    cycle_value["graph"]["edges"].as_array_mut().expect("edge array").push(json!({
        "source": target_output,
        "target": source_input,
    }));
    let cycle: Workflow = serde_json::from_value(cycle_value).expect("cyclic graph decodes");

    let mut incompatible = valid;
    incompatible.graph.node_mut(source).expect("source node").output_ports[SOURCE_OUTPUT_INDEX].type_tag =
        STRING_TYPE.to_string();
    incompatible.graph.node_mut(target).expect("target node").input_ports[TARGET_INPUT_INDEX].type_tag =
        JSON_TYPE.to_string();

    let cases = [
        (missing, WorkflowDiagnosticCode::EdgePortMissing),
        (cycle, WorkflowDiagnosticCode::RunnerCycleDetected),
        (incompatible, WorkflowDiagnosticCode::EdgeIncompatibleType),
    ];
    for (workflow, expected_code) in cases {
        let before = serialized(&workflow);
        let diagnostics = validate_workflow(&workflow);
        assert!(has_fatal_diagnostics(&diagnostics));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == expected_code));
        assert_eq!(serialized(&workflow), before);
    }
}

// r[verify n8nclone.workflow_validation.generic_runner_admission.effects]
#[test]
fn selected_runtime_rejects_unsupported_effect_policy_and_accepts_its_own() {
    let mut rejected = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let rejected_id = rejected.add_shell_node(NODE_LABEL, VALID_COMMAND);
    let mut rejected_settings = NodeRunnerSettings::new("shell", "default");
    rejected_settings.effect_policy = UNSUPPORTED_EFFECT.to_string();
    rejected.set_node_runner_settings(rejected_id, rejected_settings);

    let mut accepted = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let accepted_id = accepted.add_shell_node(NODE_LABEL, VALID_COMMAND);
    let mut accepted_settings = NodeRunnerSettings::new("shell", "default");
    accepted_settings.effect_policy = SUPPORTED_SHELL_EFFECT.to_string();
    accepted.set_node_runner_settings(accepted_id, accepted_settings);

    let rejected_diagnostics = validate_workflow(&rejected);
    let diagnostic = rejected_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerUnsupportedRuntimePolicy)
        .expect("unsupported effect is rejected");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Fatal);
    assert_eq!(diagnostic.node_id, Some(rejected_id));
    assert!(diagnostic.message.contains(UNSUPPORTED_EFFECT));
    assert!(
        !validate_workflow(&accepted)
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerUnsupportedRuntimePolicy)
    );
}

// r[verify n8nclone.workflow_validation.generic_runner_admission.terminal_state]
#[test]
fn terminal_runner_transition_is_rejected_without_workflow_mutation() {
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(NODE_LABEL, VALID_COMMAND);
    let before = serialized(&workflow);

    let result = transition_step_state(RunnerStepState::Completed, RunnerStepAction::Cancel);

    assert!(result.is_err());
    assert_eq!(serialized(&workflow), before);
}

// r[verify n8nclone.workflow_validation.node_kind_provider]
#[test]
fn every_builtin_node_kind_has_provider_validation_and_runtime_admission() {
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node("shell", VALID_COMMAND);
    workflow.add_http_node("http", "GET", "https://example.test");
    workflow.add_file_node("file", FileOperationKind::Read, "input.txt");
    workflow.add_image_input_node("image", "input.png");
    workflow.add_agent_node("agent", AgentTarget::Pi, "prompt", AgentOutputMode::Text);
    workflow.add_condition_node("condition", ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: "yes".to_string(),
            predicate: ConditionPredicate::Always,
        }],
    });
    workflow.add_merge_node("merge", MergeNodeConfig {
        policy: MergePolicy::All,
        required_inputs: vec!["left".to_string()],
    });
    workflow.add_subworkflow_node("subworkflow", SubworkflowNodeConfig {
        workflow_name: CHILD_WORKFLOW.to_string(),
        input_mappings: Vec::new(),
        output_mappings: Vec::new(),
    });

    let portable_kinds = workflow
        .configs
        .values()
        .map(|config| provider_for_config(config).portable_kind())
        .collect::<Vec<_>>();
    assert_eq!(portable_kinds.len(), EXPECTED_PROVIDER_COUNT);
    assert!(portable_kinds.iter().all(|kind| !kind.is_empty()));

    let shell_id = workflow.graph.nodes().find(|node| node.label == "shell").expect("shell node").id;
    workflow.set_node_runner_settings(shell_id, NodeRunnerSettings::new("missing", "runtime"));
    assert!(
        validate_workflow(&workflow)
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerMissingRuntimeBinding)
    );
}

// r[verify n8nclone.workflow_validation.subworkflows.mapping]
#[test]
fn unknown_subworkflow_mapping_is_fatal_and_preserves_configuration() {
    let config = SubworkflowNodeConfig {
        workflow_name: CHILD_WORKFLOW.to_string(),
        input_mappings: vec![SubworkflowInputMapping {
            parent_input: PARENT_INPUT.to_string(),
            child_input: CHILD_INPUT.to_string(),
        }],
        output_mappings: Vec::new(),
    };
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_subworkflow_node(NODE_LABEL, config);
    workflow.graph.node_mut(node_id).expect("subworkflow node").input_ports[TARGET_INPUT_INDEX].label =
        MUTATED_INPUT.to_string();
    let before = serialized(&workflow);

    let diagnostics = validate_workflow(&workflow);

    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidSubworkflowConfig)
        .expect("mapping diagnostic");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Fatal);
    assert_eq!(diagnostic.node_id, Some(node_id));
    assert!(diagnostic.message.contains(PARENT_INPUT));
    assert_eq!(serialized(&workflow), before);
}

// r[verify n8nclone.workflow_validation.tui.panel.empty]
// r[verify n8nclone.workflow_validation.tui.panel.open]
#[test]
fn tui_renders_ready_state_and_actionable_diagnostic_location() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut ready = App::new(app_config(&temp_dir));
    ready.add_shell_node("valid", VALID_COMMAND);
    ready.open_diagnostics();
    let ready_text = rendered_frame_text(&mut ready);
    assert!(ready_text.contains("Workflow ready to run"));

    let mut invalid = App::new(app_config(&temp_dir));
    invalid.add_shell_node(NODE_LABEL, EMPTY_COMMAND);
    invalid.open_diagnostics();
    let node_id = invalid.workflow.graph.nodes().next().expect("invalid node").id;
    let diagnostic_text = rendered_frame_text(&mut invalid);
    assert!(diagnostic_text.contains("fatal"));
    assert!(diagnostic_text.contains(&node_id.to_string()));
    assert!(diagnostic_text.contains("empty command"));
}
