use std::collections::BTreeSet;
use std::process::Command;

use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::engine::runner::RunnerEventKind;
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
use lattice::orchestration::OrchestrationFact;
use lattice::orchestration::OrchestrationInstance;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::dispatch_identity;
use lattice::orchestration::fact_identity;
use lattice::orchestration::new_instance;
use lattice::orchestration::transition;
use lattice::orchestration::workflow_revision;
use lattice::replay::ReplayEventRecord;
use lattice::run::inspection::BudgetSettlementState;
use lattice::run::inspection::DiagnosticCode;
use lattice::run::inspection::DispatchEvidenceState;
use lattice::run::inspection::EvidenceState;
use lattice::run::inspection::Input;
use lattice::run::inspection::OrchestrationInput;
use lattice::run::inspection::OwnerBindingState;
use lattice::run::inspection::RecoveryEvidenceState;
use lattice::run::inspection::Report;
use lattice::run::inspection::SafeOwner;
use lattice::run::inspection::inspect;
use lattice::run::inspection::render_json;
use lattice::run::inspection::render_text;
use lattice::store::RunDetailRecord;
use lattice::store::RunNodeDetail;
use lattice::store::RunNodeDetailStatus;
use lattice::store::RunOutputDetail;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::store::Store;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;

const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const RUN_ID_VALUE: u64 = 41;
const OTHER_RUN_ID_VALUE: u64 = 42;
const NODE_ID_VALUE: u64 = 7;
const PORT_ID_VALUE: u64 = 9;
const STARTED_EPOCH_MS: i64 = 1_000;
const FINISHED_EPOCH_MS: i64 = 2_000;
const NODE_COUNT: usize = 1;
const CACHE_HITS: usize = 0;
const FAILURES: usize = 0;
const FIRST_SEQUENCE: u64 = 0;
const SECOND_SEQUENCE: u64 = 1;
const INSTANCE_SEQUENCE: u64 = 3;
const ROUND_LIMIT: u64 = 2;
const ROUND_CHARGE: u64 = 1;
const PRIVATE_PROMPT: &str = "PRIVATE_PROMPT_DO_NOT_RENDER";
const PRIVATE_REASONING: &str = "<think>PRIVATE_REASONING_DO_NOT_RENDER</think>";
const PRIVATE_CREDENTIAL: &str = "sk-live-PRIVATE_CREDENTIAL_DO_NOT_RENDER";
const PRIVATE_AUTHORIZATION: &str = "Authorization: Bearer PRIVATE_AUTHORIZATION_DO_NOT_RENDER";
const ORCHESTRATION_ID: &str = "safe-inspection";
const SAFE_ACTOR: &str = "operator:safe";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 1;

fn record(owner: RunOwner) -> RunRecord {
    RunRecord {
        run_id: RunId(RUN_ID_VALUE),
        owner,
        workflow_name: PRIVATE_PROMPT.to_string(),
        started_epoch_ms: STARTED_EPOCH_MS,
        finished_epoch_ms: Some(FINISHED_EPOCH_MS),
        node_count: NODE_COUNT,
        cache_hits: CACHE_HITS,
        failures: FAILURES,
    }
}

fn details(run_id: RunId) -> RunDetailRecord {
    RunDetailRecord {
        run_id,
        nodes: vec![RunNodeDetail {
            node_id: NodeId(NODE_ID_VALUE),
            node_label: PRIVATE_REASONING.to_string(),
            status: RunNodeDetailStatus::Completed,
            cached: false,
            outputs: vec![RunOutputDetail {
                port_id: PortId(PORT_ID_VALUE),
                port_label: PRIVATE_CREDENTIAL.to_string(),
                text: PRIVATE_PROMPT.to_string(),
            }],
            error: Some(PRIVATE_AUTHORIZATION.to_string()),
        }],
    }
}

fn replay_event(run_id: RunId, sequence: u64, kind: RunnerEventKind) -> ReplayEventRecord {
    ReplayEventRecord {
        run_id,
        sequence,
        timestamp_epoch_ms: Some(STARTED_EPOCH_MS),
        node_id: None,
        port_id: None,
        parent_run_id: None,
        kind,
        summary: PRIVATE_REASONING.to_string(),
        redacted: false,
        value: None,
        artifact: None,
        contains_plaintext_secret_material: false,
    }
}

fn valid_replay(run_id: RunId) -> Vec<ReplayEventRecord> {
    vec![
        replay_event(run_id, FIRST_SEQUENCE, RunnerEventKind::RunStarted),
        replay_event(run_id, SECOND_SEQUENCE, RunnerEventKind::RunCompleted),
    ]
}

