mod common;

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetChargePlan;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::DispatchIntent;
use lattice::orchestration::DispatchStatus;
use lattice::orchestration::OrchestrationAction;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::OrchestrationFact;
use lattice::orchestration::OrchestrationInstance;
use lattice::orchestration::OrchestrationStatus;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::RetryPolicy;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::definition_to_nickel;
use lattice::orchestration::dispatch_identity;
use lattice::orchestration::fact_identity;
use lattice::orchestration::new_instance;
use lattice::orchestration::transition;
use lattice::orchestration::workflow_revision;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::store::Store;
use tempfile::tempdir;

const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const DEFINITION_FILE_NAME: &str = "orchestration.ncl";
const WORKFLOW_NAME: &str = "cli-workflow";
const DEFINITION_ID: &str = "cli-orchestration";
const OBJECTIVE: &str = "Exercise deterministic orchestration CLI";
const ACTOR: &str = "operator:cli";
const UNAUTHORIZED_ACTOR: &str = "agent:cli";
const SHELL_COMMAND: &str = "printf cli";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 1;
const ROUND_LIMIT: u64 = 2;
const ROUND_CHARGE: u64 = 1;
const RETRY_LIMIT: u64 = 1;
const PENDING_INSTANCE_SEQUENCE: u64 = 1;
const PENDING_RUN_ID: u64 = 77;
const OBSERVED_RUN_STARTED_EPOCH_MS: i64 = 1_000;
const OBSERVED_RUN_FINISHED_EPOCH_MS: i64 = 2_000;
const OBSERVED_RUN_NODE_COUNT: usize = 1;
const OBSERVED_RUN_CACHE_HITS: usize = 0;
const OBSERVED_RUN_FAILURES: usize = 0;
const OPERATOR_E2E_RUN_HISTORY_LIMIT: usize = 10;
const EXPECTED_OBSERVED_RUN_COUNT: usize = 1;
#[cfg(unix)]
const FORCED_DEATH_MARKER_FILE_NAME: &str = "orchestration-forced-death";
const RECOVERY_CONTENDER_CHILD_TEST_NAME: &str = "orchestration_recovery_contender_child_entrypoint";
const RECOVERY_CONTENDER_CONFIG_ENV: &str = "LATTICE_RECOVERY_CONTENDER_CONFIG";
const RECOVERY_CONTENDER_INSTANCE_ENV: &str = "LATTICE_RECOVERY_CONTENDER_INSTANCE";
const RECOVERY_CONTENDER_READY_ENV: &str = "LATTICE_RECOVERY_CONTENDER_READY";
const RECOVERY_CONTENDER_RELEASE_ENV: &str = "LATTICE_RECOVERY_CONTENDER_RELEASE";
const RECOVERY_CONTENDER_RESULT_ENV: &str = "LATTICE_RECOVERY_CONTENDER_RESULT";
const RECOVERY_CONTENDER_POLL_ATTEMPTS: usize = 500;
const RECOVERY_CONTENDER_POLL_DELAY_MS: u64 = 10;
const RECOVERY_CONTENDER_COUNT: usize = 2;
const RECOVERY_SETTLED_EVENT_COUNT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct RecoveryContenderResult {
    success: bool,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryContenderOutcome {
    Reconciled,
    AlreadySettled,
    ActiveOwnerRejected,
    Unexpected,
}

fn fixture(completion: CompletionPolicy) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf, Store) {
    fixture_with_shell_command(completion, SHELL_COMMAND)
}

fn effect_free_fixture(
    completion: CompletionPolicy,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf, Store) {
    fixture_with_workflow(
        completion,
        Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch()),
    )
}

fn fixture_with_shell_command(
    completion: CompletionPolicy,
    shell_command: &str,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf, Store) {
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node("cli", shell_command);
    fixture_with_workflow(completion, workflow)
}

