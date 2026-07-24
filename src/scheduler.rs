//! Pure scheduling core and scheduler-daemon outcome models.

use std::collections::HashSet;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::RunId;
use crate::headless::HeadlessRunSummary;

const MILLIS_PER_SECOND: i64 = 1_000;
const MIN_INTERVAL_SECS: u64 = 1;
const SCHEDULE_OCCURRENCE_DOMAIN: &[u8] = b"lattice-schedule-occurrence-v1";
const SCHEDULE_OCCURRENCE_ID_TAG: &[u8] = b"schedule-id";
const SCHEDULE_OCCURRENCE_DUE_TAG: &[u8] = b"due-epoch-ms";
pub const MAX_SCHEDULE_OCCURRENCE_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleOccurrenceStatus {
    PendingUnobserved,
    Succeeded,
    Failed,
    AbandonedUnknown,
}

impl ScheduleOccurrenceStatus {
    #[must_use]
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::PendingUnobserved)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleOccurrence {
    pub identity: String,
    pub schedule_id: String,
    pub workflow_name: String,
    pub workflow_revision: String,
    pub due_epoch_ms: i64,
    pub next_run_epoch_ms: i64,
    pub attempt_run_ids: Vec<RunId>,
    pub active_run_id: RunId,
    pub status: ScheduleOccurrenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleOccurrenceClaimRequest {
    pub identity: String,
    pub schedule_id: String,
    pub workflow_name: String,
    pub workflow_revision: String,
    pub due_epoch_ms: i64,
    pub next_run_epoch_ms: i64,
}

impl ScheduleOccurrenceClaimRequest {
    #[must_use]
    pub fn occurrence(&self, run_id: RunId) -> ScheduleOccurrence {
        ScheduleOccurrence {
            identity: self.identity.clone(),
            schedule_id: self.schedule_id.clone(),
            workflow_name: self.workflow_name.clone(),
            workflow_revision: self.workflow_revision.clone(),
            due_epoch_ms: self.due_epoch_ms,
            next_run_epoch_ms: self.next_run_epoch_ms,
            attempt_run_ids: vec![run_id],
            active_run_id: run_id,
            status: ScheduleOccurrenceStatus::PendingUnobserved,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleOccurrenceClaimResolution {
    Inserted(Box<ScheduleOccurrence>),
    Existing(Box<ScheduleOccurrence>),
    StaleSchedule { current_next_run_epoch_ms: i64 },
    MissingSchedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOccurrenceError {
    EmptyScheduleId,
    EmptyWorkflowName,
    EmptyWorkflowRevision,
    InvalidNextRun,
    IdentityMismatch,
    MissingAttempt,
    TooManyAttempts,
    ActiveRunMismatch,
    NotPending,
    InvalidSettlement,
    WorkflowRevisionMismatch,
    RetryLimitExhausted,
    RetryAcknowledgementRequired,
    AbandonAcknowledgementRequired,
    RecoveryOwnerMismatch,
    RecoveryRunMismatch,
    RecoveryRunNotTerminal,
}

impl fmt::Display for ScheduleOccurrenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyScheduleId => "schedule occurrence requires a non-empty schedule id",
            Self::EmptyWorkflowName => "schedule occurrence requires a non-empty workflow name",
            Self::EmptyWorkflowRevision => "schedule occurrence requires a non-empty workflow revision",
            Self::InvalidNextRun => "schedule occurrence next run must be after its due epoch",
            Self::IdentityMismatch => "schedule occurrence identity does not match its canonical inputs",
            Self::MissingAttempt => "schedule occurrence has no attempt",
            Self::TooManyAttempts => "schedule occurrence exceeds the bounded attempt limit",
            Self::ActiveRunMismatch => "schedule occurrence active run is not its latest attempt",
            Self::NotPending => "schedule occurrence is not pending",
            Self::InvalidSettlement => "schedule occurrence settlement must be succeeded or failed",
            Self::WorkflowRevisionMismatch => "saved workflow revision differs from the claimed occurrence",
            Self::RetryLimitExhausted => "schedule occurrence explicit retry is exhausted",
            Self::RetryAcknowledgementRequired => "schedule occurrence retry requires duplicate-risk acknowledgement",
            Self::AbandonAcknowledgementRequired => {
                "schedule occurrence abandonment requires unknown-outcome acknowledgement"
            }
            Self::RecoveryOwnerMismatch => "persisted run is not owned by the schedule occurrence",
            Self::RecoveryRunMismatch => "persisted run does not match the active occurrence attempt",
            Self::RecoveryRunNotTerminal => "persisted run lacks terminal evidence",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ScheduleOccurrenceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOccurrenceRunOwnerObservation {
    Exact,
    Foreign,
    Unscoped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleOccurrenceRunObservation {
    pub run_id: RunId,
    pub owner: ScheduleOccurrenceRunOwnerObservation,
    pub finished: bool,
    pub failures: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOccurrenceRecoveryDecision {
    PendingUnobserved {
        run_id: RunId,
    },
    Settle {
        run_id: RunId,
        status: ScheduleOccurrenceStatus,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleDefinition {
    pub id: String,
    pub workflow_name: String,
    pub recurrence: RecurrenceRule,
    pub enabled: bool,
    pub overlap_policy: OverlapPolicy,
    pub missed_run_policy: MissedRunPolicy,
    pub next_run_epoch_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceRule {
    EverySeconds { interval_secs: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlapPolicy {
    Forbid,
    Allow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissedRunPolicy {
    RunOnce,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleDiagnostic {
    pub schedule_id: String,
    pub code: ScheduleDiagnosticCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDiagnosticCode {
    EmptyScheduleId,
    EmptyWorkflowName,
    MissingWorkflow,
    InvalidRecurrence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueRunPlan {
    pub schedule_id: String,
    pub workflow_name: String,
    pub due_epoch_ms: i64,
    pub next_run_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedScheduleRun {
    pub schedule_id: String,
    pub workflow_name: String,
    pub reason: ScheduleSkipReason,
    pub next_run_epoch_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleSkipReason {
    Disabled,
    OverlapPrevented,
    MissedRunSkipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DueRunEvaluation {
    pub due_runs: Vec<DueRunPlan>,
    pub skipped_runs: Vec<SkippedScheduleRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleDaemonOutcome {
    Started {
        schedule_id: String,
        workflow_name: String,
        summary: Box<HeadlessRunSummary>,
        next_run_epoch_ms: i64,
    },
    Error {
        schedule_id: String,
        workflow_name: String,
        message: String,
        next_run_epoch_ms: i64,
    },
    Skipped(SkippedScheduleRun),
}

impl fmt::Display for ScheduleDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "schedule {}: {}", self.schedule_id, self.message)
    }
}

// r[impl n8nclone.scheduling.schedule_model]
pub fn validate_schedule(schedule: &ScheduleDefinition, workflow_names: &HashSet<String>) -> Vec<ScheduleDiagnostic> {
    let mut diagnostics = Vec::new();
    if schedule.id.trim().is_empty() {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::EmptyScheduleId,
            message: "schedule id cannot be empty".to_string(),
        });
    }
    if schedule.workflow_name.trim().is_empty() {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::EmptyWorkflowName,
            message: "workflow name cannot be empty".to_string(),
        });
    } else if !workflow_names.contains(&schedule.workflow_name) {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::MissingWorkflow,
            message: format!("workflow '{}' does not exist", schedule.workflow_name),
        });
    }
    if schedule.recurrence.interval_secs() < MIN_INTERVAL_SECS {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::InvalidRecurrence,
            message: "schedule interval must be at least one second".to_string(),
        });
    }
    diagnostics
}

// r[impl n8nclone.scheduling.due_evaluator]
pub fn evaluate_due_runs(
    schedules: &[ScheduleDefinition],
    current_epoch_ms: i64,
    running_schedule_ids: &HashSet<String>,
) -> DueRunEvaluation {
    let mut evaluation = DueRunEvaluation::default();
    for schedule in schedules {
        if !schedule.enabled {
            evaluation
                .skipped_runs
                .push(skipped(schedule, ScheduleSkipReason::Disabled, schedule.next_run_epoch_ms));
            continue;
        }
        if schedule.next_run_epoch_ms > current_epoch_ms {
            continue;
        }
        if schedule.overlap_policy == OverlapPolicy::Forbid && running_schedule_ids.contains(&schedule.id) {
            evaluation.skipped_runs.push(skipped(
                schedule,
                ScheduleSkipReason::OverlapPrevented,
                schedule.next_run_epoch_ms,
            ));
            continue;
        }
        if schedule.missed_run_policy == MissedRunPolicy::Skip
            && missed_by_at_least_one_interval(schedule, current_epoch_ms)
        {
            evaluation.skipped_runs.push(skipped(
                schedule,
                ScheduleSkipReason::MissedRunSkipped,
                next_after(schedule, current_epoch_ms),
            ));
            continue;
        }
        evaluation.due_runs.push(DueRunPlan {
            schedule_id: schedule.id.clone(),
            workflow_name: schedule.workflow_name.clone(),
            due_epoch_ms: schedule.next_run_epoch_ms,
            next_run_epoch_ms: next_after(schedule, current_epoch_ms),
        });
    }
    evaluation
}

impl RecurrenceRule {
    pub fn interval_secs(self) -> u64 {
        match self {
            RecurrenceRule::EverySeconds { interval_secs } => interval_secs,
        }
    }

    fn interval_ms(self) -> i64 {
        let seconds = i64::try_from(self.interval_secs()).expect("schedule interval seconds fit in i64");
        seconds.saturating_mul(MILLIS_PER_SECOND)
    }
}

fn skipped(schedule: &ScheduleDefinition, reason: ScheduleSkipReason, next_run_epoch_ms: i64) -> SkippedScheduleRun {
    SkippedScheduleRun {
        schedule_id: schedule.id.clone(),
        workflow_name: schedule.workflow_name.clone(),
        reason,
        next_run_epoch_ms,
    }
}

fn missed_by_at_least_one_interval(schedule: &ScheduleDefinition, current_epoch_ms: i64) -> bool {
    current_epoch_ms.saturating_sub(schedule.next_run_epoch_ms) >= schedule.recurrence.interval_ms()
}

fn next_after(schedule: &ScheduleDefinition, current_epoch_ms: i64) -> i64 {
    let interval_ms = schedule.recurrence.interval_ms();
    let mut next = schedule.next_run_epoch_ms;
    while next <= current_epoch_ms {
        next = next.saturating_add(interval_ms);
    }
    next
}

pub trait ScheduledWorkflowRunner {
    fn run_scheduled_workflow(&mut self, workflow_name: &str) -> Result<HeadlessRunSummary, String>;
}

// r[impl n8nclone.scheduling.daemon.run_due]
pub fn run_due_plans<R: ScheduledWorkflowRunner>(plans: &[DueRunPlan], runner: &mut R) -> Vec<ScheduleDaemonOutcome> {
    plans
        .iter()
        .map(|plan| match runner.run_scheduled_workflow(&plan.workflow_name) {
            Ok(summary) => ScheduleDaemonOutcome::Started {
                schedule_id: plan.schedule_id.clone(),
                workflow_name: plan.workflow_name.clone(),
                summary: Box::new(summary),
                next_run_epoch_ms: plan.next_run_epoch_ms,
            },
            Err(message) => ScheduleDaemonOutcome::Error {
                schedule_id: plan.schedule_id.clone(),
                workflow_name: plan.workflow_name.clone(),
                message,
                next_run_epoch_ms: plan.next_run_epoch_ms,
            },
        })
        .collect()
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn plan_schedule_occurrence(
    plan: &DueRunPlan,
    workflow_revision: &str,
) -> Result<ScheduleOccurrenceClaimRequest, ScheduleOccurrenceError> {
    if plan.schedule_id.trim().is_empty() {
        return Err(ScheduleOccurrenceError::EmptyScheduleId);
    }
    if plan.workflow_name.trim().is_empty() {
        return Err(ScheduleOccurrenceError::EmptyWorkflowName);
    }
    if workflow_revision.trim().is_empty() {
        return Err(ScheduleOccurrenceError::EmptyWorkflowRevision);
    }
    if plan.next_run_epoch_ms <= plan.due_epoch_ms {
        return Err(ScheduleOccurrenceError::InvalidNextRun);
    }
    let identity = schedule_occurrence_identity(&plan.schedule_id, plan.due_epoch_ms);
    Ok(ScheduleOccurrenceClaimRequest {
        identity,
        schedule_id: plan.schedule_id.clone(),
        workflow_name: plan.workflow_name.clone(),
        workflow_revision: workflow_revision.to_string(),
        due_epoch_ms: plan.due_epoch_ms,
        next_run_epoch_ms: plan.next_run_epoch_ms,
    })
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn validate_schedule_occurrence(occurrence: &ScheduleOccurrence) -> Result<(), ScheduleOccurrenceError> {
    if occurrence.schedule_id.trim().is_empty() {
        return Err(ScheduleOccurrenceError::EmptyScheduleId);
    }
    if occurrence.workflow_name.trim().is_empty() {
        return Err(ScheduleOccurrenceError::EmptyWorkflowName);
    }
    if occurrence.workflow_revision.trim().is_empty() {
        return Err(ScheduleOccurrenceError::EmptyWorkflowRevision);
    }
    if occurrence.next_run_epoch_ms <= occurrence.due_epoch_ms {
        return Err(ScheduleOccurrenceError::InvalidNextRun);
    }
    if occurrence.identity != schedule_occurrence_identity(&occurrence.schedule_id, occurrence.due_epoch_ms) {
        return Err(ScheduleOccurrenceError::IdentityMismatch);
    }
    if occurrence.attempt_run_ids.is_empty() {
        return Err(ScheduleOccurrenceError::MissingAttempt);
    }
    if occurrence.attempt_run_ids.len() > MAX_SCHEDULE_OCCURRENCE_ATTEMPTS {
        return Err(ScheduleOccurrenceError::TooManyAttempts);
    }
    if occurrence.attempt_run_ids.last().copied() != Some(occurrence.active_run_id) {
        return Err(ScheduleOccurrenceError::ActiveRunMismatch);
    }
    Ok(())
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn settle_schedule_occurrence(
    occurrence: &ScheduleOccurrence,
    run_id: RunId,
    status: ScheduleOccurrenceStatus,
) -> Result<ScheduleOccurrence, ScheduleOccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceError::NotPending);
    }
    if occurrence.active_run_id != run_id {
        return Err(ScheduleOccurrenceError::ActiveRunMismatch);
    }
    if !matches!(status, ScheduleOccurrenceStatus::Succeeded | ScheduleOccurrenceStatus::Failed) {
        return Err(ScheduleOccurrenceError::InvalidSettlement);
    }
    let mut settled = occurrence.clone();
    settled.status = status;
    debug_assert!(settled.status.is_terminal());
    Ok(settled)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn retry_schedule_occurrence(
    occurrence: &ScheduleOccurrence,
    workflow_revision: &str,
    run_id: RunId,
    acknowledged_duplicate_risk: bool,
) -> Result<ScheduleOccurrence, ScheduleOccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if !acknowledged_duplicate_risk {
        return Err(ScheduleOccurrenceError::RetryAcknowledgementRequired);
    }
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceError::NotPending);
    }
    if occurrence.workflow_revision != workflow_revision {
        return Err(ScheduleOccurrenceError::WorkflowRevisionMismatch);
    }
    if occurrence.attempt_run_ids.len() >= MAX_SCHEDULE_OCCURRENCE_ATTEMPTS {
        return Err(ScheduleOccurrenceError::RetryLimitExhausted);
    }
    if occurrence.attempt_run_ids.contains(&run_id) {
        return Err(ScheduleOccurrenceError::ActiveRunMismatch);
    }
    let mut retried = occurrence.clone();
    retried.attempt_run_ids.push(run_id);
    retried.active_run_id = run_id;
    debug_assert_eq!(retried.attempt_run_ids.len(), MAX_SCHEDULE_OCCURRENCE_ATTEMPTS);
    Ok(retried)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn abandon_schedule_occurrence(
    occurrence: &ScheduleOccurrence,
    acknowledged_unknown_outcome: bool,
) -> Result<ScheduleOccurrence, ScheduleOccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if !acknowledged_unknown_outcome {
        return Err(ScheduleOccurrenceError::AbandonAcknowledgementRequired);
    }
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceError::NotPending);
    }
    let mut abandoned = occurrence.clone();
    abandoned.status = ScheduleOccurrenceStatus::AbandonedUnknown;
    debug_assert!(abandoned.status.is_terminal());
    Ok(abandoned)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn classify_schedule_occurrence_recovery(
    occurrence: &ScheduleOccurrence,
    observation: Option<ScheduleOccurrenceRunObservation>,
) -> Result<ScheduleOccurrenceRecoveryDecision, ScheduleOccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceError::NotPending);
    }
    let Some(observation) = observation else {
        return Ok(ScheduleOccurrenceRecoveryDecision::PendingUnobserved {
            run_id: occurrence.active_run_id,
        });
    };
    if observation.run_id != occurrence.active_run_id {
        return Err(ScheduleOccurrenceError::RecoveryRunMismatch);
    }
    if observation.owner != ScheduleOccurrenceRunOwnerObservation::Exact {
        return Err(ScheduleOccurrenceError::RecoveryOwnerMismatch);
    }
    if !observation.finished {
        return Err(ScheduleOccurrenceError::RecoveryRunNotTerminal);
    }
    let status = if observation.failures == 0 {
        ScheduleOccurrenceStatus::Succeeded
    } else {
        ScheduleOccurrenceStatus::Failed
    };
    Ok(ScheduleOccurrenceRecoveryDecision::Settle {
        run_id: observation.run_id,
        status,
    })
}

#[must_use]
pub fn render_schedule_occurrence(occurrence: &ScheduleOccurrence) -> String {
    format!(
        "occurrence={} status={:?} due_epoch_ms={} active_run={} attempts={}",
        occurrence.identity,
        occurrence.status,
        occurrence.due_epoch_ms,
        occurrence.active_run_id,
        occurrence.attempt_run_ids.len()
    )
}

fn schedule_occurrence_identity(schedule_id: &str, due_epoch_ms: i64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SCHEDULE_OCCURRENCE_DOMAIN);
    update_occurrence_identity_part(&mut hasher, SCHEDULE_OCCURRENCE_ID_TAG, schedule_id.as_bytes());
    update_occurrence_identity_part(&mut hasher, SCHEDULE_OCCURRENCE_DUE_TAG, &due_epoch_ms.to_be_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

fn update_occurrence_identity_part(hasher: &mut blake3::Hasher, tag: &[u8], value: &[u8]) {
    let mut part = blake3::Hasher::new();
    part.update(tag);
    part.update(value);
    hasher.update(part.finalize().as_bytes());
}

pub fn render_daemon_outcome(outcome: &ScheduleDaemonOutcome) -> String {
    match outcome {
        ScheduleDaemonOutcome::Started {
            schedule_id,
            workflow_name,
            summary,
            ..
        } => format!(
            "started schedule {schedule_id} workflow {workflow_name}: {} nodes, {} failures",
            summary.node_count, summary.failures
        ),
        ScheduleDaemonOutcome::Error {
            schedule_id,
            workflow_name,
            message,
            ..
        } => format!("schedule {schedule_id} workflow {workflow_name} failed: {message}"),
        ScheduleDaemonOutcome::Skipped(skip) => {
            format!("skipped schedule {} workflow {}: {:?}", skip.schedule_id, skip.workflow_name, skip.reason)
        }
    }
}

pub mod shell;
