//! Tests for schedule persistence, pure due-run evaluation, and daemon run planning.

mod common;

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
const DENIED_PROCESS_FAILURES: usize = 1;
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
const EXPECTED_OCCURRENCE_COUNT: usize = 1;
const OCCURRENCE_LIST_LIMIT: u64 = 1;
#[cfg(unix)]
const FORCED_DEATH_MARKER_FILE_NAME: &str = "scheduler-forced-death";
#[cfg(unix)]
const RECOVERY_EFFECT_FILE_NAME: &str = "scheduler-recovery-effect";
#[cfg(unix)]
const RECOVERY_EFFECT: &str = "restarted-once";
#[cfg(unix)]
const POST_EFFECT_CHECKPOINT_FILE_NAME: &str = "scheduler-post-effect-checkpoint";
#[cfg(unix)]
const POST_EFFECT_ATTEMPTS_FILE_NAME: &str = "scheduler-post-effect-attempts";
#[cfg(unix)]
const POST_EFFECT_IDENTITY: &str = "scheduler-attempt";
#[cfg(unix)]
const CONCURRENT_READY_FILE_NAME: &str = "scheduler-concurrent-ready";
#[cfg(unix)]
const CONCURRENT_RELEASE_FILE_NAME: &str = "scheduler-concurrent-release";
#[cfg(unix)]
const CONCURRENT_EFFECT_FILE_NAME: &str = "scheduler-concurrent-effect";
#[cfg(unix)]
const CONCURRENT_EFFECT: &str = "single-owner";

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

fn scheduler_occurrences_command(config_path: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    command.arg("--config").arg(config_path).arg("scheduler-occurrences");
    command
}

fn scheduler_occurrence_command(config_path: &std::path::Path, subcommand: &str, identity: &str) -> Command {
    let mut command = scheduler_occurrences_command(config_path);
    command.arg(subcommand).arg(identity);
    command
}

