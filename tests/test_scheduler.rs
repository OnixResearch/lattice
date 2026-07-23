//! Tests for schedule persistence, pure due-run evaluation, and daemon run planning.

use std::collections::HashSet;
use std::process::Command;

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
const CLI_CONFIG_FILE_NAME: &str = "config.ncl";
const CLI_DATA_DIR_NAME: &str = "data";
const STORE_FILE_NAME: &str = "lattice.redb";
const CLI_DEFAULT_TIMEOUT_SECS: u64 = 60;
const CLI_MAX_PARALLELISM: usize = 1;
const OPERATOR_E2E_SHELL_COMMAND: &str = "printf scheduler-e2e";
const FUTURE_SCHEDULE_ID: &str = "future";
const MISSING_SCHEDULE_ID: &str = "missing-workflow";
const RUN_HISTORY_LIMIT: usize = 10;
const EXPECTED_PERSISTED_RUNS: usize = 1;

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

fn write_cli_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join(CLI_DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).expect("operator E2E data directory creates");
    let config_path = temp_dir.path().join(CLI_CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {CLI_DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {CLI_MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string(),
        ),
    )
    .expect("operator E2E config writes");
    (config_path, data_dir)
}

fn scheduler_command(config_path: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    command
        .arg("--config")
        .arg(config_path)
        .arg("scheduler-daemon")
        .arg("--now-epoch-ms")
        .arg(DUE_EPOCH_MS.to_string());
    command
}

#[test]
fn operator_e2e_scheduler_daemon_runs_due_shell_and_isolates_missing_and_future_schedules() {
    let temp_dir = tempfile::tempdir().expect("temporary operator E2E directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("operator E2E store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    workflow.add_shell_node("scheduler", OPERATOR_E2E_SHELL_COMMAND);
    store.save_workflow(&workflow).expect("operator E2E workflow saves");
    let due = schedule(START_EPOCH_MS);
    let missing = ScheduleDefinition {
        id: MISSING_SCHEDULE_ID.to_string(),
        workflow_name: MISSING_WORKFLOW_NAME.to_string(),
        ..schedule(START_EPOCH_MS)
    };
    let future = ScheduleDefinition {
        id: FUTURE_SCHEDULE_ID.to_string(),
        next_run_epoch_ms: FUTURE_EPOCH_MS,
        ..schedule(FUTURE_EPOCH_MS)
    };
    for candidate in [&due, &missing, &future] {
        store.save_schedule(candidate).expect("operator E2E schedule saves");
    }
    drop(store);

    let output = scheduler_command(&config_path).output().expect("scheduler daemon process returns");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(
        stdout.contains(&format!("started schedule {SCHEDULE_ID} workflow {WORKFLOW_NAME}")),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!("schedule {MISSING_SCHEDULE_ID} workflow {MISSING_WORKFLOW_NAME} failed")),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains(FUTURE_SCHEDULE_ID), "future schedule unexpectedly ran: {stdout}");

    let reopened = Store::new(data_dir.join(STORE_FILE_NAME)).expect("operator E2E store reopens");
    let runs = reopened.list_runs(RUN_HISTORY_LIMIT).expect("operator E2E run history lists");
    let advanced_due = reopened.load_schedule(SCHEDULE_ID).expect("due schedule loads").expect("due schedule exists");
    let advanced_missing = reopened
        .load_schedule(MISSING_SCHEDULE_ID)
        .expect("missing-workflow schedule loads")
        .expect("missing-workflow schedule exists");
    let unchanged_future = reopened
        .load_schedule(FUTURE_SCHEDULE_ID)
        .expect("future schedule loads")
        .expect("future schedule exists");

    assert_eq!(runs.len(), EXPECTED_PERSISTED_RUNS);
    assert_eq!(runs[0].workflow_name, WORKFLOW_NAME);
    assert_eq!(runs[0].failures, SUMMARY_FAILURES);
    assert!(advanced_due.next_run_epoch_ms > DUE_EPOCH_MS);
    assert!(advanced_missing.next_run_epoch_ms > DUE_EPOCH_MS);
    assert_eq!(unchanged_future.next_run_epoch_ms, FUTURE_EPOCH_MS);
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
