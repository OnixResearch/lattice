//! Tests for the pure generic runner core.

mod common;

use std::collections::BTreeMap;

use n8nclone::engine::runner::CapturingRunnerEventSink;
use n8nclone::engine::runner::DurableRunnerDecision;
use n8nclone::engine::runner::NodeRuntime;
use n8nclone::engine::runner::RunnerAdmission;
use n8nclone::engine::runner::RunnerDiagnosticCode;
use n8nclone::engine::runner::RunnerEventKind;
use n8nclone::engine::runner::RunnerEventSink;
use n8nclone::engine::runner::RunnerStepAction;
use n8nclone::engine::runner::RunnerStepState;
use n8nclone::engine::runner::RuntimeBinding;
use n8nclone::engine::runner::RuntimeInvocation;
use n8nclone::engine::runner::RuntimeOutputShape;
use n8nclone::engine::runner::RuntimeRegistry;
use n8nclone::engine::runner::RuntimeResult;
use n8nclone::engine::runner::admit_runner_plan;
use n8nclone::engine::runner::admit_runtime_outputs;
use n8nclone::engine::runner::artifact_reference_for_value;
use n8nclone::engine::runner::build_runner_plan;
use n8nclone::engine::runner::builtin_runtime_registry;
use n8nclone::engine::runner::durable_runner_decision;
use n8nclone::engine::runner::ready_nodes;
use n8nclone::engine::runner::runner_event;
use n8nclone::engine::runner::transition_step_state;
use n8nclone::engine::runner::validate_runtime_bindings;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::NodeRunnerSettings;
use n8nclone::engine::workflow::Workflow;
use pretty_assertions::assert_eq;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use serde_json::json;

const NODE_COUNT_TWO: usize = 2;
const EDGE_COUNT_ONE: usize = 1;
const BROKEN_LABEL: &str = "broken";
const RUNTIME_KIND_SHELL: &str = "shell";
const RUNTIME_KEY_DEFAULT: &str = "default";
const FIRST_READY_COUNT: usize = 1;
const FANOUT_READY_COUNT: usize = 2;
const MISSING_RUNTIME_KIND: &str = "missing-kind";
const MISSING_RUNTIME_KEY: &str = "missing-key";
const STRING_TYPE_TAG: &str = "string";
const JSON_TYPE_TAG: &str = "json";
const SECRET_FRAGMENT: &str = "secret-token";
const REDACTED_SUMMARY: &str = "[redacted]";
const ARTIFACT_BACKEND_REF: &str = "store://artifact";
const ARTIFACT_TEXT: &str = "artifact body";
const UNDECLARED_PORT_ID: u64 = 999_777;
const EVENT_INITIAL_SEQUENCE: u64 = 0;
const FAKE_RUNTIME_NODE_ID: u64 = 0;

fn input_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").input_ports[0].id
}

fn workflow_with_cycle() -> Workflow {
    let (workflow, source, target) = common::two_node_pipeline();
    let mut value = serde_json::to_value(&workflow).expect("workflow serializes");
    let source_input = input_port_id(&workflow, source).0;
    let target_stdout = common::stdout_port_id(&workflow, target).0;
    value["graph"]["edges"].as_array_mut().expect("edges array").push(json!({
        "source": target_stdout,
        "target": source_input,
    }));
    serde_json::from_value(value).expect("cyclic workflow deserializes")
}

#[derive(Debug, Clone)]
struct FakeRuntime {
    binding: RuntimeBinding,
}

impl NodeRuntime for FakeRuntime {
    fn binding(&self) -> RuntimeBinding {
        self.binding.clone()
    }

    fn validate(
        &self,
        _node: &n8nclone::engine::runner::RunnerNode,
    ) -> Vec<n8nclone::engine::runner::RunnerDiagnostic> {
        Vec::new()
    }

    fn execute(&self, _invocation: RuntimeInvocation) -> RuntimeResult {
        RuntimeResult::Failed {
            message: "fake runtime".to_string(),
        }
    }
}