fn fixture_with_workflow(
    completion: CompletionPolicy,
    workflow: Workflow,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf, Store) {
    let directory = tempdir().expect("temporary directory must exist");
    let data_dir = directory.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory must exist");
    let config_path = directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string()
        ),
    )
    .expect("config must write");
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store must open");
    store.save_workflow(&workflow).expect("workflow must persist");
    let definition = OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(&workflow).expect("workflow must hash"),
        completion,
        retry: Some(RetryPolicy {
            max_observed_failure_retries: RETRY_LIMIT,
        }),
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: ROUND_LIMIT,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge {
                rounds: ROUND_CHARGE,
                ..BudgetCharge::default()
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    let definition_path = directory.path().join(DEFINITION_FILE_NAME);
    std::fs::write(&definition_path, definition_to_nickel(&definition).expect("definition must render"))
        .expect("definition must write");
    (directory, config_path, definition_path, store)
}

fn command(config_path: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    command.arg("--config").arg(config_path).arg("orchestration");
    command
}

fn recovery_contender_command(
    config_path: &Path,
    instance_id: &str,
    ready_path: &Path,
    release_path: &Path,
    result_path: &Path,
) -> Command {
    let mut contender = Command::new(std::env::current_exe().expect("orchestration test executable resolves"));
    contender
        .arg("--exact")
        .arg(RECOVERY_CONTENDER_CHILD_TEST_NAME)
        .env(RECOVERY_CONTENDER_CONFIG_ENV, config_path)
        .env(RECOVERY_CONTENDER_INSTANCE_ENV, instance_id)
        .env(RECOVERY_CONTENDER_READY_ENV, ready_path)
        .env(RECOVERY_CONTENDER_RELEASE_ENV, release_path)
        .env(RECOVERY_CONTENDER_RESULT_ENV, result_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    contender
}

fn recovery_contender_outcome(result: &RecoveryContenderResult) -> RecoveryContenderOutcome {
    if result.success && result.stdout.contains("\"Reconciled\"") {
        return RecoveryContenderOutcome::Reconciled;
    }
    if result.success && result.stdout.contains("NothingPending") {
        return RecoveryContenderOutcome::AlreadySettled;
    }
    let normalized_stderr = result.stderr.to_ascii_lowercase();
    if !result.success
        && normalized_stderr.contains("orchestration failed:")
        && (normalized_stderr.contains("already open") || normalized_stderr.contains("acquire lock"))
    {
        return RecoveryContenderOutcome::ActiveOwnerRejected;
    }
    RecoveryContenderOutcome::Unexpected
}

fn persist_observed_failure(store: &Store, workflow: &Workflow) -> OrchestrationInstance {
    let definition = OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(workflow).expect("workflow must hash"),
        completion: CompletionPolicy::RunSucceeded,
        retry: Some(RetryPolicy {
            max_observed_failure_retries: RETRY_LIMIT,
        }),
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: ROUND_LIMIT,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge {
                rounds: ROUND_CHARGE,
                ..BudgetCharge::default()
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    let base = new_instance(definition, PENDING_INSTANCE_SEQUENCE, None).expect("instance must hash");
    let run_id = RunId(PENDING_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(&base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let start_action = OrchestrationAction::StartRound {
        intent: intent.clone(),
        planned_charge: BudgetChargePlan {
            rounds: Some(ROUND_CHARGE),
            agent_invocations: Some(0),
            tokens: Some(0),
            monetary_microunits: Some(0),
            duration_millis: Some(0),
        },
    };
    let started = transition(&base, OrchestrationFact {
        id: fact_identity(&base.id, base.version, &start_action).expect("start fact must hash"),
        action: start_action,
    })
    .expect("start must apply");
    let acknowledge_action = OrchestrationAction::DispatchAcknowledged {
        dispatch_id: intent.id.clone(),
    };
    let acknowledged = transition(&started.instance, OrchestrationFact {
        id: fact_identity(&started.instance.id, started.instance.version, &acknowledge_action)
            .expect("acknowledgement fact must hash"),
        action: acknowledge_action,
    })
    .expect("acknowledgement must apply");
    store
        .save_run(&RunRecord {
            run_id,
            owner: RunOwner::Orchestration {
                instance_id: base.id.clone(),
                dispatch_id: intent.id.clone(),
            },
            workflow_name: WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(1),
            node_count: 1,
            cache_hits: 0,
            failures: 1,
        })
        .expect("observed failed run must persist");
    let finish_action = OrchestrationAction::RunFinished {
        dispatch_id: intent.id,
        run_id,
        succeeded: false,
        retryable_observed_failure: true,
        observed_usage: BudgetCharge {
            rounds: ROUND_CHARGE,
            ..BudgetCharge::default()
        },
        safe_summary: "observed terminal failure".to_string(),
    };
    let finished = transition(&acknowledged.instance, OrchestrationFact {
        id: fact_identity(&acknowledged.instance.id, acknowledged.instance.version, &finish_action)
            .expect("finish fact must hash"),
        action: finish_action,
    })
    .expect("failure must terminalize");
    let events = vec![
        started.event.expect("start event must exist"),
        acknowledged.event.expect("acknowledgement event must exist"),
        finished.event.expect("failure event must exist"),
    ];
    store.create_orchestration(&finished.instance, &events).expect("failed instance must persist");
    finished.instance
}

fn persist_acknowledged_pending(store: &Store, workflow: &Workflow) -> OrchestrationInstance {
    let definition = OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(workflow).expect("workflow must hash"),
        completion: CompletionPolicy::RunSucceeded,
        retry: Some(RetryPolicy {
            max_observed_failure_retries: RETRY_LIMIT,
        }),
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: ROUND_LIMIT,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge {
                rounds: ROUND_CHARGE,
                ..BudgetCharge::default()
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    let base = new_instance(definition, PENDING_INSTANCE_SEQUENCE, None).expect("instance must hash");
    let run_id = RunId(PENDING_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(&base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let start_action = OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(ROUND_CHARGE),
            agent_invocations: Some(0),
            tokens: Some(0),
            monetary_microunits: Some(0),
            duration_millis: Some(0),
        },
    };
    let started = transition(&base, OrchestrationFact {
        id: fact_identity(&base.id, base.version, &start_action).expect("start fact must hash"),
        action: start_action,
    })
    .expect("start must apply");
    let dispatch_id = started.instance.pending_dispatch.as_ref().expect("dispatch must be pending").id.clone();
    let acknowledge_action = OrchestrationAction::DispatchAcknowledged { dispatch_id };
    let acknowledged = transition(&started.instance, OrchestrationFact {
        id: fact_identity(&started.instance.id, started.instance.version, &acknowledge_action)
            .expect("acknowledgement fact must hash"),
        action: acknowledge_action,
    })
    .expect("acknowledgement must apply");
    let events = vec![
        started.event.expect("start event must exist"),
        acknowledged.event.clone().expect("acknowledgement event must exist"),
    ];
    store
        .create_orchestration(&acknowledged.instance, &events)
        .expect("acknowledged pending instance must persist");
    acknowledged.instance
}

fn persist_pending_run_record(store: &Store, instance: &OrchestrationInstance, owner: RunOwner) {
    let intent = instance.pending_dispatch.as_ref().expect("pending dispatch must exist");
    store
        .save_run(&RunRecord {
            run_id: intent.run_id,
            owner,
            workflow_name: WORKFLOW_NAME.to_string(),
            started_epoch_ms: OBSERVED_RUN_STARTED_EPOCH_MS,
            finished_epoch_ms: Some(OBSERVED_RUN_FINISHED_EPOCH_MS),
            node_count: OBSERVED_RUN_NODE_COUNT,
            cache_hits: OBSERVED_RUN_CACHE_HITS,
            failures: OBSERVED_RUN_FAILURES,
        })
        .expect("pending run record must persist");
}

// r[verify lattice.agent_orchestration.cli]
#[test]
fn cli_validates_starts_and_reports_json_status() {
    let (_directory, config_path, definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    drop(store);

    let validation = command(&config_path)
        .arg("validate")
        .arg(&definition_path)
        .arg("--output")
        .arg("json")
        .output()
        .expect("validate command must execute");
    assert!(validation.status.success(), "stderr: {}", String::from_utf8_lossy(&validation.stderr));

    let start = command(&config_path)
        .arg("start")
        .arg(&definition_path)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("start command must execute");
    assert!(start.status.success(), "stderr: {}", String::from_utf8_lossy(&start.stderr));
    let instance: OrchestrationInstance = serde_json::from_slice(&start.stdout).expect("instance JSON must parse");
    assert_eq!(instance.status, OrchestrationStatus::Failed);
    assert!(instance.terminal_summary.as_deref().is_some_and(|summary| summary.contains("Failed")));

    let status = command(&config_path)
        .arg("status")
        .arg(&instance.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("status command must execute");
    assert!(status.status.success(), "stderr: {}", String::from_utf8_lossy(&status.stderr));
    let loaded: OrchestrationInstance = serde_json::from_slice(&status.stdout).expect("status JSON must parse");
    assert_eq!(loaded, instance);
}

// r[verify lattice.agent_orchestration.retry_round.scenario.success]
// r[verify lattice.agent_orchestration.cli]
#[test]
fn cli_retries_an_exact_observed_failure_and_preserves_replay() {
    let (_directory, config_path, _definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    let workflow = store
        .load_workflow(WORKFLOW_NAME)
        .expect("workflow lookup must succeed")
        .expect("workflow must exist");
    let failed = persist_observed_failure(&store, &workflow);
    drop(store);

    let retry = command(&config_path)
        .arg("retry")
        .arg(&failed.id)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("retry command must execute");
    assert!(retry.status.success(), "stderr: {}", String::from_utf8_lossy(&retry.stderr));
    let completed: OrchestrationInstance = serde_json::from_slice(&retry.stdout).expect("retry JSON must parse");
    assert_eq!(completed.status, OrchestrationStatus::Failed);
    assert_eq!(completed.round, ROUND_LIMIT);
    assert_eq!(completed.retry.admitted, RETRY_LIMIT);
    assert!(completed.terminal_summary.as_deref().is_some_and(|summary| summary.contains("Failed")));

    let replay = command(&config_path)
        .arg("replay")
        .arg(&completed.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("replay command must execute");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let report: lattice::orchestration::ReplayReport =
        serde_json::from_slice(&replay.stdout).expect("replay JSON must parse");
    assert!(report.valid);
    assert_eq!(report.instance, completed);
}

// r[verify lattice.agent_orchestration.controls]
// r[verify lattice.agent_orchestration.cli]
#[test]
fn cli_pause_resume_and_replay_share_persisted_state_machine() {
    let (_directory, config_path, definition_path, store) = effect_free_fixture(CompletionPolicy::AwaitProposal);
    drop(store);
    let start = command(&config_path)
        .arg("start")
        .arg(&definition_path)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("start command must execute");
    let waiting: OrchestrationInstance = serde_json::from_slice(&start.stdout).expect("instance JSON must parse");
    assert_eq!(waiting.status, OrchestrationStatus::WaitingForApproval);

    let pause = command(&config_path)
        .arg("pause")
        .arg(&waiting.id)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("pause command must execute");
    assert!(pause.status.success(), "stderr: {}", String::from_utf8_lossy(&pause.stderr));
    let paused: OrchestrationInstance = serde_json::from_slice(&pause.stdout).expect("pause JSON must parse");
    assert_eq!(paused.status, OrchestrationStatus::Paused);

    let resume = command(&config_path)
        .arg("resume")
        .arg(&paused.id)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("resume command must execute");
    assert!(resume.status.success(), "stderr: {}", String::from_utf8_lossy(&resume.stderr));
    let resumed: OrchestrationInstance = serde_json::from_slice(&resume.stdout).expect("resume JSON must parse");
    assert_eq!(resumed.status, OrchestrationStatus::WaitingForApproval);

    let replay = command(&config_path)
        .arg("replay")
        .arg(&resumed.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("replay command must execute");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let report: lattice::orchestration::ReplayReport =
        serde_json::from_slice(&replay.stdout).expect("replay JSON must parse");
    assert!(report.valid);
    assert_eq!(report.instance, resumed);
}

// r[verify lattice.agent_orchestration.cli]
// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn cli_recovers_cancels_and_abandons_an_unobserved_dispatch_without_retry() {
    let (_directory, config_path, _definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    let workflow = store
        .load_workflow(WORKFLOW_NAME)
        .expect("workflow lookup must succeed")
        .expect("workflow must exist");
    let instance = persist_acknowledged_pending(&store, &workflow);
    drop(store);

    let recovery = command(&config_path)
        .arg("recover")
        .arg(&instance.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("recover command must execute");
    assert!(recovery.status.success(), "stderr: {}", String::from_utf8_lossy(&recovery.stderr));
    assert!(String::from_utf8_lossy(&recovery.stdout).contains("PendingUnobserved"));

    let retry_pending = command(&config_path)
        .arg("retry")
        .arg(&instance.id)
        .arg("--actor")
        .arg(ACTOR)
        .output()
        .expect("pending retry command must execute");
    assert!(!retry_pending.status.success());
    assert!(String::from_utf8_lossy(&retry_pending.stderr).contains("pending and locally unobserved"));

    let cancel = command(&config_path)
        .arg("cancel")
        .arg(&instance.id)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("cancel command must execute");
    assert!(cancel.status.success(), "stderr: {}", String::from_utf8_lossy(&cancel.stderr));
    let cancelled: OrchestrationInstance = serde_json::from_slice(&cancel.stdout).expect("cancel JSON must parse");
    assert!(cancelled.cancellation_requested);
    assert_eq!(cancelled.status, OrchestrationStatus::Running);

    let abandon = command(&config_path)
        .arg("abandon")
        .arg(&instance.id)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("abandon command must execute");
    assert!(abandon.status.success(), "stderr: {}", String::from_utf8_lossy(&abandon.stderr));
    let abandoned: OrchestrationInstance = serde_json::from_slice(&abandon.stdout).expect("instance JSON must parse");
    assert_eq!(abandoned.status, OrchestrationStatus::Failed);
    assert!(abandoned.pending_dispatch.is_none());
    assert!(abandoned.budgets.reservations.is_empty());
    let rendered = String::from_utf8_lossy(&abandon.stdout);
    assert!(rendered.contains("remote outcome unknown"));
    assert!(!rendered.contains("remote outcome succeeded"));

    let replay = command(&config_path)
        .arg("replay")
        .arg(&instance.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("replay command must execute");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let report: lattice::orchestration::ReplayReport =
        serde_json::from_slice(&replay.stdout).expect("replay JSON must parse");
    assert!(report.valid);
    assert_eq!(report.instance, abandoned);

    let retry_abandoned = command(&config_path)
        .arg("retry")
        .arg(&instance.id)
        .arg("--actor")
        .arg(ACTOR)
        .output()
        .expect("abandoned retry command must execute");
    assert!(!retry_abandoned.status.success());
    assert!(String::from_utf8_lossy(&retry_abandoned.stderr).contains("no exact persisted terminal-failure"));

    let repeated = command(&config_path)
        .arg("abandon")
        .arg(&instance.id)
        .arg("--actor")
        .arg(ACTOR)
        .output()
        .expect("repeated abandon command must execute");
    assert!(!repeated.status.success());
    assert!(String::from_utf8_lossy(&repeated.stderr).contains("no pending dispatch"));
}

#[test]
fn orchestration_recovery_contender_child_entrypoint() {
    let Ok(config_path) = std::env::var(RECOVERY_CONTENDER_CONFIG_ENV) else {
        return;
    };
    let instance_id = std::env::var(RECOVERY_CONTENDER_INSTANCE_ENV).expect("contender instance id is configured");
    let ready_path = std::env::var_os(RECOVERY_CONTENDER_READY_ENV).expect("contender ready path is configured");
    let release_path = std::env::var_os(RECOVERY_CONTENDER_RELEASE_ENV).expect("contender release path is configured");
    let result_path = std::env::var_os(RECOVERY_CONTENDER_RESULT_ENV).expect("contender result path is configured");
    std::fs::write(&ready_path, "ready").expect("contender ready marker persists");
    common::wait_for_path(
        Path::new(&release_path),
        RECOVERY_CONTENDER_POLL_ATTEMPTS,
        Duration::from_millis(RECOVERY_CONTENDER_POLL_DELAY_MS),
    );

    let output = command(Path::new(&config_path))
        .arg("recover")
        .arg(instance_id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("contender recovery process executes");
    let result = RecoveryContenderResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    std::fs::write(&result_path, serde_json::to_vec(&result).expect("contender result serializes"))
        .expect("contender result persists");
}

#[test]
fn operator_e2e_orchestration_recovery_concurrent_processes_commit_one_terminal_transition() {
    let (directory, config_path, _definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    let workflow = store
        .load_workflow(WORKFLOW_NAME)
        .expect("concurrent recovery workflow lookup succeeds")
        .expect("concurrent recovery workflow exists");
    let pending = persist_acknowledged_pending(&store, &workflow);
    let intent = pending.pending_dispatch.as_ref().expect("concurrent recovery dispatch remains pending");
    persist_pending_run_record(&store, &pending, RunOwner::Orchestration {
        instance_id: pending.id.clone(),
        dispatch_id: intent.id.clone(),
    });
    drop(store);

    let release_path = directory.path().join("recovery-contenders-release");
    let first_ready_path = directory.path().join("recovery-contender-first-ready");
    let second_ready_path = directory.path().join("recovery-contender-second-ready");
    let first_result_path = directory.path().join("recovery-contender-first-result.json");
    let second_result_path = directory.path().join("recovery-contender-second-result.json");
    let first =
        recovery_contender_command(&config_path, &pending.id, &first_ready_path, &release_path, &first_result_path)
            .spawn()
            .expect("first recovery contender starts");
    let second =
        recovery_contender_command(&config_path, &pending.id, &second_ready_path, &release_path, &second_result_path)
            .spawn()
            .expect("second recovery contender starts");
    let first = common::ChildGuard::new(first);
    let second = common::ChildGuard::new(second);
    for ready_path in [&first_ready_path, &second_ready_path] {
        common::wait_for_path(
            ready_path,
            RECOVERY_CONTENDER_POLL_ATTEMPTS,
            Duration::from_millis(RECOVERY_CONTENDER_POLL_DELAY_MS),
        );
    }
    std::fs::write(&release_path, "release").expect("recovery contenders release together");
    let first_child = first.wait_with_output();
    let second_child = second.wait_with_output();
    assert!(first_child.status.success(), "first contender harness failed");
    assert!(second_child.status.success(), "second contender harness failed");

    let results = [&first_result_path, &second_result_path]
        .into_iter()
        .map(|path| {
            serde_json::from_slice::<RecoveryContenderResult>(
                &std::fs::read(path).expect("recovery contender result reads"),
            )
            .expect("recovery contender result parses")
        })
        .collect::<Vec<_>>();
    let outcomes = results.iter().map(recovery_contender_outcome).collect::<Vec<_>>();
    assert_eq!(
        outcomes.iter().filter(|outcome| **outcome == RecoveryContenderOutcome::Reconciled).count(),
        1,
        "contender outcomes: {outcomes:?}; results: {results:?}"
    );
    assert_eq!(outcomes.len(), RECOVERY_CONTENDER_COUNT);
    assert!(
        outcomes.iter().all(|outcome| matches!(
            outcome,
            RecoveryContenderOutcome::Reconciled
                | RecoveryContenderOutcome::AlreadySettled
                | RecoveryContenderOutcome::ActiveOwnerRejected
        )),
        "contender outcomes: {outcomes:?}; results: {results:?}"
    );

    let store_path = config_path.parent().expect("config has parent").join("data").join(STORE_FILE_NAME);
    let settled = Store::new(&store_path).expect("concurrent recovery store reopens");
    let instance = settled
        .load_orchestration(&pending.id)
        .expect("concurrent recovery instance lookup succeeds")
        .expect("concurrent recovery instance remains");
    let events = settled.load_orchestration_events(&pending.id).expect("concurrent recovery events load");
    let runs = settled.list_runs(OPERATOR_E2E_RUN_HISTORY_LIMIT).expect("concurrent recovery runs list");
    assert_eq!(instance.status, OrchestrationStatus::Completed);
    assert!(instance.pending_dispatch.is_none());
    assert_eq!(events.len(), RECOVERY_SETTLED_EVENT_COUNT);
    assert_eq!(runs.len(), EXPECTED_OBSERVED_RUN_COUNT);
    drop(settled);

    let repeated = command(&config_path)
        .arg("recover")
        .arg(&pending.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("settled recovery command executes");
    assert!(repeated.status.success(), "stderr: {}", String::from_utf8_lossy(&repeated.stderr));
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("NothingPending"));

    let replay = command(&config_path)
        .arg("replay")
        .arg(&pending.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("concurrent recovery replay executes");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let report: lattice::orchestration::ReplayReport =
        serde_json::from_slice(&replay.stdout).expect("concurrent recovery replay parses");
    assert!(report.valid);
    assert_eq!(report.instance, instance);
}

#[cfg(unix)]
#[test]
fn operator_e2e_orchestration_denies_forced_death_effect_without_current_authority() {
    let marker_directory = tempdir().expect("temporary orchestration marker directory");
    let marker_path = marker_directory.path().join(FORCED_DEATH_MARKER_FILE_NAME);
    let shell_command = common::forced_parent_death_command(&marker_path);
    let (_directory, config_path, definition_path, store) =
        fixture_with_shell_command(CompletionPolicy::RunSucceeded, &shell_command);
    drop(store);

    let denied = command(&config_path)
        .arg("start")
        .arg(&definition_path)
        .arg("--actor")
        .arg(ACTOR)
        .arg("--output")
        .arg("json")
        .output()
        .expect("orchestration process returns a denied terminal result");

    assert!(denied.status.success(), "stderr: {}", String::from_utf8_lossy(&denied.stderr));
    assert!(!marker_path.exists(), "authority denial must occur before the shell effect");
    let failed: OrchestrationInstance =
        serde_json::from_slice(&denied.stdout).expect("denied orchestration JSON parses");
    assert_eq!(failed.status, OrchestrationStatus::Failed);
    assert!(failed.pending_dispatch.is_none());
    assert!(failed.terminal_summary.as_deref().is_some_and(|summary| summary.contains("Failed")));

    let replay = command(&config_path)
        .arg("replay")
        .arg(&failed.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("denied orchestration replay process executes");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let report: lattice::orchestration::ReplayReport =
        serde_json::from_slice(&replay.stdout).expect("denied orchestration replay JSON parses");
    assert!(report.valid);
    assert_eq!(report.instance, failed);
}

#[test]
fn operator_e2e_orchestration_recovery_reconciles_exact_observed_run_without_redispatch() {
    let (_directory, config_path, _definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    let workflow = store
        .load_workflow(WORKFLOW_NAME)
        .expect("workflow lookup must succeed")
        .expect("workflow must exist");
    let pending = persist_acknowledged_pending(&store, &workflow);
    let intent = pending.pending_dispatch.as_ref().expect("pending dispatch must exist");
    persist_pending_run_record(&store, &pending, RunOwner::Orchestration {
        instance_id: pending.id.clone(),
        dispatch_id: intent.id.clone(),
    });
    drop(store);

    let recovery = command(&config_path)
        .arg("recover")
        .arg(&pending.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("recover command must execute");
    assert!(recovery.status.success(), "stderr: {}", String::from_utf8_lossy(&recovery.stderr));
    assert!(String::from_utf8_lossy(&recovery.stdout).contains("\"Reconciled\""));

    let reopened = Store::new(config_path.parent().expect("config has parent").join("data").join(STORE_FILE_NAME))
        .expect("store must reopen");
    let reconciled = reopened
        .load_orchestration(&pending.id)
        .expect("reconciled instance lookup must succeed")
        .expect("reconciled instance must exist");
    let runs = reopened.list_runs(OPERATOR_E2E_RUN_HISTORY_LIMIT).expect("reconciled run history must list");

    assert_eq!(reconciled.status, OrchestrationStatus::Completed);
    assert!(reconciled.pending_dispatch.is_none());
    assert_eq!(runs.len(), EXPECTED_OBSERVED_RUN_COUNT);
    assert_eq!(runs[0].run_id, RunId(PENDING_RUN_ID));
    drop(reopened);

    let replay = command(&config_path)
        .arg("replay")
        .arg(&pending.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("replay command must execute");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let report: lattice::orchestration::ReplayReport =
        serde_json::from_slice(&replay.stdout).expect("replay JSON must parse");
    assert!(report.valid);
    assert_eq!(report.instance, reconciled);
}

#[test]
fn operator_e2e_orchestration_recovery_rejects_unscoped_run_without_mutating_pending_state() {
    let (_directory, config_path, _definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    let workflow = store
        .load_workflow(WORKFLOW_NAME)
        .expect("workflow lookup must succeed")
        .expect("workflow must exist");
    let pending = persist_acknowledged_pending(&store, &workflow);
    persist_pending_run_record(&store, &pending, RunOwner::Unscoped);
    drop(store);

    let recovery = command(&config_path)
        .arg("recover")
        .arg(&pending.id)
        .arg("--output")
        .arg("json")
        .output()
        .expect("recover command must execute");
    assert!(!recovery.status.success());
    assert!(String::from_utf8_lossy(&recovery.stderr).contains("not owned by the pending orchestration dispatch"));

    let reopened = Store::new(config_path.parent().expect("config has parent").join("data").join(STORE_FILE_NAME))
        .expect("store must reopen");
    let unchanged = reopened
        .load_orchestration(&pending.id)
        .expect("pending instance lookup must succeed")
        .expect("pending instance must remain");

    assert_eq!(unchanged, pending);
    assert!(unchanged.pending_dispatch.is_some());
}

// r[verify lattice.agent_orchestration.cli]
// r[verify lattice.agent_orchestration.cli.scenario.invalid]
#[test]
fn unauthorized_start_fails_without_persisting_an_instance() {
    let (_directory, config_path, definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    drop(store);

    let output = command(&config_path)
        .arg("start")
        .arg(&definition_path)
        .arg("--actor")
        .arg(UNAUTHORIZED_ACTOR)
        .output()
        .expect("start command must execute");

    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("plaintext"));
    let reopened = Store::new(config_path.parent().expect("config has parent").join("data").join(STORE_FILE_NAME))
        .expect("store must reopen");
    assert!(reopened.list_orchestrations().expect("instances must list").is_empty());
}

// r[verify lattice.agent_orchestration.cli]
// r[verify lattice.agent_orchestration.cli.scenario.invalid]
#[test]
fn missing_instance_command_fails_without_creating_state() {
    let (_directory, config_path, _definition_path, store) = fixture(CompletionPolicy::RunSucceeded);
    drop(store);

    let output = command(&config_path)
        .arg("status")
        .arg("missing-instance")
        .output()
        .expect("status command must execute");

    assert!(!output.status.success());
}
