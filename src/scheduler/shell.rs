//! Imperative scheduler occurrence shell.

use crate::agent_node::AgentRuntimeConfig;
use crate::headless::HeadlessRunSummary;
use crate::headless::OwnedRunRequest;
use crate::scheduler::DueRunPlan;
use crate::scheduler::ScheduleOccurrence;
use crate::scheduler::ScheduleOccurrenceClaimResolution;
use crate::scheduler::ScheduleOccurrenceRecoveryDecision;
use crate::scheduler::ScheduleOccurrenceRunObservation;
use crate::scheduler::ScheduleOccurrenceRunOwnerObservation;
use crate::store::RunOwner;
use crate::store::Store;
use crate::store::StoreError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum ScheduleOccurrenceRecoveryOutcome {
    AlreadyTerminal { occurrence: Box<ScheduleOccurrence> },
    PendingUnobserved { occurrence: Box<ScheduleOccurrence> },
    Reconciled { occurrence: Box<ScheduleOccurrence> },
}

#[derive(Debug)]
pub enum ScheduleOccurrenceShellError {
    Store(StoreError),
    MissingOccurrence(String),
    MissingWorkflow(String),
    InvalidWorkflowRevision(String),
    InvalidOccurrence(String),
    Execution(String),
    RecoveryRequired,
}

impl std::fmt::Display for ScheduleOccurrenceShellError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::MissingOccurrence(identity) => write!(formatter, "schedule occurrence does not exist: {identity}"),
            Self::MissingWorkflow(name) => write!(formatter, "scheduled workflow does not exist: {name}"),
            Self::InvalidWorkflowRevision(message) => {
                write!(formatter, "scheduled workflow revision is invalid: {message}")
            }
            Self::InvalidOccurrence(message) => write!(formatter, "schedule occurrence is invalid: {message}"),
            Self::Execution(message) => write!(formatter, "scheduled occurrence execution failed: {message}"),
            Self::RecoveryRequired => {
                formatter.write_str("schedule occurrence has persisted run evidence; recover before another action")
            }
        }
    }
}

impl std::error::Error for ScheduleOccurrenceShellError {}