fn assert_private_content_absent(rendered: &str) {
    assert!(!rendered.contains(PRIVATE_PROMPT));
    assert!(!rendered.contains(PRIVATE_REASONING));
    assert!(!rendered.contains(PRIVATE_CREDENTIAL));
    assert!(!rendered.contains(PRIVATE_AUTHORIZATION));
    assert!(!rendered.contains("<think>"));
}

fn started_orchestration_fixture() -> (OrchestrationInstance, Vec<OrchestrationEvent>, String) {
    let workflow = Workflow::new(PRIVATE_PROMPT);
    let definition = OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: ORCHESTRATION_ID.to_string(),
        objective: PRIVATE_PROMPT.to_string(),
        workflow: workflow_revision(&workflow).expect("workflow revision"),
        completion: CompletionPolicy::RunSucceeded,
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
            approvers: BTreeSet::from([SAFE_ACTOR.to_string(), PRIVATE_CREDENTIAL.to_string()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    let initial = new_instance(definition, INSTANCE_SEQUENCE, None).expect("instance");
    let run_id = RunId(RUN_ID_VALUE);
    let dispatch_id = dispatch_identity(&initial, run_id).expect("dispatch identity");
    let intent = DispatchIntent {
        id: dispatch_id.clone(),
        run_id,
        workflow_revision: initial.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let start = apply(&initial, OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(ROUND_CHARGE),
            agent_invocations: Some(0),
            tokens: Some(0),
            monetary_microunits: Some(0),
            duration_millis: Some(0),
        },
    });
    (start.0, vec![start.1], dispatch_id)
}

fn orchestration_fixture() -> (OrchestrationInstance, Vec<OrchestrationEvent>, String) {
    let (started, mut events, dispatch_id) = started_orchestration_fixture();
    let acknowledged = apply(&started, OrchestrationAction::DispatchAcknowledged {
        dispatch_id: dispatch_id.clone(),
    });
    events.push(acknowledged.1);
    let finished = apply(&acknowledged.0, OrchestrationAction::RunFinished {
        dispatch_id: dispatch_id.clone(),
        run_id: RunId(RUN_ID_VALUE),
        succeeded: true,
        observed_usage: BudgetCharge {
            rounds: ROUND_CHARGE,
            ..BudgetCharge::default()
        },
        safe_summary: PRIVATE_REASONING.to_string(),
    });
    events.push(finished.1);
    (finished.0, events, dispatch_id)
}

fn abandoned_orchestration_fixture() -> (OrchestrationInstance, Vec<OrchestrationEvent>, String) {
    let (started, mut events, dispatch_id) = started_orchestration_fixture();
    let acknowledged = apply(&started, OrchestrationAction::DispatchAcknowledged {
        dispatch_id: dispatch_id.clone(),
    });
    events.push(acknowledged.1);
    let cancelled = apply(&acknowledged.0, OrchestrationAction::CancelRequested {
        actor: SAFE_ACTOR.to_string(),
    });
    events.push(cancelled.1);
    let abandoned = apply(&cancelled.0, OrchestrationAction::AbandonPendingDispatch {
        actor: SAFE_ACTOR.to_string(),
        dispatch_id: dispatch_id.clone(),
        run_id: RunId(RUN_ID_VALUE),
    });
    events.push(abandoned.1);
    (abandoned.0, events, dispatch_id)
}

fn apply(instance: &OrchestrationInstance, action: OrchestrationAction) -> (OrchestrationInstance, OrchestrationEvent) {
    let fact = OrchestrationFact {
        id: fact_identity(&instance.id, instance.version, &action).expect("fact identity"),
        action,
    };
    let transitioned = transition(instance, fact).expect("transition");
    (transitioned.instance, transitioned.event.expect("event"))
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn safe_projection_omits_arbitrary_persisted_content() {
    let record = record(RunOwner::Unscoped);
    let details = details(record.run_id);
    let replay = valid_replay(record.run_id);
    let report = inspect(Input {
        record: &record,
        details: Some(&details),
        replay_events: Some(&replay),
        orchestration: None,
    });
    let json = render_json(&report).expect("JSON rendering");
    let text = render_text(&report);

    assert!(report.valid);
    assert_eq!(report.details.state, EvidenceState::Valid);
    assert_eq!(report.replay.state, EvidenceState::Valid);
    assert_eq!(report.details.nodes.len(), NODE_COUNT);
    assert_eq!(report.details.nodes[0].output_count, NODE_COUNT);
    assert!(report.details.nodes[0].error_present);
    assert_private_content_absent(&json);
    assert_private_content_absent(&text);
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn missing_optional_history_is_explicit_without_invalidating_legacy_run() {
    let record = record(RunOwner::Unscoped);
    let report = inspect(Input {
        record: &record,
        details: None,
        replay_events: None,
        orchestration: None,
    });

    assert!(report.valid);
    assert_eq!(report.details.state, EvidenceState::NotRecorded);
    assert_eq!(report.replay.state, EvidenceState::NotRecorded);
    assert!(report.diagnostic_codes.is_empty());
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn present_foreign_detail_and_replay_identity_fail_closed() {
    let record = record(RunOwner::Unscoped);
    let details = details(RunId(OTHER_RUN_ID_VALUE));
    let replay = valid_replay(RunId(OTHER_RUN_ID_VALUE));
    let report = inspect(Input {
        record: &record,
        details: Some(&details),
        replay_events: Some(&replay),
        orchestration: None,
    });

    assert!(!report.valid);
    assert_eq!(report.details.state, EvidenceState::Invalid);
    assert_eq!(report.replay.state, EvidenceState::Invalid);
    assert!(report.diagnostic_codes.contains(&DiagnosticCode::DetailRunIdentityMismatch));
    assert!(report.diagnostic_codes.contains(&DiagnosticCode::ReplayRunIdentityMismatch));
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn malformed_missing_owner_identities_are_not_echoed() {
    let record = record(RunOwner::Orchestration {
        instance_id: PRIVATE_PROMPT.to_string(),
        dispatch_id: PRIVATE_CREDENTIAL.to_string(),
    });
    let report = inspect(Input {
        record: &record,
        details: None,
        replay_events: None,
        orchestration: None,
    });
    let rendered = render_json(&report).expect("JSON rendering");

    assert!(!report.valid);
    assert!(report.diagnostic_codes.contains(&DiagnosticCode::OwnerIdentityMalformed));
    assert!(report.diagnostic_codes.contains(&DiagnosticCode::OrchestrationMissing));
    assert_private_content_absent(&rendered);
    assert!(rendered.contains("[invalid]"));
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn exact_orchestration_owner_reports_finished_dispatch_and_settled_budget() {
    let (instance, events, dispatch_id) = orchestration_fixture();
    let record = record(RunOwner::Orchestration {
        instance_id: instance.id.clone(),
        dispatch_id,
    });
    let report = inspect(Input {
        record: &record,
        details: None,
        replay_events: None,
        orchestration: Some(OrchestrationInput {
            instance: &instance,
            events: &events,
        }),
    });

    assert!(report.valid, "report: {report:?}");
    assert!(matches!(report.owner, SafeOwner::Orchestration {
        binding: OwnerBindingState::Exact,
        ..
    }));
    let evidence = report.orchestration.expect("orchestration evidence");
    assert_eq!(evidence.event_chain, EvidenceState::Valid);
    assert_eq!(evidence.dispatch, DispatchEvidenceState::Finished);
    assert_eq!(evidence.recovery, RecoveryEvidenceState::Closed);
    assert_eq!(evidence.budget.settlement, BudgetSettlementState::Settled);
    assert_eq!(evidence.budget.observed.expect("observed budget").rounds, ROUND_CHARGE);
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn acknowledged_abandonment_reports_conservative_closed_settlement() {
    let (instance, events, dispatch_id) = abandoned_orchestration_fixture();
    let record = record(RunOwner::Orchestration {
        instance_id: instance.id.clone(),
        dispatch_id,
    });
    let report = inspect(Input {
        record: &record,
        details: None,
        replay_events: None,
        orchestration: Some(OrchestrationInput {
            instance: &instance,
            events: &events,
        }),
    });

    assert!(report.valid, "report: {report:?}");
    let evidence = report.orchestration.as_ref().expect("orchestration evidence");
    assert_eq!(evidence.dispatch, DispatchEvidenceState::AbandonedAcknowledged);
    assert_eq!(evidence.recovery, RecoveryEvidenceState::Closed);
    assert_eq!(evidence.budget.settlement, BudgetSettlementState::ConservativeAcknowledgedAbandonment);
    assert!(evidence.budget.observed.is_none());
    assert_private_content_absent(&render_text(&report));
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn foreign_dispatch_owner_and_duplicate_admission_are_invalid() {
    let (instance, events, dispatch_id) = orchestration_fixture();
    let foreign_record = record(RunOwner::Orchestration {
        instance_id: instance.id.clone(),
        dispatch_id: "foreign-dispatch".to_string(),
    });
    let foreign_report = inspect(Input {
        record: &foreign_record,
        details: None,
        replay_events: None,
        orchestration: Some(OrchestrationInput {
            instance: &instance,
            events: &events,
        }),
    });

    assert!(!foreign_report.valid);
    assert!(foreign_report.diagnostic_codes.contains(&DiagnosticCode::DispatchBindingMissing));
    assert!(foreign_report.diagnostic_codes.contains(&DiagnosticCode::DispatchFactMismatch));

    let mut duplicate_events = events.clone();
    duplicate_events.push(events[0].clone());
    let duplicate_record = record(RunOwner::Orchestration {
        instance_id: instance.id.clone(),
        dispatch_id,
    });
    let duplicate_report = inspect(Input {
        record: &duplicate_record,
        details: None,
        replay_events: None,
        orchestration: Some(OrchestrationInput {
            instance: &instance,
            events: &duplicate_events,
        }),
    });

    assert!(!duplicate_report.valid);
    assert!(duplicate_report.diagnostic_codes.contains(&DiagnosticCode::OrchestrationReplayInvalid));
    assert!(duplicate_report.diagnostic_codes.contains(&DiagnosticCode::DispatchBindingDuplicate));
    assert_private_content_absent(&render_json(&foreign_report).expect("foreign JSON rendering"));
    assert_private_content_absent(&render_json(&duplicate_report).expect("duplicate JSON rendering"));
}

fn cli_fixture(node_count: usize) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let data_dir = directory.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory");
    let config_path = directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string()
        ),
    )
    .expect("config write");
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store");
    let mut record = record(RunOwner::Unscoped);
    record.node_count = node_count;
    store.save_run(&record).expect("run save");
    store.save_run_details(&details(record.run_id)).expect("details save");
    store.save_run_replay_events(record.run_id, &valid_replay(record.run_id)).expect("replay save");
    drop(store);
    (directory, config_path)
}

fn orchestration_cli_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let data_dir = directory.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory");
    let config_path = directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string()
        ),
    )
    .expect("config write");
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store");
    let (instance, events, dispatch_id) = orchestration_fixture();
    store.create_orchestration(&instance, &events).expect("orchestration save");
    let record = record(RunOwner::Orchestration {
        instance_id: instance.id,
        dispatch_id,
    });
    store.save_run(&record).expect("run save");
    store.save_run_details(&details(record.run_id)).expect("details save");
    store.save_run_replay_events(record.run_id, &valid_replay(record.run_id)).expect("replay save");
    drop(store);
    (directory, config_path)
}

fn run_show(config_path: &std::path::Path, output: &str, run_id: u64) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(config_path)
        .arg("runs")
        .arg("show")
        .arg(run_id.to_string())
        .arg("--output")
        .arg(output)
        .output()
        .expect("runs show command")
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn cli_renders_safe_text_and_json_for_persisted_run() {
    let (_directory, config_path) = cli_fixture(NODE_COUNT);
    let json_output = run_show(&config_path, "json", RUN_ID_VALUE);
    let text_output = run_show(&config_path, "text", RUN_ID_VALUE);

    assert!(json_output.status.success(), "stderr: {}", String::from_utf8_lossy(&json_output.stderr));
    assert!(text_output.status.success(), "stderr: {}", String::from_utf8_lossy(&text_output.stderr));
    let parsed: Report = serde_json::from_slice(&json_output.stdout).expect("inspection JSON");
    assert!(parsed.valid);
    assert_private_content_absent(&String::from_utf8_lossy(&json_output.stdout));
    assert_private_content_absent(&String::from_utf8_lossy(&text_output.stdout));
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn cli_reports_exact_orchestration_binding_without_private_content() {
    let (_directory, config_path) = orchestration_cli_fixture();
    let output = run_show(&config_path, "json", RUN_ID_VALUE);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let report: Report = serde_json::from_slice(&output.stdout).expect("inspection JSON");
    assert!(matches!(report.owner, SafeOwner::Orchestration {
        binding: OwnerBindingState::Exact,
        ..
    }));
    assert_eq!(
        report.orchestration.as_ref().expect("orchestration evidence").dispatch,
        DispatchEvidenceState::Finished
    );
    assert_private_content_absent(&String::from_utf8_lossy(&output.stdout));
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn cli_missing_store_does_not_create_data_directory_or_database() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let data_dir = directory.path().join("missing-data");
    let config_path = directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string()
        ),
    )
    .expect("config write");

    let output = run_show(&config_path, "json", RUN_ID_VALUE);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("run evidence store is unavailable"));
    assert!(!data_dir.exists());
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn cli_not_found_and_invalid_present_evidence_exit_unsuccessfully() {
    let (_missing_directory, missing_config) = cli_fixture(NODE_COUNT);
    let missing = run_show(&missing_config, "json", OTHER_RUN_ID_VALUE);
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("run 42 not found"),
        "stderr: {}",
        String::from_utf8_lossy(&missing.stderr)
    );

    let (_invalid_directory, invalid_config) = cli_fixture(NODE_COUNT.saturating_add(NODE_COUNT));
    let invalid = run_show(&invalid_config, "json", RUN_ID_VALUE);
    assert!(!invalid.status.success());
    let report: Report = serde_json::from_slice(&invalid.stdout).expect("invalid report JSON");
    assert!(!report.valid);
    assert!(report.diagnostic_codes.contains(&DiagnosticCode::DetailNodeCountMismatch));
    assert_private_content_absent(&String::from_utf8_lossy(&invalid.stdout));
}
