//! Focused adapter pilot for product-neutral reconciliation decisions.

const OCCURRENCE_POLICY_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.policy.v1";
const OCCURRENCE_SUBJECT_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.subject.v1";
const OCCURRENCE_INPUT_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.input.v1";
const OCCURRENCE_SCOPE_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.scope.v1";
const OCCURRENCE_ATTEMPT_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.attempt.v1";
const OCCURRENCE_CONTENT_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.content.v1";
const OCCURRENCE_PREDECESSOR_DOMAIN: &[u8] = b"lattice.scheduler.occurrence.predecessor.v1";
const CORE_OPERATION_BOUND: u32 = 1;
const CORE_PREREQUISITE_BOUND: u32 = 1;
const LENGTH_FRAME_BYTES: usize = 8;

struct LatticeBlake3;

impl transactional_reconciliation_core::Blake3IdentityDeriver for LatticeBlake3 {
    fn derive_identity(
        &self,
        framed_bytes: &[u8],
    ) -> Result<transactional_reconciliation_core::Identity, transactional_reconciliation_core::CoreError> {
        transactional_reconciliation_core::Identity::new(*blake3::hash(framed_bytes).as_bytes())
    }
}

#[derive(Debug)]
pub(crate) enum PilotError {
    Occurrence(crate::scheduler::OccurrenceError),
    Core(transactional_reconciliation_core::CoreError),
    FrameLength,
}

impl From<crate::scheduler::OccurrenceError> for PilotError {
    fn from(error: crate::scheduler::OccurrenceError) -> Self {
        Self::Occurrence(error)
    }
}

impl From<transactional_reconciliation_core::CoreError> for PilotError {
    fn from(error: transactional_reconciliation_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl std::fmt::Display for PilotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Occurrence(error) => write!(formatter, "{error}"),
            Self::Core(error) => write!(formatter, "shared reconciliation rejected the occurrence: {error:?}"),
            Self::FrameLength => formatter.write_str("occurrence identity framing overflowed"),
        }
    }
}

#[derive(Debug)]
struct OccurrencePilot {
    plan: transactional_reconciliation_core::ImmutablePlan,
    current: transactional_reconciliation_core::CurrentFacts,
    reservation: transactional_reconciliation_core::AttemptReservation,
    persistence_binding: transactional_reconciliation_core::PersistenceBinding,
}

// r[impl lattice.persistence_uncertainty.shared_core]
// r[impl lattice.persistence_uncertainty.shared_core.scenario.selected]
fn plan_occurrence(occurrence: &crate::scheduler::ScheduleOccurrence) -> Result<OccurrencePilot, PilotError> {
    crate::scheduler::validate_schedule_occurrence(occurrence)?;
    debug_assert!(!occurrence.identity.is_empty());
    debug_assert_eq!(occurrence.attempt_run_ids.last().copied(), Some(occurrence.active_run_id));
    let desired_state_identity = digest(OCCURRENCE_INPUT_DOMAIN, &[occurrence.workflow_revision.as_bytes()])?;
    let policy_identity = digest(OCCURRENCE_POLICY_DOMAIN, &[])?;
    let operation = transactional_reconciliation_core::OperationDraft::new(
        digest(OCCURRENCE_SUBJECT_DOMAIN, &[occurrence.identity.as_bytes()])?,
        desired_state_identity,
        transactional_reconciliation_core::Generation::observed(occurrence.active_run_id.0),
        digest(OCCURRENCE_SCOPE_DOMAIN, &[occurrence.schedule_id.as_bytes()])?,
    );
    let revision = transactional_reconciliation_core::Revision::observed(occurrence.active_run_id.0);
    let plan = transactional_reconciliation_core::build_plan(
        &LatticeBlake3,
        transactional_reconciliation_core::Limits::new(
            transactional_reconciliation_core::Bound::new(CORE_OPERATION_BOUND)?,
            transactional_reconciliation_core::Bound::new(CORE_PREREQUISITE_BOUND)?,
        ),
        transactional_reconciliation_core::PlanningInput::new(
            revision,
            desired_state_identity,
            policy_identity,
            Vec::new(),
            vec![operation],
        ),
    )?;
    let current = transactional_reconciliation_core::CurrentFacts::new(
        revision,
        desired_state_identity,
        policy_identity,
        Vec::new(),
    );
    let operation_identity = plan.operations()[0].idempotency_identity();
    let run_id_bytes = occurrence.active_run_id.0.to_le_bytes();
    let attempt_identity = digest(OCCURRENCE_ATTEMPT_DOMAIN, &[occurrence.identity.as_bytes(), &run_id_bytes])?;
    let reservation = transactional_reconciliation_core::reserve_attempt(&plan, operation_identity, attempt_identity)?;
    let successor_content_identity = digest(OCCURRENCE_CONTENT_DOMAIN, &[
        occurrence.identity.as_bytes(),
        occurrence.workflow_revision.as_bytes(),
        &run_id_bytes,
    ])?;
    let publication = transactional_reconciliation_core::plan_publication(&plan, &current, successor_content_identity)?;
    let persistence_binding = transactional_reconciliation_core::PersistenceBinding::new(
        publication,
        reservation,
        digest(OCCURRENCE_PREDECESSOR_DOMAIN, &[])?,
    )?;
    Ok(OccurrencePilot {
        plan,
        current,
        reservation,
        persistence_binding,
    })
}

