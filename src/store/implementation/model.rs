/// Durable owner identity used to prevent one execution from satisfying another dispatch.
// r[impl lattice.agent_orchestration.run_identity]
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RunOwner {
    #[default]
    Unscoped,
    Orchestration {
        instance_id: crate::orchestration::InstanceId,
        dispatch_id: crate::orchestration::DispatchId,
    },
    ScheduleOccurrence {
        identity: String,
    },
    WorkflowApproval {
        request_id: crate::workflow::gate::ApprovalRequestId,
    },
}

/// Serializable run record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    pub run_id: crate::engine::RunId,
    #[serde(default)]
    pub owner: RunOwner,
    pub workflow_name: String,
    pub started_epoch_ms: i64,
    pub finished_epoch_ms: Option<i64>,
    pub node_count: usize,
    pub cache_hits: usize,
    pub failures: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RunNodeDetailStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunOutputDetail {
    pub port_id: crate::engine::workflow::PortId,
    pub port_label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunNodeDetail {
    pub node_id: crate::engine::workflow::NodeId,
    pub node_label: String,
    pub status: RunNodeDetailStatus,
    pub cached: bool,
    pub outputs: Vec<RunOutputDetail>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunDetailRecord {
    pub run_id: crate::engine::RunId,
    /// Canonical graph-ordered node details.
    pub nodes: Vec<RunNodeDetail>,
    /// Actual terminal observation order, separate from canonical presentation.
    #[serde(default = "empty_observed_completion_order")]
    pub observed_completion_order: Vec<crate::engine::workflow::NodeId>,
}

fn empty_observed_completion_order() -> Vec<crate::engine::workflow::NodeId> {
    Vec::new()
}

pub struct WorkflowApprovalAdvance<'a> {
    pub expected_version: u64,
    pub completed_request: &'a crate::workflow::gate::ApprovalRequest,
    pub next_request: &'a crate::workflow::gate::ApprovalRequest,
    pub run: &'a RunRecord,
    pub details: &'a RunDetailRecord,
    pub replay_events: &'a [crate::replay::ReplayEventRecord],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerAdmissionWrite {
    Inserted,
    Duplicate {
        instance_id: crate::orchestration::InstanceId,
    },
}
