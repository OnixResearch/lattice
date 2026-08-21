//! Positive and negative durable workflow approval tests.

use std::process::Command;

use lattice::engine::executor::RunId;
use lattice::engine::value::Value;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::headless::HeadlessRunStatus;
use lattice::workflow::gate::ActorId;
use lattice::workflow::gate::ApprovalAuthorityPolicy;
use lattice::workflow::gate::ApprovalExpiryPolicy;
use lattice::workflow::gate::ApprovalNodeConfig;
use lattice::workflow::gate::ApprovalReasonPolicy;
use lattice::workflow::gate::ApprovalRequestStatus;
use lattice::workflow::gate::ApprovalResumeSchema;
use lattice::workflow::gate::ApprovalTerminalPolicy;
use lattice::workflow::gate::AuthorityProfile;
use lattice::workflow::gate::ClockSourceClass;
use lattice::workflow::gate::DecisionAbility;
use lattice::workflow::gate::DecisionAuthorityEvidence;
use lattice::workflow::gate::DecisionClass;
use lattice::workflow::gate::DecisionCommand;
use lattice::workflow::gate::DecisionOutcome;
use lattice::workflow::gate::GateId;
use lattice::workflow::gate::REQUIRED_APPROVAL_NON_CLAIMS;
use lattice::workflow::gate::ResumeBindingName;
use lattice::workflow::gate::ResumeCommand;
use lattice::workflow::gate::ResumeOwnerId;
use lattice::workflow::gate::ResumeValue;
use lattice::workflow::gate::ResumeValueField;
use lattice::workflow::gate::TerminalOutcome;

const WORKFLOW_NAME: &str = "durable-approval-test";
const APPROVER: &str = "reviewer-a";
const UNAUTHORIZED_ACTOR: &str = "intruder";
const FIRST_GATE: &str = "gate-one";
const SECOND_GATE: &str = "gate-two";
const RESUME_VALUE_NAME: &str = "continue";
const SECOND_INPUT_NAME: &str = "input";
const STRING_TYPE: &str = "string";
const RESUME_TEXT: &str = "admitted";
const SECRET_TEXT: &str = "not-for-storage";
const DECISION_TTL_MILLIS: u64 = 60_000;
const RESUME_TTL_MILLIS: u64 = 60_000;
const REASON_MAX_BYTES: usize = 128;
const FIRST_RUN_ID: u64 = 41;
const SECOND_RUN_ID: u64 = 43;
const CRASH_RUN_ID: u64 = 47;
const CRASH_POLL_ATTEMPTS: usize = 1_000;
const CRASH_POLL_INTERVAL_MILLIS: u64 = 10;
const CRASH_CHILD_PARK_SECONDS: u64 = 30;
const CRASH_DATABASE_ENV: &str = "LATTICE_APPROVAL_CRASH_DATABASE";
const CRASH_MARKER_ENV: &str = "LATTICE_APPROVAL_CRASH_MARKER";
const FIRST_PORT_INDEX: usize = 0;
const TWO_NODES: usize = 2;
const FIRST_OBSERVED_MILLIS: i64 = 1_000;
const NEXT_OBSERVATION_DELTA_MILLIS: i64 = 1;
const CLI_TIMEOUT_SECS: u64 = 60;
const CLI_MAX_PARALLELISM: usize = 4;
const STORE_FILE_NAME: &str = "lattice.redb";
const DELEGATED_POLICY_REFERENCE: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DELEGATED_RECEIPT_REFERENCE: &str = "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER_DIGEST: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn actor(value: &str) -> ActorId {
    ActorId::try_from_text(value).expect("actor fixture is valid")
}

fn gate_id(value: &str) -> GateId {
    GateId::try_from_text(value).expect("gate fixture is valid")
}

fn binding(value: &str) -> ResumeBindingName {
    ResumeBindingName::try_from_text(value).expect("binding fixture is valid")
}

fn owner(value: &str) -> ResumeOwnerId {
    ResumeOwnerId::try_from_text(value).expect("owner fixture is valid")
}

fn approval_config(
    gate: &str,
    selected_inputs: Vec<String>,
    resume_values: Vec<ResumeValueField>,
) -> ApprovalNodeConfig {
    ApprovalNodeConfig {
        gate_id: gate_id(gate),
        safe_summary: format!("Review {gate}"),
        selected_input_ports: selected_inputs,
        required_artifacts: Vec::new(),
        authority: ApprovalAuthorityPolicy {
            profile: AuthorityProfile::LocalOperator,
            local_actors: vec![actor(APPROVER)],
            accepted_abilities: vec![DecisionAbility::Approve, DecisionAbility::Reject],
            prohibit_self_approval: true,
            delegated_policy_reference: None,
        },
        expiry: ApprovalExpiryPolicy {
            decision_ttl_millis: DECISION_TTL_MILLIS,
            resume_ttl_millis: RESUME_TTL_MILLIS,
            approval_must_be_fresh_at_resume: true,
            clock_source: ClockSourceClass::UtcWallClock,
        },
        reason: ApprovalReasonPolicy {
            required: false,
            identity_bound: true,
            max_bytes: REASON_MAX_BYTES,
        },
        resume: ApprovalResumeSchema {
            values: resume_values,
            credential_bindings: Vec::new(),
        },
        terminal: ApprovalTerminalPolicy {
            on_rejection: TerminalOutcome::FailWorkflow,
            on_expiry: TerminalOutcome::CancelWorkflow,
        },
        required_non_claims: REQUIRED_APPROVAL_NON_CLAIMS.map(str::to_owned).to_vec(),
    }
}

