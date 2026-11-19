//! Typed policy decisions across durable approval suspension and resume (Lattice #23).
//!
//! A decision made before an approval gate persists on the suspended run and is carried through
//! the resume with no adapter work; a decision made after the resume persists on the finished
//! run; a changed profile or artifact refuses the resume and leaves the stored record unchanged.

mod common;
#[path = "support/typed_policy.rs"]
mod fixture;

use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use lattice::engine::executor::RunId;
use lattice::engine::value::Value;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::Workflow;
use lattice::headless::HeadlessRunStatus;
use lattice::store::RunDetailRecord;
use lattice::store::RunNodeDetailStatus;
use lattice::workflow::decision::Adapter;
use lattice::workflow::decision::Response;
use lattice::workflow::decision::binding::Invocation;
use lattice::workflow::decision::ledger::Record;
use lattice::workflow::decision::ledger::ReplayRefusal;
use lattice::workflow::decision::ledger::RunOutcome;
use lattice::workflow::gate::ActorId;
use lattice::workflow::gate::ApprovalAuthorityPolicy;
use lattice::workflow::gate::ApprovalExpiryPolicy;
use lattice::workflow::gate::ApprovalNodeConfig;
use lattice::workflow::gate::ApprovalReasonPolicy;
use lattice::workflow::gate::ApprovalRequest;
use lattice::workflow::gate::ApprovalRequestStatus;
use lattice::workflow::gate::ApprovalResumeSchema;
use lattice::workflow::gate::ApprovalTerminalPolicy;
use lattice::workflow::gate::AuthorityProfile;
use lattice::workflow::gate::ClockSourceClass;
use lattice::workflow::gate::DecisionAbility;
use lattice::workflow::gate::DecisionAuthorityEvidence;
use lattice::workflow::gate::DecisionClass;
use lattice::workflow::gate::DecisionCommand;
use lattice::workflow::gate::GateId;
use lattice::workflow::gate::REQUIRED_APPROVAL_NON_CLAIMS;
use lattice::workflow::gate::ResumeBindingName;
use lattice::workflow::gate::ResumeCommand;
use lattice::workflow::gate::ResumeOwnerId;
use lattice::workflow::gate::ResumeValue;
use lattice::workflow::gate::ResumeValueField;
use lattice::workflow::gate::TerminalOutcome;
use lattice::workflow::gate::shell::Error;
use pretty_assertions::assert_eq;
use serde_json::json;

const APPROVER: &str = "reviewer-a";
const GATE_ID: &str = "release-gate";
const GATE_LABEL: &str = "release approval";
const GATE_INPUT: &str = "input";
const RESUME_VALUE_NAME: &str = "continue";
const RESUME_TEXT: &str = "admitted";
const STRING_TYPE: &str = "string";
const STDIN: &str = "stdin";
const STDOUT: &str = "stdout";
const PAGE_CASE: &str = "auth-r2-01";
const PAGE_LABEL: &str = "page";
const PAGE_CONTINUATION: &str = "page on-call";
const TTL_MILLIS: u64 = 60_000;
const REASON_MAX_BYTES: usize = 128;
const OBSERVATION_DELTA_MILLIS: i64 = 1;
const OTHER_ARTIFACT: &str = "b3:0000000000000000000000000000000000000000000000000000000000000000";

/// Counts invocations and delegates to the fixture leaf adapter.
struct Counting {
    calls: AtomicU64,
    inner: Arc<dyn Adapter>,
}

impl Counting {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicU64::new(0),
            inner: fixture::leaf_adapter(),
        })
    }

    fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Adapter for Counting {
    fn invoke(&self, invocation: &Invocation) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.invoke(invocation)
    }
}

