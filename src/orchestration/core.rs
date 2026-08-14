//! Pure durable agent-orchestration model and decision core.
//!
//! This module owns deterministic validation, lifecycle, budget, authority,
//! event, replay, and trigger-admission decisions. It performs no I/O.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::RunId;
use crate::engine::Workflow;

const ORCHESTRATION_HASH_DOMAIN: &[u8] = b"lattice-orchestration:v1";
const DEFINITION_HASH_LABEL: &[u8] = b"definition";
const WORKFLOW_HASH_LABEL: &[u8] = b"workflow";
const INSTANCE_HASH_LABEL: &[u8] = b"instance";
const PROPOSAL_HASH_LABEL: &[u8] = b"proposal";
const APPROVAL_HASH_LABEL: &[u8] = b"approval";
const AUTHORITY_HASH_LABEL: &[u8] = b"authority";
const EVENT_HASH_LABEL: &[u8] = b"event";
const STATE_HASH_LABEL: &[u8] = b"state";
const FACT_HASH_LABEL: &[u8] = b"fact";
const TRIGGER_HASH_LABEL: &[u8] = b"trigger";
const CURRENT_SCHEMA_VERSION: u32 = 1;
const HEX_CHARS_PER_BYTE: usize = 2;
const FIRST_EVENT_SEQUENCE: u64 = 0;
const EVENT_SEQUENCE_INCREMENT: u64 = 1;
const INSTANCE_VERSION_INCREMENT: u64 = 1;
const EPOCH_INCREMENT: u64 = 1;
const ROUND_INCREMENT: u64 = 1;
const MAX_REPLAY_DIAGNOSTICS: usize = 1;
const RETRY_INCREMENT: u64 = 1;
const RETRY_PREFLIGHT_RESERVATION_ID: &str = "retry-preflight";
pub const ABANDONED_ACKNOWLEDGED_SUMMARY: &str =
    "local dispatch abandoned after acknowledgement; remote outcome unknown";
pub const ABANDONED_PENDING_SUMMARY: &str =
    "local dispatch abandoned before acknowledgement; workflow shell was not invoked";