fn one_gate_workflow() -> Workflow {
    let mut workflow = Workflow::new(WORKFLOW_NAME, WorkflowTimestamp::epoch());
    workflow.add_approval_node(
        "approval",
        approval_config(FIRST_GATE, Vec::new(), vec![ResumeValueField {
            name: binding(RESUME_VALUE_NAME),
            type_tag: STRING_TYPE.to_string(),
        }]),
    );
    workflow
}

fn two_gate_workflow() -> Workflow {
    let mut workflow = Workflow::new(WORKFLOW_NAME, WorkflowTimestamp::epoch());
    let first = workflow.add_approval_node(
        "first approval",
        approval_config(FIRST_GATE, Vec::new(), vec![ResumeValueField {
            name: binding(RESUME_VALUE_NAME),
            type_tag: STRING_TYPE.to_string(),
        }]),
    );
    let second = workflow.add_approval_node(
        "second approval",
        approval_config(SECOND_GATE, vec![SECOND_INPUT_NAME.to_string()], Vec::new()),
    );
    let first_output = workflow.graph.node(first).expect("first gate").output_ports[FIRST_PORT_INDEX].id;
    let second_input = workflow.graph.node(second).expect("second gate").input_ports[FIRST_PORT_INDEX].id;
    workflow.graph.add_edge(first_output, second_input).expect("approval gates connect");
    workflow
}

fn approval_command(request: &lattice::workflow::gate::ApprovalRequest, actor_id: &str) -> DecisionCommand {
    DecisionCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        actor: actor(actor_id),
        decision: DecisionClass::Approve,
        reason: Some("reviewed".to_string()),
        observed_epoch_millis: request.requested_epoch_millis + NEXT_OBSERVATION_DELTA_MILLIS,
        evidence: DecisionAuthorityEvidence::LocalOperator,
    }
}

fn resume_command(
    request: &lattice::workflow::gate::ApprovalRequest,
    owner_id: &str,
    values: Vec<ResumeValue>,
) -> ResumeCommand {
    ResumeCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        expected_event_head: request.events.last().expect("approval event head").id.clone(),
        owner: owner(owner_id),
        observed_epoch_millis: request.requested_epoch_millis + NEXT_OBSERVATION_DELTA_MILLIS,
        values,
        credentials: Vec::new(),
    }
}

// r[verify lattice.workflow_execution.ready_frontier.boundary]
// r[verify lattice.workflow_execution.ready_frontier.boundary.scenario.approval]
// r[verify lattice.workflow_approval.suspension]
// r[verify lattice.workflow_approval.suspension.persisted]
// r[verify lattice.workflow_approval.decision]
// r[verify lattice.workflow_approval.decision.approve]
#[tokio::test]
async fn suspension_survives_restart_and_decision_does_not_resume() {
    let temporary = tempfile::tempdir().expect("temporary approval store");
    let path = temporary.path().join("approval.redb");
    let store = lattice::store::Store::new(&path).expect("store opens");
    let workflow = one_gate_workflow();
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    assert_eq!(summary.status, HeadlessRunStatus::Suspended);
    let request = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    assert_eq!(request.status, ApprovalRequestStatus::Suspended);
    assert!(
        store
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_none()
    );
    drop(store);

    let store = lattice::store::Store::new(&path).expect("store reopens");
    let denied = lattice::workflow::gate::shell::decide(&store, &approval_command(&request, UNAUTHORIZED_ACTOR));
    assert!(denied.is_err(), "unadmitted actor must not decide");
    let decision = approval_command(&request, APPROVER);
    let outcome = lattice::workflow::gate::shell::decide(&store, &decision).expect("approval records");
    assert!(matches!(outcome, DecisionOutcome::Recorded { .. }));
    let decided = store
        .load_workflow_approval(&request.id)
        .expect("decided request loads")
        .expect("decided request exists");
    assert_eq!(decided.status, ApprovalRequestStatus::Approved);
    assert!(
        store
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_none()
    );

    let repeated = lattice::workflow::gate::shell::decide(&store, &decision).expect("same decision is idempotent");
    assert!(matches!(repeated, DecisionOutcome::Idempotent { .. }));
    let mut contradictory = decision;
    contradictory.decision = DecisionClass::Reject;
    assert!(
        lattice::workflow::gate::shell::decide(&store, &contradictory).is_err(),
        "contradictory decision must fail"
    );
}

#[test]
#[ignore = "spawned by the process-crash boundary test"]
fn approval_crash_child() {
    let Some(database_path) = std::env::var_os(CRASH_DATABASE_ENV) else {
        return;
    };
    let marker_path = std::env::var_os(CRASH_MARKER_ENV).expect("crash child marker path is set");
    let store = lattice::store::Store::new(database_path).expect("crash child store opens");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("crash child runtime builds");
    let summary = runtime
        .block_on(lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(CRASH_RUN_ID)))
        .expect("crash child persists suspension");
    assert_eq!(summary.status, HeadlessRunStatus::Suspended);
    std::fs::write(marker_path, b"approval suspension committed").expect("crash child marker writes");
    std::thread::sleep(std::time::Duration::from_secs(CRASH_CHILD_PARK_SECONDS));
}

