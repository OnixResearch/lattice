//! Pure generic runner IR and admission decisions.
//!
//! This module is the functional core for generic pipeline execution. It adapts
//! persisted workflow graph data into a stable runner IR and evaluates graph
//! admission, dependency readiness, and lifecycle transitions without I/O.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::workflow::PortDirection;
use crate::engine::workflow::PortId;

/// Decision returned when durable execution considers resuming a node.
pub type DurableRunnerDecision = seaglass_execution_graph::ResumeDecision;
/// Requested transition for a runner step lifecycle.
pub type RunnerStepAction = seaglass_execution_graph::StepAction;
/// Durable lifecycle state of one runner step.
pub type RunnerStepState = seaglass_execution_graph::StepState;
/// Error returned when a runner step lifecycle transition is invalid.
pub type RunnerTransitionError = seaglass_execution_graph::StepTransitionError;

pub const LEGACY_MACHINE_STATE_SCHEMA: &str = "seaglass.execution-graph.state.v1";
pub const CHOREGRAPH_MACHINE_STATE_SCHEMA: &str = choregraph_core::MACHINE_SCHEMA_V1;
const RUNNER_PLAN_IDENTITY_DOMAIN: &[u8] = b"lattice.runner-plan.v1\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedMachineBinding {
    pub old_schema: String,
    pub new_schema: String,
    pub workflow_revision: String,
    pub machine_identity: String,
    #[serde(default = "empty_plan_identity")]
    pub runner_plan_identity: String,
}

fn empty_plan_identity() -> String {
    String::new()
}

impl PersistedMachineBinding {
    pub fn expected(workflow_revision: impl Into<String>, machine_identity: impl Into<String>) -> Self {
        Self {
            old_schema: LEGACY_MACHINE_STATE_SCHEMA.to_string(),
            new_schema: CHOREGRAPH_MACHINE_STATE_SCHEMA.to_string(),
            workflow_revision: workflow_revision.into(),
            machine_identity: machine_identity.into(),
            runner_plan_identity: String::new(),
        }
    }

