use lattice::engine::RunId;
use lattice::scheduler::DueRunPlan;
use lattice::scheduler::MAX_SCHEDULE_OCCURRENCE_ATTEMPTS;
use lattice::scheduler::ScheduleOccurrenceError;
use lattice::scheduler::ScheduleOccurrenceRecoveryDecision;
use lattice::scheduler::ScheduleOccurrenceRunObservation;
use lattice::scheduler::ScheduleOccurrenceRunOwnerObservation;
use lattice::scheduler::ScheduleOccurrenceStatus;
use lattice::scheduler::abandon_schedule_occurrence;
use lattice::scheduler::classify_schedule_occurrence_recovery;
use lattice::scheduler::plan_schedule_occurrence;
use lattice::scheduler::retry_schedule_occurrence;
use lattice::scheduler::settle_schedule_occurrence;
use lattice::scheduler::validate_schedule_occurrence;

const SCHEDULE_ID: &str = "nightly";
const OTHER_SCHEDULE_ID: &str = "hourly";
const WORKFLOW_NAME: &str = "demo";
const WORKFLOW_REVISION: &str = "b3:workflow-revision";
const OTHER_WORKFLOW_REVISION: &str = "b3:changed-workflow-revision";
const DUE_EPOCH_MS: i64 = 1_000;
const NEXT_RUN_EPOCH_MS: i64 = 2_000;
const OTHER_DUE_EPOCH_MS: i64 = 3_000;
const INITIAL_RUN_ID: RunId = RunId(7);
const RETRY_RUN_ID: RunId = RunId(8);
const FOREIGN_RUN_ID: RunId = RunId(9);
const NO_FAILURES: usize = 0;
const ONE_FAILURE: usize = 1;

fn plan(schedule_id: &str, due_epoch_ms: i64) -> DueRunPlan {
    DueRunPlan {
        schedule_id: schedule_id.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        due_epoch_ms,
        next_run_epoch_ms: NEXT_RUN_EPOCH_MS.max(due_epoch_ms.saturating_add(1)),
    }
}

fn pending_occurrence() -> lattice::scheduler::ScheduleOccurrence {
    plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("valid occurrence plans")
        .occurrence(INITIAL_RUN_ID)
}

