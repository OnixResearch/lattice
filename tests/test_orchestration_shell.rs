use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use lattice::agent_node::AgentRuntimeConfig;
use lattice::engine::NodeConfig;
use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::file_node::FileOperationKind;
use lattice::orchestration::AbandonPendingRequest;
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
use lattice::orchestration::OrchestrationEvent;
use lattice::orchestration::OrchestrationEventKind;
use lattice::orchestration::OrchestrationFact;
use lattice::orchestration::OrchestrationInstance;
use lattice::orchestration::OrchestrationStatus;
use lattice::orchestration::OrchestrationTriggerBinding;
use lattice::orchestration::OrchestrationTriggerKind;
use lattice::orchestration::RecoveryOutcome;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::RetryPolicy;
use lattice::orchestration::RetryRequest;
use lattice::orchestration::abandon_pending;
use lattice::orchestration::approve;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::dispatch_identity;
use lattice::orchestration::fact_identity;
use lattice::orchestration::make_proposal;
use lattice::orchestration::new_instance;
use lattice::orchestration::recover_pending;
use lattice::orchestration::replay;
use lattice::orchestration::retry;
use lattice::orchestration::start;
use lattice::orchestration::start_from_trigger;
use lattice::orchestration::transition;
use lattice::orchestration::validate_definition_in_store;
use lattice::orchestration::workflow_revision;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::store::Store;
use tempfile::tempdir;

const DEFINITION_ID: &str = "shell-orchestration";
const OBJECTIVE: &str = "Execute bounded workflow rounds";
const FIRST_WORKFLOW_NAME: &str = "shell-round-one";
const SECOND_WORKFLOW_NAME: &str = "shell-round-two";
const SHELL_COMMAND: &str = "printf bounded";
const ACTOR: &str = "operator:shell";
const UNAUTHORIZED_ACTOR: &str = "agent:shell";
const TRIGGER_ID: &str = "shell-webhook";
const IDEMPOTENCY_KEY: &str = "request-one";
const FIRST_SEQUENCE: u64 = 1;
const ROUND_LIMIT: u64 = 2;
const RETRY_LIMIT: u64 = 1;
const RECOVERY_RUN_ID: u64 = 77;
const RESTART_TEST_NAME: &str = "restart_duplicate_trigger_reuses_original_admission_across_processes";
const RESTART_PHASE_ENV: &str = "LATTICE_RESTART_TRIGGER_TEST_PHASE";
const RESTART_STORE_ENV: &str = "LATTICE_RESTART_TRIGGER_TEST_STORE";
const RESTART_FIRST_RESULT_ENV: &str = "LATTICE_RESTART_TRIGGER_TEST_FIRST_RESULT";
const RESTART_DUPLICATE_RESULT_ENV: &str = "LATTICE_RESTART_TRIGGER_TEST_DUPLICATE_RESULT";
const RESTART_PHASE_FIRST: &str = "first";
const RESTART_PHASE_DUPLICATE: &str = "duplicate";
const RESTART_PHASE_INVALID: &str = "invalid";
const RETRY_MARKER_CONTENT: &str = "first-attempt-observed";
const FIRST_ALLOCATED_RUN_ID: u64 = 0;
const SECOND_ALLOCATED_RUN_ID: u64 = 1;
const RETRY_RUN_LIST_LIMIT: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestartPhase {
    First,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RestartResult {
    process_id: u32,
    instance_id: String,
}

fn parse_restart_phase(value: &str) -> Result<RestartPhase, &'static str> {
    match value {
        RESTART_PHASE_FIRST => Ok(RestartPhase::First),
        RESTART_PHASE_DUPLICATE => Ok(RestartPhase::Duplicate),
        _ => Err("unknown restart trigger test phase"),
    }
}

fn encode_restart_result(result: &RestartResult) -> String {
    format!("{}\n{}", result.process_id, result.instance_id)
}

