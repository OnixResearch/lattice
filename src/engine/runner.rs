//! Pure generic runner IR and admission decisions.
//!
//! This module is the functional core for generic pipeline execution. It adapts
//! persisted workflow graph data into a stable runner IR and evaluates graph
//! admission, dependency readiness, and lifecycle transitions without I/O.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortDirection;
use rat_nodegraph::model::PortId;
use serde::Deserialize;
use serde::Serialize;

use crate::agent_node;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;

const RUNTIME_KIND_AGENT: &str = "agent";
const RUNTIME_KIND_FILE: &str = "file";
const RUNTIME_KIND_HTTP: &str = "http";
const RUNTIME_KIND_IMAGE_INPUT: &str = "image_input";
const RUNTIME_KIND_SHELL: &str = "shell";
const RUNTIME_KEY_DEFAULT: &str = "default";
const RUNTIME_KEY_READ_FILE: &str = "read";
const RUNTIME_KEY_WRITE_FILE: &str = "write";
const RUNTIME_KEY_APPEND_FILE: &str = "append";
const TYPE_TAG_ANY: &str = "any";
const EDGE_COUNT_EMPTY: usize = 0;

pub type RunnerNodeId = u64;
pub type RunnerPortId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerWorkflow {
    pub name: String,
    pub nodes: Vec<RunnerNode>,
    pub edges: Vec<RunnerEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerNode {
    pub id: RunnerNodeId,
    pub label: String,
    pub kind: String,
    pub runtime: RuntimeBinding,
    pub input_ports: Vec<RunnerPort>,
    pub output_ports: Vec<RunnerPort>,
    pub credential_ref_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBinding {
    pub kind: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerPort {
    pub id: RunnerPortId,
    pub label: String,
    pub direction: RunnerPortDirection,
    pub type_tag: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerPortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerEdge {
    pub source_node: RunnerNodeId,
    pub source_port: RunnerPortId,
    pub target_node: RunnerNodeId,
    pub target_port: RunnerPortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerDiagnosticCode {
    MissingNodeConfig,
    ConfigWithoutNode,
    EdgePortMissing,
    EdgeWrongDirection,
    EdgeSelfLoop,
    EdgeIncompatibleType,
    CycleDetected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerDiagnostic {
    pub code: RunnerDiagnosticCode,
    pub node_id: Option<NodeId>,
    pub port_id: Option<PortId>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerPlanBuild {
    pub plan: Option<RunnerWorkflow>,
    pub diagnostics: Vec<RunnerDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerStepState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerStepAction {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerTransitionError {
    InvalidTransition {
        from: RunnerStepState,
        action: RunnerStepAction,
    },
    TerminalState {
        from: RunnerStepState,
        action: RunnerStepAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerAdmission {
    Accepted { topo_order: Vec<RunnerNodeId> },
    Rejected { diagnostics: Vec<RunnerDiagnostic> },
}

// r[impl n8nclone.workflow_execution.runner_core.plan_ir]
pub fn build_runner_plan(workflow: &Workflow) -> RunnerPlanBuild {
    let mut diagnostics = Vec::new();
    let mut nodes = Vec::new();

    for node in workflow.graph.nodes() {
        let Some(config) = workflow.node_config(node.id) else {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::MissingNodeConfig,
                Some(node.id),
                None,
                format!("node '{}' has no execution config", node.label),
            ));
            continue;
        };
        nodes.push(RunnerNode {
            id: node.id.0,
            label: node.label.clone(),
            kind: config.kind_label().to_string(),
            runtime: runtime_binding_for_config(config),
            input_ports: node.input_ports.iter().map(runner_port_from_graph_port).collect(),
            output_ports: node.output_ports.iter().map(runner_port_from_graph_port).collect(),
            credential_ref_count: workflow.node_credentials(node.id).map_or(EDGE_COUNT_EMPTY, credential_ref_count),
        });
    }

    for node_id in workflow.configs.keys() {
        let node_id = NodeId(*node_id);
        if workflow.graph.node(node_id).is_none() {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::ConfigWithoutNode,
                Some(node_id),
                None,
                "node config references a missing graph node".to_string(),
            ));
        }
    }

    let edges = runner_edges_from_workflow(workflow, &mut diagnostics);
    if diagnostics.is_empty() {
        RunnerPlanBuild {
            plan: Some(RunnerWorkflow {
                name: workflow.meta.name.clone(),
                nodes,
                edges,
            }),
            diagnostics,
        }
    } else {
        RunnerPlanBuild {
            plan: None,
            diagnostics,
        }
    }
}

fn runtime_binding_for_config(config: &NodeConfig) -> RuntimeBinding {
    match config {
        NodeConfig::Shell { .. } => runtime_binding(RUNTIME_KIND_SHELL, RUNTIME_KEY_DEFAULT),
        NodeConfig::Http { .. } => runtime_binding(RUNTIME_KIND_HTTP, RUNTIME_KEY_DEFAULT),
        NodeConfig::File { operation, .. } => runtime_binding(RUNTIME_KIND_FILE, file_runtime_key(*operation)),
        NodeConfig::ImageInput { .. } => runtime_binding(RUNTIME_KIND_IMAGE_INPUT, RUNTIME_KEY_DEFAULT),
        NodeConfig::Agent(settings) => {
            runtime_binding(RUNTIME_KIND_AGENT, agent_node::agent_target_label(settings.target))
        }
    }
}

fn runtime_binding(kind: &str, key: &str) -> RuntimeBinding {
    RuntimeBinding {
        kind: kind.to_string(),
        key: key.to_string(),
    }
}

fn file_runtime_key(operation: crate::file_node::FileOperationKind) -> &'static str {
    match operation {
        crate::file_node::FileOperationKind::Read => RUNTIME_KEY_READ_FILE,
        crate::file_node::FileOperationKind::Write => RUNTIME_KEY_WRITE_FILE,
        crate::file_node::FileOperationKind::Append => RUNTIME_KEY_APPEND_FILE,
    }
}

fn credential_ref_count(refs: &crate::credentials::NodeCredentialRefs) -> usize {
    refs.env.len()
}

fn runner_port_from_graph_port(port: &rat_nodegraph::model::Port) -> RunnerPort {
    RunnerPort {
        id: port.id.0,
        label: port.label.clone(),
        direction: match port.direction {
            PortDirection::Input => RunnerPortDirection::Input,
            PortDirection::Output => RunnerPortDirection::Output,
        },
        type_tag: port.type_tag.clone(),
    }
}

fn runner_edges_from_workflow(workflow: &Workflow, diagnostics: &mut Vec<RunnerDiagnostic>) -> Vec<RunnerEdge> {
    let mut edges = Vec::new();
    for edge in workflow.graph.edges() {
        let Some(source_port) = workflow.graph.port(edge.source) else {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::EdgePortMissing,
                None,
                Some(edge.source),
                "edge source port is missing".to_string(),
            ));
            continue;
        };
        let Some(target_port) = workflow.graph.port(edge.target) else {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::EdgePortMissing,
                None,
                Some(edge.target),
                "edge target port is missing".to_string(),
            ));
            continue;
        };
        if source_port.direction != PortDirection::Output || target_port.direction != PortDirection::Input {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::EdgeWrongDirection,
                None,
                Some(edge.source),
                "edge must connect an output port to an input port".to_string(),
            ));
            continue;
        }
        let source_node = workflow.graph.port_owner(edge.source);
        let target_node = workflow.graph.port_owner(edge.target);
        if source_node == target_node {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::EdgeSelfLoop,
                source_node,
                None,
                "edge connects ports on the same node".to_string(),
            ));
            continue;
        }
        if !port_types_compatible(&source_port.type_tag, &target_port.type_tag) {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::EdgeIncompatibleType,
                target_node,
                Some(edge.target),
                format!("edge connects incompatible types '{}' to '{}'", source_port.type_tag, target_port.type_tag),
            ));
            continue;
        }
        if let (Some(source_node), Some(target_node)) = (source_node, target_node) {
            edges.push(RunnerEdge {
                source_node: source_node.0,
                source_port: edge.source.0,
                target_node: target_node.0,
                target_port: edge.target.0,
            });
        }
    }
    edges
}

