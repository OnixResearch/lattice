mod common;

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::AttemptState;
use lattice::engine::protocol::Error;
use lattice::engine::protocol::HttpAdapter;
use lattice::engine::protocol::Identity;
use lattice::engine::protocol::Issue;
use lattice::engine::protocol::LabelId;
use lattice::engine::protocol::MessageEnvelope;
use lattice::engine::protocol::PlacementKind;
use lattice::engine::protocol::RolePlacement;
use lattice::engine::protocol::SessionPlan;
use lattice::engine::protocol::SessionRecord;
use lattice::engine::protocol::fail_before_dispatch;
use lattice::engine::protocol::plan_action;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::retry_unknown;
use lattice::engine::protocol::shell::AdapterDispatch;
use lattice::engine::protocol::shell::AdapterProfile;
use lattice::engine::protocol::shell::Adapters;
use lattice::engine::protocol::shell::ProtocolAdapter;
use lattice::engine::protocol::shell::ProtocolAuthority;
use lattice::engine::protocol::shell::Runtime;
use lattice::engine::protocol::shell::SessionRepository;
use lattice::http::HttpClient;
use lattice::http::HttpError;
use lattice::http::HttpRequestPlan;
use lattice::http::HttpResponse;
use lattice::store::Store;
use serde::Deserialize;
use serde::Serialize;

fn must<T, E: Debug>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|error| panic!("fixture operation failed: {error:?}"))
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum Actual {
    NotApplied,
    Applied,
    InProgress,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum Response {
    Success,
    InternalError,
    BadGateway,
    LostAck,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct Scenario {
    actual: Actual,
    response: Response,
}

// This file is independent of Lattice's database and event trace. Responses and
// agent-facing summaries never write it. The deterministic receiver owns it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Operation {
    logical_operation: Identity,
    envelope: MessageEnvelope,
    generation: u64,
    deliveries: u32,
    effect_count: u32,
    completed: bool,
    rejected: bool,
    compensations: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct Ledger {
    operations: Vec<Operation>,
}

// A retry changes attempt, message and authority-request identities, not the
// business operation at this protocol cursor. Keep recipients distinct in fanout.
fn logical_operation(envelope: &MessageEnvelope) -> Identity {
    common::protocol::identity(&must(serde_json::to_string(&(
        &envelope.protocol,
        &envelope.complete_local_set,
        &envelope.session,
        envelope.source_role,
        envelope.destination_role,
        &envelope.source_instance,
        &envelope.destination_instance,
        &envelope.source_local_artifact,
        &envelope.destination_local_artifact,
        &envelope.step,
        envelope.label,
        envelope.payload_tag,
        envelope.value,
        &envelope.payload_reference,
        &envelope.schema_set,
        &envelope.adapter_cohort,
    ))))
}

fn ledger(path: &Path) -> Ledger {
    if !path.exists() {
        return Ledger::default();
    }
    must(serde_json::from_slice(&must(std::fs::read(path))))
}

fn save_ledger(path: &Path, ledger: &Ledger) {
    must(std::fs::write(path, must(serde_json::to_vec(ledger))));
}

struct Receiver {
    path: PathBuf,
    scenario: Scenario,
}

impl Receiver {
    fn reject_before_start(&self, envelope: &MessageEnvelope) {
        let mut records = ledger(&self.path);
        assert!(!records.operations.iter().any(|operation| operation.envelope.identity == envelope.identity));
        records.operations.push(Operation {
            logical_operation: logical_operation(envelope),
            envelope: envelope.clone(),
            generation: 1,
            deliveries: 0,
            effect_count: 0,
            completed: false,
            rejected: true,
            compensations: 0,
        });
        save_ledger(&self.path, &records);
    }
}

impl HttpClient for Receiver {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        let envelope: MessageEnvelope = must(serde_json::from_str(
            request.body.as_deref().unwrap_or_else(|| panic!("missing canonical envelope")),
        ));
        let mut records = ledger(&self.path);
        if let Some(operation) = records.operations.iter_mut().find(|item| item.envelope.identity == envelope.identity)
        {
            operation.deliveries += 1;
            // Deliberately not idempotent: an accidental resend must fail the checker.
            if matches!(self.scenario.actual, Actual::Applied) {
                operation.effect_count += 1;
            }
        } else {
            records.operations.push(Operation {
                logical_operation: logical_operation(&envelope),
                envelope,
                generation: 1,
                deliveries: 1,
                effect_count: u32::from(matches!(self.scenario.actual, Actual::Applied)),
                completed: matches!(self.scenario.actual, Actual::Applied),
                rejected: matches!(self.scenario.actual, Actual::NotApplied),
                compensations: 0,
            });
        }
        save_ledger(&self.path, &records);
        match self.scenario.response {
            Response::LostAck => Err(HttpError::Timeout("receiver ACK lost after ledger persistence".to_owned())),
            response => Ok(HttpResponse {
                status: match response {
                    Response::Success => 202,
                    Response::InternalError => 500,
                    Response::BadGateway => 502,
                    Response::LostAck => unreachable!(),
                },
                headers: HashMap::new(),
                // Labeled response data, not admitted application evidence.
                body: "{\"summary\":\"done\"}".to_owned(),
            }),
        }
    }
}

struct Adapter {
    profile: AdapterProfile,
    receiver: Receiver,
    reject_before_start: bool,
    failure_shaped: bool,
}

impl ProtocolAdapter for Adapter {
    fn profile(&self) -> &AdapterProfile {
        &self.profile
    }

    fn dispatch(
        &self,
        envelope: &MessageEnvelope,
        limits: &lattice::engine::protocol::AdapterLimits,
    ) -> AdapterDispatch {
        if self.reject_before_start {
            self.receiver.reject_before_start(envelope);
            return AdapterDispatch::RejectedBeforeStart;
        }
        let AdapterProfile::Native { cohort, runtime_route } = &self.profile else {
            panic!("native-only receiver fixture");
        };
        let transport = HttpAdapter::native(
            cohort.clone(),
            runtime_route.clone(),
            "https://deterministic-receiver.invalid/receive".to_owned(),
            &self.receiver,
        );
        let result = transport.dispatch(envelope, limits);
        if self.failure_shaped {
            AdapterDispatch::ObservedFailure
        } else {
            result
        }
    }
}

struct Authority;

impl ProtocolAuthority for Authority {
    fn authorize(&self, _session: &SessionRecord, envelope: &MessageEnvelope) -> Result<Identity, Issue> {
        Ok(common::protocol::identity(&format!("current-authority:{}", envelope.identity.as_str())))
    }
}

fn request() -> ActionRequest {
    ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("independent-receiver-payload"),
    }
}

