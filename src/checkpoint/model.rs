#![allow(
    unknown_lints,
    excessive_file_length,
    explicit_defaults,
    non_trait_imports,
    platform_dependent_cast,
    raw_arithmetic_overflow,
    numeric_units,
    path_segment_repetition,
    reason = "the checkpoint records keep their versioned wire fields and unit-bearing producer names explicit"
)]

use std::path::PathBuf;

use lattice_workflow_contract::Identity;
use lattice_workflow_contract::WorkflowAccepted;
use serde::Deserialize;
use serde::Serialize;

mod adoption;
mod refusal;
pub use adoption::AdoptionOutcome;
pub use adoption::AdoptionRecord;
pub use adoption::RestoredWorkspace;
pub use refusal::Refusal;

/// C8 manifest schema; also the manifest identity domain.
pub const MANIFEST_SCHEMA: &str = "lattice.application-checkpoint-manifest.v1";
/// C8 restart receipt schema; also the receipt identity domain.
pub const RECEIPT_SCHEMA: &str = "lattice.application-restart-receipt.v1";
/// Commit marker domain. The marker body is exactly `{ manifest }` (C8).
pub const COMMIT_SCHEMA: &str = "lattice.application-checkpoint-commit.v1";
/// Content-addressed node outcome object stored beside a manifest.
pub const NODE_OUTPUTS_SCHEMA: &str = "lattice.application-checkpoint-node-outputs.v1";
/// Kiln-supplied restart request naming the admitted checkpoint and admission reference.
pub const RESTART_REQUEST_SCHEMA: &str = "lattice.application-restart-request.v1";
/// Route checkpoint binding: which exchange route publishes under which opaque C1 profile.
pub const BINDING_SCHEMA: &str = "lattice.application-checkpoint-binding.v1";
/// Durable adoption record in the target store.
pub const ADOPTION_SCHEMA: &str = "lattice.application-checkpoint-adoption.v1";
/// Drain acknowledgement written after a quiescent manifest commits.
pub const DRAIN_SCHEMA: &str = "lattice.application-checkpoint-drain.v1";

/// The only profile this revision supports (plan D5).
pub const NODE_BOUNDARY_PROFILE: &str = "workflow-node-boundary-v1";

pub const MAXIMUM_COMPLETED_NODES: usize = lattice_workflow_contract::CONTRACT_MAXIMUM_NODES as usize;
pub const MAXIMUM_WORKSPACE_OBJECTS: usize = 4_096;
pub const MAXIMUM_WORKSPACE_PATH_BYTES: usize = 1_024;
pub const MAXIMUM_OBJECT_BYTES: u64 = 256 * 1_024 * 1_024;
pub const MAXIMUM_TEXT_BYTES: usize = 256;

pub const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "an application checkpoint does not preserve process, memory or socket state",
    "a restart receipt does not prove the source host stopped executing",
    "an uncertain node outcome is not evidence that its effect did or did not happen",
    "checkpoint admission does not establish current placement or budget authority",
];

/// Declared checkpoint support of a route's opaque C1 profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointProfile {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "workflow-node-boundary-v1")]
    WorkflowNodeBoundaryV1,
}

impl CheckpointProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::WorkflowNodeBoundaryV1 => NODE_BOUNDARY_PROFILE,
        }
    }

    /// Profiles whose content is architecture-neutral (JSON outcomes and workspace bytes only).
    #[must_use]
    pub const fn is_portable(self) -> bool {
        matches!(self, Self::WorkflowNodeBoundaryV1)
    }
}

/// Manifest profile field. Only supported profiles deserialize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManifestProfile {
    #[serde(rename = "workflow-node-boundary-v1")]
    WorkflowNodeBoundaryV1,
}

impl ManifestProfile {
    #[must_use]
    pub const fn checkpoint(self) -> CheckpointProfile {
        match self {
            Self::WorkflowNodeBoundaryV1 => CheckpointProfile::WorkflowNodeBoundaryV1,
        }
    }
}

/// The checkpoint offer a route reports. A route without hooks never advertises resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointOffer {
    Unavailable,
    Declared(CheckpointProfile),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointExecution {
    pub request: Identity,
    pub accepted: WorkflowAccepted,
    pub handler_profile: Identity,
    pub workflow_revision: Identity,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointRuntime {
    pub lattice_revision: String,
    pub lattice_binary_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletedNodeState {
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedNode {
    pub workflow_node_id: u64,
    pub adapter_identity: Identity,
    pub state: CompletedNodeState,
    pub outputs_object: Identity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_identity: Option<Identity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertainState {
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UncertainNode {
    pub workflow_node_id: u64,
    pub adapter_identity: Identity,
    pub last_state: UncertainState,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceObject {
    pub path: String,
    pub blake3: String,
    pub bytes: u64,
}

/// Budget the manifest measures and carries. Model requests and cost are not measured at node
/// boundaries; the host's per-binding budget ledger enforces them across restarts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consumed {
    pub elapsed_ms: u64,
}

/// C8 `lattice.application-checkpoint-manifest.v1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub profile: ManifestProfile,
    pub execution: CheckpointExecution,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<Identity>,
    pub producing_incarnation: u32,
    pub architecture: String,
    pub runtime: CheckpointRuntime,
    pub completed_nodes: Vec<CompletedNode>,
    pub uncertain_nodes: Vec<UncertainNode>,
    pub workspace_objects: Vec<WorkspaceObject>,
    pub consumed: Consumed,
    pub deadline_unix_ms: u64,
    pub cancellation_requested: bool,
}

/// C8 commit marker, written last.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitMarker {
    pub manifest: Identity,
}

/// Node outcome object referenced by `CompletedNode::outputs_object`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeOutputsObject {
    pub schema: String,
    pub workflow_node_id: u64,
    pub outcome: crate::workflow::gate::PersistedNodeOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestartKind {
    ApplicationRestart,
    VmContinuation,
    FreshRetry,
}

/// C8 `lattice.application-restart-receipt.v1`. `admission` is the opaque Kiln-issued reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestartReceipt {
    pub checkpoint: Identity,
    pub kind: RestartKind,
    pub restarted_incarnation: u32,
    pub admission: String,
}

/// What Kiln admitted. Lattice never mints `admission`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestartRequest {
    pub schema: String,
    pub checkpoint: Identity,
    pub admission: String,
    pub restarted_incarnation: u32,
    pub request: Identity,
    pub execution: Identity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<Identity>,
    /// Kiln's cancel decision; restart ORs it with the manifest's, so a cancel acknowledged after
    /// the last publication still holds.
    pub cancellation_requested: bool,
}

/// Binds one exchange route to an opaque C1 profile identity and its declared checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointBinding {
    pub schema: String,
    pub vm_profile: Identity,
    pub checkpoint: CheckpointProfile,
    pub graph_identity: Identity,
    pub workflow_revision: Identity,
    pub root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    pub deadline_unix_ms: u64,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrainAcknowledgement {
    pub schema: String,
    pub manifest: Identity,
}

/// Runtime facts of the restoring binary, gathered by the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRuntime {
    pub architecture: String,
    pub runtime: CheckpointRuntime,
}
