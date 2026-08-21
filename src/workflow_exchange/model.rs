#![allow(
    unknown_lints,
    non_trait_imports,
    path_segment_repetition,
    reason = "the application boundary keeps producer wire types explicit in its versioned durable records"
)]

use std::path::PathBuf;

use lattice_workflow_contract::ExecutionState;
use lattice_workflow_contract::Identity;
use lattice_workflow_contract::Label;
use lattice_workflow_contract::LogStream;
use lattice_workflow_contract::NodeState;
use lattice_workflow_contract::WorkflowAccepted;
use lattice_workflow_contract::WorkflowRequest;
use lattice_workflow_transport::TransportProfile;
use serde::Deserialize;
use serde::Serialize;

pub const HANDLER_PROFILE_SCHEMA: &str = "lattice.workflow-exchange-handler-profile.v1";
pub const EXECUTION_RECORD_SCHEMA: &str = "lattice.workflow-exchange-execution.v1";
pub const MAXIMUM_ROUTES: u32 = 1_024;
pub const MAXIMUM_ROUTE_NODES: u32 = lattice_workflow_contract::CONTRACT_MAXIMUM_NODES;
pub const MAXIMUM_WORKFLOW_NAME_BYTES: usize = 128;

pub const REQUIRED_NON_CLAIMS: [&str; 5] = [
    "handler admission does not prove workflow correctness",
    "route authority does not prove host confinement",
    "execution observation does not prove an external effect succeeded",
    "peer identity does not prove peer intent",
    "handler checks do not establish release readiness",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectAuthorityMode {
    Unavailable,
    RouteExact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceBindingMode {
    Unavailable,
    ShellEnvironment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteNode {
    pub request_node_id: Label,
    pub operation_identity: Identity,
    pub workflow_node_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRoute {
    pub profile_identity: Identity,
    pub graph_identity: Identity,
    pub workflow_name: String,
    pub workflow_revision: Identity,
    pub effect_authority: EffectAuthorityMode,
    pub source_binding: SourceBindingMode,
    pub nodes: Vec<RouteNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandlerProfile {
    pub schema: String,
    pub socket_path: PathBuf,
    pub transport: TransportProfile,
    pub maximum_active_executions: u32,
    pub routes: Vec<WorkflowRoute>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    Schema,
    SocketPath,
    Transport,
    ActiveLimit,
    RouteLimit,
    RouteOrder,
    RouteValue,
    NodeOrder,
    NodeValue,
    NonClaims,
    Encoding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowFacts {
    pub revision: Identity,
    pub node_ids: Vec<u64>,
    pub edges: Vec<(u64, u64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    Missing,
    WorkflowRevision,
    NodeCount,
    NodeMapping,
    EdgeMapping,
    SourceRequired,
    SourceUnexpected,
    SourceTarget,
    SourceCollision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurableExecutionState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}

impl DurableExecutionState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    #[must_use]
    pub const fn as_contract(self) -> ExecutionState {
        match self {
            Self::Queued => ExecutionState::Queued,
            Self::Running => ExecutionState::Running,
            Self::Completed => ExecutionState::Completed,
            Self::Failed => ExecutionState::Failed,
            Self::Cancelled => ExecutionState::Cancelled,
            Self::Unknown => ExecutionState::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableNode {
    pub request_node_id: Label,
    pub workflow_node_id: u64,
    pub adapter_identity: Identity,
    pub state: NodeState,
    pub receipt_identity: Option<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedLog {
    pub node_id: Label,
    pub stream: LogStream,
    pub cursor: u64,
    pub reference_identity: Identity,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRecord {
    pub schema: String,
    pub request: WorkflowRequest,
    pub accepted: WorkflowAccepted,
    pub handler_profile_identity: Identity,
    pub workflow_name: String,
    pub workflow_revision: Identity,
    pub run_id: u64,
    pub sequence: u64,
    pub state: DurableExecutionState,
    pub cancellation_requested: bool,
    pub nodes: Vec<DurableNode>,
    pub logs: Vec<RetainedLog>,
    pub receipt_identity: Option<Identity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableNodeResult {
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionFacts {
    pub cancelled: bool,
    pub suspended: bool,
    pub failed_before_results: bool,
    pub nodes: Vec<(u64, DurableNodeResult)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    Terminal,
    StaleSequence,
    UnsupportedControl,
    InvalidCompletion,
    Identity,
    Contract,
    LogLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionResolution {
    Inserted(ExecutionRecord),
    Existing(ExecutionRecord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationError {
    Profile,
    Store,
    Workflow,
    Route,
    Runtime,
    Transport,
    UnsupportedHost,
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Profile => "workflow exchange handler profile is invalid",
            Self::Store => "workflow exchange durable store is unavailable",
            Self::Workflow => "workflow exchange saved workflow is unavailable or invalid",
            Self::Route => "workflow exchange route does not match the saved workflow",
            Self::Runtime => "workflow exchange runtime could not start",
            Self::Transport => "workflow exchange transport failed",
            Self::UnsupportedHost => "workflow exchange service requires Linux",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ApplicationError {}