// r[verify lattice.scheduling.scheduler_occurrence_admission]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.atomic]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.ordering]
#[test]
fn occurrence_identity_is_deterministic_and_slot_scoped() {
    let request =
        plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION).expect("valid occurrence plans");
    let same =
        plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION).expect("same occurrence plans");
    let other_schedule = plan_schedule_occurrence(&plan(OTHER_SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("other schedule plans");
    let other_due = plan_schedule_occurrence(&plan(SCHEDULE_ID, OTHER_DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("other due slot plans");
    let occurrence = request.occurrence(INITIAL_RUN_ID);

    assert_eq!(occurrence.identity, same.identity);
    assert_ne!(occurrence.identity, other_schedule.identity);
    assert_ne!(occurrence.identity, other_due.identity);
    assert!(occurrence.identity.starts_with("b3:"));
    assert_eq!(occurrence.status, ScheduleOccurrenceStatus::PendingUnobserved);
    assert_eq!(occurrence.attempt_run_ids, vec![INITIAL_RUN_ID]);
    assert_eq!(validate_schedule_occurrence(&occurrence), Ok(()));
}

// r[verify lattice.scheduling.scheduler_occurrence_admission]
#[test]
fn malformed_occurrence_inputs_fail_closed() {
    let mut empty_schedule = plan(SCHEDULE_ID, DUE_EPOCH_MS);
    empty_schedule.schedule_id.clear();
    let mut empty_workflow = plan(SCHEDULE_ID, DUE_EPOCH_MS);
    empty_workflow.workflow_name.clear();
    let mut invalid_next = plan(SCHEDULE_ID, DUE_EPOCH_MS);
    invalid_next.next_run_epoch_ms = invalid_next.due_epoch_ms;

    assert_eq!(
        plan_schedule_occurrence(&empty_schedule, WORKFLOW_REVISION),
        Err(ScheduleOccurrenceError::EmptyScheduleId)
    );
    assert_eq!(
        plan_schedule_occurrence(&empty_workflow, WORKFLOW_REVISION),
        Err(ScheduleOccurrenceError::EmptyWorkflowName)
    );
    assert_eq!(
        plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), ""),
        Err(ScheduleOccurrenceError::EmptyWorkflowRevision)
    );
    assert_eq!(
        plan_schedule_occurrence(&invalid_next, WORKFLOW_REVISION),
        Err(ScheduleOccurrenceError::InvalidNextRun)
    );

    let mut tampered = pending_occurrence();
    tampered.identity = "b3:tampered".to_string();
    assert_eq!(validate_schedule_occurrence(&tampered), Err(ScheduleOccurrenceError::IdentityMismatch));
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
#[test]
fn exact_terminal_owner_settles_and_absent_record_remains_pending() {
    let occurrence = pending_occurrence();
    let pending = classify_schedule_occurrence_recovery(&occurrence, None).expect("missing run remains pending");
    let succeeded = classify_schedule_occurrence_recovery(
        &occurrence,
        Some(ScheduleOccurrenceRunObservation {
            run_id: INITIAL_RUN_ID,
            owner: ScheduleOccurrenceRunOwnerObservation::Exact,
            finished: true,
            failures: NO_FAILURES,
        }),
    )
    .expect("exact terminal run classifies");

    assert_eq!(pending, ScheduleOccurrenceRecoveryDecision::PendingUnobserved { run_id: INITIAL_RUN_ID });
    assert_eq!(succeeded, ScheduleOccurrenceRecoveryDecision::Settle {
        run_id: INITIAL_RUN_ID,
        status: ScheduleOccurrenceStatus::Succeeded,
    });
    let settled = settle_schedule_occurrence(&occurrence, INITIAL_RUN_ID, ScheduleOccurrenceStatus::Failed)
        .expect("terminal failure settles");
    assert_eq!(settled.status, ScheduleOccurrenceStatus::Failed);
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
#[test]
fn foreign_mismatched_and_nonterminal_recovery_evidence_fails_closed() {
    let occurrence = pending_occurrence();
    let observation = |run_id, owner, finished| ScheduleOccurrenceRunObservation {
        run_id,
        owner,
        finished,
        failures: ONE_FAILURE,
    };

    assert_eq!(
        classify_schedule_occurrence_recovery(
            &occurrence,
            Some(observation(INITIAL_RUN_ID, ScheduleOccurrenceRunOwnerObservation::Foreign, true,)),
        ),
        Err(ScheduleOccurrenceError::RecoveryOwnerMismatch)
    );
    assert_eq!(
        classify_schedule_occurrence_recovery(
            &occurrence,
            Some(observation(INITIAL_RUN_ID, ScheduleOccurrenceRunOwnerObservation::Unscoped, true,)),
        ),
        Err(ScheduleOccurrenceError::RecoveryOwnerMismatch)
    );
    assert_eq!(
        classify_schedule_occurrence_recovery(
            &occurrence,
            Some(observation(FOREIGN_RUN_ID, ScheduleOccurrenceRunOwnerObservation::Exact, true,)),
        ),
        Err(ScheduleOccurrenceError::RecoveryRunMismatch)
    );
    assert_eq!(
        classify_schedule_occurrence_recovery(
            &occurrence,
            Some(observation(INITIAL_RUN_ID, ScheduleOccurrenceRunOwnerObservation::Exact, false,)),
        ),
        Err(ScheduleOccurrenceError::RecoveryRunNotTerminal)
    );
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.retry]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.abandon]
#[test]
fn retry_and_abandon_require_explicit_bounded_acknowledgement() {
    let occurrence = pending_occurrence();

    assert_eq!(
        retry_schedule_occurrence(&occurrence, WORKFLOW_REVISION, RETRY_RUN_ID, false),
        Err(ScheduleOccurrenceError::RetryAcknowledgementRequired)
    );
    assert_eq!(
        retry_schedule_occurrence(&occurrence, OTHER_WORKFLOW_REVISION, RETRY_RUN_ID, true),
        Err(ScheduleOccurrenceError::WorkflowRevisionMismatch)
    );
    let retried = retry_schedule_occurrence(&occurrence, WORKFLOW_REVISION, RETRY_RUN_ID, true)
        .expect("acknowledged exact revision retries once");
    assert_eq!(retried.attempt_run_ids, vec![INITIAL_RUN_ID, RETRY_RUN_ID]);
    assert_eq!(retried.attempt_run_ids.len(), MAX_SCHEDULE_OCCURRENCE_ATTEMPTS);
    assert_eq!(
        retry_schedule_occurrence(&retried, WORKFLOW_REVISION, FOREIGN_RUN_ID, true),
        Err(ScheduleOccurrenceError::RetryLimitExhausted)
    );

    assert_eq!(
        abandon_schedule_occurrence(&occurrence, false),
        Err(ScheduleOccurrenceError::AbandonAcknowledgementRequired)
    );
    let abandoned = abandon_schedule_occurrence(&occurrence, true).expect("acknowledged occurrence abandons");
    assert_eq!(abandoned.status, ScheduleOccurrenceStatus::AbandonedUnknown);
    assert_eq!(
        settle_schedule_occurrence(&abandoned, INITIAL_RUN_ID, ScheduleOccurrenceStatus::Succeeded),
        Err(ScheduleOccurrenceError::NotPending)
    );
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.non_claim]
#[test]
fn occurrence_status_uses_local_unknown_language() {
    let rendered = lattice::scheduler::render_schedule_occurrence(&pending_occurrence());

    assert!(rendered.contains("PendingUnobserved"));
    assert!(!rendered.contains("exactly-once"));
    assert!(!rendered.contains(WORKFLOW_NAME));
}