// r[verify lattice.workflow_approval.suspension.persisted]
// r[verify lattice.workflow_approval.suspension.uncertain]
// r[verify lattice.workflow_approval.resume.valid]
#[tokio::test]
async fn process_kill_after_atomic_suspension_commit_recovers_exact_state() {
    let temporary = tempfile::tempdir().expect("temporary process-crash store");
    let database_path = temporary.path().join("approval-crash.redb");
    let marker_path = temporary.path().join("approval-committed.marker");
    let store = lattice::store::Store::new(&database_path).expect("process-crash store opens");
    let workflow = one_gate_workflow();
    store.save_workflow(&workflow).expect("process-crash workflow saves");

    let node_id = workflow.graph.nodes().next().expect("approval node exists").id;
    let uncommitted = lattice::workflow::gate::core::build_request(lattice::workflow::gate::core::RequestBuildInput {
        run_id: RunId(CRASH_RUN_ID),
        workflow: &workflow,
        node_id,
        requester: actor("crash-requester"),
        observed_epoch_millis: FIRST_OBSERVED_MILLIS,
        gate_inputs: &std::collections::HashMap::new(),
        completed_nodes: Vec::new(),
        graph_events: Vec::new(),
    })
    .expect("uncommitted request builds");
    let mut uncommitted = uncommitted;
    lattice::workflow::gate::choregraph::apply_suspend(&workflow, &mut uncommitted)
        .expect("uncommitted graph event applies");
    let uncommitted =
        lattice::workflow::gate::core::record_suspension(&uncommitted).expect("uncommitted event chain seals");
    drop(store);
    let store = lattice::store::Store::new(&database_path).expect("store reopens before commit");
    assert!(
        store.load_workflow_approval(&uncommitted.id).is_err(),
        "an absent approval table must report an explicit incomplete boundary"
    );
    drop(store);

    let executable = std::env::current_exe().expect("current test executable resolves");
    let mut child = Command::new(executable)
        .args(["--ignored", "--exact", "approval_crash_child", "--nocapture"])
        .env(CRASH_DATABASE_ENV, &database_path)
        .env(CRASH_MARKER_ENV, &marker_path)
        .spawn()
        .expect("approval crash child starts");
    for _attempt in 0..CRASH_POLL_ATTEMPTS {
        if marker_path.exists() {
            break;
        }
        if let Some(status) = child.try_wait().expect("crash child status reads") {
            panic!("approval crash child exited before the commit marker: {status}");
        }
        std::thread::sleep(std::time::Duration::from_millis(CRASH_POLL_INTERVAL_MILLIS));
    }
    assert!(marker_path.exists(), "approval suspension commit marker must become visible");
    child.kill().expect("approval crash child terminates");
    let _status = child.wait().expect("approval crash child is reaped");

    let store = lattice::store::Store::new(&database_path).expect("store reopens after process kill");
    let request = store
        .load_workflow_approval_for_run(RunId(CRASH_RUN_ID))
        .expect("committed request lookup succeeds")
        .expect("committed request survives process kill");
    assert_eq!(request.status, ApprovalRequestStatus::Suspended);
    assert_eq!(
        request.events.last().expect("suspension event exists").kind,
        lattice::workflow::gate::ApprovalEventKind::Suspended
    );
    lattice::workflow::gate::shell::decide(&store, &approval_command(&request, APPROVER))
        .expect("fresh process records approval");
    let approved = store
        .load_workflow_approval(&request.id)
        .expect("approved request reloads")
        .expect("approved request exists");
    let resumed = lattice::workflow::gate::shell::resume_with_authority_provider(
        &store,
        &resume_command(&approved, "post-crash-resume", vec![ResumeValue {
            name: binding(RESUME_VALUE_NAME),
            value: Value::from_string(RESUME_TEXT.to_string()),
        }]),
        lattice::agent_node::AgentRuntimeConfig::default(),
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
    .expect("fresh process resumes committed suspension");
    assert_eq!(resumed.summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(resumed.summary.completed, 1);
}

// r[verify lattice.workflow_approval.resume]
// r[verify lattice.workflow_approval.resume.valid]
// r[verify lattice.workflow_approval.terminal.duplicate]
#[tokio::test]
async fn resume_rejects_plaintext_secret_and_executes_each_gate_once() {
    let temporary = tempfile::tempdir().expect("temporary approval store");
    let store = lattice::store::Store::new(temporary.path().join("approval.redb")).expect("store opens");
    let workflow = two_gate_workflow();
    store.save_workflow(&workflow).expect("workflow saves");
    let initial = lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(SECOND_RUN_ID))
        .await
        .expect("first gate suspends");
    assert_eq!(initial.status, HeadlessRunStatus::Suspended);
    let first = store
        .load_workflow_approval_for_run(RunId(SECOND_RUN_ID))
        .expect("first request loads")
        .expect("first request exists");
    lattice::workflow::gate::shell::decide(&store, &approval_command(&first, APPROVER))
        .expect("first approval records");
    let approved_first = store
        .load_workflow_approval(&first.id)
        .expect("approved request loads")
        .expect("approved request exists");

    let secret_command = resume_command(&approved_first, "resume-secret-attempt", vec![ResumeValue {
        name: binding(RESUME_VALUE_NAME),
        value: Value::from_sensitive_string(SECRET_TEXT.to_string()),
    }]);
    assert!(
        lattice::workflow::gate::shell::resume_with_authority_provider(
            &store,
            &secret_command,
            lattice::agent_node::AgentRuntimeConfig::default(),
            &lattice::engine::effect::execution::UnavailableRunContextSource,
        )
        .await
        .is_err(),
        "plaintext secret resume value must fail before a claim"
    );

    let valid_command = resume_command(&approved_first, "resume-first", vec![ResumeValue {
        name: binding(RESUME_VALUE_NAME),
        value: Value::from_string(RESUME_TEXT.to_string()),
    }]);
    let first_resume = lattice::workflow::gate::shell::resume_with_authority_provider(
        &store,
        &valid_command,
        lattice::agent_node::AgentRuntimeConfig::default(),
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
    .expect("first gate resumes to second gate");
    assert_eq!(first_resume.summary.status, HeadlessRunStatus::Suspended);
    let second_id = first_resume.next_request.expect("second approval request");
    let completed_first = store
        .load_workflow_approval(&first.id)
        .expect("first request reloads")
        .expect("first request remains")
        .status;
    assert_eq!(completed_first, ApprovalRequestStatus::Resumed);
    let second = store
        .load_workflow_approval(&second_id)
        .expect("second request loads")
        .expect("second request exists");
    assert_eq!(second.completed_nodes.len(), 1);
    assert_eq!(second.status, ApprovalRequestStatus::Suspended);

    lattice::workflow::gate::shell::decide(&store, &approval_command(&second, APPROVER))
        .expect("second approval records");
    let approved_second = store
        .load_workflow_approval(&second.id)
        .expect("approved second request loads")
        .expect("approved second request exists");
    let second_resume = lattice::workflow::gate::shell::resume_with_authority_provider(
        &store,
        &resume_command(&approved_second, "resume-second", Vec::new()),
        lattice::agent_node::AgentRuntimeConfig::default(),
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
    .expect("second gate resumes");
    assert_eq!(second_resume.summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(second_resume.summary.completed, TWO_NODES);
    assert!(second_resume.next_request.is_none());
    let details = store.load_run_details(RunId(SECOND_RUN_ID)).expect("run details load").expect("run details exist");
    assert_eq!(details.nodes.len(), TWO_NODES, "completed gates must not rerun or duplicate");
}

#[tokio::test]
async fn tui_exposes_safe_inspection_and_separate_decide_and_resume_actions() {
    let temporary = tempfile::tempdir().expect("temporary TUI approval directory");
    let data_dir = temporary.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("TUI data directory creates");
    let store = lattice::store::Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    store.save_workflow(&one_gate_workflow()).expect("workflow saves");
    lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    let request = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    drop(store);

    let mut app = lattice::tui::App::new(lattice::config::AppConfig {
        data_dir,
        ..lattice::config::AppConfig::default()
    });
    app.open_approvals();
    assert!(app.is_approval_list_open());
    assert_eq!(app.approvals.records.len(), 1);
    app.inspect_selected_approval();
    let detail = app.approvals.detail.as_deref().expect("safe approval detail");
    assert!(detail.contains(request.id.as_str()));
    assert!(!detail.contains(SECRET_TEXT));

    assert!(app.decide_selected_approval(&approval_command(&request, APPROVER)));
    let approved = app
        .store
        .as_ref()
        .expect("TUI store")
        .load_workflow_approval(&request.id)
        .expect("approved request loads")
        .expect("approved request exists");
    assert_eq!(approved.status, ApprovalRequestStatus::Approved);
    assert!(
        app.store
            .as_ref()
            .expect("TUI store")
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_none(),
        "decision action must not resume the run"
    );
    let resume = resume_command(&approved, "tui-resume", vec![ResumeValue {
        name: binding(RESUME_VALUE_NAME),
        value: Value::from_string(RESUME_TEXT.to_string()),
    }]);
    assert!(app.resume_selected_approval(&resume).await);
    assert!(
        app.store
            .as_ref()
            .expect("TUI store")
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_some(),
        "separate resume action must finish the gate-only workflow"
    );
    let repeated = lattice::workflow::gate::shell::resume_with_authority_provider(
        app.store.as_ref().expect("TUI store"),
        &resume,
        lattice::agent_node::AgentRuntimeConfig::default(),
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
    .expect("same resume is idempotent");
    assert!(matches!(repeated.outcome, lattice::workflow::gate::ResumeOutcome::Idempotent { .. }));
    let conflicting = resume_command(&approved, "other-resume-owner", vec![ResumeValue {
        name: binding(RESUME_VALUE_NAME),
        value: Value::from_string(RESUME_TEXT.to_string()),
    }]);
    assert!(
        lattice::workflow::gate::shell::resume_with_authority_provider(
            app.store.as_ref().expect("TUI store"),
            &conflicting,
            lattice::agent_node::AgentRuntimeConfig::default(),
            &lattice::engine::effect::execution::UnavailableRunContextSource,
        )
        .await
        .is_err(),
        "changed completed resume facts must fail closed"
    );
}

#[tokio::test]
async fn cli_lists_decides_and_resumes_without_emitting_checkpoint_values() {
    let temporary = tempfile::tempdir().expect("temporary CLI directory");
    let data_dir = temporary.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory creates");
    let config_path = temporary.path().join("config.ncl");
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {CLI_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {CLI_MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string(),
        ),
    )
    .expect("config writes");
    let store = lattice::store::Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    store.save_workflow(&one_gate_workflow()).expect("workflow saves");
    lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    let request = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    drop(store);

    let list = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .args(["approvals", "list", "--output", "json"])
        .output()
        .expect("approval list runs");
    assert!(list.status.success());
    let rendered = String::from_utf8(list.stdout).expect("approval list is UTF-8");
    assert!(rendered.contains(request.id.as_str()));
    assert!(!rendered.contains(SECRET_TEXT), "safe approval list must omit checkpoint values");

    let decision_observed_epoch_millis = (request.requested_epoch_millis + NEXT_OBSERVATION_DELTA_MILLIS).to_string();
    let decision = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .args([
            "approvals",
            "decide",
            request.id.as_str(),
            "--workflow-revision",
            request.workflow_revision.as_str(),
            "--actor",
            APPROVER,
            "--decision",
            "approve",
            "--reason",
            "reviewed",
            "--observed-epoch-ms",
            &decision_observed_epoch_millis,
        ])
        .output()
        .expect("approval decision runs");
    assert!(decision.status.success(), "decision stderr: {}", String::from_utf8_lossy(&decision.stderr));

    let receipt = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .args(["approvals", "receipt", request.id.as_str(), "--output", "json"])
        .output()
        .expect("approval receipt runs");
    assert!(receipt.status.success());
    let receipt_text = String::from_utf8(receipt.stdout).expect("approval receipt is UTF-8");
    assert!(!receipt_text.contains("reviewed"), "receipt must omit free-form reasons");
    assert!(!receipt_text.contains(SECRET_TEXT), "receipt must omit workflow values");

    let store = lattice::store::Store::new(data_dir.join(STORE_FILE_NAME)).expect("store reopens");
    let approved = store
        .load_workflow_approval(&request.id)
        .expect("approved request loads")
        .expect("approved request exists");
    drop(store);
    let event_head = approved.events.last().expect("approved event head").id.to_string();
    let resume_observed_epoch_millis = approved
        .decision
        .as_ref()
        .expect("approval decision")
        .decided_epoch_millis
        .saturating_add(NEXT_OBSERVATION_DELTA_MILLIS)
        .to_string();
    let resume = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .args([
            "workflow",
            "resume",
            &FIRST_RUN_ID.to_string(),
            "--request",
            approved.id.as_str(),
            "--workflow-revision",
            approved.workflow_revision.as_str(),
            "--event-head",
            &event_head,
            "--owner",
            "cli-resume",
            "--value",
            "continue=string:admitted",
            "--observed-epoch-ms",
            &resume_observed_epoch_millis,
            "--output",
            "json",
        ])
        .output()
        .expect("workflow resume runs");
    assert!(resume.status.success(), "resume stderr: {}", String::from_utf8_lossy(&resume.stderr));
    let execution: serde_json::Value = serde_json::from_slice(&resume.stdout).expect("resume JSON parses");
    assert_eq!(execution["summary"]["status"], "succeeded");
}

// r[verify lattice.workflow_approval.suspension.uncertain]
// r[verify lattice.workflow_approval.resume.denied]
#[tokio::test]
async fn restart_after_resume_claim_fails_closed_without_replaying_work() {
    let temporary = tempfile::tempdir().expect("temporary crash-boundary store");
    let path = temporary.path().join("approval.redb");
    let store = lattice::store::Store::new(&path).expect("store opens");
    let workflow = one_gate_workflow();
    store.save_workflow(&workflow).expect("workflow saves");
    lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    let suspended = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    lattice::workflow::gate::shell::decide(&store, &approval_command(&suspended, APPROVER)).expect("approval records");
    let approved = store
        .load_workflow_approval(&suspended.id)
        .expect("approved request loads")
        .expect("approved request exists");
    let command = resume_command(&approved, "crashed-resume-owner", vec![ResumeValue {
        name: binding(RESUME_VALUE_NAME),
        value: Value::from_string(RESUME_TEXT.to_string()),
    }]);
    let config = match workflow.node_config(approved.node_id).expect("approval config") {
        lattice::engine::workflow::NodeConfig::Approval(config) => config,
        _ => panic!("expected approval config"),
    };
    let claim =
        lattice::workflow::gate::core::plan_resume_claim(&approved, config, &command).expect("resume claim plans");
    store
        .transition_workflow_approval(approved.version, &claim.request)
        .expect("resume claim commits before simulated crash");
    drop(store);

    let store = lattice::store::Store::new(&path).expect("store reopens after claim");
    let retry = lattice::workflow::gate::shell::resume_with_authority_provider(
        &store,
        &command,
        lattice::agent_node::AgentRuntimeConfig::default(),
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await;
    assert!(retry.is_err(), "active claim must require explicit recovery");
    let persisted = store
        .load_workflow_approval(&approved.id)
        .expect("claimed request loads")
        .expect("claimed request exists");
    assert_eq!(persisted.status, ApprovalRequestStatus::Resuming);
    assert!(
        store
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_none()
    );
}

#[test]
fn approval_integrity_rejects_changed_event_and_workflow_revision() {
    let workflow = one_gate_workflow();
    let node_id = workflow.graph.nodes().next().expect("approval node").id;
    let mut request = lattice::workflow::gate::core::build_request(lattice::workflow::gate::core::RequestBuildInput {
        run_id: RunId(FIRST_RUN_ID),
        workflow: &workflow,
        node_id,
        requester: actor("requester"),
        observed_epoch_millis: FIRST_OBSERVED_MILLIS,
        gate_inputs: &std::collections::HashMap::new(),
        completed_nodes: Vec::new(),
        graph_events: Vec::new(),
    })
    .expect("request builds");
    lattice::workflow::gate::choregraph::apply_suspend(&workflow, &mut request).expect("gate suspends");
    let request = lattice::workflow::gate::core::record_suspension(&request).expect("suspension seals");
    let config = match workflow.node_config(node_id).expect("approval config") {
        lattice::engine::workflow::NodeConfig::Approval(config) => config,
        _ => panic!("expected approval config"),
    };
    lattice::workflow::gate::core::validate_request_integrity(&request, &workflow, config)
        .expect("unchanged request is valid");
    let mut changed_event = request.clone();
    changed_event.events[0].id = lattice::workflow::gate::ApprovalEventId::try_from_hex(request.id.as_str())
        .expect("cross-role digest spelling parses at the boundary");
    assert!(
        lattice::workflow::gate::core::validate_request_integrity(&changed_event, &workflow, config).is_err(),
        "changed event identity must fail"
    );
    let mut changed_workflow = workflow.clone();
    changed_workflow.meta.description = "changed after review".to_string();
    assert!(
        lattice::workflow::gate::core::validate_request_integrity(&request, &changed_workflow, config).is_err(),
        "changed workflow revision must fail"
    );
}

// r[verify lattice.workflow_approval.node]
// r[verify lattice.workflow_approval.node.valid]
// r[verify lattice.workflow_approval.node.invalid]
#[test]
fn invalid_approval_configuration_and_duplicate_gate_ids_are_rejected() {
    let mut invalid = approval_config(FIRST_GATE, Vec::new(), Vec::new());
    invalid.required_non_claims.clear();
    invalid.resume.values = vec![
        ResumeValueField {
            name: binding(RESUME_VALUE_NAME),
            type_tag: STRING_TYPE.to_string(),
        },
        ResumeValueField {
            name: binding(RESUME_VALUE_NAME),
            type_tag: STRING_TYPE.to_string(),
        },
    ];
    let issues = lattice::workflow::gate::core::validate_node_config(&invalid);
    assert!(!issues.is_empty());

    let mut workflow = Workflow::new(WORKFLOW_NAME, WorkflowTimestamp::epoch());
    workflow.add_approval_node("first duplicate", approval_config(FIRST_GATE, Vec::new(), Vec::new()));
    workflow.add_approval_node("second duplicate", approval_config(FIRST_GATE, Vec::new(), Vec::new()));
    let diagnostics = lattice::workflow_validation::validate_workflow(&workflow);
    assert!(
        lattice::workflow_validation::has_fatal_diagnostics(&diagnostics),
        "duplicate gate identity must fail workflow validation"
    );

    let mut portable = serde_json::to_value(lattice::workflow_format::workflow_to_portable(&one_gate_workflow()))
        .expect("portable approval serializes");
    portable["configs"][0]["approval"][0]["unknown_authority"] = serde_json::Value::Bool(true);
    assert!(
        serde_json::from_value::<lattice::workflow_format::PortableWorkflow>(portable).is_err(),
        "unknown approval fields must fail closed"
    );
}

// r[verify lattice.workflow_approval.authority]
// r[verify lattice.workflow_approval.authority.valid]
// r[verify lattice.workflow_approval.authority.denied]
#[test]
fn delegated_authority_requires_exact_current_verified_facts() {
    let mut workflow = one_gate_workflow();
    let node_id = workflow.graph.nodes().next().expect("approval node").id;
    let config = match workflow.configs.get_mut(&node_id.0).expect("approval config") {
        lattice::engine::workflow::NodeConfig::Approval(config) => config,
        _ => panic!("expected approval config"),
    };
    config.authority.profile = AuthorityProfile::BasaltUcanVerified;
    config.authority.local_actors.clear();
    config.authority.delegated_policy_reference = Some(DELEGATED_POLICY_REFERENCE.to_string());
    let mut request = lattice::workflow::gate::core::build_request(lattice::workflow::gate::core::RequestBuildInput {
        run_id: RunId(FIRST_RUN_ID),
        workflow: &workflow,
        node_id,
        requester: actor("requester"),
        observed_epoch_millis: FIRST_OBSERVED_MILLIS,
        gate_inputs: &std::collections::HashMap::new(),
        completed_nodes: Vec::new(),
        graph_events: Vec::new(),
    })
    .expect("delegated request builds");
    lattice::workflow::gate::choregraph::apply_suspend(&workflow, &mut request).expect("delegated gate suspends");
    let request = lattice::workflow::gate::core::record_suspension(&request).expect("delegated suspension seals");
    let reviewer = actor(APPROVER);
    let observed_epoch_millis = request.requested_epoch_millis + NEXT_OBSERVATION_DELTA_MILLIS;
    let receipts = vec![DELEGATED_RECEIPT_REFERENCE.to_string()];
    let input_id = lattice::workflow::gate::core::delegated_authority_input_id(
        &request,
        &reviewer,
        DecisionAbility::Approve,
        observed_epoch_millis,
        &receipts,
    )
    .expect("delegated authority input identity builds");
    let facts = lattice::workflow::gate::VerifiedDelegatedAuthorityFacts {
        request_id: request.id.clone(),
        workflow_revision: request.workflow_revision.clone(),
        node_id,
        actor: reviewer.clone(),
        ability: DecisionAbility::Approve,
        policy_reference: DELEGATED_POLICY_REFERENCE.to_string(),
        verification_receipt_references: receipts,
        caveats_allow: true,
        replay_allow: true,
        current: true,
        input_id,
    };
    let command = DecisionCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        actor: reviewer,
        decision: DecisionClass::Approve,
        reason: None,
        observed_epoch_millis,
        evidence: DecisionAuthorityEvidence::BasaltUcanVerified(facts.clone()),
    };
    let config = match workflow.node_config(node_id).expect("approval config") {
        lattice::engine::workflow::NodeConfig::Approval(config) => config,
        _ => panic!("expected approval config"),
    };
    lattice::workflow::gate::core::plan_decision(&request, config, &command)
        .expect("exact delegated facts authorize the decision");

    let mut wrong_request = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_request.evidence {
        facts.request_id = lattice::workflow::gate::ApprovalRequestId::try_from_hex(OTHER_DIGEST)
            .expect("alternate request identity parses");
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_request).is_err());

    let mut wrong_revision = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_revision.evidence {
        facts.workflow_revision = lattice::orchestration::WorkflowRevision::try_from_hex(OTHER_DIGEST)
            .expect("alternate workflow revision parses");
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_revision).is_err());

    let mut wrong_actor = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_actor.evidence {
        facts.actor = actor("other-reviewer");
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_actor).is_err());

    let mut wrong_ability = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_ability.evidence {
        facts.ability = DecisionAbility::Reject;
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_ability).is_err());

    let mut wrong_policy = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_policy.evidence {
        facts.policy_reference = DELEGATED_RECEIPT_REFERENCE.to_string();
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_policy).is_err());

    let mut wrong_receipt = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_receipt.evidence {
        facts.verification_receipt_references = vec![DELEGATED_POLICY_REFERENCE.to_string()];
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_receipt).is_err());

    let actor_text_only = DecisionCommand {
        evidence: DecisionAuthorityEvidence::LocalOperator,
        ..command.clone()
    };
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &actor_text_only).is_err());

    let mut raw_token = serde_json::to_value(facts).expect("delegated authority facts serialize");
    raw_token
        .as_object_mut()
        .expect("delegated facts are a JSON object")
        .insert("raw_token".to_string(), serde_json::Value::String("unverified-token".to_string()));
    assert!(
        serde_json::from_value::<lattice::workflow::gate::VerifiedDelegatedAuthorityFacts>(raw_token).is_err(),
        "raw tokens must not enter the checked delegated-fact adapter"
    );

    let mut stale = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut stale.evidence {
        facts.current = false;
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &stale).is_err());
    let mut caveat_denied = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut caveat_denied.evidence {
        facts.caveats_allow = false;
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &caveat_denied).is_err());
    let mut replay_denied = command.clone();
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut replay_denied.evidence {
        facts.replay_allow = false;
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &replay_denied).is_err());
    let mut wrong_input = command;
    if let DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &mut wrong_input.evidence {
        facts.input_id = lattice::workflow::gate::ApprovalAuthorityInputId::try_from_hex(request.id.as_str())
            .expect("digest spelling parses for negative fixture");
    }
    assert!(lattice::workflow::gate::core::plan_decision(&request, config, &wrong_input).is_err());
}