pub(crate) fn admit_occurrence_readback(
    expected: &crate::scheduler::ScheduleOccurrence,
    observed: Option<&crate::scheduler::ScheduleOccurrence>,
) -> Result<transactional_reconciliation_core::DispatchIntent, PilotError> {
    if expected.status != crate::scheduler::ScheduleOccurrenceStatus::PendingUnobserved {
        return Err(crate::scheduler::OccurrenceError::NotPending.into());
    }
    if observed.is_some_and(|row| row != expected) {
        return Err(transactional_reconciliation_core::CoreError::PersistenceBindingMismatch.into());
    }
    let pilot = plan_occurrence(expected)?;
    let unknown = transactional_reconciliation_core::classify_commit(
        pilot.persistence_binding,
        transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
    );
    let observation = match observed {
        Some(_) => transactional_reconciliation_core::ReadBackObservation::Reservation(pilot.persistence_binding),
        None => transactional_reconciliation_core::ReadBackObservation::Missing,
    };
    let reconciled = transactional_reconciliation_core::reconcile_read_back(unknown, observation)?;
    let published = transactional_reconciliation_core::admit_published(reconciled, pilot.reservation)?;
    debug_assert_eq!(published.binding(), pilot.persistence_binding);
    admit_observation(&pilot, transactional_reconciliation_core::ReservationObservation::Durable(pilot.reservation))
}

pub(crate) fn classify_occurrence_commit(
    occurrence: &crate::scheduler::ScheduleOccurrence,
    observation: transactional_reconciliation_core::CommitObservation,
) -> Result<transactional_reconciliation_core::PersistenceDecision, PilotError> {
    let pilot = plan_occurrence(occurrence)?;
    Ok(transactional_reconciliation_core::classify_commit(pilot.persistence_binding, observation))
}

pub(crate) fn reconcile_occurrence_commit(
    decision: transactional_reconciliation_core::PersistenceDecision,
    occurrence: Option<&crate::scheduler::ScheduleOccurrence>,
) -> Result<transactional_reconciliation_core::PersistenceDecision, PilotError> {
    let observation = match occurrence {
        Some(occurrence) => {
            let pilot = plan_occurrence(occurrence)?;
            transactional_reconciliation_core::ReadBackObservation::Reservation(pilot.persistence_binding)
        }
        None => transactional_reconciliation_core::ReadBackObservation::Missing,
    };
    transactional_reconciliation_core::reconcile_read_back(decision, observation).map_err(PilotError::from)
}

fn admit_observation(
    pilot: &OccurrencePilot,
    observation: transactional_reconciliation_core::ReservationObservation,
) -> Result<transactional_reconciliation_core::DispatchIntent, PilotError> {
    transactional_reconciliation_core::admit_dispatch(&pilot.plan, &pilot.current, pilot.reservation, observation)
        .map_err(PilotError::from)
}

fn digest(domain: &[u8], parts: &[&[u8]]) -> Result<transactional_reconciliation_core::Identity, PilotError> {
    let mut hasher = blake3::Hasher::new();
    append(&mut hasher, domain)?;
    for part in parts {
        append(&mut hasher, part)?;
    }
    transactional_reconciliation_core::Identity::new(*hasher.finalize().as_bytes()).map_err(PilotError::from)
}

fn append(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), PilotError> {
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| PilotError::FrameLength)?.to_le_bytes();
    debug_assert_eq!(length_bytes.len(), LENGTH_FRAME_BYTES);
    hasher.update(&length_bytes);
    hasher.update(bytes);
    Ok(())
}

#[cfg(test)]
mod tests;