// r[verify n8nclone.workflow_execution.runner_core.plan_ir]
#[test]
fn runner_plan_preserves_typed_nodes_edges_and_runtime_bindings() {
    let (workflow, source, target) = common::two_node_pipeline();

    let build = build_runner_plan(&workflow);
    let plan = build.plan.expect("runner plan builds");

    assert!(build.diagnostics.is_empty());
    assert_eq!(plan.nodes.len(), NODE_COUNT_TWO);
    assert_eq!(plan.edges.len(), EDGE_COUNT_ONE);
    assert_eq!(plan.nodes[0].id, source.0);
    assert_eq!(plan.nodes[0].runtime.kind, RUNTIME_KIND_SHELL);
    assert_eq!(plan.nodes[0].runtime.key, RUNTIME_KEY_DEFAULT);
    assert_eq!(plan.edges[0].source_node, source.0);
    assert_eq!(plan.edges[0].target_node, target.0);
}

// r[verify n8nclone.workflow_execution.runner_core.plan_ir]
#[test]
fn runner_plan_rejects_missing_node_config_without_panicking() {
    let mut workflow = Workflow::new("missing-config");
    workflow.graph.add_node(BROKEN_LABEL, &[], &[]);

    let build = build_runner_plan(&workflow);

    assert!(build.plan.is_none());
    assert!(
        build
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == RunnerDiagnosticCode::MissingNodeConfig)
    );
}

// r[verify n8nclone.workflow_execution.runner_core.trellis_graph_admission]
#[test]
fn runner_admission_returns_deterministic_topological_order() {
    let (workflow, source, target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");

    let admission = admit_runner_plan(&plan);

    assert_eq!(admission, RunnerAdmission::Accepted {
        topo_order: vec![source.0, target.0],
    });
}

// r[verify n8nclone.workflow_execution.runner_core.trellis_graph_admission]
#[test]
fn runner_admission_rejects_cycle() {
    let workflow = workflow_with_cycle();
    let plan = build_runner_plan(&workflow).plan.expect("cyclic graph still converts to runner IR");

    let admission = admit_runner_plan(&plan);

    let RunnerAdmission::Rejected { diagnostics } = admission else {
        panic!("cycle should be rejected");
    };
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == RunnerDiagnosticCode::CycleDetected));
}