const LOCAL_CHAIN_EVIDENCE_SCOPE: &str = "local-sequence-consistency-only";
const REMOTE_EFFECT_EVIDENCE_SCOPE: &str = "remote-effects-may-be-at-least-once";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRevisionRef {
    pub workflow_name: String,
    pub content_hash: super::identity::WorkflowRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionPolicy {
    RunSucceeded,
    AwaitProposal,
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    pub max_observed_failure_retries: u64,
}

pub use budget_ledger_core::BudgetCharge;
pub use budget_ledger_core::BudgetChargePlan;
pub use budget_ledger_core::BudgetDimensions;
pub use budget_ledger_core::BudgetLimits;
pub use budget_ledger_core::BudgetPolicy;
pub use budget_ledger_core::BudgetSettlement;

pub type BudgetLedger = budget_ledger_core::BudgetLedger<super::identity::ReservationId>;
pub type BudgetReservation = budget_ledger_core::BudgetReservation<super::identity::ReservationId>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPolicy {
    pub approvers: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationTriggerKind {
    Manual,
    Schedule,
    Webhook,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrchestrationTriggerBinding {
    pub trigger_id: super::identity::TriggerId,
    pub kind: OrchestrationTriggerKind,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactionPolicy {
    pub retain_safe_summaries: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrchestrationDefinition {
    pub schema_version: u32,
    pub id: super::identity::DefinitionId,
    pub objective: String,
    pub workflow: WorkflowRevisionRef,
    pub completion: CompletionPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryPolicy>,
    pub budgets: BudgetPolicy,
    pub authority: AuthorityPolicy,
    pub triggers: Vec<OrchestrationTriggerBinding>,
    pub redaction: RedactionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryableFailure {
    pub run_id: RunId,
    pub dispatch_id: super::identity::DispatchId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryLedger {
    pub admitted: u64,
    pub retryable_failure: Option<RetryableFailure>,
}

impl RetryLedger {
    fn is_empty(&self) -> bool {
        self.admitted == 0 && self.retryable_failure.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationStatus {
    Ready,
    Running,
    WaitingForApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchStatus {
    Pending,
    Acknowledged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchIntent {
    pub id: super::identity::DispatchId,
    pub run_id: RunId,
    pub workflow_revision: WorkflowRevisionRef,
    pub status: DispatchStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionProposal {
    pub id: super::identity::ProposalId,
    pub instance_id: super::identity::InstanceId,
    pub base_definition_revision: super::identity::DefinitionRevision,
    pub base_epoch: u64,
    pub base_workflow_revision: WorkflowRevisionRef,
    pub successor: WorkflowRevisionRef,
    pub safe_summary: String,
}

// r[impl lattice.workflow_approval.compatibility]
// r[impl lattice.workflow_approval.compatibility.valid]
// r[impl lattice.workflow_approval.compatibility.denied]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub id: super::identity::ApprovalId,
    pub instance_id: super::identity::InstanceId,
    pub epoch: u64,
    pub base_workflow_revision: WorkflowRevisionRef,
    pub proposal_id: super::identity::ProposalId,
    pub authority_policy_hash: super::identity::AuthorityPolicyIdentity,
    pub actor: String,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationInstance {
    pub id: super::identity::InstanceId,
    pub sequence: u64,
    pub trigger_idempotency_key: Option<super::identity::TriggerScopedKey>,
    pub definition_revision: super::identity::DefinitionRevision,
    pub definition: OrchestrationDefinition,
    pub epoch: u64,
    pub version: u64,
    pub status: OrchestrationStatus,
    pub paused_status: Option<OrchestrationStatus>,
    pub pause_requested: bool,
    pub cancellation_requested: bool,
    pub workflow_revision: WorkflowRevisionRef,
    pub round: u64,
    pub active_run_id: Option<RunId>,
    pub pending_dispatch: Option<DispatchIntent>,
    pub pending_proposal: Option<RevisionProposal>,
    pub budgets: BudgetLedger,
    #[serde(default, skip_serializing_if = "RetryLedger::is_empty")]
    pub retry: RetryLedger,
    pub accepted_fact_ids: BTreeSet<super::identity::FactId>,
    pub last_event_sequence: Option<u64>,
    pub event_head: Option<super::identity::LedgerHeadId>,
    pub terminal_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationFact {
    pub id: super::identity::FactId,
    pub action: OrchestrationAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OrchestrationAction {
    StartRound {
        intent: DispatchIntent,
        planned_charge: BudgetChargePlan,
    },
    RetryRound {
        actor: String,
        intent: DispatchIntent,
        planned_charge: BudgetChargePlan,
    },
    DispatchAcknowledged {
        dispatch_id: super::identity::DispatchId,
    },
    RunFinished {
        dispatch_id: super::identity::DispatchId,
        run_id: RunId,
        succeeded: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        retryable_observed_failure: bool,
        observed_usage: BudgetCharge,
        safe_summary: String,
    },
    SubmitProposal {
        proposal: RevisionProposal,
    },
    DecideProposal {
        approval: ApprovalDecision,
        successor_validated: bool,
    },
    RejectProposal {
        actor: String,
        proposal_id: super::identity::ProposalId,
    },
    PauseRequested {
        actor: String,
    },
    ResumeRequested {
        actor: String,
    },
    CancelRequested {
        actor: String,
    },
    CancellationObserved {
        run_id: RunId,
        cancelled: bool,
    },
    AbandonPendingDispatch {
        actor: String,
        dispatch_id: super::identity::DispatchId,
        run_id: RunId,
    },
    BudgetExhausted {
        safe_reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationEventKind {
    RoundStarted,
    RetryRoundStarted,
    DispatchAcknowledged,
    RoundCompleted,
    RoundFailed,
    ProposalSubmitted,
    ProposalApproved,
    ProposalRejected,
    Paused,
    Resumed,
    CancellationRequested,
    CancellationObserved,
    DispatchAbandoned,
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationEvent {
    pub instance_id: super::identity::InstanceId,
    pub sequence: u64,
    pub prior_hash: Option<super::identity::PreviousEventId>,
    pub event_hash: super::identity::EventId,
    pub state_hash: super::identity::StateDigest,
    pub kind: OrchestrationEventKind,
    pub fact: OrchestrationFact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationTransition {
    pub instance: OrchestrationInstance,
    pub event: Option<OrchestrationEvent>,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationDiagnosticCode {
    InvalidSchemaVersion,
    EmptyDefinitionId,
    EmptyObjective,
    EmptyWorkflowName,
    InvalidContentHash,
    WorkflowRevisionMismatch,
    MissingApprover,
    DuplicateApprover,
    EmptyTriggerId,
    DuplicateTriggerId,
    MissingBudgetBound,
    BudgetExceeded,
    BudgetOverflow,
    DuplicateReservation,
    MissingReservation,
    InvalidLifecycleTransition,
    TerminalStateMutation,
    DispatchMismatch,
    ProposalMismatch,
    StaleApproval,
    UnauthorizedActor,
    CancellationRequired,
    RetryDisabled,
    RetryLimitExceeded,
    RetryEvidenceMissing,
    SuccessorValidationFailed,
    InvalidIdentity,
    ReplayMismatch,
    ReplayHeadMismatch,
    TriggerNotBound,
    DuplicateTrigger,
    SerializationFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationDiagnostic {
    pub code: OrchestrationDiagnosticCode,
    pub message: String,
}

impl std::fmt::Display for OrchestrationDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for OrchestrationDiagnostic {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayReport {
    pub valid: bool,
    pub instance: OrchestrationInstance,
    pub diagnostics: Vec<OrchestrationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationTriggerContext {
    pub accepted: BTreeMap<super::identity::TriggerScopedKey, super::identity::InstanceId>,
}

#[derive(Debug, Clone, Copy)]
pub struct TriggerStartRequest<'a> {
    pub definition: &'a OrchestrationDefinition,
    pub definition_revision: &'a super::identity::DefinitionRevision,
    pub trigger_id: &'a super::identity::TriggerId,
    pub trigger_kind: OrchestrationTriggerKind,
    pub idempotency_key: &'a super::identity::TriggerIdempotencyKey,
    pub next_sequence: u64,
    pub context: &'a OrchestrationTriggerContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationTriggerPlan {
    pub scoped_key: super::identity::TriggerScopedKey,
    pub instance_id: super::identity::InstanceId,
    pub definition_revision: super::identity::DefinitionRevision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestrationTriggerAdmission {
    Accepted(OrchestrationTriggerPlan),
    Duplicate { instance_id: super::identity::InstanceId },
}

pub fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

fn is_false(value: &bool) -> bool {
    !*value
}

// r[impl lattice.agent_orchestration.model]
pub fn workflow_revision(workflow: &Workflow) -> Result<WorkflowRevisionRef, OrchestrationDiagnostic> {
    let portable_workflow = crate::workflow_format::workflow_to_portable(workflow);
    let content_hash =
        super::identity::WorkflowRevision::from_generated(hash_serializable(WORKFLOW_HASH_LABEL, &portable_workflow)?);
    Ok(WorkflowRevisionRef {
        workflow_name: workflow.meta.name.clone(),
        content_hash,
    })
}

// r[impl lattice.agent_orchestration.model]
pub fn definition_revision(
    definition: &OrchestrationDefinition,
) -> Result<super::identity::DefinitionRevision, OrchestrationDiagnostic> {
    Ok(super::identity::DefinitionRevision::from_generated(hash_serializable(
        DEFINITION_HASH_LABEL,
        definition,
    )?))
}

pub fn authority_policy_hash(
    policy: &AuthorityPolicy,
) -> Result<super::identity::AuthorityPolicyIdentity, OrchestrationDiagnostic> {
    Ok(super::identity::AuthorityPolicyIdentity::from_generated(hash_serializable(
        AUTHORITY_HASH_LABEL,
        policy,
    )?))
}

pub fn proposal_identity(proposal: &RevisionProposal) -> Result<super::identity::ProposalId, OrchestrationDiagnostic> {
    let mut unhashed = proposal.clone();
    unhashed.id.clear_for_hash();
    Ok(super::identity::ProposalId::from_generated(hash_serializable(PROPOSAL_HASH_LABEL, &unhashed)?))
}

pub fn approval_identity(approval: &ApprovalDecision) -> Result<super::identity::ApprovalId, OrchestrationDiagnostic> {
    let mut unhashed = approval.clone();
    unhashed.id.clear_for_hash();
    Ok(super::identity::ApprovalId::from_generated(hash_serializable(APPROVAL_HASH_LABEL, &unhashed)?))
}

pub fn fact_identity(
    instance_id: &super::identity::InstanceId,
    version: u64,
    action: &OrchestrationAction,
) -> Result<super::identity::FactId, OrchestrationDiagnostic> {
    Ok(super::identity::FactId::from_generated(hash_serializable(
        FACT_HASH_LABEL,
        &(instance_id, version, action),
    )?))
}

// r[impl lattice.agent_orchestration.model]
pub fn validate_definition(
    definition: &OrchestrationDefinition,
    actual_workflow_revision: Option<&WorkflowRevisionRef>,
) -> Vec<OrchestrationDiagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(diagnostic) = schema_version_diagnostic(definition.schema_version) {
        diagnostics.push(diagnostic);
    }
    if let Some(diagnostic) = empty_objective_diagnostic(&definition.objective) {
        diagnostics.push(diagnostic);
    }
    validate_workflow_ref(&definition.workflow, &mut diagnostics);
    validate_actual_workflow_revision(definition, actual_workflow_revision, &mut diagnostics);
    validate_authority_policy(&definition.authority, &mut diagnostics);
    validate_trigger_bindings(&definition.triggers, &mut diagnostics);
    validate_budget_policy(&definition.budgets, &mut diagnostics);
    if let Some(diagnostic) = retry_policy_diagnostic(definition.retry) {
        diagnostics.push(diagnostic);
    }
    debug_assert!(!diagnostics.is_empty() || definition.schema_version == CURRENT_SCHEMA_VERSION);
    debug_assert!(!diagnostics.is_empty() || !definition.authority.approvers.is_empty());
    diagnostics
}

fn schema_version_diagnostic(schema_version: u32) -> Option<OrchestrationDiagnostic> {
    (schema_version != CURRENT_SCHEMA_VERSION).then(|| {
        diagnostic(
            OrchestrationDiagnosticCode::InvalidSchemaVersion,
            format!("orchestration schema version {schema_version} is unsupported; expected {CURRENT_SCHEMA_VERSION}"),
        )
    })
}

fn empty_objective_diagnostic(objective: &str) -> Option<OrchestrationDiagnostic> {
    objective
        .trim()
        .is_empty()
        .then(|| diagnostic(OrchestrationDiagnosticCode::EmptyObjective, "orchestration objective cannot be empty"))
}

fn validate_workflow_ref(reference: &WorkflowRevisionRef, diagnostics: &mut Vec<OrchestrationDiagnostic>) {
    if reference.workflow_name.trim().is_empty() {
        diagnostics
            .push(diagnostic(OrchestrationDiagnosticCode::EmptyWorkflowName, "workflow revision name cannot be empty"));
    }
}

fn validate_actual_workflow_revision(
    definition: &OrchestrationDefinition,
    actual: Option<&WorkflowRevisionRef>,
    diagnostics: &mut Vec<OrchestrationDiagnostic>,
) {
    let Some(actual) = actual else {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::WorkflowRevisionMismatch,
            "referenced workflow revision is unavailable",
        ));
        return;
    };
    if actual != &definition.workflow {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::WorkflowRevisionMismatch,
            "referenced workflow content does not match the immutable workflow revision",
        ));
    }
}

fn validate_authority_policy(policy: &AuthorityPolicy, diagnostics: &mut Vec<OrchestrationDiagnostic>) {
    if policy.approvers.is_empty() {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::MissingApprover,
            "authority policy must declare at least one approver",
        ));
    }
    if policy.approvers.iter().any(|approver| approver.trim().is_empty()) {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::DuplicateApprover,
            "authority policy approver identities must be non-empty",
        ));
    }
}

fn validate_trigger_bindings(bindings: &[OrchestrationTriggerBinding], diagnostics: &mut Vec<OrchestrationDiagnostic>) {
    let mut ids = BTreeSet::new();
    for binding in bindings {
        if !ids.insert(binding.trigger_id.clone()) {
            diagnostics.push(diagnostic(
                OrchestrationDiagnosticCode::DuplicateTriggerId,
                format!("orchestration trigger '{}' is declared more than once", binding.trigger_id),
            ));
        }
    }
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
fn retry_policy_diagnostic(policy: Option<RetryPolicy>) -> Option<OrchestrationDiagnostic> {
    match policy {
        Some(policy) if policy.max_observed_failure_retries == 0 => Some(diagnostic(
            OrchestrationDiagnosticCode::RetryDisabled,
            "enabled retry policy must allow at least one observed-failure retry",
        )),
        Some(_) | None => None,
    }
}

fn validate_budget_policy(policy: &BudgetPolicy, diagnostics: &mut Vec<OrchestrationDiagnostic>) {
    validate_configured_charge(
        policy.enforced.rounds,
        policy.limits.rounds,
        policy.per_round_upper_bound.rounds,
        "rounds",
        diagnostics,
    );
    validate_configured_charge(
        policy.enforced.agent_invocations,
        policy.limits.agent_invocations,
        policy.per_round_upper_bound.agent_invocations,
        "agent invocations",
        diagnostics,
    );
    validate_configured_charge(
        policy.enforced.tokens,
        policy.limits.tokens,
        policy.per_round_upper_bound.tokens,
        "tokens",
        diagnostics,
    );
    validate_configured_charge(
        policy.enforced.monetary_microunits,
        policy.limits.monetary_microunits,
        policy.per_round_upper_bound.monetary_microunits,
        "monetary microunits",
        diagnostics,
    );
    validate_configured_charge(
        policy.enforced.duration_millis,
        policy.limits.duration_millis,
        policy.per_round_upper_bound.duration_millis,
        "duration milliseconds",
        diagnostics,
    );
}

fn validate_configured_charge(
    enforced: bool,
    limit: u64,
    charge: u64,
    label: &str,
    diagnostics: &mut Vec<OrchestrationDiagnostic>,
) {
    if enforced && charge > limit {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::BudgetExceeded,
            format!("per-round {label} upper bound exceeds the orchestration limit"),
        ));
    }
}

pub fn new_instance(
    definition: OrchestrationDefinition,
    sequence: u64,
    trigger_idempotency_key: Option<super::identity::TriggerScopedKey>,
) -> Result<OrchestrationInstance, OrchestrationDiagnostic> {
    let definition_revision = definition_revision(&definition)?;
    let id = super::identity::InstanceId::from_generated(hash_serializable(
        INSTANCE_HASH_LABEL,
        &(
            definition_revision.as_str(),
            sequence,
            trigger_idempotency_key.as_ref().map(super::identity::TriggerScopedKey::as_str),
        ),
    )?);
    let workflow_revision = definition.workflow.clone();
    let policy = definition.budgets;
    let instance = OrchestrationInstance {
        id,
        sequence,
        trigger_idempotency_key,
        definition_revision,
        definition,
        epoch: 0,
        version: 0,
        status: OrchestrationStatus::Ready,
        paused_status: None,
        pause_requested: false,
        cancellation_requested: false,
        workflow_revision,
        round: 0,
        active_run_id: None,
        pending_dispatch: None,
        pending_proposal: None,
        budgets: BudgetLedger {
            policy,
            ..BudgetLedger::default()
        },
        retry: RetryLedger::default(),
        accepted_fact_ids: BTreeSet::new(),
        last_event_sequence: None,
        event_head: None,
        terminal_summary: None,
    };
    debug_assert_eq!(instance.workflow_revision, instance.definition.workflow);
    debug_assert_eq!(instance.budgets.policy, instance.definition.budgets);
    Ok(instance)
}

// r[impl lattice.agent_orchestration.core]
// r[impl lattice.agent_orchestration.nominal_identities.transitions]
// r[impl lattice.agent_orchestration.nominal_identities.failure_matrix]
pub fn transition(
    instance: &OrchestrationInstance,
    fact: OrchestrationFact,
) -> Result<OrchestrationTransition, OrchestrationDiagnostic> {
    if instance.accepted_fact_ids.contains(&fact.id) {
        return Ok(OrchestrationTransition {
            instance: instance.clone(),
            event: None,
            duplicate: true,
        });
    }
    if !matches!(fact.action, OrchestrationAction::RetryRound { .. }) {
        reject_terminal_mutation(instance)?;
    }
    let mut next = instance.clone();
    let kind = apply_action(&mut next, &fact.action)?;
    let is_inserted = next.accepted_fact_ids.insert(fact.id.clone());
    debug_assert!(is_inserted);
    next.version = next.version.saturating_add(INSTANCE_VERSION_INCREMENT);
    let event = build_event(instance, &next, kind, fact)?;
    next.last_event_sequence = Some(event.sequence);
    next.event_head = Some(super::identity::LedgerHeadId::from_event(&event.event_hash));
    Ok(OrchestrationTransition {
        instance: next,
        event: Some(event),
        duplicate: false,
    })
}

fn reject_terminal_mutation(instance: &OrchestrationInstance) -> Result<(), OrchestrationDiagnostic> {
    if is_terminal_status(instance.status) {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::TerminalStateMutation,
            format!("orchestration instance is terminal in state {:?}", instance.status),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct RunFinishedInput<'a> {
    dispatch_id: &'a super::identity::DispatchId,
    run_id: RunId,
    is_succeeded: bool,
    is_retryable_observed_failure: bool,
    observed_usage: BudgetCharge,
    safe_summary: &'a str,
}

fn apply_action(
    instance: &mut OrchestrationInstance,
    action: &OrchestrationAction,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    match action {
        OrchestrationAction::StartRound { intent, planned_charge } => start_round(instance, intent, *planned_charge),
        OrchestrationAction::RetryRound {
            actor,
            intent,
            planned_charge,
        } => start_retry_round(instance, actor, intent, *planned_charge),
        OrchestrationAction::DispatchAcknowledged { dispatch_id } => acknowledge_dispatch(instance, dispatch_id),
        OrchestrationAction::RunFinished {
            dispatch_id,
            run_id,
            succeeded,
            retryable_observed_failure,
            observed_usage,
            safe_summary,
        } => finish_run(instance, RunFinishedInput {
            dispatch_id,
            run_id: *run_id,
            is_succeeded: *succeeded,
            is_retryable_observed_failure: *retryable_observed_failure,
            observed_usage: *observed_usage,
            safe_summary,
        }),
        OrchestrationAction::SubmitProposal { proposal } => submit_proposal(instance, proposal),
        OrchestrationAction::DecideProposal {
            approval,
            successor_validated,
        } => decide_proposal(instance, approval, *successor_validated),
        OrchestrationAction::RejectProposal { actor, proposal_id } => reject_proposal(instance, actor, proposal_id),
        OrchestrationAction::PauseRequested { actor } => pause(instance, actor),
        OrchestrationAction::ResumeRequested { actor } => resume(instance, actor),
        OrchestrationAction::CancelRequested { actor } => cancel(instance, actor),
        OrchestrationAction::CancellationObserved { run_id, cancelled } => {
            observe_cancellation(instance, *run_id, *cancelled)
        }
        OrchestrationAction::AbandonPendingDispatch {
            actor,
            dispatch_id,
            run_id,
        } => abandon_pending_dispatch(instance, actor, dispatch_id, *run_id),
        OrchestrationAction::BudgetExhausted { safe_reason } => exhaust_budget(instance, safe_reason),
    }
}

fn start_round(
    instance: &mut OrchestrationInstance,
    intent: &DispatchIntent,
    planned_charge: BudgetChargePlan,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    if instance.status != OrchestrationStatus::Ready || instance.pause_requested {
        return Err(invalid_transition("a round can start only from an unpaused ready instance"));
    }
    begin_dispatch(instance, intent, planned_charge)?;
    Ok(OrchestrationEventKind::RoundStarted)
}

// r[impl lattice.agent_orchestration.retry_round]
fn start_retry_round(
    instance: &mut OrchestrationInstance,
    actor: &str,
    intent: &DispatchIntent,
    planned_charge: BudgetChargePlan,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    let failed = preflight_retry(instance, actor, planned_charge)?;
    if intent.run_id == failed.run_id || intent.id == failed.dispatch_id {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "retry must use run and dispatch identities distinct from the observed failure",
        ));
    }
    if dispatch_identity(instance, intent.run_id)? != intent.id {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::InvalidIdentity,
            "retry dispatch BLAKE3 identity does not match its canonical inputs",
        ));
    }
    begin_dispatch(instance, intent, planned_charge)?;
    instance.retry.admitted = instance.retry.admitted.saturating_add(RETRY_INCREMENT);
    instance.retry.retryable_failure = None;
    instance.terminal_summary = None;
    debug_assert!(instance.retry.admitted > 0);
    debug_assert!(instance.retry.retryable_failure.is_none());
    Ok(OrchestrationEventKind::RetryRoundStarted)
}

// r[impl lattice.agent_orchestration.retry_round]
pub fn preflight_retry(
    instance: &OrchestrationInstance,
    actor: &str,
    planned_charge: BudgetChargePlan,
) -> Result<RetryableFailure, OrchestrationDiagnostic> {
    let failed = retryable_failure(instance, actor)?;
    let mut budget = instance.budgets.clone();
    let preflight_id =
        super::identity::ReservationId::try_from_text(RETRY_PREFLIGHT_RESERVATION_ID).map_err(identity_error)?;
    reserve_budget(&mut budget, preflight_id, planned_charge)?;
    Ok(failed)
}

fn retryable_failure(
    instance: &OrchestrationInstance,
    actor: &str,
) -> Result<RetryableFailure, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    if instance.status != OrchestrationStatus::Failed
        || instance.active_run_id.is_some()
        || instance.pending_dispatch.is_some()
    {
        return Err(invalid_transition(
            "retry requires a terminal failed instance without an active or pending dispatch",
        ));
    }
    let policy = instance.definition.retry.ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::RetryDisabled,
            "orchestration definition does not enable observed-failure retries",
        )
    })?;
    if instance.retry.admitted >= policy.max_observed_failure_retries {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::RetryLimitExceeded,
            "observed-failure retry limit is exhausted",
        ));
    }
    let failed = instance.retry.retryable_failure.clone().ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::RetryEvidenceMissing,
            "failed orchestration has no exact persisted terminal-failure retry evidence",
        )
    })?;
    debug_assert_eq!(instance.status, OrchestrationStatus::Failed);
    debug_assert!(instance.retry.admitted < policy.max_observed_failure_retries);
    Ok(failed)
}