#[test]
fn operator_e2e_scheduler_daemon_runs_due_shell_and_isolates_missing_and_future_schedules() {
    let temp_dir = tempfile::tempdir().expect("temporary operator E2E directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("operator E2E store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
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
    assert_eq!(runs[0].failures, DENIED_PROCESS_FAILURES);
    assert!(advanced_due.next_run_epoch_ms > DUE_EPOCH_MS);
    assert!(advanced_missing.next_run_epoch_ms > DUE_EPOCH_MS);
    assert_eq!(unchanged_future.next_run_epoch_ms, FUTURE_EPOCH_MS);
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
#[test]
fn scheduler_occurrence_cli_lists_shows_and_rejects_invalid_bounds() {
    let temp_dir = tempfile::tempdir().expect("temporary occurrence CLI directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let store_path = data_dir.join(STORE_FILE_NAME);
    let store = Store::new(&store_path).expect("occurrence CLI store opens");
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("occurrence CLI workflow saves");
    store.save_schedule(&schedule(START_EPOCH_MS)).expect("occurrence CLI schedule saves");
    drop(store);

    let daemon = scheduler_command(&config_path).output().expect("occurrence CLI daemon returns");
    assert!(daemon.status.success(), "stderr: {}", String::from_utf8_lossy(&daemon.stderr));
    let store = Store::new(&store_path).expect("occurrence CLI store reopens");
    let occurrences = store.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).expect("occurrence CLI state lists");
    assert_eq!(occurrences.len(), EXPECTED_OCCURRENCE_COUNT);
    let identity = occurrences[0].identity.clone();
    drop(store);

    let list = scheduler_occurrences_command(&config_path)
        .arg("list")
        .arg("--output")
        .arg("json")
        .output()
        .expect("occurrence CLI list returns");
    assert!(list.status.success(), "stderr: {}", String::from_utf8_lossy(&list.stderr));
    assert!(String::from_utf8_lossy(&list.stdout).contains(&identity));

    let show = scheduler_occurrence_command(&config_path, "show", &identity)
        .arg("--output")
        .arg("json")
        .output()
        .expect("occurrence CLI show returns");
    assert!(show.status.success(), "stderr: {}", String::from_utf8_lossy(&show.stderr));
    assert!(String::from_utf8_lossy(&show.stdout).contains("succeeded"));

    let invalid_limit = scheduler_occurrences_command(&config_path)
        .arg("list")
        .arg("--limit")
        .arg("0")
        .output()
        .expect("invalid occurrence CLI limit returns");
    assert!(!invalid_limit.status.success());
    assert!(String::from_utf8_lossy(&invalid_limit.stderr).contains("outside the supported bound"));

    let missing = scheduler_occurrence_command(&config_path, "show", "b3:missing")
        .output()
        .expect("missing occurrence CLI show returns");
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("does not exist"));
}

// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.restart]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.retry]
#[cfg(unix)]
#[test]
fn operator_e2e_scheduler_denies_pre_effect_death_without_current_authority() {
    let temp_dir = tempfile::tempdir().expect("temporary scheduler restart directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let marker_path = temp_dir.path().join(FORCED_DEATH_MARKER_FILE_NAME);
    let recovery_effect_path = temp_dir.path().join(RECOVERY_EFFECT_FILE_NAME);
    let store_path = data_dir.join(STORE_FILE_NAME);
    let store = Store::new(&store_path).expect("scheduler restart store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(
        "scheduler-restart",
        common::forced_parent_death_once_then_write_command(&marker_path, &recovery_effect_path, RECOVERY_EFFECT),
    );
    store.save_workflow(&workflow).expect("scheduler restart workflow saves");
    store.save_schedule(&schedule(START_EPOCH_MS)).expect("scheduler restart schedule saves");
    drop(store);

    let denied = scheduler_command(&config_path).output().expect("scheduler denial returns");
    assert!(denied.status.success(), "stderr: {}", String::from_utf8_lossy(&denied.stderr));
    assert!(!marker_path.exists(), "authority denial must precede the forced-death effect");
    assert!(!recovery_effect_path.exists(), "authority denial must not write a recovery effect");

    let settled = Store::new(&store_path).expect("denied scheduler store reopens");
    let advanced = settled.load_schedule(SCHEDULE_ID).expect("schedule loads").expect("schedule remains");
    let runs = settled.list_runs(RUN_HISTORY_LIMIT).expect("denied run history lists");
    let occurrences = settled.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).expect("occurrences list");
    assert!(advanced.next_run_epoch_ms > DUE_EPOCH_MS);
    assert_eq!(runs.len(), EXPECTED_PERSISTED_RUNS);
    assert_eq!(runs[0].failures, DENIED_PROCESS_FAILURES);
    assert_eq!(occurrences.len(), EXPECTED_OCCURRENCE_COUNT);
    assert_eq!(occurrences[0].status, lattice::scheduler::ScheduleOccurrenceStatus::Failed);
}

// r[verify lattice.repository_validation.post_effect_crash_boundaries]
// r[verify lattice.repository_validation.post_effect_crash_boundaries.scenario.scheduler]
// r[verify lattice.repository_validation.post_effect_crash_boundaries.scenario.non-claim]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.restart]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.abandon]
// r[verify lattice.scheduling.scheduler_occurrence_admission.scenario.non_claim]
#[cfg(unix)]
#[test]
fn operator_e2e_scheduler_denies_post_effect_death_without_current_authority() {
    let temp_dir = tempfile::tempdir().expect("temporary scheduler post-effect directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let checkpoint_path = temp_dir.path().join(POST_EFFECT_CHECKPOINT_FILE_NAME);
    let attempts_path = temp_dir.path().join(POST_EFFECT_ATTEMPTS_FILE_NAME);
    let store_path = data_dir.join(STORE_FILE_NAME);
    let store = Store::new(&store_path).expect("scheduler post-effect store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(
        "scheduler-post-effect",
        common::post_effect_parent_death_once_then_repeat_command(
            &checkpoint_path,
            &attempts_path,
            POST_EFFECT_IDENTITY,
        ),
    );
    store.save_workflow(&workflow).expect("scheduler post-effect workflow saves");
    store.save_schedule(&schedule(START_EPOCH_MS)).expect("scheduler post-effect schedule saves");
    drop(store);

    let denied = scheduler_command(&config_path).output().expect("scheduler denial returns");
    assert!(denied.status.success(), "stderr: {}", String::from_utf8_lossy(&denied.stderr));
    assert!(!checkpoint_path.exists(), "authority denial must precede the post-effect checkpoint");
    assert!(!attempts_path.exists(), "authority denial must precede the shell attempt log");

    let settled = Store::new(&store_path).expect("denied post-effect store reopens");
    let occurrences = settled.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).expect("occurrences list");
    assert_eq!(occurrences.len(), EXPECTED_OCCURRENCE_COUNT);
    assert_eq!(occurrences[0].status, lattice::scheduler::ScheduleOccurrenceStatus::Failed);
    assert_eq!(
        settled.list_runs(RUN_HISTORY_LIMIT).expect("run history lists")[0].failures,
        DENIED_PROCESS_FAILURES
    );
}

#[cfg(unix)]
#[test]
fn operator_e2e_scheduler_denial_settles_once_without_shell_markers() {
    let temp_dir = tempfile::tempdir().expect("temporary scheduler denial directory");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let ready_path = temp_dir.path().join(CONCURRENT_READY_FILE_NAME);
    let release_path = temp_dir.path().join(CONCURRENT_RELEASE_FILE_NAME);
    let effect_path = temp_dir.path().join(CONCURRENT_EFFECT_FILE_NAME);
    let store_path = data_dir.join(STORE_FILE_NAME);
    let store = Store::new(&store_path).expect("scheduler denial store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(
        "scheduler-concurrent",
        common::gated_write_command(&ready_path, &release_path, &effect_path, CONCURRENT_EFFECT),
    );
    store.save_workflow(&workflow).expect("scheduler denial workflow saves");
    store.save_schedule(&schedule(START_EPOCH_MS)).expect("scheduler denial schedule saves");
    drop(store);

    let first = scheduler_command(&config_path).output().expect("first scheduler denial returns");
    assert!(first.status.success(), "stderr: {}", String::from_utf8_lossy(&first.stderr));
    assert!(!ready_path.exists());
    assert!(!release_path.exists());
    assert!(!effect_path.exists());

    let second = scheduler_command(&config_path).output().expect("second scheduler pass returns");
    assert!(second.status.success(), "stderr: {}", String::from_utf8_lossy(&second.stderr));
    assert!(!String::from_utf8_lossy(&second.stdout).contains("started schedule"));

    let settled = Store::new(&store_path).expect("scheduler denial store reopens");
    let runs = settled.list_runs(RUN_HISTORY_LIMIT).expect("scheduler denial runs list");
    let occurrences = settled.list_schedule_occurrences(OCCURRENCE_LIST_LIMIT).expect("occurrences list");
    assert_eq!(runs.len(), EXPECTED_PERSISTED_RUNS);
    assert_eq!(runs[0].failures, DENIED_PROCESS_FAILURES);
    assert_eq!(occurrences.len(), EXPECTED_OCCURRENCE_COUNT);
    assert_eq!(occurrences[0].status, lattice::scheduler::ScheduleOccurrenceStatus::Failed);
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
    store
        .save_workflow(&Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()))
        .expect("workflow saved");
    let workflows = store.list_workflows().expect("workflows list").into_iter().collect::<HashSet<_>>();

    assert!(validate_schedule(&schedule(START_EPOCH_MS), &workflows).is_empty());
}
