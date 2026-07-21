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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRevisionRef {
    pub workflow_name: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionPolicy {
    RunSucceeded,
    AwaitProposal,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetDimensions {
    pub rounds: bool,
    pub agent_invocations: bool,
    pub tokens: bool,
    pub monetary_microunits: bool,
    pub duration_millis: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetCharge {
    pub rounds: u64,
    pub agent_invocations: u64,
    pub tokens: u64,
    pub monetary_microunits: u64,
    pub duration_millis: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetChargePlan {
    pub rounds: Option<u64>,
    pub agent_invocations: Option<u64>,
    pub tokens: Option<u64>,
    pub monetary_microunits: Option<u64>,
    pub duration_millis: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetLimits {
    pub rounds: u64,
    pub agent_invocations: u64,
    pub tokens: u64,
    pub monetary_microunits: u64,
    pub duration_millis: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetPolicy {
    pub enforced: BudgetDimensions,
    pub limits: BudgetLimits,
    pub per_round_upper_bound: BudgetCharge,
}

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
    pub trigger_id: String,
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
    pub id: String,
    pub objective: String,
    pub workflow: WorkflowRevisionRef,
    pub completion: CompletionPolicy,
    pub budgets: BudgetPolicy,
    pub authority: AuthorityPolicy,
    pub triggers: Vec<OrchestrationTriggerBinding>,
    pub redaction: RedactionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetReservation {
    pub reservation_id: String,
    pub charge: BudgetCharge,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetLedger {
    pub policy: BudgetPolicy,
    pub consumed: BudgetCharge,
    pub reservations: BTreeMap<String, BudgetReservation>,
    pub breached: bool,
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
    pub id: String,
    pub run_id: RunId,
    pub workflow_revision: WorkflowRevisionRef,
    pub status: DispatchStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionProposal {
    pub id: String,
    pub instance_id: String,
    pub base_definition_revision: String,
    pub base_epoch: u64,
    pub base_workflow_revision: WorkflowRevisionRef,
    pub successor: WorkflowRevisionRef,
    pub safe_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub id: String,
    pub instance_id: String,
    pub epoch: u64,
    pub base_workflow_revision: WorkflowRevisionRef,
    pub proposal_id: String,
    pub authority_policy_hash: String,
    pub actor: String,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationInstance {
    pub id: String,
    pub sequence: u64,
    pub trigger_idempotency_key: Option<String>,
    pub definition_revision: String,
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
    pub accepted_fact_ids: BTreeSet<String>,
    pub last_event_sequence: Option<u64>,
    pub event_head: Option<String>,
    pub terminal_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationFact {
    pub id: String,
    pub action: OrchestrationAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OrchestrationAction {
    StartRound {
        intent: DispatchIntent,
        planned_charge: BudgetChargePlan,
    },
    DispatchAcknowledged {
        dispatch_id: String,
    },
    RunFinished {
        dispatch_id: String,
        run_id: RunId,
        succeeded: bool,
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
        proposal_id: String,
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
    BudgetExhausted {
        safe_reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationEventKind {
    RoundStarted,
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
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationEvent {
    pub instance_id: String,
    pub sequence: u64,
    pub prior_hash: Option<String>,
    pub event_hash: String,
    pub state_hash: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub valid: bool,
    pub instance: OrchestrationInstance,
    pub diagnostics: Vec<OrchestrationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationTriggerContext {
    pub accepted: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationTriggerPlan {
    pub scoped_key: String,
    pub instance_id: String,
    pub definition_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestrationTriggerAdmission {
    Accepted(OrchestrationTriggerPlan),
    Duplicate { instance_id: String },
}

pub fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

// r[impl lattice.agent_orchestration.model]
pub fn workflow_revision(workflow: &Workflow) -> Result<WorkflowRevisionRef, OrchestrationDiagnostic> {
    let content_hash = hash_serializable(WORKFLOW_HASH_LABEL, workflow)?;
    Ok(WorkflowRevisionRef {
        workflow_name: workflow.meta.name.clone(),
        content_hash,
    })
}

// r[impl lattice.agent_orchestration.model]
pub fn definition_revision(definition: &OrchestrationDefinition) -> Result<String, OrchestrationDiagnostic> {
    hash_serializable(DEFINITION_HASH_LABEL, definition)
}

pub fn authority_policy_hash(policy: &AuthorityPolicy) -> Result<String, OrchestrationDiagnostic> {
    hash_serializable(AUTHORITY_HASH_LABEL, policy)
}

pub fn proposal_identity(proposal: &RevisionProposal) -> Result<String, OrchestrationDiagnostic> {
    let mut unhashed = proposal.clone();
    unhashed.id.clear();
    hash_serializable(PROPOSAL_HASH_LABEL, &unhashed)
}

pub fn approval_identity(approval: &ApprovalDecision) -> Result<String, OrchestrationDiagnostic> {
    let mut unhashed = approval.clone();
    unhashed.id.clear();
    hash_serializable(APPROVAL_HASH_LABEL, &unhashed)
}

pub fn fact_identity(
    instance_id: &str,
    version: u64,
    action: &OrchestrationAction,
) -> Result<String, OrchestrationDiagnostic> {
    hash_serializable(FACT_HASH_LABEL, &(instance_id, version, action))
}

// r[impl lattice.agent_orchestration.model]
pub fn validate_definition(
    definition: &OrchestrationDefinition,
    actual_workflow_revision: Option<&WorkflowRevisionRef>,
) -> Vec<OrchestrationDiagnostic> {
    let mut diagnostics = Vec::new();
    if definition.schema_version != CURRENT_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::InvalidSchemaVersion,
            format!(
                "orchestration schema version {} is unsupported; expected {CURRENT_SCHEMA_VERSION}",
                definition.schema_version
            ),
        ));
    }
    if definition.id.trim().is_empty() {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::EmptyDefinitionId,
            "orchestration definition id cannot be empty",
        ));
    }
    if definition.objective.trim().is_empty() {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::EmptyObjective,
            "orchestration objective cannot be empty",
        ));
    }
    validate_workflow_ref(&definition.workflow, &mut diagnostics);
    validate_actual_workflow_revision(definition, actual_workflow_revision, &mut diagnostics);
    validate_authority_policy(&definition.authority, &mut diagnostics);
    validate_trigger_bindings(&definition.triggers, &mut diagnostics);
    validate_budget_policy(&definition.budgets, &mut diagnostics);
    diagnostics
}

fn validate_workflow_ref(reference: &WorkflowRevisionRef, diagnostics: &mut Vec<OrchestrationDiagnostic>) {
    if reference.workflow_name.trim().is_empty() {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::EmptyWorkflowName,
            "workflow revision name cannot be empty",
        ));
    }
    if !is_blake3_hex(&reference.content_hash) {
        diagnostics.push(diagnostic(
            OrchestrationDiagnosticCode::InvalidContentHash,
            "workflow revision content hash must be a BLAKE3 hex digest",
        ));
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

fn validate_trigger_bindings(
    bindings: &[OrchestrationTriggerBinding],
    diagnostics: &mut Vec<OrchestrationDiagnostic>,
) {
    let mut ids = BTreeSet::new();
    for binding in bindings {
        if binding.trigger_id.trim().is_empty() {
            diagnostics.push(diagnostic(
                OrchestrationDiagnosticCode::EmptyTriggerId,
                "orchestration trigger id cannot be empty",
            ));
        }
        if !ids.insert(binding.trigger_id.clone()) {
            diagnostics.push(diagnostic(
                OrchestrationDiagnosticCode::DuplicateTriggerId,
                format!("orchestration trigger '{}' is declared more than once", binding.trigger_id),
            ));
        }
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
    trigger_idempotency_key: Option<String>,
) -> Result<OrchestrationInstance, OrchestrationDiagnostic> {
    let definition_revision = definition_revision(&definition)?;
    let id = hash_serializable(
        INSTANCE_HASH_LABEL,
        &(definition_revision.as_str(), sequence, trigger_idempotency_key.as_deref()),
    )?;
    let workflow_revision = definition.workflow.clone();
    let policy = definition.budgets;
    Ok(OrchestrationInstance {
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
        accepted_fact_ids: BTreeSet::new(),
        last_event_sequence: None,
        event_head: None,
        terminal_summary: None,
    })
}

// r[impl lattice.agent_orchestration.core]
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
    if fact.id.trim().is_empty() {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::InvalidIdentity,
            "orchestration fact id cannot be empty",
        ));
    }
    reject_terminal_mutation(instance)?;
    let mut next = instance.clone();
    let kind = apply_action(&mut next, &fact.action)?;
    let inserted = next.accepted_fact_ids.insert(fact.id.clone());
    debug_assert!(inserted);
    next.version = next.version.saturating_add(INSTANCE_VERSION_INCREMENT);
    let event = build_event(instance, &next, kind, fact)?;
    next.last_event_sequence = Some(event.sequence);
    next.event_head = Some(event.event_hash.clone());
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

fn apply_action(
    instance: &mut OrchestrationInstance,
    action: &OrchestrationAction,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    match action {
        OrchestrationAction::StartRound {
            intent,
            planned_charge,
        } => start_round(instance, intent, *planned_charge),
        OrchestrationAction::DispatchAcknowledged { dispatch_id } => acknowledge_dispatch(instance, dispatch_id),
        OrchestrationAction::RunFinished {
            dispatch_id,
            run_id,
            succeeded,
            observed_usage,
            safe_summary,
        } => finish_run(instance, dispatch_id, *run_id, *succeeded, *observed_usage, safe_summary),
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
    if intent.workflow_revision != instance.workflow_revision {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::WorkflowRevisionMismatch,
            "dispatch intent workflow revision does not match the active orchestration revision",
        ));
    }
    reserve_budget(&mut instance.budgets, &intent.id, planned_charge)?;
    instance.round = instance.round.saturating_add(ROUND_INCREMENT);
    instance.active_run_id = Some(intent.run_id);
    instance.pending_dispatch = Some(intent.clone());
    instance.status = OrchestrationStatus::Running;
    instance.cancellation_requested = false;
    Ok(OrchestrationEventKind::RoundStarted)
}

fn acknowledge_dispatch(
    instance: &mut OrchestrationInstance,
    dispatch_id: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    let dispatch = matching_dispatch_mut(instance, dispatch_id)?;
    dispatch.status = DispatchStatus::Acknowledged;
    Ok(OrchestrationEventKind::DispatchAcknowledged)
}

fn finish_run(
    instance: &mut OrchestrationInstance,
    dispatch_id: &str,
    run_id: RunId,
    succeeded: bool,
    observed_usage: BudgetCharge,
    safe_summary: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    let dispatch = matching_dispatch(instance, dispatch_id)?;
    if dispatch.run_id != run_id || instance.active_run_id != Some(run_id) {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "run completion does not match the active dispatch identity",
        ));
    }
    let settlement = settle_budget(&mut instance.budgets, dispatch_id, observed_usage)?;
    instance.active_run_id = None;
    instance.pending_dispatch = None;
    instance.terminal_summary = instance
        .definition
        .redaction
        .retain_safe_summaries
        .then(|| safe_summary.to_string());
    let kind = if succeeded {
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
    if !succeeded {
        instance.status = OrchestrationStatus::Failed;
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
        diagnostic(
            OrchestrationDiagnosticCode::ProposalMismatch,
            "no proposal is pending for approval",
        )
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
    proposal_id: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    authorize_actor(instance, actor)?;
    if instance.status != OrchestrationStatus::WaitingForApproval {
        return Err(invalid_transition("proposal rejection requires waiting-for-approval state"));
    }
    let proposal = instance.pending_proposal.as_ref().ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::ProposalMismatch,
            "no proposal is pending for rejection",
        )
    })?;
    if proposal.id != proposal_id {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::ProposalMismatch,
            "proposal rejection identity does not match the pending proposal",
        ));
    }
    instance.pending_proposal = None;
    Ok(OrchestrationEventKind::ProposalRejected)
}

fn pause(
    instance: &mut OrchestrationInstance,
    actor: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
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
        return Err(invalid_transition(
            "cancellation observation does not match an active cancellation request",
        ));
    }
    if cancelled {
        instance.status = OrchestrationStatus::Cancelled;
        instance.active_run_id = None;
        instance.pending_dispatch = None;
    }
    Ok(OrchestrationEventKind::CancellationObserved)
}

fn exhaust_budget(
    instance: &mut OrchestrationInstance,
    safe_reason: &str,
) -> Result<OrchestrationEventKind, OrchestrationDiagnostic> {
    if instance.status == OrchestrationStatus::Running {
        return Err(invalid_transition("budget exhaustion cannot erase an active runner observation"));
    }
    instance.status = OrchestrationStatus::Exhausted;
    instance.terminal_summary = instance
        .definition
        .redaction
        .retain_safe_summaries
        .then(|| safe_reason.to_string());
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
    dispatch_id: &str,
) -> Result<&'a DispatchIntent, OrchestrationDiagnostic> {
    instance.pending_dispatch.as_ref().filter(|intent| intent.id == dispatch_id).ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::DispatchMismatch,
            "dispatch identity does not match the pending orchestration dispatch",
        )
    })
}