fn begin_dispatch(
    instance: &mut OrchestrationInstance,
    intent: &DispatchIntent,
    planned_charge: BudgetChargePlan,
) -> Result<(), OrchestrationDiagnostic> {
    if intent.workflow_revision != instance.workflow_revision {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::WorkflowRevisionMismatch,
            "dispatch intent workflow revision does not match the active orchestration revision",
        ));
    }
    if intent.status != DispatchStatus::Pending {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::InvalidLifecycleTransition,
            "new dispatch intent must begin in pending state",
        ));
    }
    reserve_budget(&mut instance.budgets, super::identity::ReservationId::from_dispatch(&intent.id), planned_charge)?;
    instance.round = instance.round.saturating_add(ROUND_INCREMENT);
    instance.active_run_id = Some(intent.run_id);
    instance.pending_dispatch = Some(intent.clone());
    instance.status = OrchestrationStatus::Running;
    instance.cancellation_requested = false;
    debug_assert_eq!(instance.active_run_id, Some(intent.run_id));
    debug_assert_eq!(instance.pending_dispatch.as_ref(), Some(intent));
    Ok(())
}

fn acknowledge_dispatch(
    instance: &mut OrchestrationInstance,
    dispatch_id: &super::identity::DispatchId,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    let dispatch = matching_dispatch_mut(instance, dispatch_id)?;
    dispatch.status = DispatchStatus::Acknowledged;
    Ok(OrchestrationEventKind::DispatchAcknowledged)
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
fn finish_run(
    instance: &mut OrchestrationInstance,
    input: RunFinishedInput<'_>,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    let dispatch = matching_dispatch(instance, input.dispatch_id)?;
    if dispatch.run_id != input.run_id || instance.active_run_id != Some(input.run_id) {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "run completion does not match the active dispatch identity",
        ));
    }
    if input.is_succeeded && input.is_retryable_observed_failure {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::RetryEvidenceMissing,
            "successful run cannot be marked as observed retryable failure",
        ));
    }
    let reservation_id = super::identity::ReservationId::from_dispatch(input.dispatch_id);
    let settlement = settle_budget(&mut instance.budgets, &reservation_id, input.observed_usage)?;
    instance.active_run_id = None;
    instance.pending_dispatch = None;
    instance.retry.retryable_failure = None;
    instance.terminal_summary =
        instance.definition.redaction.retain_safe_summaries.then(|| input.safe_summary.to_string());
    let kind = if input.is_succeeded {
        OrchestrationEventKind::RoundCompleted
    } else {
        OrchestrationEventKind::RoundFailed
    };
    if settlement.breached {
        instance.status = OrchestrationStatus::Exhausted;
        return Ok(kind);
    }
    if instance.cancellation_requested {
        instance.status = OrchestrationStatus::Cancelled;
        return Ok(kind);
    }
    if !input.is_succeeded {
        instance.status = OrchestrationStatus::Failed;
        if input.is_retryable_observed_failure && instance.definition.retry.is_some() {
            instance.retry.retryable_failure = Some(RetryableFailure {
                run_id: input.run_id,
                dispatch_id: input.dispatch_id.clone(),
            });
        }
        debug_assert!(instance.active_run_id.is_none());
        debug_assert!(instance.pending_dispatch.is_none());
        return Ok(kind);
    }
    let next_status = match instance.definition.completion {
        CompletionPolicy::RunSucceeded => OrchestrationStatus::Completed,
        CompletionPolicy::AwaitProposal => OrchestrationStatus::WaitingForApproval,
    };
    if instance.pause_requested && !is_terminal_status(next_status) {
        instance.paused_status = Some(next_status);
        instance.status = OrchestrationStatus::Paused;
    } else {
        instance.status = next_status;
    }
    debug_assert!(instance.active_run_id.is_none());
    debug_assert!(instance.pending_dispatch.is_none());
    Ok(kind)
}