// r[verify n8nclone.workflow_execution.runner_core.trellis_graph_admission]
#[test]
fn ready_nodes_require_completed_dependencies() {
    let (workflow, source, target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let mut states = BTreeMap::new();

    let initial_ready = ready_nodes(&plan, &states);
    assert_eq!(initial_ready, vec![source.0]);

    states.insert(source.0, RunnerStepState::Completed);
    let after_source_completed = ready_nodes(&plan, &states);
    assert_eq!(after_source_completed, vec![target.0]);

    states.insert(source.0, RunnerStepState::Failed);
    assert!(ready_nodes(&plan, &states).is_empty());
}

// r[verify n8nclone.workflow_execution.runner_core.trellis_graph_admission]
#[test]
fn ready_nodes_support_fanout_after_source_completes() {
    let (workflow, source, left, right, _join) = common::diamond_workflow();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let mut states = BTreeMap::new();

    let initial_ready = ready_nodes(&plan, &states);
    assert_eq!(initial_ready.len(), FIRST_READY_COUNT);
    assert_eq!(initial_ready[0], source.0);

    states.insert(source.0, RunnerStepState::Completed);
    let after_source_completed = ready_nodes(&plan, &states);
    assert_eq!(after_source_completed.len(), FANOUT_READY_COUNT);
    assert_eq!(after_source_completed, vec![left.0, right.0]);
}

// r[verify n8nclone.workflow_execution.runner_core.lifecycle]
#[test]
fn lifecycle_admits_valid_transitions() {
    let ready =
        transition_step_state(RunnerStepState::Pending, RunnerStepAction::MarkReady).expect("pending can mark ready");
    let running = transition_step_state(ready, RunnerStepAction::Start).expect("ready can start");
    let completed = transition_step_state(running, RunnerStepAction::Complete).expect("running can complete");

    assert_eq!(ready, RunnerStepState::Ready);
    assert_eq!(running, RunnerStepState::Running);
    assert_eq!(completed, RunnerStepState::Completed);
}

// r[verify n8nclone.workflow_execution.runner_core.lifecycle]
#[test]
fn lifecycle_rejects_invalid_and_terminal_transitions() {
    assert!(transition_step_state(RunnerStepState::Pending, RunnerStepAction::Complete).is_err());
    assert!(transition_step_state(RunnerStepState::Ready, RunnerStepAction::MarkReady).is_err());
    assert!(transition_step_state(RunnerStepState::Completed, RunnerStepAction::Cancel).is_err());
    assert!(transition_step_state(RunnerStepState::Failed, RunnerStepAction::Start).is_err());
    assert!(transition_step_state(RunnerStepState::Cancelled, RunnerStepAction::MarkReady).is_err());
}

// r[verify n8nclone.workflow_execution.runner_core.lifecycle]
#[test]
fn lifecycle_admits_failure_and_cancellation_paths() {
    assert_eq!(
        transition_step_state(RunnerStepState::Running, RunnerStepAction::Fail).expect("running can fail"),
        RunnerStepState::Failed,
    );
    assert_eq!(
        transition_step_state(RunnerStepState::Pending, RunnerStepAction::Cancel).expect("pending can cancel"),
        RunnerStepState::Cancelled,
    );
    assert_eq!(
        transition_step_state(RunnerStepState::Ready, RunnerStepAction::Cancel).expect("ready can cancel"),
        RunnerStepState::Cancelled,
    );
    assert_eq!(
        transition_step_state(RunnerStepState::Running, RunnerStepAction::Cancel).expect("running can cancel"),
        RunnerStepState::Cancelled,
    );
}

// r[verify n8nclone.workflow_execution.runtime_registry.dispatch]
#[test]
fn builtin_runtime_registry_accepts_default_shell_binding() {
    let (workflow, _source, _target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");

    let diagnostics = validate_runtime_bindings(&plan, &builtin_runtime_registry());

    assert!(diagnostics.is_empty());
}

// r[verify n8nclone.workflow_execution.trait_boundaries.runtime_dispatch]
#[test]
fn node_runtime_trait_exposes_binding_and_node_scoped_failure() {
    let runtime = FakeRuntime {
        binding: RuntimeBinding {
            kind: RUNTIME_KIND_SHELL.to_string(),
            key: RUNTIME_KEY_DEFAULT.to_string(),
        },
    };

    let result = runtime.execute(RuntimeInvocation {
        node_id: FAKE_RUNTIME_NODE_ID,
        binding: runtime.binding(),
        inputs: BTreeMap::new(),
        output_ports: Vec::new(),
    });

    assert_eq!(runtime.binding().kind, RUNTIME_KIND_SHELL);
    assert!(matches!(result, RuntimeResult::Failed { .. }));
}

// r[verify n8nclone.workflow_execution.runtime_registry.errors]
#[test]
fn runtime_registry_rejects_missing_binding() {
    let (mut workflow, source, _target) = common::two_node_pipeline();
    workflow.set_node_runner_settings(source, NodeRunnerSettings::new(MISSING_RUNTIME_KIND, MISSING_RUNTIME_KEY));
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let registry = RuntimeRegistry::new([RuntimeBinding {
        kind: RUNTIME_KIND_SHELL.to_string(),
        key: RUNTIME_KEY_DEFAULT.to_string(),
    }]);

    let diagnostics = validate_runtime_bindings(&plan, &registry);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == RunnerDiagnosticCode::MissingRuntimeBinding));
}