fn port_types_compatible(source: &str, target: &str) -> bool {
    source == target || source == TYPE_TAG_ANY || target == TYPE_TAG_ANY
}

// r[impl n8nclone.workflow_execution.runner_core.trellis_graph_admission]
pub fn admit_runner_plan(plan: &RunnerWorkflow) -> RunnerAdmission {
    match topological_order(plan) {
        Ok(topo_order) => RunnerAdmission::Accepted { topo_order },
        Err(diagnostic) => RunnerAdmission::Rejected {
            diagnostics: vec![diagnostic],
        },
    }
}

// r[impl n8nclone.workflow_execution.runner_core.trellis_graph_admission]
pub fn topological_order(plan: &RunnerWorkflow) -> Result<Vec<RunnerNodeId>, RunnerDiagnostic> {
    let mut incoming_counts = initial_incoming_counts(&plan.nodes, &plan.edges);
    let mut outgoing = outgoing_edges_by_source(&plan.edges);
    let mut ready = initial_zero_incoming_nodes(&incoming_counts);
    let mut topo_order = Vec::with_capacity(plan.nodes.len());

    while let Some(node_id) = pop_first_ready_node(&mut ready) {
        topo_order.push(node_id);
        if let Some(targets) = outgoing.remove(&node_id) {
            for target in targets {
                decrement_incoming_count(&mut incoming_counts, target, &mut ready);
            }
        }
    }

    if topo_order.len() == plan.nodes.len() {
        Ok(topo_order)
    } else {
        Err(runner_diagnostic(
            RunnerDiagnosticCode::CycleDetected,
            None,
            None,
            "runner graph contains a cycle".to_string(),
        ))
    }
}