fn submit_proposal(
    instance: &mut OrchestrationInstance,
    proposal: &RevisionProposal,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    if instance.status != OrchestrationStatus::WaitingForApproval || instance.pending_proposal.is_some() {
        return Err(invalid_transition("a proposal requires an empty waiting-for-approval state"));
    }
    validate_proposal(instance, proposal)?;
    instance.pending_proposal = Some(proposal.clone());
    Ok(OrchestrationEventKind::ProposalSubmitted)
}

fn decide_proposal(
    instance: &mut OrchestrationInstance,
    approval: &ApprovalDecision,
    successor_validated: bool,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    if instance.status != OrchestrationStatus::WaitingForApproval {
        return Err(invalid_transition("proposal decisions require waiting-for-approval state"));
    }
    let proposal = instance.pending_proposal.clone().ok_or_else(|| {
        diagnostic(OrchestrationDiagnosticCode::ProposalMismatch, "no proposal is pending for approval")
    })?;
    validate_approval(instance, &proposal, approval)?;
    if !approval.approved {
        instance.pending_proposal = None;
        return Ok(OrchestrationEventKind::ProposalRejected);
    }
    if !successor_validated {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::SuccessorValidationFailed,
            "approval cannot bypass successor workflow validation",
        ));
    }
    instance.workflow_revision = proposal.successor;
    instance.pending_proposal = None;
    instance.epoch = instance.epoch.saturating_add(EPOCH_INCREMENT);
    instance.status = OrchestrationStatus::Ready;
    Ok(OrchestrationEventKind::ProposalApproved)
}

