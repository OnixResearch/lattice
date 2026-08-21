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

fn plan_occurrence(occurrence: &crate::scheduler::ScheduleOccurrence) -> Result<OccurrencePilot, PilotError> {
    crate::scheduler::validate_schedule_occurrence(occurrence)?;
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

pub(crate) fn admit_occurrence(
    occurrence: &crate::scheduler::ScheduleOccurrence,
) -> Result<transactional_reconciliation_core::DispatchIntent, PilotError> {
    let pilot = plan_occurrence(occurrence)?;
    let persisted = transactional_reconciliation_core::classify_commit(
        pilot.persistence_binding,
        transactional_reconciliation_core::CommitObservation::Applied(pilot.persistence_binding),
    );
    let published = transactional_reconciliation_core::admit_published(persisted, pilot.reservation)?;
    debug_assert_eq!(published.binding(), pilot.persistence_binding);
    admit_observation(&pilot, transactional_reconciliation_core::ReservationObservation::Durable(pilot.reservation))
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
mod tests {
    const SCHEDULE_ID: &str = "daily";
    const WORKFLOW_NAME: &str = "digest";
    const WORKFLOW_REVISION: &str = "workflow-v1";
    const DUE_EPOCH_MS: i64 = 1_000;
    const NEXT_RUN_EPOCH_MS: i64 = 2_000;
    const RUN_ID: u64 = 17;

    fn occurrence() -> Result<crate::scheduler::ScheduleOccurrence, super::PilotError> {
        let plan = crate::scheduler::DueRunPlan {
            schedule_id: SCHEDULE_ID.to_string(),
            workflow_name: WORKFLOW_NAME.to_string(),
            due_epoch_ms: DUE_EPOCH_MS,
            next_run_epoch_ms: NEXT_RUN_EPOCH_MS,
        };
        Ok(crate::scheduler::plan_schedule_occurrence(&plan, WORKFLOW_REVISION)?
            .occurrence(crate::engine::RunId(RUN_ID)))
    }

    #[test]
    fn exact_occurrence_reservation_admits_shared_dispatch() -> Result<(), super::PilotError> {
        let occurrence = occurrence()?;
        let pilot = super::plan_occurrence(&occurrence)?;
        let dispatch = super::admit_occurrence(&occurrence)?;

        assert_eq!(dispatch.plan_identity(), pilot.plan.identity());
        assert_eq!(dispatch.attempt_identity(), pilot.reservation.attempt_identity());
        Ok(())
    }

    #[test]
    fn uncertain_publication_quarantines_until_exact_reopen() -> Result<(), super::PilotError> {
        let pilot = super::plan_occurrence(&occurrence()?)?;
        let uncertain = transactional_reconciliation_core::classify_commit(
            pilot.persistence_binding,
            transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
        );
        let reconciled = transactional_reconciliation_core::reconcile_read_back(
            uncertain,
            transactional_reconciliation_core::ReadBackObservation::Reservation(pilot.persistence_binding),
        )?;

        assert!(matches!(
            uncertain.quarantine(),
            transactional_reconciliation_core::QuarantineStatus::Quarantined(_)
        ));
        assert_eq!(reconciled.state(), transactional_reconciliation_core::PersistenceState::Published);
        assert_eq!(reconciled.quarantine(), transactional_reconciliation_core::QuarantineStatus::Clear);
        Ok(())
    }

    #[test]
    fn missing_reservation_blocks_shared_dispatch() -> Result<(), super::PilotError> {
        let pilot = super::plan_occurrence(&occurrence()?)?;
        let result =
            super::admit_observation(&pilot, transactional_reconciliation_core::ReservationObservation::Missing);

        assert!(matches!(
            result,
            Err(super::PilotError::Core(transactional_reconciliation_core::CoreError::ReservationMismatch))
        ));
        Ok(())
    }
}
