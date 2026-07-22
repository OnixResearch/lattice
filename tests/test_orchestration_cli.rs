use std::collections::BTreeSet;
use std::process::Command;

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

fn fixture(completion: CompletionPolicy) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf, Store) {
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
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    workflow.add_shell_node("cli", SHELL_COMMAND);
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
    assert_eq!(instance.status, OrchestrationStatus::Completed);

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
    assert_eq!(completed.status, OrchestrationStatus::Completed);
    assert_eq!(completed.round, ROUND_LIMIT);
    assert_eq!(completed.retry.admitted, RETRY_LIMIT);
    assert!(completed.retry.retryable_failure.is_none());

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
    let (_directory, config_path, definition_path, store) = fixture(CompletionPolicy::AwaitProposal);
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
