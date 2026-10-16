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
    Ok(crate::scheduler::plan_schedule_occurrence(&plan, WORKFLOW_REVISION)?.occurrence(crate::engine::RunId(RUN_ID)))
}

#[test]
fn exact_occurrence_reservation_admits_shared_dispatch() -> Result<(), super::PilotError> {
    let occurrence = occurrence()?;
    let pilot = super::plan_occurrence(&occurrence)?;
    let dispatch = super::admit_occurrence_readback(&occurrence, Some(&occurrence))?;

    // r[verify lattice.persistence_uncertainty.commit_outcomes.scenario.published]
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
    let result = super::admit_observation(&pilot, transactional_reconciliation_core::ReservationObservation::Missing);

    assert!(matches!(
        result,
        Err(super::PilotError::Core(transactional_reconciliation_core::CoreError::ReservationMismatch))
    ));
    Ok(())
}

#[test]
fn unknown_and_foreign_reservations_cannot_authorize_dispatch() -> Result<(), super::PilotError> {
    let occurrence = occurrence()?;
    let pilot = super::plan_occurrence(&occurrence)?;
    let unknown = super::classify_occurrence_commit(
        &occurrence,
        transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
    )?;
    // r[verify lattice.persistence_uncertainty.effect_gate.scenario.unknown_blocked]
    assert_eq!(unknown.state(), transactional_reconciliation_core::PersistenceState::PublicationUnknown);
    assert_eq!(
        transactional_reconciliation_core::admit_published(unknown, pilot.reservation),
        Err(transactional_reconciliation_core::CoreError::PersistenceQuarantined)
    );

    let mut foreign = occurrence.clone();
    foreign.workflow_revision = "foreign-workflow-revision".to_string();
    let conflict = super::reconcile_occurrence_commit(unknown, Some(&foreign))?;
    // r[verify lattice.persistence_uncertainty.reconciliation.scenario.conflict_blocks]
    assert_eq!(conflict.state(), transactional_reconciliation_core::PersistenceState::Conflicting);
    assert!(matches!(conflict.quarantine(), transactional_reconciliation_core::QuarantineStatus::Quarantined(_)));
    assert_eq!(
        transactional_reconciliation_core::admit_published(conflict, pilot.reservation),
        Err(transactional_reconciliation_core::CoreError::PersistenceQuarantined)
    );
    Ok(())
}

#[test]
fn readback_admission_rejects_missing_or_nonexact_occurrence() -> Result<(), super::PilotError> {
    let expected = occurrence()?;
    assert!(matches!(
        super::admit_occurrence_readback(&expected, None),
        Err(super::PilotError::Core(transactional_reconciliation_core::CoreError::PersistenceQuarantined))
    ));
    let mut observed = expected.clone();
    observed.next_run_epoch_ms += 1;
    assert!(matches!(
        super::admit_occurrence_readback(&expected, Some(&observed)),
        Err(super::PilotError::Core(transactional_reconciliation_core::CoreError::PersistenceBindingMismatch))
    ));
    Ok(())
}