fn reject_proposal(
    instance: &mut OrchestrationInstance,
    actor: &str,
    proposal_id: &super::identity::ProposalId,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    if instance.status != OrchestrationStatus::WaitingForApproval {
        return Err(invalid_transition("proposal rejection requires waiting-for-approval state"));
    }
    let proposal = instance.pending_proposal.as_ref().ok_or_else(|| {
        diagnostic(OrchestrationDiagnosticCode::ProposalMismatch, "no proposal is pending for rejection")
    })?;
    if &proposal.id != proposal_id {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::ProposalMismatch,
            "proposal rejection identity does not match the pending proposal",
        ));
    }
    instance.pending_proposal = None;
    Ok(OrchestrationEventKind::ProposalRejected)
}

fn pause(instance: &mut OrchestrationInstance, actor: &str) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    if instance.pause_requested || instance.status == OrchestrationStatus::Paused {
        return Err(invalid_transition("orchestration instance is already paused"));
    }
    instance.pause_requested = true;
    if instance.status != OrchestrationStatus::Running {
        instance.paused_status = Some(instance.status);
        instance.status = OrchestrationStatus::Paused;
    }
    Ok(OrchestrationEventKind::Paused)
}

fn resume(
    instance: &mut OrchestrationInstance,
    actor: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    if !instance.pause_requested {
        return Err(invalid_transition("orchestration instance is not paused"));
    }
    instance.pause_requested = false;
    if instance.status == OrchestrationStatus::Paused {
        instance.status = instance.paused_status.take().ok_or_else(|| {
            diagnostic(
                OrchestrationDiagnosticCode::InvalidLifecycleTransition,
                "paused instance is missing its prior lifecycle state",
            )
        })?;
    }
    Ok(OrchestrationEventKind::Resumed)
}

