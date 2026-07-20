//! Tests for schedule persistence, pure due-run evaluation, and daemon run planning.

use std::collections::HashSet;

use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::headless::HeadlessRunStatus;
use lattice::headless::HeadlessRunSummary;
use lattice::scheduler::DueRunPlan;
use lattice::scheduler::MissedRunPolicy;
use lattice::scheduler::OverlapPolicy;
use lattice::scheduler::RecurrenceRule;
use lattice::scheduler::ScheduleDaemonOutcome;
use lattice::scheduler::ScheduleDefinition;
use lattice::scheduler::ScheduleDiagnosticCode;
use lattice::scheduler::ScheduleSkipReason;
use lattice::scheduler::ScheduledWorkflowRunner;
use lattice::scheduler::evaluate_due_runs;
use lattice::scheduler::run_due_plans;
use lattice::scheduler::validate_schedule;
use lattice::store::Store;
use pretty_assertions::assert_eq;

const SCHEDULE_ID: &str = "nightly";
const OTHER_SCHEDULE_ID: &str = "hourly";
const WORKFLOW_NAME: &str = "demo";
const MISSING_WORKFLOW_NAME: &str = "missing";
const START_EPOCH_MS: i64 = 1_000;
const DUE_EPOCH_MS: i64 = 2_000;
const FUTURE_EPOCH_MS: i64 = 10_000;
const INTERVAL_SECS: u64 = 5;
const RUN_ID_VALUE: u64 = 7;
const SUMMARY_NODE_COUNT: usize = 1;
const SUMMARY_STEPS_RUN: usize = 1;
const SUMMARY_COMPLETED: usize = 1;
const SUMMARY_FAILURES: usize = 0;
const SUMMARY_CACHE_HITS: usize = 0;

#[derive(Default)]
struct FakeRunner {
    started: Vec<String>,
    fail_workflow: Option<String>,
}

impl ScheduledWorkflowRunner for FakeRunner {
    fn run_scheduled_workflow(&mut self, workflow_name: &str) -> Result<HeadlessRunSummary, String> {
        self.started.push(workflow_name.to_string());
        if self.fail_workflow.as_deref() == Some(workflow_name) {
            return Err("load failed".to_string());
        }
        Ok(HeadlessRunSummary {
            run_id: RunId(RUN_ID_VALUE),
            workflow_name: workflow_name.to_string(),
            status: HeadlessRunStatus::Succeeded,
            node_count: SUMMARY_NODE_COUNT,
            steps_run: SUMMARY_STEPS_RUN,
            completed: SUMMARY_COMPLETED,
            failures: SUMMARY_FAILURES,
            cache_hits: SUMMARY_CACHE_HITS,
        })
    }
}

fn schedule(next_run_epoch_ms: i64) -> ScheduleDefinition {
    ScheduleDefinition {
        id: SCHEDULE_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        recurrence: RecurrenceRule::EverySeconds {
            interval_secs: INTERVAL_SECS,
        },
        enabled: true,
        overlap_policy: OverlapPolicy::Forbid,
        missed_run_policy: MissedRunPolicy::RunOnce,
        next_run_epoch_ms,
    }
}

// r[verify n8nclone.scheduling.schedule_model]
// r[verify n8nclone.scheduling.schedule_model.save]
#[test]
fn store_persists_schedule_definitions() {
    let temp_dir = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temp_dir.path().join("scheduler.redb")).expect("store opens");
    let schedule = schedule(START_EPOCH_MS);

    store.save_schedule(&schedule).expect("schedule saved");

    assert_eq!(store.load_schedule(SCHEDULE_ID).expect("schedule loads"), Some(schedule.clone()));
    assert_eq!(store.list_schedules().expect("schedules list"), vec![schedule]);
}

// r[verify n8nclone.scheduling.schedule_model.missing_workflow]
#[test]
fn validation_rejects_schedule_for_missing_workflow() {
    let schedule = ScheduleDefinition {
        workflow_name: MISSING_WORKFLOW_NAME.to_string(),
        ..schedule(START_EPOCH_MS)
    };
    let workflows = HashSet::from([WORKFLOW_NAME.to_string()]);

    let diagnostics = validate_schedule(&schedule, &workflows);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == ScheduleDiagnosticCode::MissingWorkflow));
}

