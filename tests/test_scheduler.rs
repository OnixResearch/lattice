//! Tests for schedule persistence, pure due-run evaluation, and daemon run planning.

mod common;

use std::collections::HashSet;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

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
#[cfg(unix)]
const FORCED_DEATH_MARKER_FILE_NAME: &str = "scheduler-forced-death";
#[cfg(unix)]
const CRASH_SNAPSHOT_FILE_NAME: &str = "scheduler-crashed-state.redb";
#[cfg(unix)]
const RECOVERY_EFFECT_FILE_NAME: &str = "scheduler-recovery-effect";
#[cfg(unix)]
const RECOVERY_EFFECT: &str = "restarted-once";
#[cfg(unix)]
const CONCURRENT_READY_FILE_NAME: &str = "scheduler-concurrent-ready";
#[cfg(unix)]
const CONCURRENT_RELEASE_FILE_NAME: &str = "scheduler-concurrent-release";
#[cfg(unix)]
const CONCURRENT_EFFECT_FILE_NAME: &str = "scheduler-concurrent-effect";
#[cfg(unix)]
const CONCURRENT_EFFECT: &str = "single-owner";
#[cfg(unix)]
const CONCURRENT_GATE_POLL_ATTEMPTS: usize = 500;
#[cfg(unix)]
const CONCURRENT_GATE_POLL_DELAY_MS: u64 = 10;

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