impl From<StoreError> for ScheduleOccurrenceShellError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn claim_due_occurrence(
    store: &Store,
    plan: &DueRunPlan,
) -> Result<ScheduleOccurrenceClaimResolution, ScheduleOccurrenceShellError> {
    let workflow = store
        .load_workflow(&plan.workflow_name)?
        .ok_or_else(|| ScheduleOccurrenceShellError::MissingWorkflow(plan.workflow_name.clone()))?;
    let revision = crate::orchestration::workflow_revision(&workflow)
        .map_err(|error| ScheduleOccurrenceShellError::InvalidWorkflowRevision(error.to_string()))?;
    let request = super::plan_schedule_occurrence(plan, &revision.content_hash)
        .map_err(|error| ScheduleOccurrenceShellError::InvalidOccurrence(error.to_string()))?;
    store.claim_schedule_occurrence(&request).map_err(Into::into)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub async fn execute_occurrence(
    store: &Store,
    occurrence: &ScheduleOccurrence,
    agent_runtime: AgentRuntimeConfig,
) -> Result<HeadlessRunSummary, ScheduleOccurrenceShellError> {
    super::validate_schedule_occurrence(occurrence)
        .map_err(|error| ScheduleOccurrenceShellError::InvalidOccurrence(error.to_string()))?;
    if occurrence.status != super::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceShellError::InvalidOccurrence(
            super::ScheduleOccurrenceError::NotPending.to_string(),
        ));
    }
    let summary = crate::headless::run_owned_workflow(store, OwnedRunRequest {
        workflow_name: occurrence.workflow_name.clone(),
        run_id: occurrence.active_run_id,
        owner: RunOwner::ScheduleOccurrence {
            identity: occurrence.identity.clone(),
        },
        agent_runtime,
    })
    .await
    .map_err(|error| ScheduleOccurrenceShellError::Execution(error.to_string()))?;
    let recovered = recover_occurrence(store, &occurrence.identity)?;
    if !matches!(recovered, ScheduleOccurrenceRecoveryOutcome::Reconciled { .. }) {
        return Err(ScheduleOccurrenceShellError::Execution(
            "terminal run did not reconcile the active occurrence".to_string(),
        ));
    }
    Ok(summary)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn recover_occurrence(
    store: &Store,
    identity: &str,
) -> Result<ScheduleOccurrenceRecoveryOutcome, ScheduleOccurrenceShellError> {
    let occurrence = load_occurrence(store, identity)?;
    if occurrence.status.is_terminal() {
        return Ok(ScheduleOccurrenceRecoveryOutcome::AlreadyTerminal {
            occurrence: Box::new(occurrence),
        });
    }
    let record = store.load_run(occurrence.active_run_id)?;
    let observation = record.as_ref().map(|record| ScheduleOccurrenceRunObservation {
        run_id: record.run_id,
        owner: owner_observation(&occurrence.identity, &record.owner),
        finished: record.finished_epoch_ms.is_some(),
        failures: record.failures,
    });
    match super::classify_schedule_occurrence_recovery(&occurrence, observation)
        .map_err(|error| ScheduleOccurrenceShellError::InvalidOccurrence(error.to_string()))?
    {
        ScheduleOccurrenceRecoveryDecision::PendingUnobserved { .. } => {
            Ok(ScheduleOccurrenceRecoveryOutcome::PendingUnobserved {
                occurrence: Box::new(occurrence),
            })
        }
        ScheduleOccurrenceRecoveryDecision::Settle { run_id, status } => {
            let occurrence = store.settle_schedule_occurrence(identity, run_id, status)?;
            Ok(ScheduleOccurrenceRecoveryOutcome::Reconciled {
                occurrence: Box::new(occurrence),
            })
        }
    }
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub async fn retry_occurrence(
    store: &Store,
    identity: &str,
    acknowledged_duplicate_risk: bool,
    agent_runtime: AgentRuntimeConfig,
) -> Result<ScheduleOccurrence, ScheduleOccurrenceShellError> {
    let current = load_occurrence(store, identity)?;
    if current.status != super::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceShellError::InvalidOccurrence(
            super::ScheduleOccurrenceError::NotPending.to_string(),
        ));
    }
    if store.load_run(current.active_run_id)?.is_some() {
        return Err(ScheduleOccurrenceShellError::RecoveryRequired);
    }
    let workflow = store
        .load_workflow(&current.workflow_name)?
        .ok_or_else(|| ScheduleOccurrenceShellError::MissingWorkflow(current.workflow_name.clone()))?;
    let revision = crate::orchestration::workflow_revision(&workflow)
        .map_err(|error| ScheduleOccurrenceShellError::InvalidWorkflowRevision(error.to_string()))?;
    let retried = store.retry_schedule_occurrence(identity, &revision.content_hash, acknowledged_duplicate_risk)?;
    execute_occurrence(store, &retried, agent_runtime).await?;
    load_occurrence(store, identity)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn abandon_occurrence(
    store: &Store,
    identity: &str,
    acknowledged_unknown_outcome: bool,
) -> Result<ScheduleOccurrence, ScheduleOccurrenceShellError> {
    let current = load_occurrence(store, identity)?;
    if current.status != super::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(ScheduleOccurrenceShellError::InvalidOccurrence(
            super::ScheduleOccurrenceError::NotPending.to_string(),
        ));
    }
    if store.load_run(current.active_run_id)?.is_some() {
        return Err(ScheduleOccurrenceShellError::RecoveryRequired);
    }
    store.abandon_schedule_occurrence(identity, acknowledged_unknown_outcome).map_err(Into::into)
}

fn load_occurrence(store: &Store, identity: &str) -> Result<ScheduleOccurrence, ScheduleOccurrenceShellError> {
    store
        .load_schedule_occurrence(identity)?
        .ok_or_else(|| ScheduleOccurrenceShellError::MissingOccurrence(identity.to_string()))
}

fn owner_observation(identity: &str, owner: &RunOwner) -> ScheduleOccurrenceRunOwnerObservation {
    match owner {
        RunOwner::ScheduleOccurrence {
            identity: owner_identity,
        } if owner_identity == identity => ScheduleOccurrenceRunOwnerObservation::Exact,
        RunOwner::Unscoped => ScheduleOccurrenceRunOwnerObservation::Unscoped,
        RunOwner::Orchestration { .. } | RunOwner::ScheduleOccurrence { .. } => {
            ScheduleOccurrenceRunOwnerObservation::Foreign
        }
    }
}