// r[verify n8nclone.scheduling.due_evaluator]
// r[verify n8nclone.scheduling.due_evaluator.due]
#[test]
fn due_evaluator_returns_plan_for_due_schedule() {
    let schedules = vec![schedule(START_EPOCH_MS)];

    let evaluation = evaluate_due_runs(&schedules, DUE_EPOCH_MS, &HashSet::new());

    assert_eq!(evaluation.due_runs.len(), SUMMARY_NODE_COUNT);
    assert_eq!(evaluation.due_runs[0].schedule_id, SCHEDULE_ID);
    assert_eq!(evaluation.due_runs[0].workflow_name, WORKFLOW_NAME);
    assert!(evaluation.due_runs[0].next_run_epoch_ms > DUE_EPOCH_MS);
}

// r[verify n8nclone.scheduling.due_evaluator.not_due]
#[test]
fn due_evaluator_returns_no_plan_for_future_schedule() {
    let schedules = vec![schedule(FUTURE_EPOCH_MS)];

    let evaluation = evaluate_due_runs(&schedules, DUE_EPOCH_MS, &HashSet::new());

    assert!(evaluation.due_runs.is_empty());
}

// r[verify n8nclone.scheduling.overlap_policy]
// r[verify n8nclone.scheduling.overlap_policy.prevent]
#[test]
fn overlap_policy_prevents_second_run() {
    let schedules = vec![schedule(START_EPOCH_MS)];
    let running = HashSet::from([SCHEDULE_ID.to_string()]);

    let evaluation = evaluate_due_runs(&schedules, DUE_EPOCH_MS, &running);

    assert!(evaluation.due_runs.is_empty());
    assert_eq!(evaluation.skipped_runs[0].reason, ScheduleSkipReason::OverlapPrevented);
}

// r[verify n8nclone.scheduling.missed_runs]
// r[verify n8nclone.scheduling.missed_runs.restart]
#[test]
fn missed_run_skip_policy_advances_without_plan() {
    let mut missed = schedule(START_EPOCH_MS);
    missed.missed_run_policy = MissedRunPolicy::Skip;
    let schedules = vec![missed];

    let evaluation = evaluate_due_runs(&schedules, FUTURE_EPOCH_MS, &HashSet::new());

    assert!(evaluation.due_runs.is_empty());
    assert_eq!(evaluation.skipped_runs[0].reason, ScheduleSkipReason::MissedRunSkipped);
    assert!(evaluation.skipped_runs[0].next_run_epoch_ms > FUTURE_EPOCH_MS);
}

// r[verify n8nclone.scheduling.daemon.run_due]
// r[verify n8nclone.scheduling.daemon.run_due.start]
#[test]
fn daemon_runner_starts_due_workflow() {
    let plans = vec![DueRunPlan {
        schedule_id: SCHEDULE_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        due_epoch_ms: START_EPOCH_MS,
        next_run_epoch_ms: DUE_EPOCH_MS,
    }];
    let mut runner = FakeRunner::default();

    let outcomes = run_due_plans(&plans, &mut runner);

    assert_eq!(runner.started, vec![WORKFLOW_NAME.to_string()]);
    assert!(matches!(outcomes[0], ScheduleDaemonOutcome::Started { .. }));
}

// r[verify n8nclone.scheduling.daemon.run_due.load_failure]
#[test]
fn daemon_runner_reports_load_failure_without_blocking_other_schedules() {
    let plans = vec![
        DueRunPlan {
            schedule_id: SCHEDULE_ID.to_string(),
            workflow_name: MISSING_WORKFLOW_NAME.to_string(),
            due_epoch_ms: START_EPOCH_MS,
            next_run_epoch_ms: DUE_EPOCH_MS,
        },
        DueRunPlan {
            schedule_id: OTHER_SCHEDULE_ID.to_string(),
            workflow_name: WORKFLOW_NAME.to_string(),
            due_epoch_ms: START_EPOCH_MS,
            next_run_epoch_ms: DUE_EPOCH_MS,
        },
    ];
    let mut runner = FakeRunner {
        fail_workflow: Some(MISSING_WORKFLOW_NAME.to_string()),
        ..FakeRunner::default()
    };

    let outcomes = run_due_plans(&plans, &mut runner);

    assert!(matches!(outcomes[0], ScheduleDaemonOutcome::Error { .. }));
    assert!(matches!(outcomes[1], ScheduleDaemonOutcome::Started { .. }));
}

#[test]
fn validation_accepts_schedule_for_existing_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temp_dir.path().join("workflow-schedule.redb")).expect("store opens");
    store.save_workflow(&Workflow::new(WORKFLOW_NAME)).expect("workflow saved");
    let workflows = store.list_workflows().expect("workflows list").into_iter().collect::<HashSet<_>>();

    assert!(validate_schedule(&schedule(START_EPOCH_MS), &workflows).is_empty());
}