fn parse_restart_result(value: &str) -> Result<RestartResult, &'static str> {
    let Some((process_id, instance_id)) = value.split_once('\n') else {
        return Err("restart result is missing its identity separator");
    };
    let process_id = process_id.parse::<u32>().map_err(|_| "restart process identity is invalid")?;
    if instance_id.is_empty() {
        return Err("restart instance identity is empty");
    }
    Ok(RestartResult {
        process_id,
        instance_id: instance_id.to_string(),
    })
}

fn workflow(name: &str) -> Workflow {
    let mut workflow = Workflow::new(name);
    workflow.add_shell_node("bounded", SHELL_COMMAND);
    workflow
}

fn fail_once_workflow(name: &str, marker: &Path) -> Workflow {
    let marker_path = marker.to_string_lossy().into_owned();
    let allowed_root = marker.parent().expect("retry marker must have a parent").to_string_lossy().into_owned();
    let mut workflow = Workflow::new(name);
    let node_id = workflow.add_file_node("bounded", FileOperationKind::Read, &marker_path);
    workflow.set_node_config(node_id, NodeConfig::File {
        operation: FileOperationKind::Read,
        path: marker_path,
        content: None,
        overwrite: false,
        allowed_roots: vec![allowed_root],
    });
    workflow
}

fn definition(workflow: &Workflow, completion: CompletionPolicy) -> OrchestrationDefinition {
    OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(workflow).expect("workflow must hash"),
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
                rounds: FIRST_SEQUENCE,
                ..BudgetCharge::default()
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: vec![OrchestrationTriggerBinding {
            trigger_id: TRIGGER_ID.to_string(),
            kind: OrchestrationTriggerKind::Webhook,
        }],
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let directory = tempdir().expect("temporary directory must exist");
    let store = Store::new(directory.path().join("shell.redb")).expect("store must open");
    (directory, store)
}

