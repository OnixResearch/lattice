mod common;

use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::scheduler::DueRunPlan;
use lattice::scheduler::MAX_SCHEDULE_OCCURRENCE_ATTEMPTS;
use lattice::scheduler::MissedRunPolicy;
use lattice::scheduler::OccurrenceError as ScheduleOccurrenceError;
use lattice::scheduler::OverlapPolicy;
use lattice::scheduler::RecurrenceRule;
use lattice::scheduler::ScheduleDefinition;
use lattice::scheduler::ScheduleOccurrenceClaimResolution;
use lattice::scheduler::ScheduleOccurrenceRecoveryDecision;
use lattice::scheduler::ScheduleOccurrenceRunObservation;
use lattice::scheduler::ScheduleOccurrenceRunOwnerObservation;
use lattice::scheduler::ScheduleOccurrenceStatus;
use lattice::scheduler::abandon_schedule_occurrence;
use lattice::scheduler::classify_schedule_occurrence_recovery;
use lattice::scheduler::plan_schedule_occurrence;
use lattice::scheduler::retry_schedule_occurrence;
use lattice::scheduler::settle_schedule_occurrence;
use lattice::scheduler::shell::ScheduleOccurrenceRecoveryOutcome;
use lattice::scheduler::shell::abandon_occurrence;
use lattice::scheduler::shell::claim_due_occurrence;
use lattice::scheduler::shell::execute_occurrence;
use lattice::scheduler::shell::recover_occurrence;
use lattice::scheduler::shell::retry_occurrence;
use lattice::scheduler::validate_schedule_occurrence;
use lattice::scheduler::validate_schedule_occurrence_claim;
use lattice::store::RunOwner;
use lattice::store::RunRecord;

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
const INTERVAL_SECONDS: u64 = 1;
const CLAIM_CONTENDER_COUNT: usize = 8;
const EXPECTED_INSERTED_COUNT: usize = 1;
const EXPECTED_EXISTING_COUNT: usize = CLAIM_CONTENDER_COUNT - EXPECTED_INSERTED_COUNT;
const EXPECTED_OCCURRENCE_COUNT: usize = 1;
const OCCURRENCE_LIST_LIMIT: u64 = 1;
const RUN_STARTED_EPOCH_MS: i64 = 10;
const RUN_FINISHED_EPOCH_MS: i64 = 20;
const ZERO_NODE_COUNT: usize = 0;
const ZERO_CACHE_HITS: usize = 0;
const RUN_ID_BEFORE_EXHAUSTION: RunId = RunId(u64::MAX - 1);
const MISALIGNED_NEXT_RUN_EPOCH_MS: i64 = 1_500;
const CHANGED_WORKFLOW_COMMAND: &str = "printf changed-workflow";

fn plan(schedule_id: &str, due_epoch_ms: i64) -> DueRunPlan {
    DueRunPlan {
        schedule_id: schedule_id.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        due_epoch_ms,
        next_run_epoch_ms: NEXT_RUN_EPOCH_MS.max(due_epoch_ms.saturating_add(1)),
    }
}

fn schedule(next_run_epoch_ms: i64) -> ScheduleDefinition {
    ScheduleDefinition {
        id: SCHEDULE_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        recurrence: RecurrenceRule::EverySeconds {
            interval_secs: INTERVAL_SECONDS,
        },
        enabled: true,
        overlap_policy: OverlapPolicy::Forbid,
        missed_run_policy: MissedRunPolicy::RunOnce,
        next_run_epoch_ms,
    }
}