fn matching_dispatch_mut<'a>(
    instance: &'a mut OrchestrationInstance,
    dispatch_id: &str,
) -> Result<&'a mut DispatchIntent, OrchestrationDiagnostic> {
    instance
        .pending_dispatch
        .as_mut()
        .filter(|intent| intent.id == dispatch_id)
        .ok_or_else(|| {
            diagnostic(
                OrchestrationDiagnosticCode::DispatchMismatch,
                "dispatch identity does not match the pending orchestration dispatch",
            )
        })
}

// r[impl lattice.agent_orchestration.budgets]
pub fn reserve_budget(
    ledger: &mut BudgetLedger,
    reservation_id: &str,
    plan: BudgetChargePlan,
) -> Result<BudgetCharge, OrchestrationDiagnostic> {
    if ledger.reservations.contains_key(reservation_id) {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::DuplicateReservation,
            "budget reservation identity was already admitted",
        ));
    }
    let charge = admitted_charge(&ledger.policy, plan)?;
    let reserved = total_reserved(&ledger.reservations)?;
    let committed = checked_add_charge(ledger.consumed, reserved)?;
    let requested = checked_add_charge(committed, charge)?;
    ensure_within_limits(&ledger.policy, requested)?;
    ledger.reservations.insert(
        reservation_id.to_string(),
        BudgetReservation {
            reservation_id: reservation_id.to_string(),
            charge,
        },
    );
    Ok(charge)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetSettlement {
    pub breached: bool,
}