#[cfg(unix)]
#[test]
fn operator_e2e_scheduler_restart_retries_due_run_after_forced_process_death() {
    let temp_dir = tempfile::tempdir().expect("temporary scheduler restart directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let marker_path = temp_dir.path().join(FORCED_DEATH_MARKER_FILE_NAME);
    let recovery_effect_path = temp_dir.path().join(RECOVERY_EFFECT_FILE_NAME);
    let store_path = data_dir.join(STORE_FILE_NAME);
    let store = Store::new(&store_path).expect("scheduler restart store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    workflow.add_shell_node(
        "scheduler-restart",
        common::forced_parent_death_once_then_write_command(&marker_path, &recovery_effect_path, RECOVERY_EFFECT),
    );
    store.save_workflow(&workflow).expect("scheduler restart workflow saves");
    store.save_schedule(&schedule(START_EPOCH_MS)).expect("scheduler restart schedule saves");
    drop(store);

    let crashed = scheduler_command(&config_path).output().expect("first scheduler process starts");

    assert!(!crashed.status.success(), "forced-death scheduler unexpectedly succeeded");
    assert!(crashed.status.code().is_none(), "forced-death scheduler was not terminated by a signal");
    assert_eq!(
        std::fs::read_to_string(&marker_path).expect("forced-death marker persists"),
        common::FORCED_DEATH_MARKER_CONTENT,
    );
    assert!(!recovery_effect_path.exists(), "recovery effect must not occur before restart");

    let crash_snapshot_path = temp_dir.path().join(CRASH_SNAPSHOT_FILE_NAME);
    let snapshot_bytes =
        std::fs::copy(&store_path, &crash_snapshot_path).expect("crashed scheduler store snapshot copies");
    assert!(snapshot_bytes > 0, "crashed scheduler store snapshot must not be empty");
    let after_crash = Store::new(&crash_snapshot_path).expect("crashed scheduler snapshot repairs and reopens");
    let unchanged = after_crash
        .load_schedule(SCHEDULE_ID)
        .expect("crashed schedule loads")
        .expect("crashed schedule remains");
    assert_eq!(unchanged.next_run_epoch_ms, START_EPOCH_MS);
    assert!(after_crash.list_runs(RUN_HISTORY_LIMIT).expect("crashed run history lists").is_empty());
    drop(after_crash);

    let restarted = scheduler_command(&config_path).output().expect("restarted scheduler process returns");
    let restarted_stdout = String::from_utf8_lossy(&restarted.stdout);
    let restarted_stderr = String::from_utf8_lossy(&restarted.stderr);

    assert!(restarted.status.success(), "stderr: {restarted_stderr}");
    assert!(
        restarted_stdout.contains(&format!("started schedule {SCHEDULE_ID} workflow {WORKFLOW_NAME}")),
        "stdout: {restarted_stdout}"
    );
    assert_eq!(std::fs::read_to_string(&recovery_effect_path).expect("recovery effect persists"), RECOVERY_EFFECT,);

    let recovered = Store::new(&store_path).expect("restarted scheduler store reopens");
    let runs = recovered.list_runs(RUN_HISTORY_LIMIT).expect("restarted run history lists");
    let advanced = recovered
        .load_schedule(SCHEDULE_ID)
        .expect("restarted schedule loads")
        .expect("restarted schedule remains");
    assert_eq!(runs.len(), EXPECTED_PERSISTED_RUNS);
    assert_eq!(runs[0].workflow_name, WORKFLOW_NAME);
    assert!(advanced.next_run_epoch_ms > DUE_EPOCH_MS);
}

#[cfg(unix)]
#[test]
fn operator_e2e_scheduler_concurrent_daemons_admit_one_store_owner_and_one_durable_run() {
    let temp_dir = tempfile::tempdir().expect("temporary concurrent scheduler directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let ready_path = temp_dir.path().join(CONCURRENT_READY_FILE_NAME);
    let release_path = temp_dir.path().join(CONCURRENT_RELEASE_FILE_NAME);
    let effect_path = temp_dir.path().join(CONCURRENT_EFFECT_FILE_NAME);
    let store_path = data_dir.join(STORE_FILE_NAME);
    let store = Store::new(&store_path).expect("concurrent scheduler store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    workflow.add_shell_node(
        "scheduler-concurrent",
        common::gated_write_command(&ready_path, &release_path, &effect_path, CONCURRENT_EFFECT),
    );
    store.save_workflow(&workflow).expect("concurrent scheduler workflow saves");
    store.save_schedule(&schedule(START_EPOCH_MS)).expect("concurrent scheduler schedule saves");
    drop(store);

    let first = scheduler_command(&config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("first concurrent scheduler starts");
    let mut first = common::ChildGuard::new(first);
    common::wait_for_path(
        &ready_path,
        CONCURRENT_GATE_POLL_ATTEMPTS,
        Duration::from_millis(CONCURRENT_GATE_POLL_DELAY_MS),
    );
    assert_eq!(
        std::fs::read_to_string(&ready_path).expect("concurrent scheduler ready marker reads"),
        common::PROCESS_GATE_READY_CONTENT,
    );
    assert!(
        first.child_mut().try_wait().expect("first concurrent scheduler remains observable").is_none(),
        "first concurrent scheduler must still own the store while gated"
    );

    let contender = scheduler_command(&config_path).output().expect("second concurrent scheduler returns");
    let contender_stderr = String::from_utf8_lossy(&contender.stderr);
    let normalized_contender_stderr = contender_stderr.to_ascii_lowercase();
    assert!(!contender.status.success(), "second concurrent scheduler unexpectedly acquired the store");
    assert!(contender_stderr.contains("Scheduler failed:"), "stderr: {contender_stderr}");
    assert!(
        normalized_contender_stderr.contains("already open") || normalized_contender_stderr.contains("acquire lock"),
        "second scheduler did not report active store ownership: {contender_stderr}"
    );
    assert!(!effect_path.exists(), "gated workflow effect must not occur before release");

    std::fs::write(&release_path, "release").expect("concurrent scheduler gate releases");
    let first_output = first.wait_with_output();
    let first_stdout = String::from_utf8_lossy(&first_output.stdout);
    let first_stderr = String::from_utf8_lossy(&first_output.stderr);
    assert!(first_output.status.success(), "stderr: {first_stderr}");
    assert!(
        first_stdout.contains(&format!("started schedule {SCHEDULE_ID} workflow {WORKFLOW_NAME}")),
        "stdout: {first_stdout}"
    );
    assert_eq!(std::fs::read_to_string(&effect_path).expect("concurrent scheduler effect reads"), CONCURRENT_EFFECT);

    let settled = Store::new(&store_path).expect("concurrent scheduler store reopens");
    let runs = settled.list_runs(RUN_HISTORY_LIMIT).expect("concurrent scheduler runs list");
    let advanced = settled
        .load_schedule(SCHEDULE_ID)
        .expect("concurrent scheduler schedule loads")
        .expect("concurrent scheduler schedule remains");
    assert_eq!(runs.len(), EXPECTED_PERSISTED_RUNS);
    assert!(advanced.next_run_epoch_ms > DUE_EPOCH_MS);
    drop(settled);

    let after_advance = scheduler_command(&config_path).output().expect("post-advance scheduler returns");
    assert!(after_advance.status.success(), "stderr: {}", String::from_utf8_lossy(&after_advance.stderr));
    assert!(!String::from_utf8_lossy(&after_advance.stdout).contains("started schedule"));
    let final_store = Store::new(&store_path).expect("final concurrent scheduler store reopens");
    assert_eq!(
        final_store.list_runs(RUN_HISTORY_LIMIT).expect("final concurrent scheduler runs list").len(),
        EXPECTED_PERSISTED_RUNS,
    );
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