fn run_record(run_id: RunId, owner: RunOwner, failures: usize) -> RunRecord {
    RunRecord {
        run_id,
        owner,
        workflow_name: WORKFLOW_NAME.to_string(),
        started_epoch_ms: RUN_STARTED_EPOCH_MS,
        finished_epoch_ms: Some(RUN_FINISHED_EPOCH_MS),
        node_count: ZERO_NODE_COUNT,
        cache_hits: ZERO_CACHE_HITS,
        failures,
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

    let request = plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("claim validation request plans");
    let mut disabled = schedule(DUE_EPOCH_MS);
    disabled.enabled = false;
    let mut mismatched = schedule(DUE_EPOCH_MS);
    mismatched.workflow_name = OTHER_SCHEDULE_ID.to_string();
    let mut misaligned = request.clone();
    misaligned.next_run_epoch_ms = MISALIGNED_NEXT_RUN_EPOCH_MS;
    assert_eq!(
        validate_schedule_occurrence_claim(&disabled, &request),
        Err(ScheduleOccurrenceError::ScheduleDisabled)
    );
    assert_eq!(
        validate_schedule_occurrence_claim(&mismatched, &request),
        Err(ScheduleOccurrenceError::ScheduleMismatch)
    );
    assert_eq!(
        validate_schedule_occurrence_claim(&schedule(DUE_EPOCH_MS), &misaligned),
        Err(ScheduleOccurrenceError::NextRunMisaligned)
    );
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

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.atomic]
#[test]
fn redb_claim_race_selects_one_occurrence_and_advances_once() {
    let (store, _dir) = common::temp_store();
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("due schedule saves");
    let store = Arc::new(store);
    let request = Arc::new(
        plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION).expect("race occurrence plans"),
    );
    let barrier = Arc::new(Barrier::new(CLAIM_CONTENDER_COUNT));
    let mut contenders = Vec::with_capacity(CLAIM_CONTENDER_COUNT);
    for _ in 0..CLAIM_CONTENDER_COUNT {
        let store = Arc::clone(&store);
        let request = Arc::clone(&request);
        let barrier = Arc::clone(&barrier);
        contenders.push(thread::spawn(move || {
            barrier.wait();
            store.claim_schedule_occurrence(&request).expect("occurrence contender resolves")
        }));
    }
    let resolutions = contenders
        .into_iter()
        .map(|contender| contender.join().expect("occurrence contender joins"))
        .collect::<Vec<_>>();
    let inserted = resolutions
        .iter()
        .filter_map(|resolution| match resolution {
            ScheduleOccurrenceClaimResolution::Inserted(occurrence) => Some(occurrence.active_run_id),
            ScheduleOccurrenceClaimResolution::Existing(_)
            | ScheduleOccurrenceClaimResolution::StaleSchedule { .. }
            | ScheduleOccurrenceClaimResolution::MissingSchedule => None,
        })
        .collect::<Vec<_>>();
    let existing = resolutions
        .iter()
        .filter_map(|resolution| match resolution {
            ScheduleOccurrenceClaimResolution::Existing(occurrence) => Some(occurrence.active_run_id),
            ScheduleOccurrenceClaimResolution::Inserted(_)
            | ScheduleOccurrenceClaimResolution::StaleSchedule { .. }
            | ScheduleOccurrenceClaimResolution::MissingSchedule => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(inserted.len(), EXPECTED_INSERTED_COUNT);
    assert_eq!(existing.len(), EXPECTED_EXISTING_COUNT);
    assert!(existing.iter().all(|run_id| *run_id == inserted[0]));
    let advanced = store
        .load_schedule(SCHEDULE_ID)
        .expect("advanced schedule loads")
        .expect("advanced schedule exists");
    assert_eq!(advanced.next_run_epoch_ms, NEXT_RUN_EPOCH_MS);
    let occurrences = store.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).expect("occurrences list");
    assert_eq!(occurrences.len(), EXPECTED_OCCURRENCE_COUNT);
    assert_eq!(occurrences[0].active_run_id, inserted[0]);
    assert_eq!(store.next_run_id().expect("following run reserves").0, inserted[0].0 + 1);
}

// r[verify lattice.persistence_uncertainty.commit_outcomes]
// r[verify lattice.persistence_uncertainty.quarantine]
// r[verify lattice.persistence_uncertainty.reconciliation]
// r[verify lattice.persistence_uncertainty.effect_gate]
// r[verify lattice.persistence_uncertainty.validation]
#[test]
#[doc = "r[verify lattice.persistence_uncertainty.operator_actions.scenario.retry_requires_acknowledgement]"]
fn uncertain_claim_commit_quarantines_until_exact_readback() {
    let (published_store, _published_dir) = common::temp_store();
    published_store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("published fault schedule saves");
    let request = plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("uncertain occurrence plans");
    // r[verify lattice.persistence_uncertainty.commit_outcomes.scenario.uncertain]
    assert!(matches!(
        published_store.claim_schedule_occurrence_with_fault(&request, lattice::store::CommitFault::ObservationLost,),
        Err(lattice::store::StoreError::PublicationUnknown)
    ));
    assert!(published_store.is_quarantined().expect("quarantine status reads"));
    // r[verify lattice.persistence_uncertainty.quarantine.scenario.workflow_command_rejected]
    assert!(matches!(
        published_store.claim_schedule_occurrence(&request),
        Err(lattice::store::StoreError::Quarantined)
    ));
    // r[verify lattice.persistence_uncertainty.reconciliation.scenario.match_publishes]
    assert_eq!(
        published_store.reconcile_schedule_occurrence_publication().expect("exact successor reconciles"),
        transactional_reconciliation_core::PersistenceState::Published
    );
    assert!(!published_store.is_quarantined().expect("cleared status reads"));
    assert!(
        published_store
            .load_schedule_occurrence(&request.identity)
            .expect("reconciled occurrence loads")
            .is_some()
    );

    let (missing_store, _missing_dir) = common::temp_store();
    missing_store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("missing fault schedule saves");
    assert!(matches!(
        missing_store
            .claim_schedule_occurrence_with_fault(&request, lattice::store::CommitFault::CommitOutcomeUnknown,),
        Err(lattice::store::StoreError::PublicationUnknown)
    ));
    assert_eq!(
        missing_store.reconcile_schedule_occurrence_publication().expect("missing row stays classified"),
        transactional_reconciliation_core::PersistenceState::PublicationUnknown
    );
    assert!(missing_store.is_quarantined().expect("missing quarantine reads"));
    assert!(matches!(
        missing_store.retry_schedule_occurrence(&request.identity, WORKFLOW_REVISION, false),
        Err(lattice::store::StoreError::Conflict(_))
    ));
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.atomic]
#[test]
fn missing_stale_and_exhausted_claims_do_not_mutate_schedule_state() {
    let (missing_store, _missing_dir) = common::temp_store();
    let request = plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("missing occurrence plans");
    assert_eq!(
        missing_store.claim_schedule_occurrence(&request).expect("missing schedule resolves"),
        ScheduleOccurrenceClaimResolution::MissingSchedule
    );
    assert!(missing_store.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).unwrap().is_empty());

    let (stale_store, _stale_dir) = common::temp_store();
    stale_store.save_schedule(&schedule(OTHER_DUE_EPOCH_MS)).expect("stale schedule saves");
    assert_eq!(
        stale_store.claim_schedule_occurrence(&request).expect("stale schedule resolves"),
        ScheduleOccurrenceClaimResolution::StaleSchedule {
            current_next_run_epoch_ms: OTHER_DUE_EPOCH_MS,
        }
    );
    assert_eq!(stale_store.load_schedule(SCHEDULE_ID).unwrap().unwrap().next_run_epoch_ms, OTHER_DUE_EPOCH_MS);

    let (exhausted_store, _exhausted_dir) = common::temp_store();
    exhausted_store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("exhaustion schedule saves");
    exhausted_store
        .save_run(&run_record(RUN_ID_BEFORE_EXHAUSTION, RunOwner::Unscoped, NO_FAILURES))
        .expect("run before exhaustion saves");
    let error = exhausted_store.claim_schedule_occurrence(&request).expect_err("run id exhaustion rejects claim");
    assert!(error.to_string().contains("exhausted"));
    assert_eq!(exhausted_store.load_schedule(SCHEDULE_ID).unwrap().unwrap().next_run_epoch_ms, DUE_EPOCH_MS);
    assert!(exhausted_store.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).unwrap().is_empty());
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.retry]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.abandon]
#[test]
fn durable_transitions_require_missing_active_run_and_remain_bounded() {
    let (store, _dir) = common::temp_store();
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("transition schedule saves");
    let request = plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION)
        .expect("transition occurrence plans");
    let ScheduleOccurrenceClaimResolution::Inserted(initial) =
        store.claim_schedule_occurrence(&request).expect("transition occurrence claims")
    else {
        panic!("first transition claim must insert");
    };

    assert!(
        store
            .retry_schedule_occurrence(&initial.identity, WORKFLOW_REVISION, false)
            .expect_err("unacknowledged retry rejects")
            .to_string()
            .contains("acknowledgement")
    );
    let retried = store
        .retry_schedule_occurrence(&initial.identity, WORKFLOW_REVISION, true)
        .expect("acknowledged retry reserves once");
    assert_eq!(retried.attempt_run_ids.len(), MAX_SCHEDULE_OCCURRENCE_ATTEMPTS);
    assert!(
        store
            .retry_schedule_occurrence(&initial.identity, WORKFLOW_REVISION, true)
            .expect_err("second explicit retry rejects")
            .to_string()
            .contains("exhausted")
    );
    let abandoned = store
        .abandon_schedule_occurrence(&initial.identity, true)
        .expect("pending retried occurrence abandons");
    assert_eq!(abandoned.status, ScheduleOccurrenceStatus::AbandonedUnknown);

    let (observed_store, _observed_dir) = common::temp_store();
    observed_store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("observed schedule saves");
    let ScheduleOccurrenceClaimResolution::Inserted(observed) =
        observed_store.claim_schedule_occurrence(&request).expect("observed occurrence claims")
    else {
        panic!("first observed claim must insert");
    };
    observed_store
        .save_run(&run_record(
            observed.active_run_id,
            RunOwner::ScheduleOccurrence {
                identity: observed.identity.clone(),
            },
            NO_FAILURES,
        ))
        .expect("owned terminal run saves");
    assert!(
        observed_store
            .retry_schedule_occurrence(&observed.identity, WORKFLOW_REVISION, true)
            .expect_err("observed run blocks retry")
            .to_string()
            .contains("recover")
    );
    assert!(
        observed_store
            .abandon_schedule_occurrence(&observed.identity, true)
            .expect_err("observed run blocks abandonment")
            .to_string()
            .contains("recover")
    );
    let settled = observed_store
        .settle_schedule_occurrence(&observed.identity, observed.active_run_id, ScheduleOccurrenceStatus::Succeeded)
        .expect("exact observed occurrence settles");
    assert_eq!(settled.status, ScheduleOccurrenceStatus::Succeeded);
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.ordering]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
#[tokio::test]
async fn shell_claims_before_execution_and_settles_exact_owned_run() {
    let (store, _dir) = common::temp_store();
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("workflow saves");
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("shell schedule saves");
    let ScheduleOccurrenceClaimResolution::Inserted(occurrence) =
        claim_due_occurrence(&store, &plan(SCHEDULE_ID, DUE_EPOCH_MS)).expect("shell occurrence claims")
    else {
        panic!("first shell occurrence must insert");
    };

    assert_eq!(store.load_schedule(SCHEDULE_ID).unwrap().unwrap().next_run_epoch_ms, NEXT_RUN_EPOCH_MS);
    assert!(store.load_run(occurrence.active_run_id).expect("pre-effect run loads").is_none());
    let summary = execute_occurrence(&store, &occurrence, lattice::agent_node::AgentRuntimeConfig::default())
        .await
        .expect("owned occurrence executes");
    let settled = store
        .load_schedule_occurrence(&occurrence.identity)
        .expect("settled occurrence loads")
        .expect("settled occurrence exists");
    let run = store.load_run(summary.run_id).expect("owned run loads").expect("owned run exists");

    assert_eq!(settled.status, ScheduleOccurrenceStatus::Succeeded);
    assert_eq!(run.run_id, occurrence.active_run_id);
    assert_eq!(run.owner, RunOwner::ScheduleOccurrence {
        identity: occurrence.identity.clone(),
    });
    assert!(matches!(
        recover_occurrence(&store, &occurrence.identity).expect("terminal occurrence recovers idempotently"),
        ScheduleOccurrenceRecoveryOutcome::AlreadyTerminal { .. }
    ));
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.ordering]
#[tokio::test]
async fn shell_rejects_changed_workflow_revision_before_effects() {
    let (store, _dir) = common::temp_store();
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("initial workflow saves");
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("revision schedule saves");
    let ScheduleOccurrenceClaimResolution::Inserted(occurrence) =
        claim_due_occurrence(&store, &plan(SCHEDULE_ID, DUE_EPOCH_MS)).expect("revision occurrence claims")
    else {
        panic!("first revision occurrence must insert");
    };
    let mut changed = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    changed.add_shell_node("changed", CHANGED_WORKFLOW_COMMAND);
    store.save_workflow(&changed).expect("changed workflow saves");

    let error = execute_occurrence(&store, &occurrence, lattice::agent_node::AgentRuntimeConfig::default())
        .await
        .expect_err("changed workflow revision rejects before execution");

    assert!(error.to_string().contains("revision differs"));
    assert!(store.load_run(occurrence.active_run_id).expect("changed run lookup succeeds").is_none());
    assert_eq!(
        store.load_schedule_occurrence(&occurrence.identity).unwrap().unwrap().status,
        ScheduleOccurrenceStatus::PendingUnobserved
    );
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
#[test]
fn shell_recovery_rejects_legacy_unscoped_run_evidence() {
    let (store, _dir) = common::temp_store();
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("workflow saves");
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("legacy schedule saves");
    let ScheduleOccurrenceClaimResolution::Inserted(occurrence) =
        claim_due_occurrence(&store, &plan(SCHEDULE_ID, DUE_EPOCH_MS)).expect("legacy occurrence claims")
    else {
        panic!("first legacy occurrence must insert");
    };
    store
        .save_run(&run_record(occurrence.active_run_id, RunOwner::Unscoped, NO_FAILURES))
        .expect("legacy unscoped run saves");

    let error = recover_occurrence(&store, &occurrence.identity).expect_err("unscoped evidence rejects");
    assert!(error.to_string().contains("not owned"));
    assert_eq!(
        store.load_schedule_occurrence(&occurrence.identity).unwrap().unwrap().status,
        ScheduleOccurrenceStatus::PendingUnobserved
    );
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.retry]
#[tokio::test]
async fn shell_retry_is_explicit_fresh_and_terminal() {
    let (store, _dir) = common::temp_store();
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("workflow saves");
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("retry shell schedule saves");
    let ScheduleOccurrenceClaimResolution::Inserted(initial) =
        claim_due_occurrence(&store, &plan(SCHEDULE_ID, DUE_EPOCH_MS)).expect("retry shell occurrence claims")
    else {
        panic!("first retry shell occurrence must insert");
    };

    assert!(
        retry_occurrence(&store, &initial.identity, false, lattice::agent_node::AgentRuntimeConfig::default(),)
            .await
            .expect_err("unacknowledged shell retry rejects")
            .to_string()
            .contains("acknowledgement")
    );
    let retried = retry_occurrence(&store, &initial.identity, true, lattice::agent_node::AgentRuntimeConfig::default())
        .await
        .expect("acknowledged shell retry executes");

    assert_eq!(retried.status, ScheduleOccurrenceStatus::Succeeded);
    assert_eq!(retried.attempt_run_ids.len(), MAX_SCHEDULE_OCCURRENCE_ATTEMPTS);
    assert_ne!(retried.active_run_id, initial.active_run_id);
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.abandon]
#[test]
fn shell_abandon_closes_without_creating_run_evidence() {
    let (store, _dir) = common::temp_store();
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("workflow saves");
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("abandon shell schedule saves");
    let ScheduleOccurrenceClaimResolution::Inserted(initial) =
        claim_due_occurrence(&store, &plan(SCHEDULE_ID, DUE_EPOCH_MS)).expect("abandon shell occurrence claims")
    else {
        panic!("first abandon shell occurrence must insert");
    };

    assert!(
        abandon_occurrence(&store, &initial.identity, false)
            .expect_err("unacknowledged shell abandonment rejects")
            .to_string()
            .contains("acknowledgement")
    );
    let abandoned = abandon_occurrence(&store, &initial.identity, true).expect("acknowledged shell abandonment closes");

    assert_eq!(abandoned.status, ScheduleOccurrenceStatus::AbandonedUnknown);
    assert!(store.load_run(initial.active_run_id).expect("abandoned run lookup succeeds").is_none());
}

#[tokio::test]
async fn reopened_pending_reservation_blocks_allowed_effect_until_acknowledged_retry() {
    let directory = tempfile::tempdir().expect("temporary store");
    let path = directory.path().join("repair-effects.redb");
    let marker = directory.path().join("effect-marker");
    let store = lattice::store::Store::new(&path).expect("store");
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node("observable effect", format!("printf executed > '{}'", marker.display()));
    store.save_workflow(&workflow).expect("workflow persists");
    store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("schedule persists");
    let ScheduleOccurrenceClaimResolution::Inserted(first) =
        claim_due_occurrence(&store, &plan(SCHEDULE_ID, DUE_EPOCH_MS)).expect("first claim")
    else {
        panic!("fresh claim must insert")
    };
    let authority = common::authority::TestRunContextProvider::new();
    let completed = lattice::scheduler::shell::execute_occurrence_with_authority_provider(
        &store,
        &first,
        lattice::agent_node::AgentRuntimeConfig::default(),
        &authority,
    )
    .await
    .expect("current admitted authority permits the actual shell effect");
    assert_eq!(completed.status, lattice::headless::HeadlessRunStatus::Succeeded);
    assert_eq!(std::fs::read(&marker).expect("first effect"), b"executed");
    std::fs::remove_file(&marker).expect("clear observed effect");

    let mut second_schedule = schedule(DUE_EPOCH_MS);
    second_schedule.id = "reopened-pending".to_string();
    store.save_schedule(&second_schedule).expect("second schedule");
    let ScheduleOccurrenceClaimResolution::Inserted(pending) =
        claim_due_occurrence(&store, &plan("reopened-pending", DUE_EPOCH_MS)).expect("pending claim")
    else {
        panic!("fresh pending claim must insert")
    };
    drop(store);
    lattice::run::shell::repair_path(&path).expect("structural repair/open");
    let store = lattice::store::Store::new(&path).expect("reopened store");
    assert!(store.is_quarantined().expect("durable fence"));
    // r[verify lattice.persistence_uncertainty.repair_boundary.scenario.records_stay_blocked]
    assert!(matches!(
        lattice::scheduler::shell::execute_occurrence_with_authority_provider(
            &store,
            &pending,
            lattice::agent_node::AgentRuntimeConfig::default(),
            &authority,
        )
        .await,
        Err(lattice::scheduler::shell::Error::Store(lattice::store::StoreError::Quarantined))
    ));
    assert!(!marker.exists());
    assert!(store.load_run(pending.active_run_id).expect("run remains absent").is_none());
    assert_eq!(
        store.reconcile_schedule_occurrence_publication().expect("readback cannot clear repair fence"),
        transactional_reconciliation_core::PersistenceState::PublicationUnknown
    );
    assert!(store.retry_schedule_occurrence(&pending.identity, &pending.workflow_revision, false).is_err());
    let retry = store
        .retry_schedule_occurrence(&pending.identity, &pending.workflow_revision, true)
        .expect("acknowledged fresh reservation closes uncertainty");
    assert_ne!(retry.active_run_id, pending.active_run_id);
    let completed = lattice::scheduler::shell::execute_occurrence_with_authority_provider(
        &store,
        &retry,
        lattice::agent_node::AgentRuntimeConfig::default(),
        &authority,
    )
    .await
    .expect("explicit fresh reservation can dispatch");
    assert_eq!(completed.status, lattice::headless::HeadlessRunStatus::Succeeded);
    assert_eq!(std::fs::read(marker).expect("explicit retry effect"), b"executed");
}

#[test]
fn reopened_fence_requires_exact_terminal_run_and_ack_does_not_bypass_ownership() {
    for exact_owner in [false, true] {
        let directory = tempfile::tempdir().expect("temporary store");
        let path = directory.path().join("terminal-repair.redb");
        let store = lattice::store::Store::new(&path).expect("store");
        store.save_schedule(&schedule(DUE_EPOCH_MS)).expect("schedule");
        let request = plan_schedule_occurrence(&plan(SCHEDULE_ID, DUE_EPOCH_MS), WORKFLOW_REVISION).expect("request");
        let ScheduleOccurrenceClaimResolution::Inserted(pending) =
            store.claim_schedule_occurrence(&request).expect("claim")
        else {
            panic!("fresh claim must insert")
        };
        let owner = if exact_owner {
            RunOwner::ScheduleOccurrence {
                identity: pending.identity.clone(),
            }
        } else {
            RunOwner::Unscoped
        };
        store.save_run(&run_record(pending.active_run_id, owner, NO_FAILURES)).expect("terminal evidence");
        drop(store);
        let store = lattice::store::Store::new(&path).expect("reopened store");
        assert!(store.is_quarantined().expect("fence"));
        assert!(store.retry_schedule_occurrence(&pending.identity, WORKFLOW_REVISION, true).is_err());
        assert!(store.abandon_schedule_occurrence(&pending.identity, true).is_err());
        let recovery = recover_occurrence(&store, &pending.identity);
        if exact_owner {
            assert!(matches!(recovery, Ok(ScheduleOccurrenceRecoveryOutcome::Reconciled { .. })));
            assert!(!store.is_quarantined().expect("exact terminal evidence closes fence"));
        } else {
            assert!(recovery.is_err());
            assert!(store.is_quarantined().expect("foreign ownership stays blocked"));
        }
    }
}
