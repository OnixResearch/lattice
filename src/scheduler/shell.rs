//! Imperative scheduler occurrence shell.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum ScheduleOccurrenceRecoveryOutcome {
    AlreadyTerminal { occurrence: Box<super::ScheduleOccurrence> },
    PendingUnobserved { occurrence: Box<super::ScheduleOccurrence> },
    Reconciled { occurrence: Box<super::ScheduleOccurrence> },
}

#[derive(Debug)]
pub enum Error {
    Store(crate::store::StoreError),
    MissingOccurrence(String),
    MissingWorkflow(String),
    InvalidWorkflowRevision(String),
    InvalidOccurrence(String),
    Execution(String),
    RecoveryRequired,
}

impl std::fmt::Display for Error {
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

impl std::error::Error for Error {}

impl From<crate::store::StoreError> for Error {
    fn from(error: crate::store::StoreError) -> Self {
        Self::Store(error)
    }
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn claim_due_occurrence(
    store: &crate::store::Store,
    plan: &super::DueRunPlan,
) -> Result<super::ScheduleOccurrenceClaimResolution, Error> {
    let revision = load_workflow_revision(store, &plan.workflow_name)?;
    let request = super::plan_schedule_occurrence(plan, &revision)
        .map_err(|error| Error::InvalidOccurrence(error.to_string()))?;
    store.claim_schedule_occurrence(&request).map_err(Into::into)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub async fn execute_occurrence(
    store: &crate::store::Store,
    occurrence: &super::ScheduleOccurrence,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
) -> Result<crate::headless::HeadlessRunSummary, Error> {
    execute_occurrence_with_authority_provider(
        store,
        occurrence,
        agent_runtime,
        &crate::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
}

pub async fn execute_occurrence_with_authority_provider(
    store: &crate::store::Store,
    occurrence: &super::ScheduleOccurrence,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<crate::headless::HeadlessRunSummary, Error> {
    super::validate_schedule_occurrence(occurrence).map_err(|error| Error::InvalidOccurrence(error.to_string()))?;
    debug_assert!(!occurrence.identity.is_empty());
    debug_assert_eq!(occurrence.attempt_run_ids.last().copied(), Some(occurrence.active_run_id));
    if occurrence.status != super::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(Error::InvalidOccurrence(super::OccurrenceError::NotPending.to_string()));
    }
    let revision = load_workflow_revision(store, &occurrence.workflow_name)?;
    if revision != occurrence.workflow_revision {
        return Err(Error::InvalidOccurrence(super::OccurrenceError::WorkflowRevisionMismatch.to_string()));
    }
    let dispatch = store.admit_schedule_occurrence_dispatch(occurrence)?;
    debug_assert_eq!(dispatch.attempt_identity().as_bytes().len(), blake3::OUT_LEN);
    let summary = crate::headless::run_owned_workflow_with_authority_provider(
        store,
        crate::headless::OwnedRunRequest {
            workflow_name: occurrence.workflow_name.clone(),
            run_id: occurrence.active_run_id,
            owner: crate::store::RunOwner::ScheduleOccurrence {
                identity: occurrence.identity.clone(),
            },
            agent_runtime,
        },
        authority_provider,
    )
    .await
    .map_err(|error| Error::Execution(error.to_string()))?;
    let recovered = recover_occurrence(store, &occurrence.identity)?;
    if !matches!(recovered, ScheduleOccurrenceRecoveryOutcome::Reconciled { .. }) {
        return Err(Error::Execution("terminal run did not reconcile the active occurrence".to_string()));
    }
    debug_assert_eq!(summary.run_id, occurrence.active_run_id);
    Ok(summary)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn recover_occurrence(
    store: &crate::store::Store,
    identity: &str,
) -> Result<ScheduleOccurrenceRecoveryOutcome, Error> {
    let occurrence = load_occurrence(store, identity)?;
    debug_assert_eq!(occurrence.identity, identity);
    debug_assert_eq!(occurrence.attempt_run_ids.last().copied(), Some(occurrence.active_run_id));
    if occurrence.status.is_terminal() {
        return Ok(ScheduleOccurrenceRecoveryOutcome::AlreadyTerminal {
            occurrence: Box::new(occurrence),
        });
    }
    let record = store.load_run(occurrence.active_run_id)?;
    let observation = record.as_ref().map(|record| super::ScheduleOccurrenceRunObservation {
        run_id: record.run_id,
        owner: owner_observation(&occurrence.identity, &record.owner),
        finished: record.finished_epoch_ms.is_some(),
        failures: record.failures,
    });
    match super::classify_schedule_occurrence_recovery(&occurrence, observation)
        .map_err(|error| Error::InvalidOccurrence(error.to_string()))?
    {
        super::ScheduleOccurrenceRecoveryDecision::PendingUnobserved { .. } => {
            Ok(ScheduleOccurrenceRecoveryOutcome::PendingUnobserved {
                occurrence: Box::new(occurrence),
            })
        }
        super::ScheduleOccurrenceRecoveryDecision::Settle { run_id, status } => {
            let occurrence = store.settle_schedule_occurrence(identity, run_id, status)?;
            Ok(ScheduleOccurrenceRecoveryOutcome::Reconciled {
                occurrence: Box::new(occurrence),
            })
        }
    }
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub async fn retry_occurrence(
    store: &crate::store::Store,
    identity: &str,
    is_duplicate_risk_acknowledged: bool,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
) -> Result<super::ScheduleOccurrence, Error> {
    let current = load_occurrence(store, identity)?;
    if current.status != super::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(Error::InvalidOccurrence(super::OccurrenceError::NotPending.to_string()));
    }
    if store.load_run(current.active_run_id)?.is_some() {
        return Err(Error::RecoveryRequired);
    }
    let revision = load_workflow_revision(store, &current.workflow_name)?;
    let retried = store.retry_schedule_occurrence(identity, &revision, is_duplicate_risk_acknowledged)?;
    execute_occurrence(store, &retried, agent_runtime).await?;
    load_occurrence(store, identity)
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn abandon_occurrence(
    store: &crate::store::Store,
    identity: &str,
    is_unknown_outcome_acknowledged: bool,
) -> Result<super::ScheduleOccurrence, Error> {
    let current = load_occurrence(store, identity)?;
    if current.status != super::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(Error::InvalidOccurrence(super::OccurrenceError::NotPending.to_string()));
    }
    if store.load_run(current.active_run_id)?.is_some() {
        return Err(Error::RecoveryRequired);
    }
    store.abandon_schedule_occurrence(identity, is_unknown_outcome_acknowledged).map_err(Into::into)
}

fn load_workflow_revision(store: &crate::store::Store, workflow_name: &str) -> Result<String, Error> {
    let workflow = store
        .load_workflow(workflow_name)?
        .ok_or_else(|| Error::MissingWorkflow(workflow_name.to_string()))?;
    crate::orchestration::workflow_revision(&workflow)
        .map(|revision| revision.content_hash.to_string())
        .map_err(|error| Error::InvalidWorkflowRevision(error.to_string()))
}

fn load_occurrence(store: &crate::store::Store, identity: &str) -> Result<super::ScheduleOccurrence, Error> {
    store
        .load_schedule_occurrence_for_recovery(identity)?
        .ok_or_else(|| Error::MissingOccurrence(identity.to_string()))
}

fn owner_observation(identity: &str, owner: &crate::store::RunOwner) -> super::ScheduleOccurrenceRunOwnerObservation {
    match owner {
        crate::store::RunOwner::ScheduleOccurrence {
            identity: owner_identity,
        } if owner_identity == identity => super::ScheduleOccurrenceRunOwnerObservation::Exact,
        crate::store::RunOwner::Unscoped => super::ScheduleOccurrenceRunOwnerObservation::Unscoped,
        crate::store::RunOwner::Orchestration { .. }
        | crate::store::RunOwner::ScheduleOccurrence { .. }
        | crate::store::RunOwner::WorkflowApproval { .. } => super::ScheduleOccurrenceRunOwnerObservation::Foreign,
    }
}