fn cancel(
    instance: &mut OrchestrationInstance,
    actor: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    instance.cancellation_requested = true;
    if instance.status != OrchestrationStatus::Running {
        instance.status = OrchestrationStatus::Cancelled;
    }
    Ok(OrchestrationEventKind::CancellationRequested)
}

fn observe_cancellation(
    instance: &mut OrchestrationInstance,
    run_id: RunId,
    cancelled: bool,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    if !instance.cancellation_requested || instance.active_run_id != Some(run_id) {
        return Err(invalid_transition("cancellation observation does not match an active cancellation request"));
    }
    if cancelled {
        instance.status = OrchestrationStatus::Cancelled;
        instance.active_run_id = None;
        instance.pending_dispatch = None;
    }
    Ok(OrchestrationEventKind::CancellationObserved)
}

// r[impl lattice.agent_orchestration.unobserved_abandon]
fn abandon_pending_dispatch(
    instance: &mut OrchestrationInstance,
    actor: &str,
    dispatch_id: &super::identity::DispatchId,
    run_id: RunId,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    if !instance.cancellation_requested {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::CancellationRequired,
            "pending dispatch abandonment requires a prior cancellation request",
        ));
    }
    let dispatch = matching_dispatch(instance, dispatch_id)?;
    if dispatch.run_id != run_id || instance.active_run_id != Some(run_id) {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "abandonment does not match the active dispatch and run identities",
        ));
    }
    let dispatch_status = dispatch.status;
    let observed_usage = match dispatch_status {
        DispatchStatus::Pending => BudgetCharge::default(),
        DispatchStatus::Acknowledged => {
            instance
                .budgets
                .reservations
                .get(&super::identity::ReservationId::from_dispatch(dispatch_id))
                .ok_or_else(|| {
                    diagnostic(
                        OrchestrationDiagnosticCode::MissingReservation,
                        "acknowledged abandonment is missing its conservative budget reservation",
                    )
                })?
                .charge
        }
    };
    let reservation_id = super::identity::ReservationId::from_dispatch(dispatch_id);
    let settlement = settle_budget(&mut instance.budgets, &reservation_id, observed_usage)?;
    let safe_summary = match dispatch_status {
        DispatchStatus::Pending => ABANDONED_PENDING_SUMMARY,
        DispatchStatus::Acknowledged => ABANDONED_ACKNOWLEDGED_SUMMARY,
    };
    instance.active_run_id = None;
    instance.pending_dispatch = None;
    instance.pause_requested = false;
    instance.paused_status = None;
    instance.cancellation_requested = false;
    instance.retry.retryable_failure = None;
    instance.status = OrchestrationStatus::Failed;
    instance.terminal_summary = instance.definition.redaction.retain_safe_summaries.then(|| safe_summary.to_string());
    debug_assert!(!settlement.breached);
    debug_assert!(instance.active_run_id.is_none());
    debug_assert!(instance.pending_dispatch.is_none());
    Ok(OrchestrationEventKind::DispatchAbandoned)
}

fn exhaust_budget(
    instance: &mut OrchestrationInstance,
    safe_reason: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    if instance.status == OrchestrationStatus::Running {
        return Err(invalid_transition("budget exhaustion cannot erase an active runner observation"));
    }
    instance.status = OrchestrationStatus::Exhausted;
    instance.terminal_summary = instance.definition.redaction.retain_safe_summaries.then(|| safe_reason.to_string());
    Ok(OrchestrationEventKind::BudgetExhausted)
}

fn validate_proposal(
    instance: &OrchestrationInstance,
    proposal: &RevisionProposal,
) -> Result<(), OrchestrationDiagnostic> {
    if proposal.instance_id != instance.id
        || proposal.base_definition_revision != instance.definition_revision
        || proposal.base_epoch != instance.epoch
        || proposal.base_workflow_revision != instance.workflow_revision
    {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::ProposalMismatch,
            "proposal does not bind the current orchestration base state",
        ));
    }
    if proposal_identity(proposal)? != proposal.id {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::InvalidIdentity,
            "proposal BLAKE3 identity does not match its canonical content",
        ));
    }
    let mut diagnostics = Vec::new();
    validate_workflow_ref(&proposal.successor, &mut diagnostics);
    if let Some(diagnostic) = diagnostics.into_iter().next() {
        return Err(diagnostic);
    }
    Ok(())
}

fn validate_approval(
    instance: &OrchestrationInstance,
    proposal: &RevisionProposal,
    approval: &ApprovalDecision,
) -> Result<(), OrchestrationDiagnostic> {
    authorize_actor(instance, &approval.actor)?;
    let expected_authority_hash = authority_policy_hash(&instance.definition.authority)?;
    if approval.instance_id != instance.id
        || approval.epoch != instance.epoch
        || approval.base_workflow_revision != instance.workflow_revision
        || approval.proposal_id != proposal.id
        || approval.authority_policy_hash != expected_authority_hash
    {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::StaleApproval,
            "approval does not bind the current instance, epoch, revision, proposal, and authority policy",
        ));
    }
    if approval_identity(approval)? != approval.id {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::InvalidIdentity,
            "approval BLAKE3 identity does not match its canonical content",
        ));
    }
    Ok(())
}

pub fn authorize_definition_actor(
    definition: &OrchestrationDefinition,
    actor: &str,
) -> Result<(), OrchestrationDiagnostic> {
    if definition.authority.approvers.contains(actor) {
        Ok(())
    } else {
        Err(diagnostic(
            OrchestrationDiagnosticCode::UnauthorizedActor,
            format!("actor '{actor}' is not authorized for this orchestration"),
        ))
    }
}

fn authorize_actor(instance: &OrchestrationInstance, actor: &str) -> Result<(), OrchestrationDiagnostic> {
    if instance.definition.authority.approvers.contains(actor) {
        Ok(())
    } else {
        Err(diagnostic(
            OrchestrationDiagnosticCode::UnauthorizedActor,
            format!("actor '{actor}' is not authorized for this orchestration"),
        ))
    }
}