pub fn settle_budget(
    ledger: &mut BudgetLedger,
    reservation_id: &str,
    observed: BudgetCharge,
) -> Result<BudgetSettlement, OrchestrationDiagnostic> {
    let reservation = ledger.reservations.remove(reservation_id).ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::MissingReservation,
            "budget settlement does not match a pending reservation",
        )
    })?;
    let breached = charge_exceeds(observed, reservation.charge);
    ledger.consumed = checked_add_charge(ledger.consumed, observed)?;
    if breached || ensure_within_limits(&ledger.policy, ledger.consumed).is_err() {
        ledger.breached = true;
    }
    Ok(BudgetSettlement {
        breached: ledger.breached,
    })
}

fn admitted_charge(
    policy: &BudgetPolicy,
    plan: BudgetChargePlan,
) -> Result<BudgetCharge, OrchestrationDiagnostic> {
    Ok(BudgetCharge {
        rounds: admitted_dimension(policy.enforced.rounds, plan.rounds, "rounds")?,
        agent_invocations: admitted_dimension(
            policy.enforced.agent_invocations,
            plan.agent_invocations,
            "agent invocations",
        )?,
        tokens: admitted_dimension(policy.enforced.tokens, plan.tokens, "tokens")?,
        monetary_microunits: admitted_dimension(
            policy.enforced.monetary_microunits,
            plan.monetary_microunits,
            "monetary microunits",
        )?,
        duration_millis: admitted_dimension(
            policy.enforced.duration_millis,
            plan.duration_millis,
            "duration milliseconds",
        )?,
    })
}