#[test]
fn approval_node_round_trips_through_portable_and_nickel_formats() {
    let workflow = two_gate_workflow();
    let portable = lattice::workflow_format::workflow_to_portable(&workflow);
    let restored =
        lattice::workflow_format::portable_to_workflow(portable).expect("portable approval workflow imports");
    assert_eq!(
        lattice::orchestration::workflow_revision(&workflow).expect("original revision").content_hash,
        lattice::orchestration::workflow_revision(&restored).expect("restored revision").content_hash,
    );
    let nickel = lattice::workflow_format::workflow_to_nickel(&workflow);
    assert!(nickel.contains("approval"));
    assert!(nickel.contains(FIRST_GATE));
    assert!(nickel.contains(SECOND_GATE));
    let temporary = tempfile::tempdir().expect("temporary Nickel workflow directory");
    let path = temporary.path().join("approval.ncl");
    std::fs::write(&path, nickel).expect("approval Nickel writes");
    let loaded =
        lattice::workflow_format::load_workflow_nickel_file(&path).expect("approval Nickel evaluates and imports");
    assert_eq!(
        lattice::orchestration::workflow_revision(&workflow).expect("original revision").content_hash,
        lattice::orchestration::workflow_revision(&loaded).expect("Nickel revision").content_hash,
    );
}