fn matching_dispatch<'a>(
    instance: &'a OrchestrationInstance,
    dispatch_id: &super::identity::DispatchId,
) -> Result<&'a DispatchIntent, OrchestrationDiagnostic> {
    instance.pending_dispatch.as_ref().filter(|intent| &intent.id == dispatch_id).ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "dispatch identity does not match the pending orchestration dispatch",
        )
    })
}

fn matching_dispatch_mut<'a>(
    instance: &'a mut OrchestrationInstance,
    dispatch_id: &super::identity::DispatchId,
) -> Result<&'a mut DispatchIntent, OrchestrationDiagnostic> {
    instance.pending_dispatch.as_mut().filter(|intent| &intent.id == dispatch_id).ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "dispatch identity does not match the pending orchestration dispatch",
        )
    })
}

// r[impl lattice.agent_orchestration.budgets]
// r[impl lattice.budget_ledger_incubation.boundary]
pub fn reserve_budget(
    ledger: &mut BudgetLedger,
    reservation_id: super::identity::ReservationId,
    plan: BudgetChargePlan,
) -> Result<BudgetCharge, OrchestrationDiagnostic> {
    budget_ledger_core::reserve_budget_for(ledger, reservation_id, plan).map_err(ledger_error)
}

pub fn settle_budget(
    ledger: &mut BudgetLedger,
    reservation_id: &super::identity::ReservationId,
    observed: BudgetCharge,
) -> Result<BudgetSettlement, OrchestrationDiagnostic> {
    budget_ledger_core::settle_budget_for(ledger, reservation_id, observed).map_err(ledger_error)
}

fn ledger_error(error: budget_ledger_core::LedgerError) -> OrchestrationDiagnostic {
    match error {
        budget_ledger_core::LedgerError::MissingBudgetBound { label } => diagnostic(
            OrchestrationDiagnosticCode::MissingBudgetBound,
            format!("enforced {label} budget has no admitted upper bound"),
        ),
        budget_ledger_core::LedgerError::BudgetExceeded { label } => {
            diagnostic(OrchestrationDiagnosticCode::BudgetExceeded, format!("{label} budget is exhausted"))
        }
        budget_ledger_core::LedgerError::BudgetOverflow { label } => {
            diagnostic(OrchestrationDiagnosticCode::BudgetOverflow, format!("{label} budget accounting overflowed"))
        }
        budget_ledger_core::LedgerError::DuplicateReservation => diagnostic(
            OrchestrationDiagnosticCode::DuplicateReservation,
            "budget reservation identity was already admitted",
        ),
        budget_ledger_core::LedgerError::MissingReservation => diagnostic(
            OrchestrationDiagnosticCode::MissingReservation,
            "budget settlement does not match a pending reservation",
        ),
    }
}

fn build_event(
    prior: &OrchestrationInstance,
    next: &OrchestrationInstance,
    kind: OrchestrationEventKind,
    fact: OrchestrationFact,
) -> Result<OrchestrationEvent, OrchestrationDiagnostic> {
    debug_assert_eq!(prior.id, next.id);
    debug_assert_eq!(next.version, prior.version.saturating_add(INSTANCE_VERSION_INCREMENT));
    let sequence = prior
        .last_event_sequence
        .map(|sequence| sequence.saturating_add(EVENT_SEQUENCE_INCREMENT))
        .unwrap_or(FIRST_EVENT_SEQUENCE);
    let state_hash = semantic_state_hash(next)?;
    let mut event = OrchestrationEvent {
        instance_id: next.id.clone(),
        sequence,
        prior_hash: prior.event_head.as_ref().map(super::identity::PreviousEventId::from_ledger_head),
        event_hash: super::identity::EventId::from_generated(
            "0".repeat(blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE)),
        ),
        state_hash,
        kind,
        fact,
    };
    event.event_hash = event_identity(&event)?;
    Ok(event)
}

fn event_identity(event: &OrchestrationEvent) -> Result<super::identity::EventId, OrchestrationDiagnostic> {
    let mut unhashed = event.clone();
    unhashed.event_hash.clear_for_hash();
    Ok(super::identity::EventId::from_generated(hash_serializable(EVENT_HASH_LABEL, &unhashed)?))
}

fn semantic_state_hash(
    instance: &OrchestrationInstance,
) -> Result<super::identity::StateDigest, OrchestrationDiagnostic> {
    let mut state = instance.clone();
    state.last_event_sequence = None;
    state.event_head = None;
    Ok(super::identity::StateDigest::from_generated(hash_serializable(STATE_HASH_LABEL, &state)?))
}

// r[impl lattice.agent_orchestration.events]
// r[impl lattice.agent_orchestration.nominal_identities.events]
pub fn replay_events(
    initial: &OrchestrationInstance,
    events: &[OrchestrationEvent],
    expected_head: Option<&super::identity::LedgerHeadId>,
) -> ReplayReport {
    let mut instance = initial.clone();
    let mut diagnostics = Vec::with_capacity(MAX_REPLAY_DIAGNOSTICS);
    for recorded in events {
        match transition(&instance, recorded.fact.clone()) {
            Ok(transition) => {
                let Some(generated) = transition.event else {
                    diagnostics.push(diagnostic(
                        OrchestrationDiagnosticCode::ReplayMismatch,
                        "replay event repeats an already accepted fact identity",
                    ));
                    break;
                };
                if generated != *recorded {
                    diagnostics.push(diagnostic(
                        OrchestrationDiagnosticCode::ReplayMismatch,
                        format!(
                            "orchestration event sequence {} does not match deterministic replay",
                            recorded.sequence
                        ),
                    ));
                    break;
                }
                instance = transition.instance;
            }
            Err(error) => {
                diagnostics.push(diagnostic(
                    OrchestrationDiagnosticCode::ReplayMismatch,
                    format!("orchestration replay rejected event {}: {}", recorded.sequence, error.message),
                ));
                break;
            }
        }
    }
    if replay_head_mismatch(&diagnostics, instance.event_head.as_ref(), expected_head) {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::ReplayHeadMismatch,
            "orchestration event chain does not match the supplied current head or checkpoint",
        ));
    }
    ReplayReport {
        valid: diagnostics.is_empty(),
        instance,
        diagnostics,
    }
}

