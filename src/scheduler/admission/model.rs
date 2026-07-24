//! Durable occurrence state and transition facts.

pub const MAX_SCHEDULE_OCCURRENCE_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScheduleOccurrence {
    pub identity: String,
    pub schedule_id: String,
    pub workflow_name: String,
    pub workflow_revision: String,
    pub due_epoch_ms: i64,
    pub next_run_epoch_ms: i64,
    pub attempt_run_ids: Vec<crate::engine::RunId>,
    pub active_run_id: crate::engine::RunId,
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
    pub fn occurrence(&self, run_id: crate::engine::RunId) -> ScheduleOccurrence {
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
pub enum OccurrenceError {
    EmptyScheduleId,
    EmptyWorkflowName,
    EmptyWorkflowRevision,
    InvalidNextRun,
    ScheduleDisabled,
    ScheduleMismatch,
    NextRunMisaligned,
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

impl std::fmt::Display for OccurrenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyScheduleId => "schedule occurrence requires a non-empty schedule id",
            Self::EmptyWorkflowName => "schedule occurrence requires a non-empty workflow name",
            Self::EmptyWorkflowRevision => "schedule occurrence requires a non-empty workflow revision",
            Self::InvalidNextRun => "schedule occurrence next run must be after its due epoch",
            Self::ScheduleDisabled => "schedule occurrence cannot claim a disabled schedule",
            Self::ScheduleMismatch => "schedule occurrence does not match the persisted due slot",
            Self::NextRunMisaligned => "schedule occurrence next run is not aligned to its recurrence",
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

impl std::error::Error for OccurrenceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOccurrenceRunOwnerObservation {
    Exact,
    Foreign,
    Unscoped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleOccurrenceRunObservation {
    pub run_id: crate::engine::RunId,
    pub owner: ScheduleOccurrenceRunOwnerObservation,
    pub finished: bool,
    pub failures: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOccurrenceRecoveryDecision {
    PendingUnobserved {
        run_id: crate::engine::RunId,
    },
    Settle {
        run_id: crate::engine::RunId,
        status: ScheduleOccurrenceStatus,
    },
}