    pub fn expected_with_runner_plan(
        workflow_revision: impl Into<String>,
        machine_identity: impl Into<String>,
        runner_plan_identity: impl Into<String>,
    ) -> Self {
        Self {
            old_schema: LEGACY_MACHINE_STATE_SCHEMA.to_string(),
            new_schema: CHOREGRAPH_MACHINE_STATE_SCHEMA.to_string(),
            workflow_revision: workflow_revision.into(),
            machine_identity: machine_identity.into(),
            runner_plan_identity: runner_plan_identity.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedStepState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedStepStateRecord {
    pub binding: PersistedMachineBinding,
    pub node_id: RunnerNodeId,
    pub state: PersistedStepState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistedStateIssue {
    UnsupportedSchema,
    StaleWorkflowRevision,
    StaleMachineIdentity,
    StaleRunnerPlanIdentity,
    ForeignNode,
    AmbiguousNode,
    PartialState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedReplayIdentity {
    pub expected_machine: [u8; choregraph_core::TRACE_BINDING_BYTES],
    pub expected_initial_state: [u8; choregraph_core::TRACE_BINDING_BYTES],
    pub observed_machine: [u8; choregraph_core::TRACE_BINDING_BYTES],
    pub observed_initial_state: [u8; choregraph_core::TRACE_BINDING_BYTES],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedMachineEvent {
    pub node_id: RunnerNodeId,
    pub action: RunnerStepAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedReplayError {
    Graph(RunnerDiagnostic),
    State(seaglass_execution_graph::StateConversionError),
}

use crate::agent_node;
use crate::engine::value::Value;
use crate::engine::value::ValueData;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::NodeRunnerSettings;
use crate::engine::workflow::Workflow;

const RUNTIME_KIND_AGENT: &str = "agent";
const RUNTIME_KIND_CONDITION: &str = "condition";
const RUNTIME_KIND_FILE: &str = "file";
const RUNTIME_KIND_HTTP: &str = "http";
const RUNTIME_KIND_IMAGE_INPUT: &str = "image_input";
const RUNTIME_KIND_MERGE: &str = "merge";
const RUNTIME_KIND_PROCESS: &str = "process";
const RUNTIME_KIND_SHELL: &str = "shell";
const RUNTIME_KIND_SUBWORKFLOW: &str = "subworkflow";
const RUNTIME_KIND_APPROVAL: &str = "approval";
const RUNTIME_KEY_DEFAULT: &str = "default";
const RUNTIME_KEY_READ_FILE: &str = "read";
const RUNTIME_KEY_WRITE_FILE: &str = "write";
const RUNTIME_KEY_APPEND_FILE: &str = "append";
const EFFECT_POLICY_AGENT: &str = "model";
const EFFECT_POLICY_FILE: &str = "filesystem";
const EFFECT_POLICY_HTTP: &str = "network";
const EFFECT_POLICY_PROCESS: &str = "process";
const EFFECT_POLICY_PURE: &str = "pure";
const TYPE_TAG_ANY: &str = "any";
const EMPTY_CREDENTIAL_REF_COUNT: usize = 0;
const RUNNER_EVENT_INITIAL_SEQUENCE: u64 = 0;
const RUNNER_EVENT_SEQUENCE_INCREMENT: u64 = 1;
const REDACTED_SUMMARY: &str = "[redacted]";

pub type RunnerNodeId = u64;
pub type RunnerPortId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerWorkflow {
    pub name: String,
    pub nodes: Vec<RunnerNode>,
    pub edges: Vec<RunnerEdge>,
    pub expression_dependencies: Vec<crate::expression_dependencies::ExpressionDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerNode {
    pub id: RunnerNodeId,
    pub label: String,
    pub kind: String,
    pub runtime: RuntimeBinding,
    pub policy: RunnerPolicy,
    pub input_ports: Vec<RunnerPort>,
    pub output_ports: Vec<RunnerPort>,
    pub credential_ref_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuntimeBinding {
    pub kind: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerPolicy {
    pub effect_policy: String,
    pub cache_policy: String,
    pub retry_policy: String,
    pub artifact_policy: String,
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
    InvalidExpressionDependency,
    CycleDetected,
    MissingRuntimeBinding,
    UnsupportedRuntimePolicy,
    RuntimeOutputPortMissing,
    RuntimeOutputTypeMismatch,
    RuntimeFailure,
    TerminalStateMutation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerDiagnostic {
    pub code: RunnerDiagnosticCode,
    pub node_id: Option<crate::engine::workflow::NodeId>,
    pub port_id: Option<PortId>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerPlanBuild {
    pub plan: Option<RunnerWorkflow>,
    pub diagnostics: Vec<RunnerDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerAdmission {
    Accepted { topo_order: Vec<RunnerNodeId> },
    Rejected { diagnostics: Vec<RunnerDiagnostic> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRegistry {
    bindings: BTreeSet<RuntimeBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOutputShape {
    pub port_id: RunnerPortId,
    pub type_tag: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeInvocation {
    pub node_id: RunnerNodeId,
    pub binding: RuntimeBinding,
    pub inputs: BTreeMap<RunnerPortId, Value>,
    pub output_ports: Vec<RunnerPort>,
}

#[derive(Debug, Clone)]
pub struct RuntimeOutputValue {
    pub port_id: RunnerPortId,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum RuntimeResult {
    Completed { outputs: Vec<RuntimeOutputValue> },
    Failed { message: String },
}

// r[impl n8nclone.workflow_execution.trait_boundaries.runtime_dispatch]
pub trait NodeRuntime: Send + Sync {
    fn binding(&self) -> RuntimeBinding;
    fn validate(&self, node: &RunnerNode) -> Vec<RunnerDiagnostic>;
    fn execute(&self, invocation: RuntimeInvocation) -> RuntimeResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerEventKind {
    RunStarted,
    NodeReady,
    NodeStarted,
    CacheHit,
    RuntimeStdout,
    RuntimeStderr,
    ArtifactCreated,
    NodeCompleted,
    NodeFailed,
    NodeCancelled,
    RunCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerEvent {
    pub run_id: String,
    pub sequence: u64,
    pub node_id: Option<RunnerNodeId>,
    pub port_id: Option<RunnerPortId>,
    pub kind: RunnerEventKind,
    pub summary: String,
    pub redacted: bool,
}

// r[impl n8nclone.workflow_execution.trait_boundaries.event_sink]
pub trait RunnerEventSink {
    fn emit(&mut self, event: RunnerEvent);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapturingRunnerEventSink {
    pub events: Vec<RunnerEvent>,
}

// r[impl n8nclone.workflow_execution.trait_boundaries.event_sink]
impl RunnerEventSink for CapturingRunnerEventSink {
    fn emit(&mut self, event: RunnerEvent) {
        self.events.push(event);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub content_hash: String,
    pub type_tag: String,
    pub media_type: Option<String>,
    pub byte_length: usize,
    pub backend_ref: String,
    pub sensitive: bool,
    pub summary: String,
}

/// Node ranked by the number of transitive dependents it can block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bottleneck {
    /// Runner node identifier.
    pub node_id: RunnerNodeId,
    /// Number of transitive dependents blocked by this node.
    pub dependent_count: u32,
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
        let runner_settings = workflow.node_runner_settings(node.id);
        nodes.push(RunnerNode {
            id: node.id.0,
            label: node.label.clone(),
            kind: config.kind_label().to_string(),
            runtime: runtime_binding_for_node(config, runner_settings),
            policy: runner_policy_for_node(runner_settings),
            input_ports: node.input_ports.iter().map(runner_port_from_graph_port).collect(),
            output_ports: node.output_ports.iter().map(runner_port_from_graph_port).collect(),
            credential_ref_count: workflow
                .node_credentials(node.id)
                .map_or(EMPTY_CREDENTIAL_REF_COUNT, credential_ref_count),
        });
    }

    for node_id in workflow.configs.keys() {
        let node_id = crate::engine::workflow::NodeId(*node_id);
        if workflow.graph.node(node_id).is_none() {
            diagnostics.push(runner_diagnostic(
                RunnerDiagnosticCode::ConfigWithoutNode,
                Some(node_id),
                None,
                "node config references a missing graph node".to_string(),
            ));
        }
    }

    debug_assert!(nodes.len() <= workflow.configs.len());
    debug_assert!(nodes.iter().all(|node| workflow.graph.node(crate::engine::workflow::NodeId(node.id)).is_some()));

    let edges = runner_edges_from_workflow(workflow, &mut diagnostics);
    let expression_dependencies = match expression_dependencies_for_workflow(workflow) {
        Ok(dependencies) => dependencies,
        Err(expression_diagnostics) => {
            diagnostics.extend(expression_diagnostics);
            Vec::new()
        }
    };
    if diagnostics.is_empty() {
        RunnerPlanBuild {
            plan: Some(RunnerWorkflow {
                name: workflow.meta.name.clone(),
                nodes,
                edges,
                expression_dependencies,
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

fn expression_dependencies_for_workflow(
    workflow: &Workflow,
) -> Result<Vec<crate::expression_dependencies::ExpressionDependency>, Vec<RunnerDiagnostic>> {
    crate::expression_dependencies::workflow_projection(workflow)
        .map(|projection| projection.expression_dependencies)
        .map_err(|issues| {
            issues
                .into_iter()
                .map(|issue| {
                    runner_diagnostic(
                        RunnerDiagnosticCode::InvalidExpressionDependency,
                        Some(crate::engine::workflow::NodeId(issue.consumer_node)),
                        issue.producer_output_port.map(|port| PortId(port.0)),
                        format!("{}: {}", issue.config_field_path, issue.message),
                    )
                })
                .collect()
        })
}

fn runtime_binding_for_node(config: &NodeConfig, settings: Option<&NodeRunnerSettings>) -> RuntimeBinding {
    settings
        .map(|settings| runtime_binding(&settings.runtime_kind, &settings.runtime_key))
        .unwrap_or_else(|| runtime_binding_for_config(config))
}

fn runner_policy_for_node(settings: Option<&NodeRunnerSettings>) -> RunnerPolicy {
    settings.map(runner_policy_from_settings).unwrap_or_else(default_runner_policy)
}

fn runner_policy_from_settings(settings: &NodeRunnerSettings) -> RunnerPolicy {
    RunnerPolicy {
        effect_policy: settings.effect_policy.clone(),
        cache_policy: settings.cache_policy.clone(),
        retry_policy: settings.retry_policy.clone(),
        artifact_policy: settings.artifact_policy.clone(),
    }
}

fn default_runner_policy() -> RunnerPolicy {
    RunnerPolicy {
        effect_policy: String::new(),
        cache_policy: String::new(),
        retry_policy: String::new(),
        artifact_policy: String::new(),
    }
}

pub(crate) fn runtime_binding_for_config(config: &NodeConfig) -> RuntimeBinding {
    match config {
        NodeConfig::Shell { .. } => runtime_binding(RUNTIME_KIND_SHELL, RUNTIME_KEY_DEFAULT),
        NodeConfig::Http { .. } => runtime_binding(RUNTIME_KIND_HTTP, RUNTIME_KEY_DEFAULT),
        NodeConfig::File { operation, .. } => runtime_binding(RUNTIME_KIND_FILE, file_runtime_key(*operation)),
        NodeConfig::ImageInput { .. } => runtime_binding(RUNTIME_KIND_IMAGE_INPUT, RUNTIME_KEY_DEFAULT),
        NodeConfig::Agent(settings) => {
            runtime_binding(RUNTIME_KIND_AGENT, agent_node::agent_target_label(settings.target))
        }
        NodeConfig::Operation(plan) => runtime_binding(&plan.runtime.kind, &plan.runtime.key),
        NodeConfig::Condition(_) => runtime_binding(RUNTIME_KIND_CONDITION, RUNTIME_KEY_DEFAULT),
        NodeConfig::Merge(_) => runtime_binding(RUNTIME_KIND_MERGE, RUNTIME_KEY_DEFAULT),
        NodeConfig::Subworkflow(_) => runtime_binding(RUNTIME_KIND_SUBWORKFLOW, RUNTIME_KEY_DEFAULT),
        NodeConfig::Approval(_) => runtime_binding(RUNTIME_KIND_APPROVAL, RUNTIME_KEY_DEFAULT),
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

fn runner_port_from_graph_port(port: &crate::engine::workflow::Port) -> RunnerPort {
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
pub fn plan_identity(plan: &RunnerWorkflow) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_vec(plan)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUNNER_PLAN_IDENTITY_DOMAIN);
    hasher.update(&encoded);
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

pub fn admit_runner_plan(plan: &RunnerWorkflow) -> RunnerAdmission {
    match topological_order(plan) {
        Ok(topo_order) => RunnerAdmission::Accepted { topo_order },
        Err(diagnostic) => RunnerAdmission::Rejected {
            diagnostics: vec![diagnostic],
        },
    }
}

// r[impl n8nclone.workflow_execution.runner_core.trellis_graph_admission]
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
pub fn topological_order(plan: &RunnerWorkflow) -> Result<Vec<RunnerNodeId>, RunnerDiagnostic> {
    let shared_plan = admitted_execution_plan(plan)?;
    parse_node_ids(&shared_plan.topological_order)
}

// r[impl n8nclone.workflow_execution.runner_core.trellis_graph_admission]
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
pub fn ready_nodes(plan: &RunnerWorkflow, states: &BTreeMap<RunnerNodeId, RunnerStepState>) -> Vec<RunnerNodeId> {
    let Ok(shared_plan) = admitted_execution_plan(plan) else {
        return Vec::new();
    };
    let shared_states = shared_step_states(states);
    seaglass_execution_graph::ready_nodes(&shared_plan, &shared_states)
        .iter()
        .filter_map(|node_id| node_id.parse::<RunnerNodeId>().ok())
        .collect()
}

/// Return one deterministic longest dependency path through an admitted workflow.
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
pub fn critical_path(plan: &RunnerWorkflow) -> Result<Vec<RunnerNodeId>, RunnerDiagnostic> {
    let shared_plan = admitted_execution_plan(plan)?;
    parse_node_ids(&shared_plan.critical_path())
}

/// Rank admitted workflow nodes by their transitive dependent counts.
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
pub fn bottlenecks(plan: &RunnerWorkflow) -> Result<Vec<Bottleneck>, RunnerDiagnostic> {
    let shared_plan = admitted_execution_plan(plan)?;
    shared_plan
        .bottlenecks()
        .into_iter()
        .map(|bottleneck| {
            parse_node_id(&bottleneck.node_id).map(|node_id| Bottleneck {
                node_id,
                dependent_count: bottleneck.dependent_count,
            })
        })
        .collect()
}

// r[impl n8nclone.workflow_execution.runner_core.lifecycle]
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
/// Apply one deterministic runner lifecycle transition.
pub fn transition_step_state(
    from: RunnerStepState,
    action: RunnerStepAction,
) -> Result<RunnerStepState, RunnerTransitionError> {
    seaglass_execution_graph::transition_step_state(from, action)
}

/// Return whether a runner step state forbids further execution transitions.
pub fn is_terminal_step_state(state: RunnerStepState) -> bool {
    seaglass_execution_graph::is_terminal_state(state)
}

// r[impl lattice.choregraph_adoption.adapter]
fn admitted_execution_plan(plan: &RunnerWorkflow) -> Result<seaglass_execution_graph::ExecutionPlan, RunnerDiagnostic> {
    seaglass_execution_graph::admit_graph(&shared_execution_graph(plan)).map_err(|diagnostics| {
        diagnostics.first().map_or_else(invalid_shared_graph_diagnostic, shared_graph_diagnostic)
    })
}

fn shared_execution_graph(plan: &RunnerWorkflow) -> seaglass_execution_graph::ExecutionGraph {
    let nodes = plan
        .nodes
        .iter()
        .map(|node| seaglass_execution_graph::ExecutionNode::new(node.id.to_string()))
        .collect();
    debug_assert!(plan.edges.len() <= choregraph_core::DEFAULT_MAX_EDGES);
    debug_assert!(plan.expression_dependencies.len() <= choregraph_core::DEFAULT_MAX_EDGES);
    let edges = plan
        .edges
        .iter()
        .map(|edge| (edge.source_node, edge.target_node))
        .chain(
            plan.expression_dependencies
                .iter()
                .map(|dependency| (dependency.producer_node, dependency.consumer_node)),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|(source, target)| seaglass_execution_graph::ExecutionEdge::new(source.to_string(), target.to_string()))
        .collect();
    seaglass_execution_graph::ExecutionGraph::new(nodes, edges)
}

fn shared_step_states(
    states: &BTreeMap<RunnerNodeId, RunnerStepState>,
) -> BTreeMap<String, seaglass_execution_graph::StepState> {
    states.iter().map(|(node_id, state)| (node_id.to_string(), *state)).collect()
}

const fn persisted_step_state(state: PersistedStepState) -> RunnerStepState {
    match state {
        PersistedStepState::Pending => RunnerStepState::Pending,
        PersistedStepState::Ready => RunnerStepState::Ready,
        PersistedStepState::Running => RunnerStepState::Running,
        PersistedStepState::Completed => RunnerStepState::Completed,
        PersistedStepState::Failed => RunnerStepState::Failed,
        PersistedStepState::Cancelled => RunnerStepState::Cancelled,
    }
}

fn binding_uses_supported_schemas(binding: &PersistedMachineBinding) -> bool {
    let is_legacy_schema = binding.old_schema == LEGACY_MACHINE_STATE_SCHEMA;
    let is_choregraph_schema = binding.new_schema == CHOREGRAPH_MACHINE_STATE_SCHEMA;
    is_legacy_schema && is_choregraph_schema
}

fn validate_persisted_binding(
    expected: &PersistedMachineBinding,
    observed: &PersistedMachineBinding,
) -> Result<(), PersistedStateIssue> {
    if !binding_uses_supported_schemas(observed) || !binding_uses_supported_schemas(expected) {
        return Err(PersistedStateIssue::UnsupportedSchema);
    }
    if observed.workflow_revision != expected.workflow_revision {
        return Err(PersistedStateIssue::StaleWorkflowRevision);
    }
    if observed.machine_identity != expected.machine_identity {
        return Err(PersistedStateIssue::StaleMachineIdentity);
    }
    if observed.runner_plan_identity != expected.runner_plan_identity {
        return Err(PersistedStateIssue::StaleRunnerPlanIdentity);
    }
    Ok(())
}

fn migrate_step_records(
    expected: &PersistedMachineBinding,
    expected_nodes: &BTreeSet<RunnerNodeId>,
    records: &[PersistedStepStateRecord],
) -> Result<BTreeMap<RunnerNodeId, RunnerStepState>, PersistedStateIssue> {
    let mut states = BTreeMap::new();
    for record in records {
        validate_persisted_binding(expected, &record.binding)?;
        if !expected_nodes.contains(&record.node_id) {
            return Err(PersistedStateIssue::ForeignNode);
        }
        if states.insert(record.node_id, persisted_step_state(record.state)).is_some() {
            return Err(PersistedStateIssue::AmbiguousNode);
        }
    }
    Ok(states)
}

// r[impl lattice.choregraph_adoption.persistence]
pub fn migrate_persisted_states(
    plan: &RunnerWorkflow,
    expected: &PersistedMachineBinding,
    records: &[PersistedStepStateRecord],
) -> Result<BTreeMap<RunnerNodeId, RunnerStepState>, PersistedStateIssue> {
    validate_persisted_binding(expected, expected)?;
    let expected_nodes = plan.nodes.iter().map(|node| node.id).collect::<BTreeSet<_>>();
    if records.len() != expected_nodes.len() {
        return Err(PersistedStateIssue::PartialState);
    }
    let states = migrate_step_records(expected, &expected_nodes, records)?;
    if states.len() != expected_nodes.len() {
        return Err(PersistedStateIssue::PartialState);
    }
    Ok(states)
}

const fn core_action(action: RunnerStepAction) -> choregraph_core::Action {
    match action {
        RunnerStepAction::MarkReady => choregraph_core::Action::MarkReady,
        RunnerStepAction::Start => choregraph_core::Action::Start,
        RunnerStepAction::Complete => choregraph_core::Action::Complete,
        RunnerStepAction::Fail => choregraph_core::Action::Fail,
        RunnerStepAction::Cancel => choregraph_core::Action::Cancel,
    }
}

fn replay_events(events: &[PersistedMachineEvent]) -> Vec<choregraph_core::Event> {
    events
        .iter()
        .map(|event| choregraph_core::Event::new(event.node_id.to_string(), core_action(event.action)))
        .collect()
}

fn replay_bindings(
    identity: PersistedReplayIdentity,
) -> (choregraph_core::ReplayBindings, choregraph_core::ReplayBindings) {
    let expected = choregraph_core::ReplayBindings::new(
        choregraph_core::TraceBinding::from_bytes(identity.expected_machine),
        choregraph_core::TraceBinding::from_bytes(identity.expected_initial_state),
    );
    let observed = choregraph_core::ReplayBindings::new(
        choregraph_core::TraceBinding::from_bytes(identity.observed_machine),
        choregraph_core::TraceBinding::from_bytes(identity.observed_initial_state),
    );
    (expected, observed)
}

// r[impl lattice.choregraph_adoption.replay]
pub fn replay_persisted_events(
    plan: &RunnerWorkflow,
    states: &BTreeMap<RunnerNodeId, RunnerStepState>,
    identity: PersistedReplayIdentity,
    events: &[PersistedMachineEvent],
) -> Result<choregraph_core::ReplayDecision, PersistedReplayError> {
    let shared_plan = admitted_execution_plan(plan).map_err(PersistedReplayError::Graph)?;
    let initial_state = seaglass_execution_graph::expand_state(&shared_plan, &shared_step_states(states))
        .map_err(PersistedReplayError::State)?;
    let core_events = replay_events(events);
    let (expected_bindings, observed_bindings) = replay_bindings(identity);
    let trace = choregraph_core::BoundedTrace::new(observed_bindings, &core_events);
    let input = choregraph_core::ReplayInput::new(
        shared_plan.machine(),
        &initial_state,
        expected_bindings,
        trace,
        choregraph_core::ReplayLimits {
            max_events: choregraph_core::DEFAULT_MAX_TRACE_EVENTS,
        },
    );
    Ok(choregraph_core::replay(input))
}

fn parse_node_ids(node_ids: &[String]) -> Result<Vec<RunnerNodeId>, RunnerDiagnostic> {
    node_ids.iter().map(|node_id| parse_node_id(node_id)).collect()
}

fn parse_node_id(node_id: &str) -> Result<RunnerNodeId, RunnerDiagnostic> {
    match node_id.parse::<RunnerNodeId>() {
        Ok(parsed_node_id) => Ok(parsed_node_id),
        Err(_) => Err(runner_diagnostic(
            RunnerDiagnosticCode::RuntimeFailure,
            None,
            None,
            format!("shared graph core returned non-numeric runner node id '{node_id}'"),
        )),
    }
}

fn shared_graph_diagnostic(diagnostic: &seaglass_execution_graph::GraphDiagnostic) -> RunnerDiagnostic {
    let code = match diagnostic.code {
        seaglass_execution_graph::GraphDiagnosticCode::CycleDetected => RunnerDiagnosticCode::CycleDetected,
        seaglass_execution_graph::GraphDiagnosticCode::SelfLoop => RunnerDiagnosticCode::EdgeSelfLoop,
        seaglass_execution_graph::GraphDiagnosticCode::EmptyNodeId
        | seaglass_execution_graph::GraphDiagnosticCode::DuplicateNodeId
        | seaglass_execution_graph::GraphDiagnosticCode::DuplicateEdge
        | seaglass_execution_graph::GraphDiagnosticCode::MissingSourceNode
        | seaglass_execution_graph::GraphDiagnosticCode::MissingTargetNode
        | seaglass_execution_graph::GraphDiagnosticCode::GraphTooLarge => RunnerDiagnosticCode::RuntimeFailure,
    };
    let node_id = diagnostic
        .node_id
        .as_deref()
        .and_then(|node_id| node_id.parse::<RunnerNodeId>().ok())
        .map(crate::engine::workflow::NodeId);
    runner_diagnostic(code, node_id, None, diagnostic.to_string())
}

fn invalid_shared_graph_diagnostic() -> RunnerDiagnostic {
    runner_diagnostic(
        RunnerDiagnosticCode::RuntimeFailure,
        None,
        None,
        "shared graph core rejected the runner plan without a diagnostic".to_string(),
    )
}

// r[impl n8nclone.workflow_execution.runtime_registry.dispatch]
pub fn builtin_runtime_registry() -> RuntimeRegistry {
    RuntimeRegistry::new([
        runtime_binding(RUNTIME_KIND_SHELL, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_HTTP, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_FILE, RUNTIME_KEY_READ_FILE),
        runtime_binding(RUNTIME_KIND_FILE, RUNTIME_KEY_WRITE_FILE),
        runtime_binding(RUNTIME_KIND_FILE, RUNTIME_KEY_APPEND_FILE),
        runtime_binding(RUNTIME_KIND_IMAGE_INPUT, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_CONDITION, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_MERGE, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_SUBWORKFLOW, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_APPROVAL, RUNTIME_KEY_DEFAULT),
        runtime_binding(RUNTIME_KIND_PROCESS, crate::operation::runtime::CAIRN_RUNTIME_KEY),
        runtime_binding(RUNTIME_KIND_AGENT, agent_node::agent_target_label(agent_node::AgentTarget::Pi)),
        runtime_binding(RUNTIME_KIND_AGENT, agent_node::agent_target_label(agent_node::AgentTarget::Clankers)),
        runtime_binding(RUNTIME_KIND_AGENT, agent_node::agent_target_label(agent_node::AgentTarget::Animus)),
    ])
}

impl RuntimeRegistry {
    pub fn new(bindings: impl IntoIterator<Item = RuntimeBinding>) -> Self {
        Self {
            bindings: bindings.into_iter().collect(),
        }
    }

    pub fn contains(&self, binding: &RuntimeBinding) -> bool {
        self.bindings.contains(binding)
    }
}

// r[impl n8nclone.workflow_execution.runtime_registry.dispatch]
pub fn validate_runtime_bindings(plan: &RunnerWorkflow, registry: &RuntimeRegistry) -> Vec<RunnerDiagnostic> {
    plan.nodes
        .iter()
        .filter(|node| !registry.contains(&node.runtime))
        .map(|node| {
            runner_diagnostic(
                RunnerDiagnosticCode::MissingRuntimeBinding,
                Some(crate::engine::workflow::NodeId(node.id)),
                None,
                format!(
                    "node '{}' references missing runtime '{}:{}'",
                    node.label, node.runtime.kind, node.runtime.key
                ),
            )
        })
        .collect()
}

pub fn validate_runtime_policies(plan: &RunnerWorkflow) -> Vec<RunnerDiagnostic> {
    plan.nodes.iter().filter_map(unsupported_runtime_policy_diagnostic).collect()
}

fn unsupported_runtime_policy_diagnostic(node: &RunnerNode) -> Option<RunnerDiagnostic> {
    let declared = node.policy.effect_policy.trim();
    if declared.is_empty() {
        return None;
    }
    let compatible_policy = runtime_effect_policy(&node.runtime);
    if compatible_policy == Some(declared) {
        return None;
    }
    Some(runner_diagnostic(
        RunnerDiagnosticCode::UnsupportedRuntimePolicy,
        Some(crate::engine::workflow::NodeId(node.id)),
        None,
        format!(
            "node '{}' declares unsupported effect policy '{}' for runtime '{}:{}'",
            node.label, declared, node.runtime.kind, node.runtime.key
        ),
    ))
}

fn runtime_effect_policy(binding: &RuntimeBinding) -> Option<&'static str> {
    match binding.kind.as_str() {
        RUNTIME_KIND_AGENT => Some(EFFECT_POLICY_AGENT),
        RUNTIME_KIND_FILE | RUNTIME_KIND_IMAGE_INPUT => Some(EFFECT_POLICY_FILE),
        RUNTIME_KIND_HTTP => Some(EFFECT_POLICY_HTTP),
        RUNTIME_KIND_SHELL | RUNTIME_KIND_PROCESS => Some(EFFECT_POLICY_PROCESS),
        RUNTIME_KIND_CONDITION | RUNTIME_KIND_MERGE | RUNTIME_KIND_SUBWORKFLOW | RUNTIME_KIND_APPROVAL => {
            Some(EFFECT_POLICY_PURE)
        }
        _ => None,
    }
}

// r[impl n8nclone.workflow_execution.runtime_registry.typed_output_contract]
pub fn admit_runtime_outputs(node: &RunnerNode, outputs: &[RuntimeOutputShape]) -> Result<(), Vec<RunnerDiagnostic>> {
    let declared = node.output_ports.iter().map(|port| (port.id, port.type_tag.as_str())).collect::<BTreeMap<_, _>>();
    let diagnostics = outputs
        .iter()
        .filter_map(|output| runtime_output_diagnostic(node, &declared, output))
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn runtime_output_diagnostic(
    node: &RunnerNode,
    declared: &BTreeMap<RunnerPortId, &str>,
    output: &RuntimeOutputShape,
) -> Option<RunnerDiagnostic> {
    let Some(expected_type) = declared.get(&output.port_id) else {
        return Some(runner_diagnostic(
            RunnerDiagnosticCode::RuntimeOutputPortMissing,
            Some(crate::engine::workflow::NodeId(node.id)),
            Some(PortId(output.port_id)),
            format!("runtime output for node '{}' references undeclared output port", node.label),
        ));
    };
    if !port_types_compatible(&output.type_tag, expected_type) {
        return Some(runner_diagnostic(
            RunnerDiagnosticCode::RuntimeOutputTypeMismatch,
            Some(crate::engine::workflow::NodeId(node.id)),
            Some(PortId(output.port_id)),
            format!(
                "runtime output for node '{}' has type '{}' but port expects '{}'",
                node.label, output.type_tag, expected_type
            ),
        ));
    }
    None
}

// r[impl n8nclone.workflow_execution.runner_events.stream]
pub fn runner_event(
    run_id: impl Into<String>,
    prior_events: &[RunnerEvent],
    node_id: Option<RunnerNodeId>,
    port_id: Option<RunnerPortId>,
    kind: RunnerEventKind,
    summary: impl Into<String>,
    sensitive_fragments: &[String],
) -> RunnerEvent {
    let raw_summary = summary.into();
    let redacted = contains_sensitive_fragment(&raw_summary, sensitive_fragments);
    RunnerEvent {
        run_id: run_id.into(),
        sequence: next_event_sequence(prior_events),
        node_id,
        port_id,
        kind,
        summary: if redacted {
            REDACTED_SUMMARY.to_string()
        } else {
            raw_summary
        },
        redacted,
    }
}

fn next_event_sequence(prior_events: &[RunnerEvent]) -> u64 {
    prior_events
        .last()
        .map(|event| event.sequence + RUNNER_EVENT_SEQUENCE_INCREMENT)
        .unwrap_or(RUNNER_EVENT_INITIAL_SEQUENCE)
}

// r[impl n8nclone.workflow_execution.runner_events.redaction]
fn contains_sensitive_fragment(summary: &str, sensitive_fragments: &[String]) -> bool {
    sensitive_fragments
        .iter()
        .filter(|fragment| !fragment.is_empty())
        .any(|fragment| summary.contains(fragment))
}

// r[impl n8nclone.workflow_execution.artifact_references.model]
// r[impl n8nclone.workflow_execution.artifact_references.trellis_identity]
pub fn artifact_reference_for_value(value: &Value, backend_ref: impl Into<String>) -> ArtifactReference {
    ArtifactReference {
        content_hash: value.hash.to_hex(),
        type_tag: value.type_tag().as_str().to_string(),
        media_type: value_media_type(value),
        byte_length: value_byte_length(value),
        backend_ref: backend_ref.into(),
        sensitive: !value.sensitive_fragments.is_empty(),
        summary: value.as_display_string(),
    }
}

fn value_media_type(value: &Value) -> Option<String> {
    match &value.data {
        ValueData::Image(image) => Some(image.media_type.clone()),
        _ => None,
    }
}

fn value_byte_length(value: &Value) -> usize {
    match &value.data {
        ValueData::String(value) => value.len(),
        ValueData::Bytes(value) => value.len(),
        ValueData::Json(value) => serde_json::to_vec(value).map(|bytes| bytes.len()).unwrap_or_default(),
        ValueData::Image(image) => image.bytes.len(),
        ValueData::ExitCode(_) => std::mem::size_of::<i32>(),
        ValueData::Error { message, stderr } => message.len() + stderr.len(),
    }
}

// r[impl n8nclone.workflow_execution.durable_runner.resume]
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
pub fn durable_runner_decision(
    plan: &RunnerWorkflow,
    states: &BTreeMap<RunnerNodeId, RunnerStepState>,
    node_id: RunnerNodeId,
) -> DurableRunnerDecision {
    let Ok(shared_plan) = admitted_execution_plan(plan) else {
        return DurableRunnerDecision::RejectTerminalMutation;
    };
    seaglass_execution_graph::resume_decision(&shared_plan, &shared_step_states(states), &node_id.to_string())
}

fn runner_diagnostic(
    code: RunnerDiagnosticCode,
    node_id: Option<crate::engine::workflow::NodeId>,
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