// r[impl lattice.agent_orchestration.triggers]
pub fn admit_trigger_start(
    request: TriggerStartRequest<'_>,
) -> Result<OrchestrationTriggerAdmission, OrchestrationDiagnostic> {
    if !request
        .definition
        .triggers
        .iter()
        .any(|binding| &binding.trigger_id == request.trigger_id && binding.kind == request.trigger_kind)
    {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::TriggerNotBound,
            format!("trigger '{}' is not bound to orchestration '{}'", request.trigger_id, request.definition.id),
        ));
    }
    debug_assert!(
        request
            .definition
            .triggers
            .iter()
            .any(|binding| &binding.trigger_id == request.trigger_id && binding.kind == request.trigger_kind)
    );
    let scoped_key = scoped_trigger_key(request.definition_revision, request.trigger_id, request.idempotency_key)?;
    debug_assert!(!scoped_key.as_str().is_empty());
    if let Some(instance_id) = request.context.accepted.get(&scoped_key) {
        return Ok(OrchestrationTriggerAdmission::Duplicate {
            instance_id: instance_id.clone(),
        });
    }
    let instance_id = super::identity::InstanceId::from_generated(hash_serializable(
        INSTANCE_HASH_LABEL,
        &(request.definition_revision, request.next_sequence, Some(scoped_key.as_str())),
    )?);
    Ok(OrchestrationTriggerAdmission::Accepted(OrchestrationTriggerPlan {
        scoped_key,
        instance_id,
        definition_revision: request.definition_revision.clone(),
    }))
}

fn replay_head_mismatch(
    diagnostics: &[OrchestrationDiagnostic],
    actual_head: Option<&super::identity::LedgerHeadId>,
    expected_head: Option<&super::identity::LedgerHeadId>,
) -> bool {
    if !diagnostics.is_empty() {
        return false;
    }
    actual_head != expected_head
}

pub fn scoped_trigger_key(
    definition_revision: &super::identity::DefinitionRevision,
    trigger_id: &super::identity::TriggerId,
    idempotency_key: &super::identity::TriggerIdempotencyKey,
) -> Result<super::identity::TriggerScopedKey, OrchestrationDiagnostic> {
    Ok(super::identity::TriggerScopedKey::from_generated(hash_serializable(
        TRIGGER_HASH_LABEL,
        &(definition_revision, trigger_id, idempotency_key),
    )?))
}

pub fn make_proposal(
    instance: &OrchestrationInstance,
    successor: WorkflowRevisionRef,
    safe_summary: impl Into<String>,
) -> Result<RevisionProposal, OrchestrationDiagnostic> {
    let mut proposal = RevisionProposal {
        id: super::identity::ProposalId::from_generated("0".repeat(blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE))),
        instance_id: instance.id.clone(),
        base_definition_revision: instance.definition_revision.clone(),
        base_epoch: instance.epoch,
        base_workflow_revision: instance.workflow_revision.clone(),
        successor,
        safe_summary: safe_summary.into(),
    };
    proposal.id = proposal_identity(&proposal)?;
    Ok(proposal)
}

pub fn make_approval(
    instance: &OrchestrationInstance,
    proposal: &RevisionProposal,
    actor: impl Into<String>,
    approved: bool,
) -> Result<ApprovalDecision, OrchestrationDiagnostic> {
    let mut approval = ApprovalDecision {
        id: super::identity::ApprovalId::from_generated("0".repeat(blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE))),
        instance_id: instance.id.clone(),
        epoch: instance.epoch,
        base_workflow_revision: instance.workflow_revision.clone(),
        proposal_id: proposal.id.clone(),
        authority_policy_hash: authority_policy_hash(&instance.definition.authority)?,
        actor: actor.into(),
        approved,
    };
    approval.id = approval_identity(&approval)?;
    Ok(approval)
}

pub fn dispatch_identity(
    instance: &OrchestrationInstance,
    run_id: RunId,
) -> Result<super::identity::DispatchId, OrchestrationDiagnostic> {
    Ok(super::identity::DispatchId::from_generated(hash_serializable(
        INSTANCE_HASH_LABEL,
        &(
            instance.id.as_str(),
            instance.epoch,
            instance.round.saturating_add(ROUND_INCREMENT),
            instance.workflow_revision.content_hash.as_str(),
            run_id,
        ),
    )?))
}

pub fn is_terminal_status(status: OrchestrationStatus) -> bool {
    matches!(
        status,
        OrchestrationStatus::Completed
            | OrchestrationStatus::Failed
            | OrchestrationStatus::Cancelled
            | OrchestrationStatus::Exhausted
    )
}

pub fn render_status_text(instance: &OrchestrationInstance) -> String {
    debug_assert!(!instance.id.as_str().is_empty());
    debug_assert!(!instance.workflow_revision.content_hash.as_str().is_empty());
    let active_run = instance.active_run_id.map_or_else(|| "-".to_string(), |run_id| run_id.to_string());
    let proposal = instance.pending_proposal.as_ref().map_or("-", |proposal| proposal.id.as_str());
    let event_sequence = instance.last_event_sequence.map_or_else(|| "-".to_string(), |sequence| sequence.to_string());
    let event_head = instance.event_head.as_ref().map_or("-", super::identity::LedgerHeadId::as_str);
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
        LOCAL_CHAIN_EVIDENCE_SCOPE,
        REMOTE_EFFECT_EVIDENCE_SCOPE,
    )
}

fn next_action(instance: &OrchestrationInstance) -> &'static str {
    match instance.status {
        OrchestrationStatus::Ready => "start_round",
        OrchestrationStatus::Running if instance.cancellation_requested => "observe_cancellation",
        OrchestrationStatus::Running => "wait_for_run",
        OrchestrationStatus::WaitingForApproval if instance.pending_proposal.is_some() => "approve_or_reject",
        OrchestrationStatus::WaitingForApproval => "submit_proposal",
        OrchestrationStatus::Paused => "resume_or_cancel",
        OrchestrationStatus::Failed if instance.retry.retryable_failure.is_some() => "retry",
        OrchestrationStatus::Completed
        | OrchestrationStatus::Failed
        | OrchestrationStatus::Cancelled
        | OrchestrationStatus::Exhausted => "none",
    }
}

fn hash_serializable<T: Serialize + ?Sized>(label: &[u8], value: &T) -> Result<String, OrchestrationDiagnostic> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        diagnostic(
            OrchestrationDiagnosticCode::SerializationFailure,
            format!("failed to serialize orchestration identity input: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(ORCHESTRATION_HASH_DOMAIN);
    hasher.update(label);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

fn identity_error(error: super::identity::AdmissionError) -> OrchestrationDiagnostic {
    diagnostic(OrchestrationDiagnosticCode::InvalidIdentity, error.to_string())
}

fn invalid_transition(message: impl Into<String>) -> OrchestrationDiagnostic {
    diagnostic(OrchestrationDiagnosticCode::InvalidLifecycleTransition, message)
}

fn diagnostic(code: OrchestrationDiagnosticCode, message: impl Into<String>) -> OrchestrationDiagnostic {
    OrchestrationDiagnostic {
        code,
        message: message.into(),
    }
}