fn admitted_dimension(
    enforced: bool,
    planned: Option<u64>,
    label: &str,
) -> Result<u64, OrchestrationDiagnostic> {
    if enforced {
        planned.ok_or_else(|| {
            diagnostic(
                OrchestrationDiagnosticCode::MissingBudgetBound,
                format!("enforced {label} budget has no admitted upper bound"),
            )
        })
    } else {
        Ok(planned.unwrap_or_default())
    }
}

fn total_reserved(
    reservations: &BTreeMap<String, BudgetReservation>,
) -> Result<BudgetCharge, OrchestrationDiagnostic> {
    reservations
        .values()
        .try_fold(BudgetCharge::default(), |total, reservation| checked_add_charge(total, reservation.charge))
}

fn checked_add_charge(
    left: BudgetCharge,
    right: BudgetCharge,
) -> Result<BudgetCharge, OrchestrationDiagnostic> {
    Ok(BudgetCharge {
        rounds: checked_add_dimension(left.rounds, right.rounds, "rounds")?,
        agent_invocations: checked_add_dimension(left.agent_invocations, right.agent_invocations, "agent invocations")?,
        tokens: checked_add_dimension(left.tokens, right.tokens, "tokens")?,
        monetary_microunits: checked_add_dimension(
            left.monetary_microunits,
            right.monetary_microunits,
            "monetary microunits",
        )?,
        duration_millis: checked_add_dimension(left.duration_millis, right.duration_millis, "duration milliseconds")?,
    })
}

