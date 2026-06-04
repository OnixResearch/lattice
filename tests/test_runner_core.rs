//! Tests for the pure generic runner core.

mod common;

use std::collections::BTreeMap;

use n8nclone::engine::runner::RunnerAdmission;
use n8nclone::engine::runner::RunnerDiagnosticCode;
use n8nclone::engine::runner::RunnerStepAction;
use n8nclone::engine::runner::RunnerStepState;
use n8nclone::engine::runner::admit_runner_plan;
use n8nclone::engine::runner::build_runner_plan;
use n8nclone::engine::runner::ready_nodes;
use n8nclone::engine::runner::transition_step_state;
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
