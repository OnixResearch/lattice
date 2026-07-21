//! Serialization-stable, content-omitting inspection model.

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    NotRecorded,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerBindingState {
    NotApplicable,
    Exact,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeNodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchEvidenceState {
    NotApplicable,
    Reserved,
    Acknowledged,
    Finished,
    AbandonedPending,
    AbandonedAcknowledged,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryEvidenceState {
    NotApplicable,
    RunObservedPendingReconciliation,
    Closed,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetSettlementState {
    NotApplicable,
    Reserved,
    Settled,
    ConservativePendingAbandonment,
    ConservativeAcknowledgedAbandonment,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    RunLifecycleInvalid,
    DetailRunIdentityMismatch,
    DetailNodeCountMismatch,
    DetailNodeIdentityDuplicate,
    ReplayRunIdentityMismatch,
    ReplayInvalid,
    OwnerIdentityMalformed,
    OrchestrationMissing,
    OrchestrationIdentityMismatch,
    OrchestrationReplayInvalid,
    DispatchBindingMissing,
    DispatchBindingDuplicate,
    DispatchFactMismatch,
    DispatchTerminalConflict,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SafeLifecycle {
    pub started_epoch_ms: i64,
    pub finished_epoch_ms: Option<i64>,
    pub node_count: usize,
    pub cache_hits: usize,
    pub failures: usize,
    pub terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SafeNode {
    pub node_id: u64,
    pub status: SafeNodeStatus,
    pub cached: bool,
    pub output_count: usize,
    pub error_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SafeDetails {
    pub state: EvidenceState,
    pub nodes: Vec<SafeNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SafeReplay {
    pub state: EvidenceState,
    pub event_count: usize,
    pub diagnostic_codes: Vec<crate::replay::ReplayDiagnosticCode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SafeOwner {
    Unscoped,
    Orchestration {
        instance_id: String,
        dispatch_id: String,
        binding: OwnerBindingState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SafeBudgetEvidence {
    pub settlement: BudgetSettlementState,
    pub planned_upper_bound: Option<crate::orchestration::BudgetChargePlan>,
    pub observed: Option<crate::orchestration::BudgetCharge>,
    pub orchestration_ledger_breached: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SafeOrchestrationEvidence {
    pub event_chain: EvidenceState,
    pub status: crate::orchestration::OrchestrationStatus,
    pub dispatch: DispatchEvidenceState,
    pub recovery: RecoveryEvidenceState,
    pub cancellation_requested: bool,
    pub run_succeeded: Option<bool>,
    pub budget: SafeBudgetEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub valid: bool,
    pub run_id: u64,
    pub workflow_identity: String,
    pub lifecycle: SafeLifecycle,
    pub owner: SafeOwner,
    pub details: SafeDetails,
    pub replay: SafeReplay,
    pub orchestration: Option<SafeOrchestrationEvidence>,
    pub diagnostic_codes: Vec<DiagnosticCode>,
}

pub struct OrchestrationInput<'a> {
    pub instance: &'a crate::orchestration::OrchestrationInstance,
    pub events: &'a [crate::orchestration::OrchestrationEvent],
}

pub struct Input<'a> {
    pub record: &'a crate::store::RunRecord,
    pub details: Option<&'a crate::store::RunDetailRecord>,
    pub replay_events: Option<&'a [crate::replay::ReplayEventRecord]>,
    pub orchestration: Option<OrchestrationInput<'a>>,
}