fn plan(fanout: bool) -> (SessionPlan, Vec<RolePlacement>) {
    use common::protocol as fixture;
    let mut source =
        fixture::native_source(fixture::transfer(fixture::ROLE_A, fixture::ROLE_B, fixture::LABEL_A, fixture::end()));
    if fanout {
        let mut third = source.roles[1].clone();
        third.id = fixture::ROLE_C;
        source.roles.push(third);
        let mut choice = fixture::choice();
        let choregraph_choreography::model::tree::Node::Choice { branches, .. } = &mut choice else {
            panic!("choice fixture lost its choice");
        };
        for branch in branches {
            let payload = if branch.label == fixture::LABEL_A {
                fixture::PAYLOAD_A
            } else {
                fixture::PAYLOAD_B
            };
            branch.body = fixture::communication(
                fixture::ROLE_A,
                fixture::ROLE_B,
                branch.label,
                payload,
                fixture::communication(fixture::ROLE_A, fixture::ROLE_C, branch.label, payload, fixture::end()),
            );
        }
        source.protocol = choice;
    }
    let cohort = fixture::cohort(source);
    let placements = fixture::native_placements(&cohort);
    let plan = must(plan_session(cohort, placements.clone(), fixture::identity("independent-receiver-session")));
    (plan, placements)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum Later {
    Normal,
    Missing,
    Mismatched,
    Reject,
    RejectAll,
    Unknown,
    FailureShaped,
}

fn adapters(path: &Path, placements: &[RolePlacement], scenario: Scenario, later: Later) -> Adapters {
    let mut native: HashMap<String, Box<dyn ProtocolAdapter>> = HashMap::new();
    for placement in placements {
        let PlacementKind::Native { runtime_route } = &placement.kind else {
            panic!("unexpected external fixture");
        };
        let is_later = placement.role == common::protocol::lattice_role(common::protocol::ROLE_C);
        if is_later && matches!(later, Later::Missing) {
            continue;
        }
        let cohort = if is_later && matches!(later, Later::Mismatched) {
            common::protocol::identity("wrong-adapter-cohort")
        } else {
            placement.adapter_cohort.clone()
        };
        let mut receiver_scenario = scenario;
        if is_later && matches!(later, Later::Unknown) {
            receiver_scenario = Scenario {
                actual: Actual::InProgress,
                response: Response::LostAck,
            };
        }
        native.insert(
            runtime_route.as_str().to_owned(),
            Box::new(Adapter {
                profile: AdapterProfile::Native {
                    cohort,
                    runtime_route: runtime_route.clone(),
                },
                receiver: Receiver {
                    path: path.to_owned(),
                    scenario: receiver_scenario,
                },
                reject_before_start: matches!(later, Later::RejectAll) || (is_later && matches!(later, Later::Reject)),
                failure_shaped: matches!(later, Later::FailureShaped),
            }),
        );
    }
    Adapters::new(native, HashMap::new())
}

fn load(store: &Store, plan: &SessionPlan) -> SessionRecord {
    must(store.load(&plan.session)).unwrap_or_else(|| panic!("session missing after reopen"))
}

fn dispatch_fixture(root: &Path, scenario: Scenario) {
    let (plan, placements) = plan(false);
    let store = must(Store::new(root.join("lattice.redb")));
    let selected = adapters(&root.join("receiver.json"), &placements, scenario, Later::Normal);
    let runtime = Runtime::new(&store, &Authority, &selected);
    let initial = must(runtime.create(plan));
    let result = runtime.dispatch(&initial, request());
    if matches!(scenario.response, Response::Success) {
        assert_eq!(must(result).dispatched_messages, 1);
    } else {
        assert_eq!(result, Err(Error::Issue(Issue::AdapterFailed)));
    }
}

fn dispatch_fanout_fixture(root: &Path, scenario: Scenario, later: Later) {
    let (plan, placements) = plan(true);
    let store = must(Store::new(root.join("lattice.redb")));
    let selected = adapters(&root.join("receiver.json"), &placements, scenario, later);
    let runtime = Runtime::new(&store, &Authority, &selected);
    let initial = must(runtime.create(plan));
    assert_eq!(runtime.dispatch(&initial, request()), Err(Error::Issue(Issue::AdapterFailed)));
}

// A real process boundary: the child exits after the ambiguous dispatch; the
// parent opens a fresh database and independent ledger, never a serialized agent state.
#[test]
fn receiver_restart_worker() {
    let Some(root) = std::env::var_os("LATTICE_PROTOCOL_RECEIVER_WORKER") else {
        return;
    };
    let root = PathBuf::from(root);
    let scenario: Scenario = must(serde_json::from_slice(&must(std::fs::read(root.join("scenario.json")))));
    if root.join("fanout.json").exists() {
        let later: Later = must(serde_json::from_slice(&must(std::fs::read(root.join("fanout.json")))));
        dispatch_fanout_fixture(&root, scenario, later);
    } else {
        dispatch_fixture(&root, scenario);
    }
}

#[derive(Clone, Copy, Debug)]
enum History {
    Unchanged,
    TruthfulCorrection,
    RewrittenDone,
    StaleFailure,
}

fn history(history: History) -> Vec<&'static str> {
    match history {
        History::Unchanged => vec!["original operation attempted; no admitted observation"],
        History::TruthfulCorrection => vec![
            "original operation attempted; no admitted observation",
            "correction: transport response is not completion; preserve original operation identity",
        ],
        History::RewrittenDone => vec!["FALSE INPUT: correction already completed; operation done"],
        History::StaleFailure => vec!["FALSE INPUT: operation failed; retry it"],
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Intent {
    Reconcile,
    ClaimCompletion,
    Retry,
}

// Deterministic stand-in for a controller reading a transcript. Every line is
// consumed; a later correction replaces the proposed intent, not durable facts.
fn summary_intent(summary: &[&str]) -> Intent {
    assert!(!summary.is_empty());
    let mut intent = Intent::Reconcile;
    for line in summary {
        intent = match *line {
            "original operation attempted; no admitted observation"
            | "correction: transport response is not completion; preserve original operation identity" => {
                Intent::Reconcile
            }
            "FALSE INPUT: correction already completed; operation done" => Intent::ClaimCompletion,
            "FALSE INPUT: operation failed; retry it" => Intent::Retry,
            _ => panic!("unknown deterministic summary input: {line}"),
        };
    }
    intent
}

struct Handoff {
    suggested: Intent,
    permitted: Intent,
    session: SessionRecord,
    completed: Vec<Identity>,
}

fn reconstruct_handoff(runtime: &Runtime<'_>, session: &Identity, path: &Path, summary: &[&str]) -> Handoff {
    let suggested = summary_intent(summary);
    let session = must(runtime.replay(session));
    let records = ledger(path);
    // This receiver profile equates its exact accepted observation with completed
    // work. Neither receiver completion alone nor a summary admits that observation.
    let completed = records
        .operations
        .iter()
        .filter(|operation| {
            operation.completed
                && session.attempts.iter().any(|attempt| {
                    attempt.messages.contains(&operation.envelope)
                        && attempt.observations.iter().any(|observation| {
                            observation.message == operation.envelope.identity && observation.accepted
                        })
                })
        })
        .map(|operation| operation.envelope.identity.clone())
        .collect();
    let permitted = if session.attempts.last().is_some_and(|attempt| attempt.state == AttemptState::Committed) {
        Intent::ClaimCompletion
    } else {
        Intent::Reconcile
    };
    Handoff {
        suggested,
        permitted,
        session,
        completed,
    }
}

fn broken_completion_claims(summary: &[&str], messages: &[MessageEnvelope]) -> Vec<Identity> {
    if summary_intent(summary) == Intent::ClaimCompletion {
        messages.iter().map(|message| message.identity.clone()).collect()
    } else {
        Vec::new()
    }
}

fn broken_retry(session: &SessionRecord, summary: &[&str], fresh_attempt: bool) -> Option<MessageEnvelope> {
    if summary_intent(summary) != Intent::Retry {
        return None;
    }
    let original = &session.attempts[0];
    if fresh_attempt {
        // Deliberately broken: the controller invents operator risk acknowledgment
        // from a stale summary and then sends the fresh canonical retry message.
        let retry = must(retry_unknown(session, &original.identity, true));
        Some(retry.prepared.attempts[1].messages[0].clone())
    } else {
        Some(original.messages[0].clone())
    }
}

// Current receiver-profile contract: missing/unavailable is not rejection. Only
// an exact current receiver record may be passed to Runtime::observe. The runtime
// does not authenticate arbitrary caller-supplied evidence references itself.
fn observe(
    runtime: &Runtime<'_>,
    session: &SessionRecord,
    path: &Path,
    candidate: &Operation,
) -> Result<SessionRecord, Error> {
    let current = ledger(path);
    let exact = current
        .operations
        .iter()
        .find(|operation| operation.envelope.identity == candidate.envelope.identity);
    if exact != Some(candidate)
        || !((candidate.completed && !candidate.rejected && candidate.effect_count == 1)
            || (candidate.rejected && !candidate.completed && candidate.effect_count == 0))
    {
        return Err(Error::Issue(Issue::ObservationMismatch));
    }
    let evidence = common::protocol::identity(&must(serde_json::to_string(candidate)));
    runtime.observe(
        session,
        &candidate.envelope,
        candidate.completed,
        evidence,
        if candidate.rejected {
            Some(Issue::AdapterFailed)
        } else {
            None
        },
    )
}

// Independent oracle, not a check of whether the internal trace looks happy.
// Aggregate logical operations across canonical messages and fresh retry attempts.
fn check(records: &Ledger, completed: &[Identity]) -> Result<(), &'static str> {
    let mut totals = HashMap::<&str, (u64, u64)>::new();
    for operation in &records.operations {
        if operation.logical_operation != logical_operation(&operation.envelope) {
            return Err("incorrect logical-operation identity");
        }
        let counts = totals.entry(operation.logical_operation.as_str()).or_default();
        counts.0 += u64::from(operation.effect_count);
        counts.1 += u64::from(operation.deliveries);
        if operation.effect_count > 1 || operation.deliveries > 1 || operation.compensations != 0 {
            return Err("duplicated effect, delivery, or unjustified compensation");
        }
    }
    if totals.values().any(|(effects, deliveries)| *effects > 1 || *deliveries > 1) {
        return Err("duplicated logical operation across messages or attempts");
    }
    for identity in completed {
        if !records.operations.iter().any(|operation| {
            operation.envelope.identity == *identity && operation.completed && operation.effect_count == 1
        }) {
            return Err("false external-effect completion");
        }
    }
    Ok(())
}

// r[verify lattice.role_protocol.uncertainty.history]
// r[verify lattice.role_protocol.uncertainty.http]
// r[verify lattice.role_protocol.uncertainty.crash]
// r[verify lattice.role_protocol.replay.send]
#[test]
fn independent_receiver_matrix_ignores_responses_and_handoff_summaries_across_restart() {
    for actual in [Actual::NotApplied, Actual::Applied, Actual::InProgress] {
        for response in [
            Response::Success,
            Response::InternalError,
            Response::BadGateway,
            Response::LostAck,
        ] {
            for restart in [false, true] {
                let root = must(tempfile::tempdir());
                let scenario = Scenario { actual, response };
                if restart {
                    must(std::fs::write(root.path().join("scenario.json"), must(serde_json::to_vec(&scenario))));
                    let status = must(
                        std::process::Command::new(must(std::env::current_exe()))
                            .args(["--exact", "receiver_restart_worker", "--nocapture"])
                            .env("LATTICE_PROTOCOL_RECEIVER_WORKER", root.path())
                            .status(),
                    );
                    assert!(status.success());
                } else {
                    dispatch_fixture(root.path(), scenario);
                }
                let (plan, _) = plan(false);
                let path = root.path().join("receiver.json");
                let records = ledger(&path);
                assert_eq!(records.operations.len(), 1);
                assert_eq!(records.operations[0].effect_count, u32::from(matches!(actual, Actual::Applied)));
                let store = must(Store::new(root.path().join("lattice.redb")));
                let empty = Adapters::new(HashMap::new(), HashMap::new());
                let runtime = Runtime::new(&store, &Authority, &empty);
                let unknown = load(&store, &plan);
                assert_eq!(unknown.attempts[0].state, AttemptState::DispatchedUnknown);
                assert_eq!(unknown.owners[0].owner, request().source);
                let attempt = &unknown.attempts[0];
                assert_eq!(
                    fail_before_dispatch(&unknown, &attempt.identity, Issue::AdapterFailed),
                    Err(Error::Issue(Issue::ReorderedMessage))
                );
                assert_eq!(
                    retry_unknown(&unknown, &attempt.identity, false),
                    Err(Error::Issue(Issue::AcknowledgementRequired))
                );
                assert_eq!(plan_action(&unknown, request()), Err(Error::Issue(Issue::AttemptInFlight)));
                assert_eq!(runtime.dispatch(&unknown, request()), Err(Error::Issue(Issue::AttemptInFlight)));
                for representation in [
                    History::Unchanged,
                    History::TruthfulCorrection,
                    History::RewrittenDone,
                    History::StaleFailure,
                ] {
                    let handoff = history(representation);
                    let reconstructed = reconstruct_handoff(&runtime, &plan.session, &path, &handoff);
                    assert_eq!(reconstructed.suggested, match representation {
                        History::Unchanged | History::TruthfulCorrection => Intent::Reconcile,
                        History::RewrittenDone => Intent::ClaimCompletion,
                        History::StaleFailure => Intent::Retry,
                    });
                    if matches!(representation, History::TruthfulCorrection) {
                        assert_eq!(handoff[0], history(History::Unchanged)[0]);
                    }
                    assert_eq!(reconstructed.permitted, Intent::Reconcile);
                    assert!(reconstructed.completed.is_empty());
                    assert_eq!(reconstructed.session, unknown);
                    assert_eq!(load(&store, &plan), unknown);
                    assert_eq!(ledger(&path), records);
                    assert_eq!(check(&ledger(&path), &[]), Ok(()));
                }
                assert_eq!(runtime.settle(&unknown, &attempt.identity), Err(Error::Issue(Issue::ObservationMismatch)));
                let operation = records.operations[0].clone();
                if matches!(actual, Actual::InProgress) {
                    assert_eq!(
                        observe(&runtime, &unknown, &path, &operation),
                        Err(Error::Issue(Issue::ObservationMismatch))
                    );
                    assert_eq!(must(runtime.replay(&plan.session)), unknown);
                    // The same receiver operation finishes; a new generation is required.
                    let mut finished = records.clone();
                    finished.operations[0].generation += 1;
                    finished.operations[0].completed = true;
                    finished.operations[0].effect_count = 1;
                    save_ledger(&path, &finished);
                    assert_eq!(
                        observe(&runtime, &unknown, &path, &operation),
                        Err(Error::Issue(Issue::ObservationMismatch))
                    );
                }
                let current = ledger(&path).operations[0].clone();
                let observed = must(observe(&runtime, &unknown, &path, &current));
                let settled = must(runtime.settle(&observed, &attempt.identity));
                if current.completed {
                    assert_eq!(settled.attempts[0].state, AttemptState::Committed);
                    assert_eq!(settled.owners[0].owner, common::protocol::lattice_role(common::protocol::ROLE_B));
                    assert_eq!(check(&ledger(&path), std::slice::from_ref(&current.envelope.identity)), Ok(()));
                } else {
                    assert_eq!(settled.attempts[0].state, AttemptState::Failed);
                    assert_eq!(settled.owners, unknown.owners);
                    assert!(check(&ledger(&path), std::slice::from_ref(&current.envelope.identity)).is_err());
                }
                assert_eq!(must(runtime.replay(&plan.session)), settled);
                assert_eq!(ledger(&path).operations[0].deliveries, 1);
            }
        }
    }
}

// r[verify lattice.role_protocol.uncertainty.fanout]
#[test]
fn fanout_preflights_later_missing_and_mismatched_adapters_before_any_receiver_work() {
    for later in [Later::Missing, Later::Mismatched] {
        let root = must(tempfile::tempdir());
        let path = root.path().join("receiver.json");
        let (plan, placements) = plan(true);
        let store = must(Store::new(root.path().join("lattice.redb")));
        let selected = adapters(
            &path,
            &placements,
            Scenario {
                actual: Actual::Applied,
                response: Response::Success,
            },
            later,
        );
        let runtime = Runtime::new(&store, &Authority, &selected);
        let initial = must(runtime.create(plan.clone()));
        assert_eq!(must(plan_action(&initial, request())).messages.len(), 2);
        assert_eq!(runtime.dispatch(&initial, request()), Err(Error::Issue(Issue::AdapterFailed)));
        let failed = load(&store, &plan);
        assert_eq!(failed.attempts[0].state, AttemptState::Failed);
        assert!(ledger(&path).operations.is_empty());
        assert_eq!(must(runtime.replay(&plan.session)), failed);
    }
}

// r[verify lattice.role_protocol.uncertainty.fanout]
#[test]
fn partial_fanout_requires_each_exact_observation_and_explicit_retry_risk() {
    for later in [Later::Reject, Later::Unknown] {
        for restart in [false, true] {
            let root = must(tempfile::tempdir());
            let path = root.path().join("receiver.json");
            let scenario = Scenario {
                actual: Actual::Applied,
                response: Response::Success,
            };
            if restart {
                must(std::fs::write(root.path().join("scenario.json"), must(serde_json::to_vec(&scenario))));
                must(std::fs::write(root.path().join("fanout.json"), must(serde_json::to_vec(&later))));
                let status = must(
                    std::process::Command::new(must(std::env::current_exe()))
                        .args(["--exact", "receiver_restart_worker", "--nocapture"])
                        .env("LATTICE_PROTOCOL_RECEIVER_WORKER", root.path())
                        .status(),
                );
                assert!(status.success());
            } else {
                dispatch_fanout_fixture(root.path(), scenario, later);
            }
            let (plan, _) = plan(true);
            let store = must(Store::new(root.path().join("lattice.redb")));
            let selected = Adapters::new(HashMap::new(), HashMap::new());
            let runtime = Runtime::new(&store, &Authority, &selected);
            let initial = must(lattice::engine::protocol::create_session(plan.clone()));
            let unknown = load(&store, &plan);
            let attempt = &unknown.attempts[0];
            assert_eq!(attempt.messages.len(), 2);
            assert_eq!(attempt.state, AttemptState::DispatchedUnknown);
            assert_eq!(plan_action(&unknown, request()), Err(Error::Issue(Issue::AttemptInFlight)));
            let before = ledger(&path);
            assert_eq!(before.operations[0].effect_count, 1);
            let observed_b = must(observe(&runtime, &unknown, &path, &before.operations[0]));
            assert_eq!(runtime.settle(&observed_b, &attempt.identity), Err(Error::Issue(Issue::ObservationMismatch)));
            assert_eq!(observed_b.cursors, initial.cursors);
            assert_eq!(must(runtime.replay(&plan.session)), observed_b);
            assert_eq!(ledger(&path), before);
            for representation in [
                History::Unchanged,
                History::TruthfulCorrection,
                History::RewrittenDone,
                History::StaleFailure,
            ] {
                let handoff = history(representation);
                let reconstructed = reconstruct_handoff(&runtime, &plan.session, &path, &handoff);
                assert_eq!(reconstructed.suggested, match representation {
                    History::Unchanged | History::TruthfulCorrection => Intent::Reconcile,
                    History::RewrittenDone => Intent::ClaimCompletion,
                    History::StaleFailure => Intent::Retry,
                });
                assert_eq!(reconstructed.permitted, Intent::Reconcile);
                assert_eq!(reconstructed.session, observed_b);
                assert_eq!(reconstructed.completed, vec![before.operations[0].envelope.identity.clone()]);
                assert_eq!(check(&before, &reconstructed.completed), Ok(()));
                assert_eq!(ledger(&path), before);
            }
            assert_eq!(
                retry_unknown(&observed_b, &attempt.identity, false),
                Err(Error::Issue(Issue::AcknowledgementRequired))
            );
            if matches!(later, Later::Unknown) {
                let mut finished = before.clone();
                finished.operations[1].generation += 1;
                finished.operations[1].completed = true;
                finished.operations[1].effect_count = 1;
                save_ledger(&path, &finished);
                let observed_c = must(observe(&runtime, &observed_b, &path, &finished.operations[1]));
                let settled = must(runtime.settle(&observed_c, &attempt.identity));
                assert_eq!(settled.attempts[0].state, AttemptState::Committed);
                assert_eq!(
                    check(
                        &ledger(&path),
                        &attempt.messages.iter().map(|message| message.identity.clone()).collect::<Vec<_>>()
                    ),
                    Ok(())
                );
            } else {
                // Risk acknowledgment plans a new identity, but never executes it.
                let retry = must(retry_unknown(&observed_b, &attempt.identity, true));
                assert_ne!(retry.prepared.attempts[1].identity, attempt.identity);
                assert_eq!(retry.prepared.attempts[0].state, AttemptState::AbandonedUnknown);
                assert_eq!(ledger(&path), before);
                // C's no-start guarantee was independently persisted with zero
                // deliveries/effects. Admit only that exact current receiver record.
                let rejection = &before.operations[1];
                assert_eq!(rejection.envelope, attempt.messages[1]);
                assert!(rejection.rejected);
                assert_eq!((rejection.deliveries, rejection.effect_count), (0, 0));
                let rejected = must(observe(&runtime, &observed_b, &path, rejection));
                assert_eq!(runtime.settle(&rejected, &attempt.identity), Err(Error::Issue(Issue::ObservationMismatch)));
                assert_eq!(plan_action(&rejected, request()), Err(Error::Issue(Issue::AttemptInFlight)));
                assert_eq!(must(runtime.replay(&plan.session)), rejected);
                assert_eq!(ledger(&path), before);
            }
        }
    }
}

// r[verify lattice.role_protocol.uncertainty.observations]
#[test]
fn substituted_wrong_and_stale_observations_cannot_resolve_unknown() {
    let root = must(tempfile::tempdir());
    let path = root.path().join("receiver.json");
    dispatch_fixture(root.path(), Scenario {
        actual: Actual::Applied,
        response: Response::BadGateway,
    });
    let (plan, _) = plan(false);
    let store = must(Store::new(root.path().join("lattice.redb")));
    let empty = Adapters::new(HashMap::new(), HashMap::new());
    let runtime = Runtime::new(&store, &Authority, &empty);
    let unknown = load(&store, &plan);
    let operation = ledger(&path).operations[0].clone();
    let mut wrong_session = operation.clone();
    wrong_session.envelope.session = common::protocol::identity("other-session");
    let mut wrong_attempt = operation.clone();
    wrong_attempt.envelope.attempt = common::protocol::identity("other-attempt");
    let mut wrong_message = operation.clone();
    wrong_message.envelope.identity = common::protocol::identity("other-message");
    let mut wrong_role = operation.clone();
    wrong_role.envelope.destination_role = request().source;
    let mut wrong_adapter = operation.clone();
    wrong_adapter.envelope.adapter_cohort = common::protocol::identity("substituted-adapter");
    let mut stale = operation.clone();
    stale.generation = 0;
    let mut substituted = operation.clone();
    substituted.completed = false;
    for forged in [
        wrong_session,
        wrong_attempt,
        wrong_message,
        wrong_role,
        wrong_adapter,
        stale,
        substituted,
    ] {
        assert!(observe(&runtime, &unknown, &path, &forged).is_err());
        // Core envelope admission independently checks all canonical context.
        if forged.envelope != operation.envelope {
            assert!(
                runtime
                    .observe(&unknown, &forged.envelope, true, common::protocol::identity("forged-evidence"), None)
                    .is_err()
            );
        }
        assert_eq!(load(&store, &plan), unknown);
    }
    let observed = must(observe(&runtime, &unknown, &path, &operation));
    assert!(observe(&runtime, &observed, &path, &operation).is_err());
    let settled = must(runtime.settle(&observed, &unknown.attempts[0].identity));
    assert!(observe(&runtime, &settled, &path, &operation).is_err());
    let retry = must(retry_unknown(&unknown, &unknown.attempts[0].identity, true));
    let prepared = retry.prepared;
    let attempt = &prepared.attempts[1];
    let authorized = must(lattice::engine::protocol::record_authorization(&prepared, &attempt.identity, vec![
        common::protocol::identity("new-current-authority"),
    ]));
    let new_unknown = must(lattice::engine::protocol::mark_dispatched_unknown(&authorized, &attempt.identity));
    assert!(
        lattice::engine::protocol::admit_observation(
            &new_unknown,
            &operation.envelope,
            true,
            common::protocol::identity("cross-attempt-evidence"),
            None
        )
        .is_err()
    );
}

// r[verify lattice.role_protocol.uncertainty.control]
#[test]
fn independent_checker_detects_a_broken_summary_or_response_trusting_control() {
    let root = must(tempfile::tempdir());
    let scenario = Scenario {
        actual: Actual::NotApplied,
        response: Response::Success,
    };
    dispatch_fixture(root.path(), scenario);
    let path = root.path().join("receiver.json");
    let actual = ledger(&path);
    let operation = &actual.operations[0];
    assert_eq!(check(&actual, &[]), Ok(()));
    // The broken control actually reads the selected false summary; truthful
    // representations must not produce this completion claim.
    for representation in [History::Unchanged, History::TruthfulCorrection, History::StaleFailure] {
        assert!(
            broken_completion_claims(&history(representation), std::slice::from_ref(&operation.envelope)).is_empty()
        );
    }
    let claims = broken_completion_claims(&history(History::RewrittenDone), std::slice::from_ref(&operation.envelope));
    assert_eq!(claims, vec![operation.envelope.identity.clone()]);
    assert_eq!(check(&actual, &claims), Err("false external-effect completion"));
    // Separately broken response controller trusts the actual scenario's 202.
    let response_claims = if matches!(scenario.response, Response::Success) {
        vec![operation.envelope.identity.clone()]
    } else {
        Vec::new()
    };
    assert_eq!(check(&actual, &response_claims), Err("false external-effect completion"));
    let mut duplicated = actual.clone();
    duplicated.operations[0].effect_count = 2;
    assert!(check(&duplicated, &[]).is_err());
    let mut compensated = actual;
    compensated.operations[0].compensations = 1;
    assert!(check(&compensated, &[]).is_err());
}

#[test]
fn failure_shaped_adapter_response_without_receipt_stays_unknown() {
    let root = must(tempfile::tempdir());
    let path = root.path().join("receiver.json");
    let (plan, placements) = plan(false);
    let store = must(Store::new(root.path().join("lattice.redb")));
    let selected = adapters(
        &path,
        &placements,
        Scenario {
            actual: Actual::Applied,
            response: Response::Success,
        },
        Later::FailureShaped,
    );
    let runtime = Runtime::new(&store, &Authority, &selected);
    let initial = must(runtime.create(plan.clone()));
    assert_eq!(runtime.dispatch(&initial, request()), Err(Error::Issue(Issue::AdapterFailed)));
    assert_eq!(load(&store, &plan).attempts[0].state, AttemptState::DispatchedUnknown);
    assert_eq!(ledger(&path).operations[0].effect_count, 1);
}

// r[verify lattice.role_protocol.uncertainty.rejection]
#[test]
fn genuine_first_rejection_is_terminal_and_starts_no_receiver_work() {
    let root = must(tempfile::tempdir());
    let path = root.path().join("receiver.json");
    let (plan, placements) = plan(true);
    let store = must(Store::new(root.path().join("lattice.redb")));
    let selected = adapters(
        &path,
        &placements,
        Scenario {
            actual: Actual::Applied,
            response: Response::Success,
        },
        Later::RejectAll,
    );
    let runtime = Runtime::new(&store, &Authority, &selected);
    let initial = must(runtime.create(plan.clone()));
    assert_eq!(runtime.dispatch(&initial, request()), Err(Error::Issue(Issue::AdapterFailed)));
    let failed = load(&store, &plan);
    assert_eq!(failed.attempts[0].state, AttemptState::Failed);
    let records = ledger(&path);
    assert_eq!(records.operations.len(), 1);
    assert!(records.operations[0].rejected);
    assert_eq!((records.operations[0].deliveries, records.operations[0].effect_count), (0, 0));
    assert_eq!(check(&records, &[]), Ok(()));
    assert_eq!(must(runtime.replay(&plan.session)), failed);
}

// r[verify lattice.role_protocol.uncertainty.control]
#[test]
fn broken_failed_summary_retry_is_detected_in_the_actual_persisted_effect_ledger() {
    for fresh_attempt in [false, true] {
        let root = must(tempfile::tempdir());
        let scenario = Scenario {
            actual: Actual::Applied,
            response: Response::InternalError,
        };
        dispatch_fixture(root.path(), scenario);
        let path = root.path().join("receiver.json");
        let original = ledger(&path);
        assert_eq!(original.operations[0].effect_count, 1);
        assert_eq!(check(&original, &[]), Ok(()));
        let (plan, _) = plan(false);
        let store = must(Store::new(root.path().join("lattice.redb")));
        let unknown = load(&store, &plan);
        assert!(broken_retry(&unknown, &history(History::TruthfulCorrection), fresh_attempt).is_none());
        let envelope = broken_retry(&unknown, &history(History::StaleFailure), fresh_attempt)
            .unwrap_or_else(|| panic!("broken controller ignored false retry input"));
        assert_eq!(logical_operation(&envelope), original.operations[0].logical_operation);
        let receiver = Receiver {
            path: path.clone(),
            scenario,
        };
        let response = must(receiver.execute(&HttpRequestPlan {
            method: "POST".to_owned(),
            url: "https://deterministic-receiver.invalid/receive".to_owned(),
            headers: HashMap::new(),
            body: Some(must(serde_json::to_string(&envelope))),
            timeout_secs: 1,
        }));
        assert_eq!(response.status, 500);
        // Reopen actual receiver output; a fresh retry creates a second record
        // with independently persisted work, not a fabricated duplicate counter.
        let duplicated = ledger(&path);
        if fresh_attempt {
            assert_ne!(envelope.identity, original.operations[0].envelope.identity);
            assert_ne!(envelope.attempt, original.operations[0].envelope.attempt);
            assert_eq!(duplicated.operations.len(), 2);
            for operation in &duplicated.operations {
                assert_eq!((operation.effect_count, operation.deliveries), (1, 1));
            }
            assert_eq!(check(&duplicated, &[]), Err("duplicated logical operation across messages or attempts"));
        } else {
            assert_eq!(duplicated.operations.len(), 1);
            assert_eq!(duplicated.operations[0].effect_count, 2);
            assert_eq!(duplicated.operations[0].deliveries, 2);
            assert!(check(&duplicated, &[]).is_err());
        }
        assert_eq!(load(&store, &plan), unknown);
    }
}