// r[verify lattice.workflow_approval.resume.inputs]
#[tokio::test]
async fn resume_persists_only_declared_opaque_credential_references() {
    let temporary = tempfile::tempdir().expect("temporary credential approval store");
    let store = lattice::store::Store::new(temporary.path().join("approval.redb")).expect("store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME, WorkflowTimestamp::epoch());
    let mut config = approval_config(FIRST_GATE, Vec::new(), Vec::new());
    config.resume.credential_bindings = vec![binding("token")];
    workflow.add_approval_node("credential approval", config);
    store.save_workflow(&workflow).expect("workflow saves");
    lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    let request = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    lattice::workflow::gate::shell::decide(&store, &approval_command(&request, APPROVER)).expect("approval records");
    let approved = store
        .load_workflow_approval(&request.id)
        .expect("approved request loads")
        .expect("approved request exists");
    let mut invalid = resume_command(&approved, "credential-resume-invalid", Vec::new());
    invalid.credentials.push(lattice::workflow::gate::ResumeCredentialBinding {
        name: binding("token"),
        reference: lattice::credentials::SecretRef::new("", "TOKEN", None),
    });
    assert!(
        lattice::workflow::gate::shell::resume_with_authority_provider(
            &store,
            &invalid,
            lattice::agent_node::AgentRuntimeConfig::default(),
            &lattice::engine::effect::execution::UnavailableRunContextSource,
        )
        .await
        .is_err(),
        "malformed credential reference must fail before resume claim"
    );
    let mut valid = resume_command(&approved, "credential-resume", Vec::new());
    valid.credentials.push(lattice::workflow::gate::ResumeCredentialBinding {
        name: binding("token"),
        reference: lattice::credentials::SecretRef::new("env", "TOKEN", None),
    });
    lattice::workflow::gate::shell::resume_with_authority_provider(
        &store,
        &valid,
        lattice::agent_node::AgentRuntimeConfig::default(),
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
    .expect("opaque credential reference resumes");
    let completed = store
        .load_workflow_approval(&approved.id)
        .expect("completed request loads")
        .expect("completed request exists");
    let persisted = serde_json::to_string(&completed).expect("completed request serializes");
    assert!(persisted.contains("env:TOKEN"));
    assert!(!persisted.contains("resolved-secret-value"));
}

// r[verify lattice.workflow_approval.decision.reject]
// r[verify lattice.workflow_approval.terminal]
#[tokio::test]
async fn rejection_is_terminal_and_self_approval_fails_closed() {
    let temporary = tempfile::tempdir().expect("temporary rejection store");
    let store = lattice::store::Store::new(temporary.path().join("approval.redb")).expect("store opens");
    let mut workflow = one_gate_workflow();
    let node_id = workflow.graph.nodes().next().expect("approval node").id;
    let config = match workflow.configs.get_mut(&node_id.0).expect("approval config") {
        lattice::engine::workflow::NodeConfig::Approval(config) => config,
        _ => panic!("expected approval config"),
    };
    config.authority.local_actors.push(actor("lattice-runtime"));
    store.save_workflow(&workflow).expect("workflow saves");
    lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    let request = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    assert!(
        lattice::workflow::gate::shell::decide(&store, &approval_command(&request, "lattice-runtime")).is_err(),
        "self approval must fail even for an allowed local actor"
    );
    let mut rejection = approval_command(&request, APPROVER);
    rejection.decision = DecisionClass::Reject;
    lattice::workflow::gate::shell::decide(&store, &rejection).expect("rejection records");
    let rejected = store
        .load_workflow_approval(&request.id)
        .expect("rejected request loads")
        .expect("rejected request exists");
    assert_eq!(rejected.status, ApprovalRequestStatus::Rejected);
    assert!(
        store
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_some()
    );
    assert!(
        lattice::workflow::gate::shell::resume_with_authority_provider(
            &store,
            &resume_command(&rejected, "rejected-resume", vec![ResumeValue {
                name: binding(RESUME_VALUE_NAME),
                value: Value::from_string(RESUME_TEXT.to_string()),
            }],),
            lattice::agent_node::AgentRuntimeConfig::default(),
            &lattice::engine::effect::execution::UnavailableRunContextSource,
        )
        .await
        .is_err(),
        "rejected approval must never resume"
    );
}

// r[verify lattice.workflow_approval.terminal.expired]
#[tokio::test]
async fn expired_resume_becomes_terminal_without_running_the_gate() {
    let temporary = tempfile::tempdir().expect("temporary expiry store");
    let store = lattice::store::Store::new(temporary.path().join("approval.redb")).expect("store opens");
    store.save_workflow(&one_gate_workflow()).expect("workflow saves");
    lattice::headless::run_saved_workflow(&store, WORKFLOW_NAME, RunId(FIRST_RUN_ID))
        .await
        .expect("workflow suspends");
    let request = store
        .load_workflow_approval_for_run(RunId(FIRST_RUN_ID))
        .expect("request loads")
        .expect("request exists");
    lattice::workflow::gate::shell::decide(&store, &approval_command(&request, APPROVER)).expect("approval records");
    let approved = store
        .load_workflow_approval(&request.id)
        .expect("approved request loads")
        .expect("approved request exists");
    let mut command = resume_command(&approved, "expired-resume", vec![ResumeValue {
        name: binding(RESUME_VALUE_NAME),
        value: Value::from_string(RESUME_TEXT.to_string()),
    }]);
    command.observed_epoch_millis = approved.decision.as_ref().expect("approval decision").decided_epoch_millis
        + i64::try_from(RESUME_TTL_MILLIS).expect("resume TTL fits")
        + NEXT_OBSERVATION_DELTA_MILLIS;
    assert!(
        lattice::workflow::gate::shell::resume_with_authority_provider(
            &store,
            &command,
            lattice::agent_node::AgentRuntimeConfig::default(),
            &lattice::engine::effect::execution::UnavailableRunContextSource,
        )
        .await
        .is_err()
    );
    let expired = store
        .load_workflow_approval(&approved.id)
        .expect("expired request loads")
        .expect("expired request exists");
    assert_eq!(expired.status, ApprovalRequestStatus::Expired);
    assert!(
        store
            .load_run(RunId(FIRST_RUN_ID))
            .expect("run loads")
            .expect("run exists")
            .finished_epoch_ms
            .is_some()
    );
}

#[test]
fn expired_decision_is_terminal_without_resume_authority() {
    let workflow = one_gate_workflow();
    let node_id = workflow.graph.nodes().next().expect("approval node").id;
    let mut request = lattice::workflow::gate::core::build_request(lattice::workflow::gate::core::RequestBuildInput {
        run_id: RunId(FIRST_RUN_ID),
        workflow: &workflow,
        node_id,
        requester: actor("requester"),
        observed_epoch_millis: FIRST_OBSERVED_MILLIS,
        gate_inputs: &std::collections::HashMap::new(),
        completed_nodes: Vec::new(),
        graph_events: Vec::new(),
    })
    .expect("request builds");
    lattice::workflow::gate::choregraph::apply_suspend(&workflow, &mut request).expect("gate suspends");
    let request = lattice::workflow::gate::core::record_suspension(&request).expect("suspension seals");
    let config = match workflow.node_config(node_id).expect("approval config") {
        lattice::engine::workflow::NodeConfig::Approval(config) => config,
        _ => panic!("expected approval config"),
    };
    let command = DecisionCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        actor: actor(APPROVER),
        decision: DecisionClass::Approve,
        reason: None,
        observed_epoch_millis: request.decision_deadline_epoch_millis + NEXT_OBSERVATION_DELTA_MILLIS,
        evidence: DecisionAuthorityEvidence::LocalOperator,
    };
    let plan = lattice::workflow::gate::core::plan_decision(&request, config, &command)
        .expect("expiry becomes a terminal plan");
    assert!(matches!(plan.outcome, DecisionOutcome::Expired { .. }));
    assert_eq!(plan.request.status, ApprovalRequestStatus::Expired);
}