fn checked_add_dimension(left: u64, right: u64, label: &str) -> Result<u64, OrchestrationDiagnostic> {
    left.checked_add(right).ok_or_else(|| {
        diagnostic(
            OrchestrationDiagnosticCode::BudgetOverflow,
            format!("{label} budget accounting overflowed"),
        )
    })
}

fn ensure_within_limits(policy: &BudgetPolicy, charge: BudgetCharge) -> Result<(), OrchestrationDiagnostic> {
    ensure_dimension(policy.enforced.rounds, charge.rounds, policy.limits.rounds, "rounds")?;
    ensure_dimension(
        policy.enforced.agent_invocations,
        charge.agent_invocations,
        policy.limits.agent_invocations,
        "agent invocations",
    )?;
    ensure_dimension(policy.enforced.tokens, charge.tokens, policy.limits.tokens, "tokens")?;
    ensure_dimension(
        policy.enforced.monetary_microunits,
        charge.monetary_microunits,
        policy.limits.monetary_microunits,
        "monetary microunits",
    )?;
    ensure_dimension(
        policy.enforced.duration_millis,
        charge.duration_millis,
        policy.limits.duration_millis,
        "duration milliseconds",
    )
}

fn ensure_dimension(enforced: bool, value: u64, limit: u64, label: &str) -> Result<(), OrchestrationDiagnostic> {
    if enforced && value > limit {
        Err(diagnostic(
            OrchestrationDiagnosticCode::BudgetExceeded,
            format!("{label} budget is exhausted"),
        ))
    } else {
        Ok(())
    }
}

fn charge_exceeds(observed: BudgetCharge, reserved: BudgetCharge) -> bool {
    observed.rounds > reserved.rounds
        || observed.agent_invocations > reserved.agent_invocations
        || observed.tokens > reserved.tokens
        || observed.monetary_microunits > reserved.monetary_microunits
        || observed.duration_millis > reserved.duration_millis
}

fn build_event(
    prior: &OrchestrationInstance,
    next: &OrchestrationInstance,
    kind: OrchestrationEventKind,
    fact: OrchestrationFact,
) -> Result<OrchestrationEvent, OrchestrationDiagnostic> {
    let sequence = prior
        .last_event_sequence
        .map(|sequence| sequence.saturating_add(EVENT_SEQUENCE_INCREMENT))
        .unwrap_or(FIRST_EVENT_SEQUENCE);
    let state_hash = semantic_state_hash(next)?;
    let mut event = OrchestrationEvent {
        instance_id: next.id.clone(),
        sequence,
        prior_hash: prior.event_head.clone(),
        event_hash: String::new(),
        state_hash,
        kind,
        fact,
    };
    event.event_hash = event_identity(&event)?;
    Ok(event)
}

