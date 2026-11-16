#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RetryableFailure {
    pub run_id: crate::engine::RunId,
    pub dispatch_id: crate::orchestration::identity::DispatchId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RetryLedger {
    pub admitted: u64,
    pub retryable_failure: Option<RetryableFailure>,
}

impl RetryLedger {
    fn is_empty(&self) -> bool {
        self.admitted == 0 && self.retryable_failure.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ready,
    Running,
    WaitingForApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchStatus {
    Pending,
    Acknowledged,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DispatchIntent {
    pub id: crate::orchestration::identity::DispatchId,
    pub run_id: crate::engine::RunId,
    pub workflow_revision: super::super::WorkflowRevisionRef,
    pub status: DispatchStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RevisionProposal {
    pub id: crate::orchestration::identity::ProposalId,
    pub instance_id: crate::orchestration::identity::InstanceId,
    pub base_definition_revision: crate::orchestration::identity::DefinitionRevision,
    pub base_epoch: u64,
    pub base_workflow_revision: super::super::WorkflowRevisionRef,
    pub successor: super::super::WorkflowRevisionRef,
    pub safe_summary: String,
}

// r[impl lattice.workflow_approval.compatibility]
// r[impl lattice.workflow_approval.compatibility.valid]
// r[impl lattice.workflow_approval.compatibility.denied]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ApprovalDecision {
    pub id: crate::orchestration::identity::ApprovalId,
    pub instance_id: crate::orchestration::identity::InstanceId,
    pub epoch: u64,
    pub base_workflow_revision: super::super::WorkflowRevisionRef,
    pub proposal_id: crate::orchestration::identity::ProposalId,
    pub authority_policy_hash: crate::orchestration::identity::AuthorityPolicyIdentity,
    pub actor: String,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance {
    pub id: crate::orchestration::identity::InstanceId,
    pub sequence: u64,
    pub trigger_idempotency_key: Option<crate::orchestration::identity::TriggerScopedKey>,
    pub definition_revision: crate::orchestration::identity::DefinitionRevision,
    pub definition: super::super::Definition,
    pub epoch: u64,
    pub version: u64,
    pub status: Status,
    pub paused_status: Option<Status>,
    pub pause_requested: bool,
    pub cancellation_requested: bool,
    pub workflow_revision: super::super::WorkflowRevisionRef,
    pub round: u64,
    pub active_run_id: Option<crate::engine::RunId>,
    pub pending_dispatch: Option<DispatchIntent>,
    pub pending_proposal: Option<RevisionProposal>,
    pub budgets: super::super::BudgetLedger,
    #[serde(default = "empty_retry_ledger", skip_serializing_if = "RetryLedger::is_empty")]
    pub retry: RetryLedger,
    pub accepted_fact_ids: std::collections::BTreeSet<crate::orchestration::identity::FactId>,
    pub last_event_sequence: Option<u64>,
    pub event_head: Option<crate::orchestration::identity::LedgerHeadId>,
    pub terminal_summary: Option<String>,
    #[serde(
        default = "reserved_usage_basis",
        skip_serializing_if = "BudgetUsageBasis::is_reserved_fallback"
    )]
    pub last_usage_basis: BudgetUsageBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fact {
    pub id: crate::orchestration::identity::FactId,
    pub action: super::super::Action,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetUsageBasis {
    #[default]
    ReservedFallback,
    /// Frozen conservative charge with separately retained host/provider evidence.
    ConservativeObservation,
    Measured,
    MeasuredTokens,
    MeasuredDuration,
    MeasuredCost,
    MeasuredTokensAndDuration,
    MeasuredTokensAndCost,
    MeasuredDurationAndCost,
    MeasuredTokensDurationAndCost,
}

impl BudgetUsageBasis {
    pub(super) fn is_reserved_fallback(&self) -> bool {
        matches!(self, Self::ReservedFallback)
    }

    pub(in crate::orchestration::core) fn is_measured(&self) -> bool {
        matches!(self, Self::Measured)
    }
}

pub fn new_instance(
    definition: super::super::Definition,
    sequence: u64,
    trigger_idempotency_key: Option<crate::orchestration::identity::TriggerScopedKey>,
) -> Result<Instance, super::super::Diagnostic> {
    let definition_revision = super::super::definition_revision(&definition)?;
    let id = crate::orchestration::identity::InstanceId::from_generated(super::canonical::hash_serializable(
        super::super::INSTANCE_HASH_LABEL,
        &(
            definition_revision.as_str(),
            sequence,
            trigger_idempotency_key.as_ref().map(crate::orchestration::identity::TriggerScopedKey::as_str),
        ),
    )?);
    let workflow_revision = definition.workflow.clone();
    let policy = definition.budgets;
    let instance = Instance {
        id,
        sequence,
        trigger_idempotency_key,
        definition_revision,
        definition,
        epoch: 0,
        version: 0,
        status: Status::Ready,
        paused_status: None,
        pause_requested: false,
        cancellation_requested: false,
        workflow_revision,
        round: 0,
        active_run_id: None,
        pending_dispatch: None,
        pending_proposal: None,
        budgets: super::super::BudgetLedger {
            policy,
            consumed: super::super::BudgetCharge {
                rounds: 0,
                agent_invocations: 0,
                tokens: 0,
                monetary_microunits: 0,
                duration_millis: 0,
            },
            reservations: std::collections::BTreeMap::new(),
            breached: false,
        },
        retry: RetryLedger::default(),
        accepted_fact_ids: std::collections::BTreeSet::new(),
        last_event_sequence: None,
        event_head: None,
        terminal_summary: None,
        last_usage_basis: BudgetUsageBasis::ReservedFallback,
    };
    debug_assert_eq!(instance.workflow_revision, instance.definition.workflow);
    debug_assert_eq!(instance.budgets.policy, instance.definition.budgets);
    Ok(instance)
}

pub fn is_terminal_status(status: Status) -> bool {
    matches!(status, Status::Completed | Status::Failed | Status::Cancelled | Status::Exhausted)
}

pub fn render_status_text(instance: &Instance) -> String {
    debug_assert!(!instance.id.as_str().is_empty());
    debug_assert!(!instance.workflow_revision.content_hash.as_str().is_empty());
    let active_run = instance.active_run_id.map_or_else(|| "-".to_string(), |run_id| run_id.to_string());
    let proposal = instance.pending_proposal.as_ref().map_or("-", |proposal| proposal.id.as_str());
    let event_sequence = instance.last_event_sequence.map_or_else(|| "-".to_string(), |sequence| sequence.to_string());
    let event_head = instance.event_head.as_ref().map_or("-", crate::orchestration::identity::LedgerHeadId::as_str);
    let remaining = budget_ledger_core::remaining_budget_for(&instance.budgets);
    format!(
        "{} | {:?} | revision {} | round {} | run {} | proposal {} | remaining {}/{}/{}/{}/{} | breached {} | event {}/{} | next {} | evidence {},{}",
        instance.id,
        instance.status,
        instance.workflow_revision.content_hash,
        instance.round,
        active_run,
        proposal,
        remaining.rounds,
        remaining.agent_invocations,
        remaining.tokens,
        remaining.monetary_microunits,
        remaining.duration_millis,
        instance.budgets.breached,
        event_sequence,
        event_head,
        next_action(instance),
        super::super::LOCAL_CHAIN_EVIDENCE_SCOPE,
        super::super::REMOTE_EFFECT_EVIDENCE_SCOPE,
    )
}

pub(super) fn next_action(instance: &Instance) -> &'static str {
    match instance.status {
        Status::Ready => "start_round",
        Status::Running if instance.cancellation_requested => "observe_cancellation",
        Status::Running => "wait_for_run",
        Status::WaitingForApproval if instance.pending_proposal.is_some() => "approve_or_reject",
        Status::WaitingForApproval => "submit_proposal",
        Status::Paused => "resume_or_cancel",
        Status::Failed if instance.retry.retryable_failure.is_some() => "retry",
        Status::Completed | Status::Failed | Status::Cancelled | Status::Exhausted => "none",
    }
}

fn empty_retry_ledger() -> RetryLedger {
    RetryLedger {
        admitted: 0,
        retryable_failure: None,
    }
}

const fn reserved_usage_basis() -> BudgetUsageBasis {
    BudgetUsageBasis::ReservedFallback
}