fn cancelled_acknowledged_instance(workflow: &Workflow) -> (OrchestrationInstance, Vec<OrchestrationEvent>) {
    let base = new_instance(definition(workflow, CompletionPolicy::RunSucceeded), FIRST_SEQUENCE, None)
        .expect("instance must hash");
    let run_id = RunId(RECOVERY_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(&base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let start_action = OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(FIRST_SEQUENCE),
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
    let cancel_action = OrchestrationAction::CancelRequested {
        actor: ACTOR.to_string(),
    };
    let cancelled = transition(&acknowledged.instance, OrchestrationFact {
        id: fact_identity(&acknowledged.instance.id, acknowledged.instance.version, &cancel_action)
            .expect("cancellation fact must hash"),
        action: cancel_action,
    })
    .expect("cancellation must apply");
    let events = vec![
        started.event.expect("start event must exist"),
        acknowledged.event.expect("acknowledgement event must exist"),
        cancelled.event.expect("cancellation event must exist"),
    ];
    (cancelled.instance, events)
}

// r[verify lattice.agent_orchestration.rounds]
// r[verify lattice.agent_orchestration.cli]
// r[verify lattice.workflow_execution.headless_run_details.scenario.orchestration]
#[tokio::test]
async fn start_executes_one_seaglass_admitted_round_and_persists_replay() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");

    let instance =
        start(&store, definition(&workflow, CompletionPolicy::RunSucceeded), ACTOR, AgentRuntimeConfig::default())
            .await
            .expect("orchestration must run");

    assert_eq!(instance.status, OrchestrationStatus::Completed);
    assert_eq!(instance.round, FIRST_SEQUENCE);
    assert!(instance.active_run_id.is_none());
    let runs = store.list_runs(ROUND_LIMIT as usize).expect("runs must list");
    assert_eq!(runs.len(), FIRST_SEQUENCE as usize);
    let RunOwner::Orchestration {
        instance_id,
        dispatch_id,
    } = &runs[0].owner
    else {
        panic!("orchestration run must retain its owner identity");
    };
    assert_eq!(instance_id, &instance.id);
    assert!(!dispatch_id.is_empty());
    let details = store
        .load_run_details(runs[0].run_id)
        .expect("run details must load")
        .expect("run details must persist");
    assert_eq!(details.nodes.len(), FIRST_SEQUENCE as usize);
    let report = replay(&store, &instance.id).expect("side-effect-free replay must validate");
    assert!(report.valid);
    assert_eq!(report.instance, instance);
}

// r[verify lattice.agent_orchestration.model]
#[test]
fn definition_validation_rejects_stale_workflow_revision() {
    let (_directory, store) = store();
    let mut workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    workflow.add_shell_node("drift", SHELL_COMMAND);
    store.save_workflow(&workflow).expect("drifted workflow must persist");

    let report = validate_definition_in_store(&store, &definition).expect("validation must run");

    assert!(!report.valid());
    assert!(!report.orchestration_diagnostics.is_empty());
}

// r[verify lattice.agent_orchestration.authority]
// r[verify lattice.agent_orchestration.rounds]
#[tokio::test]
async fn approved_successor_runs_as_a_new_immutable_round() {
    let (_directory, store) = store();
    let first = workflow(FIRST_WORKFLOW_NAME);
    let second = workflow(SECOND_WORKFLOW_NAME);
    store.save_workflow(&first).expect("first workflow must persist");
    store.save_workflow(&second).expect("second workflow must persist");
    let waiting =
        start(&store, definition(&first, CompletionPolicy::AwaitProposal), ACTOR, AgentRuntimeConfig::default())
            .await
            .expect("first round must run");
    assert_eq!(waiting.status, OrchestrationStatus::WaitingForApproval);

    let proposal = make_proposal(
        &waiting,
        workflow_revision(&second).expect("successor must hash"),
        "operator-reviewed successor",
    )
    .expect("proposal must hash");
    let advanced = approve(&store, &waiting.id, proposal, ACTOR, AgentRuntimeConfig::default())
        .await
        .expect("approved successor must run");

    assert_eq!(advanced.status, OrchestrationStatus::WaitingForApproval);
    assert_eq!(advanced.round, ROUND_LIMIT);
    assert_eq!(advanced.workflow_revision.workflow_name, SECOND_WORKFLOW_NAME);
    assert_eq!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").len(), ROUND_LIMIT as usize);
}

// r[verify lattice.agent_orchestration.triggers]
#[tokio::test]
async fn duplicate_trigger_returns_the_original_instance_without_another_run() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    store.save_workflow(&workflow).expect("workflow must persist");
    store.save_orchestration_definition(&definition).expect("definition must persist");

    let first = start_from_trigger(
        &store,
        DEFINITION_ID,
        TRIGGER_ID,
        OrchestrationTriggerKind::Webhook,
        IDEMPOTENCY_KEY,
        AgentRuntimeConfig::default(),
    )
    .await
    .expect("first trigger must run");
    let duplicate = start_from_trigger(
        &store,
        DEFINITION_ID,
        TRIGGER_ID,
        OrchestrationTriggerKind::Webhook,
        IDEMPOTENCY_KEY,
        AgentRuntimeConfig::default(),
    )
    .await
    .expect("duplicate trigger must return prior result");

    assert_eq!(duplicate.id, first.id);
    assert_eq!(store.list_orchestrations().expect("instances must list").len(), FIRST_SEQUENCE as usize);
    assert_eq!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").len(), FIRST_SEQUENCE as usize);
}