// r[verify n8nclone.workflow_execution.runtime_registry.typed_output_contract]
#[test]
fn runtime_output_contract_accepts_declared_matching_output() {
    let (workflow, _source, _target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let node = plan.nodes.first().expect("source node exists");
    let stdout = node.output_ports.first().expect("stdout port exists");

    let result = admit_runtime_outputs(node, &[RuntimeOutputShape {
        port_id: stdout.id,
        type_tag: STRING_TYPE_TAG.to_string(),
    }]);

    assert!(result.is_ok());
}

// r[verify n8nclone.workflow_execution.runtime_registry.errors]
#[test]
fn runtime_output_contract_rejects_wrong_port_and_type() {
    let (workflow, _source, _target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let node = plan.nodes.first().expect("source node exists");
    let stdout = node.output_ports.first().expect("stdout port exists");

    let diagnostics = admit_runtime_outputs(node, &[
        RuntimeOutputShape {
            port_id: stdout.id,
            type_tag: JSON_TYPE_TAG.to_string(),
        },
        RuntimeOutputShape {
            port_id: UNDECLARED_PORT_ID,
            type_tag: STRING_TYPE_TAG.to_string(),
        },
    ])
    .expect_err("invalid outputs rejected");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == RunnerDiagnosticCode::RuntimeOutputTypeMismatch)
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == RunnerDiagnosticCode::RuntimeOutputPortMissing)
    );
}

// r[verify n8nclone.workflow_execution.runner_events.stream]
#[test]
fn runner_events_assign_ordered_sequences() {
    let first = runner_event("run:test", &[], None, None, RunnerEventKind::RunStarted, "started", &[]);
    let second = runner_event(
        "run:test",
        std::slice::from_ref(&first),
        None,
        None,
        RunnerEventKind::RunCompleted,
        "completed",
        &[],
    );

    assert_eq!(first.sequence, EVENT_INITIAL_SEQUENCE);
    assert!(second.sequence > first.sequence);
}

// r[verify n8nclone.workflow_execution.runner_events.redaction]
#[test]
fn runner_event_redacts_sensitive_summary() {
    let event = runner_event(
        "run:test",
        &[],
        None,
        None,
        RunnerEventKind::RuntimeStdout,
        format!("value={SECRET_FRAGMENT}"),
        &[SECRET_FRAGMENT.to_string()],
    );

    assert!(event.redacted);
    assert_eq!(event.summary, REDACTED_SUMMARY);
}

// r[verify n8nclone.workflow_execution.trait_boundaries.event_sink]
#[test]
fn capturing_event_sink_records_ordered_events() {
    let first = runner_event("run:test", &[], None, None, RunnerEventKind::RunStarted, "started", &[]);
    let second = runner_event(
        "run:test",
        std::slice::from_ref(&first),
        None,
        None,
        RunnerEventKind::RunCompleted,
        "completed",
        &[],
    );
    let mut sink = CapturingRunnerEventSink::default();

    sink.emit(first.clone());
    sink.emit(second.clone());

    assert_eq!(sink.events, vec![first, second]);
}

// r[verify n8nclone.workflow_execution.artifact_references.model]
// r[verify n8nclone.workflow_execution.artifact_references.trellis_identity]
#[test]
fn artifact_reference_carries_hash_type_length_and_backend() {
    let value = Value::from_string(ARTIFACT_TEXT.to_string());
    let reference = artifact_reference_for_value(&value, ARTIFACT_BACKEND_REF);

    assert_eq!(reference.content_hash, value.hash.to_hex());
    assert_eq!(reference.type_tag, STRING_TYPE_TAG);
    assert_eq!(reference.byte_length, ARTIFACT_TEXT.len());
    assert_eq!(reference.backend_ref, ARTIFACT_BACKEND_REF);
}

// r[verify n8nclone.workflow_execution.durable_runner.resume]
#[test]
fn durable_runner_decision_skips_completed_and_blocks_failed_dependencies() {
    let (workflow, source, target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let mut states = BTreeMap::new();
    states.insert(source.0, RunnerStepState::Completed);

    assert_eq!(durable_runner_decision(&plan, &states, source.0), DurableRunnerDecision::SkipCompleted);
    assert_eq!(durable_runner_decision(&plan, &states, target.0), DurableRunnerDecision::Execute);

    states.insert(source.0, RunnerStepState::Failed);
    assert_eq!(durable_runner_decision(&plan, &states, target.0), DurableRunnerDecision::BlockedByFailedDependency,);
}
