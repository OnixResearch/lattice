//! Pure durable scheduler-occurrence admission and recovery contract.

const IDENTITY_DOMAIN: &[u8] = b"lattice-schedule-occurrence-v1";
const ID_TAG: &[u8] = b"schedule-id";
const DUE_TAG: &[u8] = b"due-epoch-ms";
mod model;

pub use model::*;

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn plan_schedule_occurrence(
    plan: &super::DueRunPlan,
    workflow_revision: &str,
) -> Result<ScheduleOccurrenceClaimRequest, OccurrenceError> {
    if plan.schedule_id.trim().is_empty() {
        return Err(OccurrenceError::EmptyScheduleId);
    }
    if plan.workflow_name.trim().is_empty() {
        return Err(OccurrenceError::EmptyWorkflowName);
    }
    if workflow_revision.trim().is_empty() {
        return Err(OccurrenceError::EmptyWorkflowRevision);
    }
    if plan.next_run_epoch_ms <= plan.due_epoch_ms {
        return Err(OccurrenceError::InvalidNextRun);
    }
    debug_assert!(!plan.schedule_id.trim().is_empty());
    debug_assert!(plan.next_run_epoch_ms > plan.due_epoch_ms);
    let identity = identity(&plan.schedule_id, plan.due_epoch_ms);
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
pub fn validate_schedule_occurrence_claim(
    schedule: &super::ScheduleDefinition,
    request: &ScheduleOccurrenceClaimRequest,
) -> Result<(), OccurrenceError> {
    if !schedule.enabled {
        return Err(OccurrenceError::ScheduleDisabled);
    }
    if schedule.id != request.schedule_id
        || schedule.workflow_name != request.workflow_name
        || schedule.next_run_epoch_ms != request.due_epoch_ms
    {
        return Err(OccurrenceError::ScheduleMismatch);
    }
    let interval_ms = schedule.recurrence.interval_ms();
    let advance_ms = request.next_run_epoch_ms.saturating_sub(request.due_epoch_ms);
    if interval_ms <= 0 || advance_ms <= 0 || advance_ms % interval_ms != 0 {
        return Err(OccurrenceError::NextRunMisaligned);
    }
    debug_assert!(schedule.enabled);
    debug_assert_eq!(schedule.next_run_epoch_ms, request.due_epoch_ms);
    Ok(())
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn validate_schedule_occurrence(occurrence: &ScheduleOccurrence) -> Result<(), OccurrenceError> {
    if occurrence.schedule_id.trim().is_empty() {
        return Err(OccurrenceError::EmptyScheduleId);
    }
    if occurrence.workflow_name.trim().is_empty() {
        return Err(OccurrenceError::EmptyWorkflowName);
    }
    if occurrence.workflow_revision.trim().is_empty() {
        return Err(OccurrenceError::EmptyWorkflowRevision);
    }
    if occurrence.next_run_epoch_ms <= occurrence.due_epoch_ms {
        return Err(OccurrenceError::InvalidNextRun);
    }
    if occurrence.identity != identity(&occurrence.schedule_id, occurrence.due_epoch_ms) {
        return Err(OccurrenceError::IdentityMismatch);
    }
    if occurrence.attempt_run_ids.is_empty() {
        return Err(OccurrenceError::MissingAttempt);
    }
    if occurrence.attempt_run_ids.len() > MAX_SCHEDULE_OCCURRENCE_ATTEMPTS {
        return Err(OccurrenceError::TooManyAttempts);
    }
    if occurrence.attempt_run_ids.last().copied() != Some(occurrence.active_run_id) {
        return Err(OccurrenceError::ActiveRunMismatch);
    }
    debug_assert!(!occurrence.attempt_run_ids.is_empty());
    debug_assert_eq!(occurrence.attempt_run_ids.last().copied(), Some(occurrence.active_run_id));
    Ok(())
}

// r[impl lattice.scheduling.scheduler_occurrence_admission]
pub fn settle_schedule_occurrence(
    occurrence: &ScheduleOccurrence,
    run_id: crate::engine::RunId,
    status: ScheduleOccurrenceStatus,
) -> Result<ScheduleOccurrence, OccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(OccurrenceError::NotPending);
    }
    if occurrence.active_run_id != run_id {
        return Err(OccurrenceError::ActiveRunMismatch);
    }
    if !matches!(status, ScheduleOccurrenceStatus::Succeeded | ScheduleOccurrenceStatus::Failed) {
        return Err(OccurrenceError::InvalidSettlement);
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
    run_id: crate::engine::RunId,
    is_duplicate_risk_acknowledged: bool,
) -> Result<ScheduleOccurrence, OccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if !is_duplicate_risk_acknowledged {
        return Err(OccurrenceError::RetryAcknowledgementRequired);
    }
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(OccurrenceError::NotPending);
    }
    if occurrence.workflow_revision != workflow_revision {
        return Err(OccurrenceError::WorkflowRevisionMismatch);
    }
    if occurrence.attempt_run_ids.len() >= MAX_SCHEDULE_OCCURRENCE_ATTEMPTS {
        return Err(OccurrenceError::RetryLimitExhausted);
    }
    if occurrence.attempt_run_ids.contains(&run_id) {
        return Err(OccurrenceError::ActiveRunMismatch);
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
    is_unknown_outcome_acknowledged: bool,
) -> Result<ScheduleOccurrence, OccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    if !is_unknown_outcome_acknowledged {
        return Err(OccurrenceError::AbandonAcknowledgementRequired);
    }
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(OccurrenceError::NotPending);
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
) -> Result<ScheduleOccurrenceRecoveryDecision, OccurrenceError> {
    validate_schedule_occurrence(occurrence)?;
    debug_assert!(!occurrence.attempt_run_ids.is_empty());
    debug_assert_eq!(occurrence.attempt_run_ids.last().copied(), Some(occurrence.active_run_id));
    if occurrence.status != ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(OccurrenceError::NotPending);
    }
    let Some(observation) = observation else {
        return Ok(ScheduleOccurrenceRecoveryDecision::PendingUnobserved {
            run_id: occurrence.active_run_id,
        });
    };
    if observation.run_id != occurrence.active_run_id {
        return Err(OccurrenceError::RecoveryRunMismatch);
    }
    if observation.owner != ScheduleOccurrenceRunOwnerObservation::Exact {
        return Err(OccurrenceError::RecoveryOwnerMismatch);
    }
    if !observation.finished {
        return Err(OccurrenceError::RecoveryRunNotTerminal);
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

fn identity(schedule_id: &str, due_epoch_ms: i64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(IDENTITY_DOMAIN);
    update_identity_part(&mut hasher, ID_TAG, schedule_id.as_bytes());
    update_identity_part(&mut hasher, DUE_TAG, &due_epoch_ms.to_be_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

fn update_identity_part(hasher: &mut blake3::Hasher, tag: &[u8], value: &[u8]) {
    let mut part = blake3::Hasher::new();
    part.update(tag);
    part.update(value);
    hasher.update(part.finalize().as_bytes());
}