fn gate_config(selected_inputs: Vec<String>, resume_values: Vec<ResumeValueField>) -> ApprovalNodeConfig {
    ApprovalNodeConfig {
        gate_id: GateId::try_from_text(GATE_ID).expect("gate fixture is valid"),
        safe_summary: format!("Review {GATE_ID}"),
        selected_input_ports: selected_inputs,
        required_artifacts: Vec::new(),
        authority: ApprovalAuthorityPolicy {
            profile: AuthorityProfile::LocalOperator,
            local_actors: vec![ActorId::try_from_text(APPROVER).expect("actor fixture is valid")],
            accepted_abilities: vec![DecisionAbility::Approve, DecisionAbility::Reject],
            prohibit_self_approval: true,
            delegated_policy_reference: None,
        },
        expiry: ApprovalExpiryPolicy {
            decision_ttl_millis: TTL_MILLIS,
            resume_ttl_millis: TTL_MILLIS,
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

fn page_lines() -> Vec<String> {
    fixture::cases().cases.into_iter().find(|case| case.id == PAGE_CASE).expect("case exists").lines
}

fn routing_workflow() -> Workflow {
    fixture::routing_workflow(&page_lines()).0
}

/// The routing fixture with an approval gate consuming the routed `page` continuation: the
/// Condition decides before the run suspends.
fn gate_after_route() -> Workflow {
    let mut workflow = routing_workflow();
    let gate = workflow.add_approval_node(GATE_LABEL, gate_config(vec![GATE_INPUT.to_owned()], Vec::new()));
    let continuation = fixture::node_by_label(&workflow, PAGE_CONTINUATION);
    fixture::connect(&mut workflow, (continuation, STDOUT), (gate, GATE_INPUT));
    workflow
}

/// The routing fixture with an approval gate ahead of the log source: the Condition decides only
/// after the run resumes.
fn gate_before_route() -> Workflow {
    let mut workflow = routing_workflow();
    let gate = workflow.add_approval_node(
        GATE_LABEL,
        gate_config(Vec::new(), vec![ResumeValueField {
            name: ResumeBindingName::try_from_text(RESUME_VALUE_NAME).expect("binding fixture is valid"),
            type_tag: STRING_TYPE.to_owned(),
        }]),
    );
    let source = fixture::node_by_label(&workflow, fixture::SOURCE_LABEL);
    let gate_output = workflow.graph.node(gate).expect("gate exists").output_ports[0].label.clone();
    fixture::connect(&mut workflow, (gate, &gate_output), (source, STDIN));
    // The source now receives the resume value on stdin. Drain it before printing so the shell
    // never exits while stdin is still being written (a broken pipe fails the dispatch).
    let command = format!("cat > /dev/null; {}", fixture::printf_command(&page_lines()));
    fixture::set_shell_command(&mut workflow, source, &command);
    workflow
}

fn edited_profile(edit: impl FnOnce(&mut serde_json::Value)) -> serde_json::Value {
    let mut profile = fixture::profile_json();
    edit(&mut profile);
    profile
}

fn runtime(profile: &serde_json::Value, adapter: &Arc<Counting>) -> lattice::workflow::decision::Runtime {
    fixture::runtime(profile, Arc::clone(adapter) as Arc<dyn Adapter>)
}

struct Suspended {
    store: lattice::store::Store,
    _directory: tempfile::TempDir,
    run_id: RunId,
    request: ApprovalRequest,
}

/// Saves `workflow`, runs it with the fixture profile until its gate suspends, and approves it.
async fn suspend_and_approve(workflow: &Workflow, run_id: RunId, adapter: &Arc<Counting>) -> Suspended {
    let (store, directory) = common::temp_store();
    store.save_workflow(workflow).expect("workflow saves");
    let authority = common::authority::TestRunContextProvider::new();
    let summary = lattice::headless::run_saved_workflow_with_typed_policy(
        lattice::headless::TypedPolicyRunRequest {
            store: &store,
            workflow_name: fixture::WORKFLOW_NAME,
            run_id,
            authority_provider: &authority,
        },
        runtime(&fixture::profile_json(), adapter),
    )
    .await
    .expect("typed policy run suspends");
    assert_eq!(summary.status, HeadlessRunStatus::Suspended);
    let pending = store.load_workflow_approval_for_run(run_id).expect("request loads").expect("request exists");
    lattice::workflow::gate::shell::decide(&store, &DecisionCommand {
        request_id: pending.id.clone(),
        expected_workflow_revision: pending.workflow_revision.clone(),
        actor: ActorId::try_from_text(APPROVER).expect("actor fixture is valid"),
        decision: DecisionClass::Approve,
        reason: Some("reviewed".to_owned()),
        observed_epoch_millis: pending.requested_epoch_millis + OBSERVATION_DELTA_MILLIS,
        evidence: DecisionAuthorityEvidence::LocalOperator,
    })
    .expect("approval records");
    let request = store
        .load_workflow_approval(&pending.id)
        .expect("approved request loads")
        .expect("approved request exists");
    Suspended {
        store,
        _directory: directory,
        run_id,
        request,
    }
}

fn resume_command(request: &ApprovalRequest, owner: &str, values: Vec<ResumeValue>) -> ResumeCommand {
    ResumeCommand {
        request_id: request.id.clone(),
        expected_workflow_revision: request.workflow_revision.clone(),
        expected_event_head: request.events.last().expect("approval event head").id.clone(),
        owner: ResumeOwnerId::try_from_text(owner).expect("owner fixture is valid"),
        observed_epoch_millis: request.requested_epoch_millis + OBSERVATION_DELTA_MILLIS,
        values,
        credentials: Vec::new(),
    }
}

async fn resume(
    suspended: &Suspended,
    owner: &str,
    values: Vec<ResumeValue>,
    typed_policy: Option<lattice::workflow::decision::Runtime>,
) -> Result<lattice::workflow::gate::shell::ResumeExecution, Error> {
    let command = resume_command(&suspended.request, owner, values);
    let authority = common::authority::TestRunContextProvider::new();
    let agent_runtime = lattice::agent_node::RuntimeOptions::default();
    match typed_policy {
        Some(runtime) => {
            lattice::workflow::gate::shell::resume_with_typed_policy(
                &suspended.store,
                &command,
                agent_runtime,
                &authority,
                runtime,
            )
            .await
        }
        None => {
            lattice::workflow::gate::shell::resume_with_authority_provider(
                &suspended.store,
                &command,
                agent_runtime,
                &authority,
            )
            .await
        }
    }
}

fn details(suspended: &Suspended) -> RunDetailRecord {
    suspended.store.load_run_details(suspended.run_id).expect("details load").expect("details exist")
}

fn decisions(details: &RunDetailRecord) -> Vec<(NodeId, Record)> {
    details
        .nodes
        .iter()
        .filter_map(|node| node.policy_decision.clone().map(|record| (node.node_id, record)))
        .collect()
}

fn single_decision(details: &RunDetailRecord) -> Record {
    let decided = decisions(details);
    assert_eq!(decided.len(), 1, "exactly one Condition node decided");
    let route = details
        .nodes
        .iter()
        .find(|node| node.node_label == fixture::ROUTE_LABEL)
        .expect("route node detail")
        .node_id;
    assert_eq!(decided[0].0, route);
    decided[0].1.clone()
}

fn assert_only_page_continued(details: &RunDetailRecord) {
    let completed = fixture::CONTINUATIONS
        .iter()
        .filter(|(_, label)| {
            details
                .nodes
                .iter()
                .any(|node| node.node_label == *label && node.status == RunNodeDetailStatus::Completed)
        })
        .map(|(_, label)| *label)
        .collect::<Vec<_>>();
    assert_eq!(completed, vec![PAGE_CONTINUATION]);
}

// r[verify lattice.typed_policy.record.scenario.suspended]
// r[verify lattice.typed_policy.replay.scenario.resumed]
#[tokio::test]
async fn a_decision_before_suspension_survives_resume_without_adapter_work() {
    let before = Counting::new();
    let suspended = suspend_and_approve(&gate_after_route(), RunId(61), &before).await;
    assert_eq!(before.calls(), 1);
    let stored = single_decision(&details(&suspended));
    assert_eq!(stored.routed_label.as_deref(), Some(PAGE_LABEL));
    assert_eq!(stored.workflow_outcome, None, "a suspended run has no eventual outcome yet");

    let after = Counting::new();
    let resumed = resume(&suspended, "resume-carried", Vec::new(), Some(runtime(&fixture::profile_json(), &after)))
        .await
        .expect("resume carries the stored decision");
    assert_eq!(resumed.summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(after.calls(), 0, "the stored decision is reused with no adapter call");

    let finished = details(&suspended);
    assert_only_page_continued(&finished);
    let carried = single_decision(&finished);
    assert_eq!(carried, Record {
        workflow_outcome: Some(RunOutcome::Completed),
        ..stored
    });
    let labels = fixture::LABELS.map(str::to_owned).to_vec();
    let replayed =
        lattice::workflow::decision::ledger::replay(&carried, &fixture::admit(&fixture::profile_json()), &labels);
    assert_eq!(replayed, Ok(PAGE_LABEL.to_owned()));
    assert_eq!(after.calls(), 0);
}

// r[verify lattice.typed_policy.record.scenario.suspended]
#[tokio::test]
async fn a_decision_made_after_resume_is_persisted() {
    let before = Counting::new();
    let suspended = suspend_and_approve(&gate_before_route(), RunId(62), &before).await;
    assert_eq!(before.calls(), 0, "the Condition is behind the gate");
    assert!(decisions(&details(&suspended)).is_empty());

    let after = Counting::new();
    let resumed = resume(
        &suspended,
        "resume-decides",
        vec![ResumeValue {
            name: ResumeBindingName::try_from_text(RESUME_VALUE_NAME).expect("binding fixture is valid"),
            value: Value::from_string(RESUME_TEXT.to_owned()),
        }],
        Some(runtime(&fixture::profile_json(), &after)),
    )
    .await
    .expect("resume decides and completes");
    assert_eq!(resumed.summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(after.calls(), 1);

    let finished = details(&suspended);
    assert_only_page_continued(&finished);
    let record = single_decision(&finished);
    assert_eq!(record.profile, fixture::admit(&fixture::profile_json()).identity);
    assert_eq!(record.routed_label.as_deref(), Some(PAGE_LABEL));
    assert_eq!(record.workflow_outcome, Some(RunOutcome::Completed));
}

// r[verify lattice.typed_policy.replay.scenario.changed]
// r[verify lattice.typed_policy.replay.scenario.resumed]
#[tokio::test]
async fn a_stale_profile_or_artifact_refuses_resume_and_leaves_the_record_unchanged() {
    let before = Counting::new();
    let suspended = suspend_and_approve(&gate_after_route(), RunId(63), &before).await;
    let stored_details = details(&suspended);
    let stored_request = suspended.request.clone();

    let after = Counting::new();
    let revised = edited_profile(|profile| profile["version"] = json!(2));
    let changed_artifact = edited_profile(|profile| profile["cohort"]["artifact"] = json!(OTHER_ARTIFACT));
    let disabled = edited_profile(|profile| profile["enabled"] = json!(false));
    for (owner, typed_policy) in [
        ("resume-revised", Some(runtime(&revised, &after))),
        ("resume-artifact", Some(runtime(&changed_artifact, &after))),
        ("resume-disabled", Some(runtime(&disabled, &after))),
        ("resume-unbound", None),
    ] {
        let refused = resume(&suspended, owner, Vec::new(), typed_policy).await;
        assert!(
            matches!(refused, Err(Error::PolicyDecisionStale(ReplayRefusal::ProfileChanged))),
            "{owner}: {refused:?}"
        );
        assert!(
            refused.err().is_some_and(|error| error.to_string().contains("stale typed policy decision")),
            "{owner}: the refusal names the stale decision"
        );
        assert_eq!(after.calls(), 0, "{owner}: a stale decision is never re-decided");
        assert_eq!(details(&suspended), stored_details, "{owner}: the stored record is unchanged");
        let request = suspended
            .store
            .load_workflow_approval(&stored_request.id)
            .expect("request reloads")
            .expect("request exists");
        assert_eq!(
            serde_json::to_value(&request).expect("request encodes"),
            serde_json::to_value(&stored_request).expect("request encodes"),
            "{owner}: the refused resume claims nothing"
        );
        assert_eq!(request.status, ApprovalRequestStatus::Approved);
    }

    let resumed = resume(&suspended, "resume-current", Vec::new(), Some(runtime(&fixture::profile_json(), &after)))
        .await
        .expect("the current profile still resumes");
    assert_eq!(resumed.summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(after.calls(), 0);
}
