//! Positive and negative evidence for explicit workflow expression dependencies.

use std::collections::HashMap;

use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::runner;
use lattice::engine::value::Value;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::Workflow;
use lattice::expression_dependencies;
use lattice::expressions::ExpressionPortId;
use lattice::workflow_format;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::validate_workflow;

const PRODUCER_LABEL: &str = "producer";
const CONSUMER_LABEL: &str = "consumer";
const UNRELATED_LABEL: &str = "unrelated";
const PRODUCER_COMMAND: &str = "printf producer";
const CONSUMER_COMMAND: &str = "printf consumer";
const SHELL_STDIN_INPUT_INDEX: usize = 0;
const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
const EXPRESSION_FIELD_COUNT: usize = 18;
const RESOLVED_OUTPUT: &str = "expression-ok";
const STORED_WORKFLOW_REVISION: &str = "blake3:stored-workflow";
const STORED_MACHINE_IDENTITY: &str = "blake3:stored-machine";
const PLAIN_RUNNER_IDENTITY: &str = "blake3:75effcf83d3c2e3bc0fbdcd025b9dadacf1f1e79fb664c0b56e4768993f7c8e8";
const PROJECTED_RUNNER_IDENTITY: &str = "blake3:a0469c4f953a465715c787b572835795c664f3b846e8dcd56cfed0b8ba952a1c";
const PLAIN_MACHINE_IDENTITY: &str = "47fcaf97770cc65b0fb954690eff5f35599f08474a99cecd0554b894e37478d6";
const PROJECTED_MACHINE_IDENTITY: &str = "dfed62b3fcc0efba5dc6bd43aaadd293d3e00b2c48a4ed3229168b6938043c78";
const PORTABLE_BYTE_LENGTH: usize = 2_273;
const PORTABLE_BLAKE3: &str = "3d44a887d162cf7fee87ff760cfd89fbbb9304c25447f03eb5711a77ee04b7a0";

fn stdout_port(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_STDOUT_OUTPUT_INDEX].id
}

fn stdin_port(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").input_ports[SHELL_STDIN_INPUT_INDEX].id
}

fn reference(node_id: NodeId, port_id: PortId) -> String {
    format!("{{{{node:{}:port:{}}}}}", node_id.0, port_id.0)
}