// r[verify lattice.agent_orchestration.triggers.scenario.process-restart]
#[test]
fn restart_duplicate_trigger_reuses_original_admission_across_processes() {
    if run_restart_child_phase() {
        return;
    }

    let directory = tempdir().expect("temporary directory must exist");
    let store_path = directory.path().join("restart.redb");
    let first_result_path = directory.path().join("first-instance-id");
    let duplicate_result_path = directory.path().join("duplicate-instance-id");
    let first_status =
        invoke_restart_child(&store_path, &first_result_path, &duplicate_result_path, RESTART_PHASE_FIRST);
    assert!(first_status.success(), "first process must admit and complete the trigger");
    let duplicate_status =
        invoke_restart_child(&store_path, &first_result_path, &duplicate_result_path, RESTART_PHASE_DUPLICATE);
    assert!(duplicate_status.success(), "restarted process must reuse the trigger admission");

    let first_result = parse_restart_result(
        &std::fs::read_to_string(&first_result_path).expect("first process must record its safe identity"),
    )
    .expect("first process result must decode");
    let duplicate_result = parse_restart_result(
        &std::fs::read_to_string(&duplicate_result_path).expect("duplicate process must record its safe identity"),
    )
    .expect("duplicate process result must decode");
    assert_ne!(duplicate_result.process_id, first_result.process_id);
    assert_eq!(duplicate_result.instance_id, first_result.instance_id);

    let store = Store::new(&store_path).expect("parent must reopen the released store");
    let admissions = store.list_orchestration_trigger_admissions().expect("admissions must list");
    let instances = store.list_orchestrations().expect("instances must list");
    let runs = store.list_runs(ROUND_LIMIT as usize).expect("runs must list");
    assert_eq!(admissions.len(), FIRST_SEQUENCE as usize);
    assert_eq!(instances.len(), FIRST_SEQUENCE as usize);
    assert_eq!(runs.len(), FIRST_SEQUENCE as usize);
    let instance = &instances[0];
    let expected_scoped_key =
        lattice::orchestration::scoped_trigger_key(&instance.definition_revision, TRIGGER_ID, IDEMPOTENCY_KEY)
            .expect("scoped trigger identity must hash");
    assert_eq!(admissions.get(&expected_scoped_key).map(String::as_str), Some(first_result.instance_id.as_str()));
    assert_eq!(instance.trigger_idempotency_key.as_deref(), Some(expected_scoped_key.as_str()));
    assert_eq!(instance.id, first_result.instance_id);
    assert_eq!(instance.status, OrchestrationStatus::Completed);
    assert_eq!(instance.budgets.consumed, BudgetCharge {
        rounds: FIRST_SEQUENCE,
        ..BudgetCharge::default()
    });
    assert!(instance.budgets.reservations.is_empty());
    assert!(!instance.budgets.breached);
    assert_eq!(instance.round, FIRST_SEQUENCE);
    assert!(instance.active_run_id.is_none());
    assert!(instance.pending_dispatch.is_none());
    let RunOwner::Orchestration {
        instance_id,
        dispatch_id,
    } = &runs[0].owner
    else {
        panic!("restarted trigger run must retain orchestration ownership");
    };
    assert_eq!(instance_id, &instance.id);
    assert!(!dispatch_id.is_empty());
    let report = replay(&store, &instance.id).expect("restarted instance replay must validate");
    assert!(report.valid);
}

// r[verify lattice.agent_orchestration.triggers.scenario.process-restart]
#[test]
fn restart_child_rejects_unknown_phase_before_creating_store() {
    if run_restart_child_phase() {
        return;
    }

    let directory = tempdir().expect("temporary directory must exist");
    let store_path = directory.path().join("must-not-exist.redb");
    let first_result_path = directory.path().join("must-not-write-first");
    let duplicate_result_path = directory.path().join("must-not-write-duplicate");
    let status = invoke_restart_child(&store_path, &first_result_path, &duplicate_result_path, RESTART_PHASE_INVALID);

    assert!(!status.success());
    assert!(!store_path.exists());
    assert!(!first_result_path.exists());
    assert!(!duplicate_result_path.exists());
}