fn initial_incoming_counts(nodes: &[RunnerNode], edges: &[RunnerEdge]) -> BTreeMap<RunnerNodeId, usize> {
    let mut incoming_counts = nodes.iter().map(|node| (node.id, EDGE_COUNT_EMPTY)).collect::<BTreeMap<_, _>>();
    for edge in edges {
        if let Some(incoming_count) = incoming_counts.get_mut(&edge.target_node) {
            *incoming_count += 1;
        }
    }
    incoming_counts
}

fn outgoing_edges_by_source(edges: &[RunnerEdge]) -> BTreeMap<RunnerNodeId, Vec<RunnerNodeId>> {
    let mut outgoing = BTreeMap::new();
    for edge in edges {
        outgoing.entry(edge.source_node).or_insert_with(Vec::new).push(edge.target_node);
    }
    for targets in outgoing.values_mut() {
        targets.sort_unstable();
    }
    outgoing
}

fn initial_zero_incoming_nodes(incoming_counts: &BTreeMap<RunnerNodeId, usize>) -> BTreeSet<RunnerNodeId> {
    incoming_counts
        .iter()
        .filter_map(|(node_id, incoming_count)| (*incoming_count == EDGE_COUNT_EMPTY).then_some(*node_id))
        .collect()
}

fn pop_first_ready_node(ready: &mut BTreeSet<RunnerNodeId>) -> Option<RunnerNodeId> {
    let node_id = ready.first().copied()?;
    ready.remove(&node_id);
    Some(node_id)
}

fn decrement_incoming_count(
    incoming_counts: &mut BTreeMap<RunnerNodeId, usize>,
    node_id: RunnerNodeId,
    ready: &mut BTreeSet<RunnerNodeId>,
) {
    let Some(incoming_count) = incoming_counts.get_mut(&node_id) else {
        return;
    };
    if *incoming_count == EDGE_COUNT_EMPTY {
        return;
    }
    *incoming_count -= 1;
    if *incoming_count == EDGE_COUNT_EMPTY {
        ready.insert(node_id);
    }
}

// r[impl n8nclone.workflow_execution.runner_core.trellis_graph_admission]
pub fn ready_nodes(plan: &RunnerWorkflow, states: &BTreeMap<RunnerNodeId, RunnerStepState>) -> Vec<RunnerNodeId> {
    let mut ready = Vec::new();
    for node in &plan.nodes {
        if step_state_for_node(states, node.id) == RunnerStepState::Pending
            && dependencies_completed(plan, states, node.id)
        {
            ready.push(node.id);
        }
    }
    ready.sort_unstable();
    ready
}

fn step_state_for_node(states: &BTreeMap<RunnerNodeId, RunnerStepState>, node_id: RunnerNodeId) -> RunnerStepState {
    states.get(&node_id).copied().unwrap_or(RunnerStepState::Pending)
}

fn dependencies_completed(
    plan: &RunnerWorkflow,
    states: &BTreeMap<RunnerNodeId, RunnerStepState>,
    node_id: RunnerNodeId,
) -> bool {
    plan.edges
        .iter()
        .filter(|edge| edge.target_node == node_id)
        .all(|edge| step_state_for_node(states, edge.source_node) == RunnerStepState::Completed)
}

// r[impl n8nclone.workflow_execution.runner_core.lifecycle]
pub fn transition_step_state(
    from: RunnerStepState,
    action: RunnerStepAction,
) -> Result<RunnerStepState, RunnerTransitionError> {
    if is_terminal_step_state(from) {
        return Err(RunnerTransitionError::TerminalState { from, action });
    }
    match (from, action) {
        (RunnerStepState::Pending, RunnerStepAction::MarkReady) => Ok(RunnerStepState::Ready),
        (RunnerStepState::Ready, RunnerStepAction::Start) => Ok(RunnerStepState::Running),
        (RunnerStepState::Running, RunnerStepAction::Complete) => Ok(RunnerStepState::Completed),
        (RunnerStepState::Running, RunnerStepAction::Fail) => Ok(RunnerStepState::Failed),
        (RunnerStepState::Pending | RunnerStepState::Ready | RunnerStepState::Running, RunnerStepAction::Cancel) => {
            Ok(RunnerStepState::Cancelled)
        }
        _ => Err(RunnerTransitionError::InvalidTransition { from, action }),
    }
}

pub fn is_terminal_step_state(state: RunnerStepState) -> bool {
    matches!(state, RunnerStepState::Completed | RunnerStepState::Failed | RunnerStepState::Cancelled)
}

fn runner_diagnostic(
    code: RunnerDiagnosticCode,
    node_id: Option<NodeId>,
    port_id: Option<PortId>,
    message: String,
) -> RunnerDiagnostic {
    RunnerDiagnostic {
        code,
        node_id,
        port_id,
        message,
    }
}