fn independent_expression_workflow() -> (Workflow, NodeId, NodeId, PortId) {
    let mut workflow = Workflow::new("explicit-expression", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let producer = workflow.add_shell_node(PRODUCER_LABEL, PRODUCER_COMMAND);
    let consumer = workflow.add_shell_node(CONSUMER_LABEL, CONSUMER_COMMAND);
    let producer_stdout = stdout_port(&workflow, producer);
    workflow.set_node_config(consumer, NodeConfig::shell(reference(producer, producer_stdout)));
    (workflow, producer, consumer, producer_stdout)
}

// r[verify lattice.workflow_validation.explicit_dependencies.extraction]
// r[verify lattice.workflow_validation.explicit_dependencies.extraction.supported-reference]
// r[verify lattice.workflow_validation.explicit_dependencies.projection]
// r[verify lattice.workflow_validation.explicit_dependencies.projection.derived-edge]
#[test]
fn expression_reference_adds_one_admitted_execution_dependency() {
    let (workflow, producer, consumer, producer_stdout) = independent_expression_workflow();

    let projection = expression_dependencies::workflow_projection(&workflow).expect("expression dependency projects");
    let plan = runner::build_runner_plan(&workflow).plan.expect("runner plan builds");
    let order = runner::topological_order(&plan).expect("combined dependency graph admits");

    assert_eq!(projection.expression_dependencies.len(), 1);
    assert_eq!(projection.expression_dependencies[0].producer_node, producer.0);
    assert_eq!(projection.expression_dependencies[0].consumer_node, consumer.0);
    assert_eq!(projection.expression_dependencies[0].producer_output_port.0, producer_stdout.0);
    assert_eq!(order, vec![producer.0, consumer.0]);
}

// r[verify lattice.workflow_validation.explicit_dependencies.inventory]
// r[verify lattice.workflow_validation.explicit_dependencies.inventory.unclassified-field]
#[test]
fn every_supported_expression_field_is_in_the_checked_inventory() {
    let mut workflow = Workflow::new("expression-inventory", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let producer = workflow.add_shell_node(PRODUCER_LABEL, PRODUCER_COMMAND);
    let producer_stdout = stdout_port(&workflow, producer);
    let expression = reference(producer, producer_stdout);

    let shell = workflow.add_shell_node("shell", CONSUMER_COMMAND);
    let mut shell_env = HashMap::new();
    shell_env.insert("VALUE".to_string(), expression.clone());
    workflow.set_node_config(shell, NodeConfig::Shell {
        command: expression.clone(),
        cwd: Some(expression.clone()),
        env: shell_env,
        timeout_secs: 1,
        pipe_stdin: false,
    });

    let http = workflow.add_http_node("http", "GET", "https://example.com");
    let mut headers = HashMap::new();
    headers.insert("X-Value".to_string(), expression.clone());
    workflow.set_node_config(http, NodeConfig::Http {
        method: expression.clone(),
        url: expression.clone(),
        headers,
        body: Some(expression.clone()),
        timeout_secs: 1,
    });

    let file = workflow.add_file_node("file", lattice::file_node::FileOperationKind::Write, "./value");
    workflow.set_node_config(file, NodeConfig::File {
        operation: lattice::file_node::FileOperationKind::Write,
        path: expression.clone(),
        content: Some(expression.clone()),
        overwrite: false,
        allowed_roots: vec![expression.clone()],
    });

    let image = workflow.add_image_input_node("image", "./image.png");
    workflow.set_node_config(image, NodeConfig::ImageInput {
        path: expression.clone(),
        allowed_roots: vec![expression.clone()],
        max_bytes: 1,
    });

    let agent = workflow.add_agent_node(
        "agent",
        lattice::agent_node::AgentTarget::Pi,
        "prompt",
        lattice::agent_node::AgentOutputMode::Text,
    );
    let mut settings = lattice::agent_node::AgentNodeSettings::new(
        lattice::agent_node::AgentTarget::Pi,
        expression.clone(),
        lattice::agent_node::AgentOutputMode::Text,
    );
    settings.model_hint = Some(expression.clone());
    settings.tool_policy = expression.clone();
    settings.context_refs = vec![expression.clone()];
    settings.cwd = Some(expression.clone());
    settings.session = Some(expression);
    workflow.set_node_config(agent, NodeConfig::Agent(settings));

    let projection = expression_dependencies::workflow_projection(&workflow).expect("inventory fields project");
    let paths = projection
        .expression_dependencies
        .iter()
        .map(|dependency| dependency.config_field_path.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(paths.len(), EXPRESSION_FIELD_COUNT);
    assert!(paths.contains("shell.command"));
    assert!(paths.contains("http.body"));
    assert!(paths.contains("file.allowed_roots[0]"));
    assert!(paths.contains("image_input.path"));
    assert!(paths.contains("agent.session"));
}

// r[verify lattice.workflow_validation.explicit_dependencies.compatibility]
// r[verify lattice.workflow_validation.explicit_dependencies.compatibility.reopen]
#[test]
fn duplicate_reference_and_existing_graph_edge_normalize_without_source_mutation() {
    let (mut workflow, producer, consumer, producer_stdout) = independent_expression_workflow();
    let consumer_stdin = stdin_port(&workflow, consumer);
    workflow.graph.add_edge(producer_stdout, consumer_stdin).expect("compatible edge");
    let duplicate = format!("{} {}", reference(producer, producer_stdout), reference(producer, producer_stdout));
    workflow.set_node_config(consumer, NodeConfig::shell(duplicate));
    let portable = workflow_format::workflow_to_portable(&workflow);
    let source_before = serde_json::to_vec(&portable).expect("portable source encodes");
    let runtime_projection = expression_dependencies::workflow_projection(&workflow).expect("runtime projection");
    let portable_projection = expression_dependencies::portable_projection(&portable).expect("portable projection");

    let lowered = workflow_format::choregraph::lower_portable(&portable).expect("combined dependency lowers");
    let source_after = serde_json::to_vec(&portable).expect("portable source re-encodes");
    let reopened = workflow_format::from_nickel_value(
        serde_json::to_value(&portable).expect("portable value encodes"),
        &std::collections::BTreeMap::new(),
    )
    .expect("portable expression workflow reopens");
    let reopened_source =
        serde_json::to_vec(&workflow_format::workflow_to_portable(&reopened)).expect("reopened source encodes");

    assert_eq!(source_after, source_before);
    assert_eq!(reopened_source, source_before);
    assert_eq!(portable_projection, runtime_projection);
    assert_eq!(lowered.machine.edges().len(), 1);
}

// r[verify lattice.workflow_validation.explicit_dependencies.projection]
// r[verify lattice.workflow_validation.explicit_dependencies.baseline]
// r[verify lattice.workflow_validation.explicit_dependencies.baseline.incidental-order]
// r[verify lattice.workflow_validation.explicit_dependencies.evidence]
// r[verify lattice.workflow_validation.explicit_dependencies.evidence.non-claim]
// r[verify lattice.workflow_validation.explicit_dependencies.projection.stale-plan]
#[test]
fn runner_and_machine_identities_bind_the_derived_dependency() {
    let mut without_dependency =
        Workflow::new("identity-binding", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let producer = without_dependency.add_shell_node(PRODUCER_LABEL, PRODUCER_COMMAND);
    let consumer = without_dependency.add_shell_node(CONSUMER_LABEL, CONSUMER_COMMAND);
    let producer_stdout = stdout_port(&without_dependency, producer);
    let mut with_dependency = without_dependency.clone();
    with_dependency.set_node_config(consumer, NodeConfig::shell(reference(producer, producer_stdout)));

    let plain_plan = runner::build_runner_plan(&without_dependency).plan.expect("plain plan builds");
    let projected_plan = runner::build_runner_plan(&with_dependency).plan.expect("projected plan builds");
    let portable_bytes =
        serde_json::to_vec(&workflow_format::workflow_to_portable(&with_dependency)).expect("projected portable bytes");
    let plain_runner_identity = runner::plan_identity(&plain_plan).expect("plain plan identity");
    let projected_runner_identity = runner::plan_identity(&projected_plan).expect("projected plan identity");
    let plain_lowered =
        workflow_format::choregraph::lower_portable(&workflow_format::workflow_to_portable(&without_dependency))
            .expect("plain workflow lowers");
    let projected_lowered =
        workflow_format::choregraph::lower_portable(&workflow_format::workflow_to_portable(&with_dependency))
            .expect("projected workflow lowers");
    let plain_machine =
        choregraph_kamacite::build_concrete_machine_artifact(&plain_lowered.machine).expect("plain machine identity");
    let projected_machine = choregraph_kamacite::build_concrete_machine_artifact(&projected_lowered.machine)
        .expect("projected machine identity");

    assert_eq!(portable_bytes.len(), PORTABLE_BYTE_LENGTH);
    assert_eq!(blake3::hash(&portable_bytes).to_hex().as_str(), PORTABLE_BLAKE3);
    assert_eq!(plain_runner_identity, PLAIN_RUNNER_IDENTITY);
    assert_eq!(projected_runner_identity, PROJECTED_RUNNER_IDENTITY);
    assert_eq!(plain_machine.identity.hex, PLAIN_MACHINE_IDENTITY);
    assert_eq!(projected_machine.identity.hex, PROJECTED_MACHINE_IDENTITY);
    assert_ne!(plain_runner_identity, projected_runner_identity);
    assert_ne!(plain_machine.identity.hex, projected_machine.identity.hex);

    let expected = runner::PersistedMachineBinding::expected_with_runner_plan(
        STORED_WORKFLOW_REVISION,
        STORED_MACHINE_IDENTITY,
        &projected_runner_identity,
    );
    let observed = runner::PersistedMachineBinding::expected_with_runner_plan(
        STORED_WORKFLOW_REVISION,
        STORED_MACHINE_IDENTITY,
        &plain_runner_identity,
    );
    let records = projected_plan
        .nodes
        .iter()
        .map(|node| runner::PersistedStepStateRecord {
            binding: observed.clone(),
            node_id: node.id,
            state: runner::PersistedStepState::Pending,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        runner::migrate_persisted_states(&projected_plan, &expected, &records),
        Err(runner::PersistedStateIssue::StaleRunnerPlanIdentity)
    );
}

// r[verify lattice.workflow_validation.explicit_dependencies.context]
// r[verify lattice.workflow_validation.explicit_dependencies.compatibility]
#[cfg(unix)]
#[tokio::test]
async fn serial_runtime_uses_derived_order_and_expression_aware_cache_keys() {
    let mut workflow = Workflow::new("runtime-expression", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let producer = workflow.add_shell_node(PRODUCER_LABEL, format!("printf 'printf {RESOLVED_OUTPUT}'"));
    let consumer = workflow.add_shell_node(CONSUMER_LABEL, CONSUMER_COMMAND);
    let producer_stdout = stdout_port(&workflow, producer);
    let consumer_stdout = stdout_port(&workflow, consumer);
    workflow.set_node_config(consumer, NodeConfig::shell(reference(producer, producer_stdout)));
    let mut executor = Executor::new();

    let (_run_id, first_steps) = executor.run(&workflow, &HashMap::new()).await;
    let first_consumer = first_steps.iter().find(|step| step.node_id == consumer).expect("consumer step exists");
    let first_outputs = match &first_consumer.status {
        NodeStatus::Completed { outputs, cached } => {
            assert!(!cached);
            outputs
        }
        status => panic!("consumer must complete: {status:?}"),
    };
    assert_eq!(first_outputs.get(&consumer_stdout).expect("consumer stdout").as_display_string(), RESOLVED_OUTPUT);

    let cache = first_steps
        .iter()
        .filter_map(|step| match &step.status {
            NodeStatus::Completed { outputs, .. } => Some((step.input_hash, outputs.clone())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let (_run_id, cached_steps) = executor.run(&workflow, &cache).await;

    assert!(cached_steps.iter().all(|step| matches!(step.status, NodeStatus::Completed { cached: true, .. })));
}

// r[verify lattice.workflow_validation.explicit_dependencies.context]
// r[verify lattice.workflow_validation.explicit_dependencies.context.unrelated-output]
#[test]
fn expression_context_contains_only_admitted_references() {
    let (mut workflow, producer, consumer, producer_stdout) = independent_expression_workflow();
    let unrelated = workflow.add_shell_node(UNRELATED_LABEL, "printf unrelated");
    let unrelated_stdout = stdout_port(&workflow, unrelated);
    let projection = expression_dependencies::workflow_projection(&workflow).expect("expression dependency projects");
    let mut values = HashMap::new();
    values.insert(producer_stdout, Value::from_string("producer-value".to_string()));
    values.insert(unrelated_stdout, Value::from_string("unrelated-value".to_string()));

    let context =
        expression_dependencies::context_for_consumer(&projection.expression_dependencies, consumer.0, &values)
            .expect("required value is available");

    assert!(context.contains(producer.0, ExpressionPortId(producer_stdout.0)));
    assert!(!context.contains(unrelated.0, ExpressionPortId(unrelated_stdout.0)));
}

// r[verify lattice.workflow_validation.explicit_dependencies.admission]
// r[verify lattice.workflow_validation.explicit_dependencies.admission.missing-port]
#[test]
fn missing_producer_port_and_self_reference_fail_validation() {
    let (mut missing_port, producer, consumer, _producer_stdout) = independent_expression_workflow();
    missing_port.set_node_config(consumer, NodeConfig::shell(format!("{{{{node:{}:port:{}}}}}", producer.0, u64::MAX)));
    let missing_diagnostics = validate_workflow(&missing_port);

    let (mut self_reference, _producer, consumer, _producer_stdout) = independent_expression_workflow();
    let consumer_stdout = stdout_port(&self_reference, consumer);
    self_reference.set_node_config(consumer, NodeConfig::shell(reference(consumer, consumer_stdout)));
    let self_diagnostics = validate_workflow(&self_reference);

    assert!(
        missing_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::ExpressionOutputPortMissing)
    );
    assert!(
        self_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::ExpressionSelfReference)
    );
}

// r[verify lattice.workflow_validation.explicit_dependencies.admission]
// r[verify lattice.workflow_validation.explicit_dependencies.admission.cycle]
#[test]
fn combined_dependency_cycle_fails_before_execution() {
    let (mut workflow, producer, consumer, producer_stdout) = independent_expression_workflow();
    workflow
        .graph
        .add_edge(stdout_port(&workflow, consumer), stdin_port(&workflow, producer))
        .expect("reverse graph edge is shape-valid");
    let diagnostics = validate_workflow(&workflow);
    let portable = workflow_format::workflow_to_portable(&workflow);
    let lowering = workflow_format::choregraph::lower_portable(&portable);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerCycleDetected));
    assert!(lowering.is_err());
    assert_eq!(producer_stdout, stdout_port(&workflow, producer));
}

// r[verify lattice.workflow_validation.explicit_dependencies.negative]
#[test]
fn malformed_and_unsupported_expression_forms_fail_closed() {
    let (mut malformed, _producer, consumer, _producer_stdout) = independent_expression_workflow();
    malformed.set_node_config(consumer, NodeConfig::shell("unexpected }}"));
    let malformed_diagnostics = validate_workflow(&malformed);

    let (mut unsupported, _producer, consumer, _producer_stdout) = independent_expression_workflow();
    let mut settings = lattice::agent_node::AgentNodeSettings::new(
        lattice::agent_node::AgentTarget::Pi,
        "prompt",
        lattice::agent_node::AgentOutputMode::Text,
    );
    settings.prompt_file = Some("{{node:1:port:2}}".to_string());
    unsupported.set_node_config(consumer, NodeConfig::Agent(settings));
    let unsupported_diagnostics = validate_workflow(&unsupported);

    assert!(
        malformed_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidExpressionSyntax)
    );
    assert!(
        unsupported_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::UnsupportedExpressionField)
    );
}

// r[verify lattice.workflow_validation.explicit_dependencies.negative]
// r[verify lattice.workflow_validation.explicit_dependencies.negative.serial-order]
// r[verify lattice.workflow_validation.explicit_dependencies.validation]
// r[verify lattice.workflow_validation.explicit_dependencies.validation.context-rail]
#[cfg(unix)]
#[tokio::test]
async fn invalid_dependency_prevents_shell_effects() {
    let temp = tempfile::tempdir().expect("temporary effect directory");
    let marker = temp.path().join("effect-marker");
    let mut workflow = Workflow::new("pre-effect-rejection", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let command = format!("printf touched > {}; {{{{node:{}:port:{}}}}}", marker.display(), u64::MAX, u64::MAX);
    workflow.add_shell_node(CONSUMER_LABEL, command);
    let mut executor = Executor::new();

    let (_run_id, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert!(steps.is_empty());
    assert!(!marker.exists());
}

// r[verify lattice.workflow_validation.explicit_dependencies.context]
// r[verify lattice.workflow_validation.explicit_dependencies.context.missing-value]
#[test]
fn missing_runtime_expression_value_fails_context_preparation() {
    let (workflow, _producer, consumer, _producer_stdout) = independent_expression_workflow();
    let projection = expression_dependencies::workflow_projection(&workflow).expect("expression dependency projects");

    let error =
        expression_dependencies::context_for_consumer(&projection.expression_dependencies, consumer.0, &HashMap::new())
            .expect_err("missing admitted value rejects context");

    assert_eq!(error.consumer_node, consumer.0);
    assert_eq!(error.config_field_path, "shell.command");
}