fn run_restart_child_phase() -> bool {
    let Ok(phase_value) = std::env::var(RESTART_PHASE_ENV) else {
        return false;
    };
    let phase = parse_restart_phase(&phase_value).expect("restart child phase must be recognized");
    let store_path = std::env::var_os(RESTART_STORE_ENV).expect("restart child store path must be present");
    let first_result_path =
        std::env::var_os(RESTART_FIRST_RESULT_ENV).expect("restart child first result path must be present");
    let duplicate_result_path =
        std::env::var_os(RESTART_DUPLICATE_RESULT_ENV).expect("restart child duplicate result path must be present");
    run_restart_child(phase, Path::new(&store_path), Path::new(&first_result_path), Path::new(&duplicate_result_path));
    true
}

fn run_restart_child(phase: RestartPhase, store_path: &Path, first_result_path: &Path, duplicate_result_path: &Path) {
    let store = Store::new(store_path).expect("restart child store must open");
    if phase == RestartPhase::First {
        let workflow = workflow(FIRST_WORKFLOW_NAME);
        let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
        store.save_workflow(&workflow).expect("restart workflow must persist");
        store.save_orchestration_definition(&definition).expect("restart definition must persist");
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("restart child runtime must build");
    let instance = runtime
        .block_on(start_from_trigger(
            &store,
            DEFINITION_ID,
            TRIGGER_ID,
            OrchestrationTriggerKind::Webhook,
            IDEMPOTENCY_KEY,
            AgentRuntimeConfig::default(),
        ))
        .expect("restart child trigger must succeed");
    let result = RestartResult {
        process_id: std::process::id(),
        instance_id: instance.id.clone(),
    };
    match phase {
        RestartPhase::First => {
            std::fs::write(first_result_path, encode_restart_result(&result))
                .expect("first child must record safe process and instance identities");
        }
        RestartPhase::Duplicate => {
            let expected = parse_restart_result(
                &std::fs::read_to_string(first_result_path)
                    .expect("duplicate child must read first process and instance identities"),
            )
            .expect("first process result must decode");
            assert_ne!(result.process_id, expected.process_id);
            assert_eq!(result.instance_id, expected.instance_id);
            std::fs::write(duplicate_result_path, encode_restart_result(&result))
                .expect("duplicate child must record safe process and instance identities");
        }
    }
}

fn invoke_restart_child(
    store_path: &Path,
    first_result_path: &Path,
    duplicate_result_path: &Path,
    phase: &str,
) -> std::process::ExitStatus {
    Command::new(std::env::current_exe().expect("current integration-test executable must resolve"))
        .arg("--exact")
        .arg(RESTART_TEST_NAME)
        .env(RESTART_PHASE_ENV, phase)
        .env(RESTART_STORE_ENV, store_path)
        .env(RESTART_FIRST_RESULT_ENV, first_result_path)
        .env(RESTART_DUPLICATE_RESULT_ENV, duplicate_result_path)
        .status()
        .expect("restart child process must launch")
}

// r[verify lattice.agent_orchestration.observed_failure_retry.scenario.observed]
// r[verify lattice.agent_orchestration.retry_round.scenario.success]
#[tokio::test]
async fn exact_observed_failure_retries_with_fresh_identity_budget_and_replay() {
    let (directory, store) = store();
    let marker = directory.path().join("retry-marker");
    let workflow = fail_once_workflow(FIRST_WORKFLOW_NAME, &marker);
    store.save_workflow(&workflow).expect("retry workflow must persist");

    let failed =
        start(&store, definition(&workflow, CompletionPolicy::RunSucceeded), ACTOR, AgentRuntimeConfig::default())
            .await
            .expect("first observed attempt must terminalize");
    let first_record = store
        .load_run(RunId(FIRST_ALLOCATED_RUN_ID))
        .expect("first run lookup must succeed")
        .expect("first run must persist");
    let first_failure = failed.retry.retryable_failure.clone().unwrap_or_else(|| {
        panic!("exact failed run must become retryable: record={first_record:?}, instance={failed:?}")
    });
    assert_eq!(failed.status, OrchestrationStatus::Failed);
    assert_eq!(first_failure.run_id, RunId(FIRST_ALLOCATED_RUN_ID));
    assert_eq!(failed.budgets.consumed.rounds, FIRST_SEQUENCE);
    assert!(failed.budgets.reservations.is_empty());
    assert!(!marker.exists());
    let unauthorized = retry(
        &store,
        RetryRequest {
            instance_id: &failed.id,
            actor: UNAUTHORIZED_ACTOR,
        },
        AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("unauthorized retry must fail before dispatch");
    assert!(unauthorized.to_string().contains("not authorized"));
    assert_eq!(store.list_runs(RETRY_RUN_LIST_LIMIT).expect("run list must load").len(), FIRST_SEQUENCE as usize);
    assert_eq!(store.load_orchestration(&failed.id).expect("failed instance must load"), Some(failed.clone()));
    std::fs::write(&marker, RETRY_MARKER_CONTENT).expect("retry marker must persist");

    let completed = retry(
        &store,
        RetryRequest {
            instance_id: &failed.id,
            actor: ACTOR,
        },
        AgentRuntimeConfig::default(),
    )
    .await
    .expect("authorized observed failure retry must run");

    assert_eq!(completed.status, OrchestrationStatus::Completed);
    assert_eq!(completed.round, ROUND_LIMIT);
    assert_eq!(completed.retry.admitted, RETRY_LIMIT);
    assert!(completed.retry.retryable_failure.is_none());
    assert_eq!(completed.budgets.consumed.rounds, ROUND_LIMIT);
    assert!(completed.budgets.reservations.is_empty());
    let runs = store.list_runs(RETRY_RUN_LIST_LIMIT).expect("retry run history must load");
    assert_eq!(runs.len(), RETRY_RUN_LIST_LIMIT);
    let first = runs
        .iter()
        .find(|record| record.run_id == RunId(FIRST_ALLOCATED_RUN_ID))
        .expect("first run must remain durable");
    let second = runs
        .iter()
        .find(|record| record.run_id == RunId(SECOND_ALLOCATED_RUN_ID))
        .expect("retry run must remain durable");
    assert!(first.failures > 0);
    assert_eq!(second.failures, 0);
    let RunOwner::Orchestration {
        instance_id: retry_instance,
        dispatch_id: retry_dispatch,
    } = &second.owner
    else {
        panic!("retry run must have orchestration ownership");
    };
    assert_eq!(retry_instance, &completed.id);
    assert_ne!(retry_dispatch, &first_failure.dispatch_id);
    let events = store.load_orchestration_events(&completed.id).expect("retry events must load");
    assert_eq!(
        events.iter().filter(|event| event.kind == OrchestrationEventKind::RetryRoundStarted).count(),
        RETRY_LIMIT as usize
    );
    let report = replay(&store, &completed.id).expect("retry replay must validate");
    assert!(report.valid);
    assert_eq!(report.instance, completed);
}

// r[verify lattice.agent_orchestration.retry_round.scenario.pending-unobserved]
// r[verify lattice.agent_orchestration.retry_round.scenario.unsafe-evidence]
#[tokio::test]
async fn pending_unobserved_and_abandoned_dispatches_never_retry() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (pending, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&pending, &events).expect("pending instance must persist");

    let pending_error = retry(
        &store,
        RetryRequest {
            instance_id: &pending.id,
            actor: ACTOR,
        },
        AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("pending-unobserved dispatch cannot retry");
    assert!(pending_error.to_string().contains("pending and locally unobserved"));
    assert_eq!(store.load_orchestration(&pending.id).expect("pending instance must load"), Some(pending.clone()));
    assert!(store.list_runs(RETRY_RUN_LIST_LIMIT).expect("run list must load").is_empty());

    let abandoned = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &pending.id,
        actor: ACTOR,
    })
    .expect("authorized abandonment must close uncertainty");
    let abandoned_error = retry(
        &store,
        RetryRequest {
            instance_id: &abandoned.id,
            actor: ACTOR,
        },
        AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("abandoned dispatch cannot become retry authority");
    assert!(abandoned_error.to_string().contains("no exact persisted terminal-failure retry evidence"));
    assert_eq!(store.load_orchestration(&abandoned.id).expect("abandoned instance must load"), Some(abandoned));
    assert!(store.list_runs(RETRY_RUN_LIST_LIMIT).expect("run list must load").is_empty());

    let foreign_directory = tempdir().expect("foreign temporary directory must exist");
    let foreign_store =
        Store::new(foreign_directory.path().join("foreign-shell.redb")).expect("foreign store must open");
    foreign_store.save_workflow(&workflow).expect("foreign workflow must persist");
    let (foreign_pending, foreign_events) = cancelled_acknowledged_instance(&workflow);
    foreign_store
        .create_orchestration(&foreign_pending, &foreign_events)
        .expect("foreign pending instance must persist");
    foreign_store
        .save_run(&RunRecord {
            run_id: RunId(RECOVERY_RUN_ID),
            owner: RunOwner::Unscoped,
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: FIRST_SEQUENCE as usize,
        })
        .expect("foreign run must persist");
    let foreign_error = retry(
        &foreign_store,
        RetryRequest {
            instance_id: &foreign_pending.id,
            actor: ACTOR,
        },
        AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("foreign owner cannot provide retry evidence");
    assert!(foreign_error.to_string().contains("not owned by the pending orchestration dispatch"));
    assert_eq!(
        foreign_store.load_orchestration(&foreign_pending.id).expect("foreign pending instance must load"),
        Some(foreign_pending)
    );
}

// r[verify lattice.agent_orchestration.durability]
#[test]
fn recovery_reconciles_persisted_run_without_redispatch() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    store.save_workflow(&workflow).expect("workflow must persist");
    let base = new_instance(definition, FIRST_SEQUENCE, None).expect("instance must hash");
    let run_id = RunId(RECOVERY_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(&base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let action = OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(FIRST_SEQUENCE),
            agent_invocations: Some(0),
            tokens: Some(0),
            monetary_microunits: Some(0),
            duration_millis: Some(0),
        },
    };
    let started = transition(&base, OrchestrationFact {
        id: fact_identity(&base.id, base.version, &action).expect("fact must hash"),
        action,
    })
    .expect("start must apply");
    store
        .create_orchestration(
            &started.instance,
            std::slice::from_ref(started.event.as_ref().expect("event must exist")),
        )
        .expect("started instance must persist");
    store
        .save_run(&RunRecord {
            run_id,
            owner: RunOwner::Orchestration {
                instance_id: started.instance.id.clone(),
                dispatch_id: started
                    .instance
                    .pending_dispatch
                    .as_ref()
                    .expect("dispatch must remain pending")
                    .id
                    .clone(),
            },
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: 0,
        })
        .expect("observed run must persist");

    let outcome = recover_pending(&store, &started.instance.id).expect("recovery must run");
    let RecoveryOutcome::Reconciled(reconciled) = outcome else {
        panic!("existing run must reconcile");
    };
    assert_eq!(reconciled.status, OrchestrationStatus::Completed);
    assert!(reconciled.pending_dispatch.is_none());
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn unobserved_cancelled_dispatch_requires_authority_then_abandons_without_retry() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (instance, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&instance, &events).expect("pending instance must persist");

    let outcome = recover_pending(&store, &instance.id).expect("recovery must inspect pending run");
    assert_eq!(outcome, RecoveryOutcome::PendingUnobserved {
        run_id: RunId(RECOVERY_RUN_ID),
    });
    let unauthorized = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: UNAUTHORIZED_ACTOR,
    })
    .expect_err("unauthorized abandonment must fail");
    assert!(unauthorized.to_string().contains("not authorized"));
    assert_eq!(store.load_orchestration(&instance.id).expect("instance must load"), Some(instance.clone()));

    let abandoned = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect("authorized abandonment must apply");
    assert_eq!(abandoned.status, OrchestrationStatus::Failed);
    assert!(abandoned.pending_dispatch.is_none());
    assert!(abandoned.active_run_id.is_none());
    assert!(abandoned.budgets.reservations.is_empty());
    assert_eq!(abandoned.budgets.consumed.rounds, FIRST_SEQUENCE);
    let report = replay(&store, &instance.id).expect("abandonment replay must validate");
    assert!(report.valid);
    assert_eq!(report.instance, abandoned);

    let repeated = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect_err("terminal abandonment cannot repeat");
    assert!(repeated.to_string().contains("no pending dispatch"));
    assert!(store.load_run(RunId(RECOVERY_RUN_ID)).expect("run lookup must succeed").is_none());
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn observed_pending_run_rejects_abandonment_and_uses_recovery() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (instance, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&instance, &events).expect("pending instance must persist");
    store
        .save_run(&RunRecord {
            run_id: RunId(RECOVERY_RUN_ID),
            owner: RunOwner::Orchestration {
                instance_id: instance.id.clone(),
                dispatch_id: instance.pending_dispatch.as_ref().expect("dispatch must remain pending").id.clone(),
            },
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: 0,
        })
        .expect("observed run must persist");

    let rejected = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect_err("observed run cannot be abandoned");
    assert!(rejected.to_string().contains("use orchestration recover"));
    assert_eq!(store.load_orchestration(&instance.id).expect("instance must load"), Some(instance.clone()));

    let outcome = recover_pending(&store, &instance.id).expect("ordinary recovery must reconcile");
    let RecoveryOutcome::Reconciled(reconciled) = outcome else {
        panic!("observed run must reconcile");
    };
    assert_eq!(reconciled.status, OrchestrationStatus::Cancelled);
    assert!(reconciled.pending_dispatch.is_none());
}