fn event_identity(event: &OrchestrationEvent) -> Result<String, OrchestrationDiagnostic> {
    let mut unhashed = event.clone();
    unhashed.event_hash.clear();
    hash_serializable(EVENT_HASH_LABEL, &unhashed)
}

fn semantic_state_hash(instance: &OrchestrationInstance) -> Result<String, OrchestrationDiagnostic> {
    let mut state = instance.clone();
    state.last_event_sequence = None;
    state.event_head = None;
    hash_serializable(STATE_HASH_LABEL, &state)
}

// r[impl lattice.agent_orchestration.events]
pub fn replay_events(
    initial: &OrchestrationInstance,
    events: &[OrchestrationEvent],
    expected_head: Option<&str>,
) -> ReplayReport {
    let mut instance = initial.clone();
    let mut diagnostics = Vec::new();
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
                        format!("orchestration event sequence {} does not match deterministic replay", recorded.sequence),
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
    if diagnostics.is_empty() && instance.event_head.as_deref() != expected_head {
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
    definition: &OrchestrationDefinition,
    definition_revision: &str,
    trigger_id: &str,
    idempotency_key: &str,
    next_sequence: u64,
    context: &OrchestrationTriggerContext,
) -> Result<OrchestrationTriggerAdmission, OrchestrationDiagnostic> {
    if !definition.triggers.iter().any(|binding| binding.trigger_id == trigger_id) {
        return Err(diagnostic(
            OrchestrationDiagnosticCode::TriggerNotBound,
            format!("trigger '{trigger_id}' is not bound to orchestration '{}'", definition.id),
        ));
    }
    let scoped_key = scoped_trigger_key(definition_revision, trigger_id, idempotency_key)?;
    if let Some(instance_id) = context.accepted.get(&scoped_key) {
        return Ok(OrchestrationTriggerAdmission::Duplicate {
            instance_id: instance_id.clone(),
        });
    }
    let instance_id = hash_serializable(
        INSTANCE_HASH_LABEL,
        &(definition_revision, next_sequence, Some(scoped_key.as_str())),
    )?;
    Ok(OrchestrationTriggerAdmission::Accepted(OrchestrationTriggerPlan {
        scoped_key,
        instance_id,
        definition_revision: definition_revision.to_string(),
    }))
}

pub fn scoped_trigger_key(
    definition_revision: &str,
    trigger_id: &str,
    idempotency_key: &str,
) -> Result<String, OrchestrationDiagnostic> {
    hash_serializable(TRIGGER_HASH_LABEL, &(definition_revision, trigger_id, idempotency_key))
}

pub fn make_proposal(
    instance: &OrchestrationInstance,
    successor: WorkflowRevisionRef,
    safe_summary: impl Into<String>,
) -> Result<RevisionProposal, OrchestrationDiagnostic> {
    let mut proposal = RevisionProposal {
        id: String::new(),
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
        id: String::new(),
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
) -> Result<String, OrchestrationDiagnostic> {
    hash_serializable(
        INSTANCE_HASH_LABEL,
        &(
            instance.id.as_str(),
            instance.epoch,
            instance.round.saturating_add(ROUND_INCREMENT),
            instance.workflow_revision.content_hash.as_str(),
            run_id,
        ),
    )
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
    let active_run = instance.active_run_id.map_or_else(|| "-".to_string(), |run_id| run_id.to_string());
    let proposal = instance.pending_proposal.as_ref().map_or("-", |proposal| proposal.id.as_str());
    format!(
        "{} | {:?} | revision {} | round {} | run {} | proposal {} | next {}",
        instance.id,
        instance.status,
        instance.workflow_revision.content_hash,
        instance.round,
        active_run,
        proposal,
        next_action(instance),
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
        OrchestrationStatus::Completed
        | OrchestrationStatus::Failed
        | OrchestrationStatus::Cancelled
        | OrchestrationStatus::Exhausted => "none",
    }
}

fn hash_serializable<T: Serialize + ?Sized>(
    label: &[u8],
    value: &T,
) -> Result<String, OrchestrationDiagnostic> {
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

fn is_blake3_hex(value: &str) -> bool {
    value.len() == blake3::OUT_LEN * HEX_CHARS_PER_BYTE && value.bytes().all(|byte| byte.is_ascii_hexdigit())
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