// r[verify lattice.agent_orchestration.run_identity]
#[test]
fn foreign_run_record_cannot_satisfy_or_abandon_a_pending_dispatch() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (instance, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&instance, &events).expect("pending instance must persist");
    store
        .save_run(&RunRecord {
            run_id: RunId(RECOVERY_RUN_ID),
            owner: RunOwner::Unscoped,
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: 0,
        })
        .expect("foreign run must persist");

    let recovery_error = recover_pending(&store, &instance.id).expect_err("foreign run cannot satisfy recovery");
    let abandon_error = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect_err("foreign run blocks abandonment");
    let persisted = store
        .load_orchestration(&instance.id)
        .expect("instance lookup must succeed")
        .expect("instance remains persisted");

    assert!(recovery_error.to_string().contains("not owned by the pending orchestration dispatch"));
    assert!(abandon_error.to_string().contains("not owned by the pending orchestration dispatch"));
    assert_eq!(persisted, instance);
    assert!(persisted.pending_dispatch.is_some());
    assert_eq!(persisted.budgets.reservations.len(), FIRST_SEQUENCE as usize);
}

// r[verify lattice.agent_orchestration.triggers]
#[tokio::test]
async fn unbound_trigger_fails_without_instance_or_run() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    store.save_workflow(&workflow).expect("workflow must persist");
    store.save_orchestration_definition(&definition).expect("definition must persist");

    let error = start_from_trigger(
        &store,
        DEFINITION_ID,
        "missing-trigger",
        OrchestrationTriggerKind::Schedule,
        IDEMPOTENCY_KEY,
        AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("unbound trigger must fail");

    assert!(error.to_string().contains("not bound"));
    assert!(store.list_orchestrations().expect("instances must list").is_empty());
    assert!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").is_empty());
}
