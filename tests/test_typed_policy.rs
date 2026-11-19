//! Optional bounded typed policy operation on a declared Condition node (Lattice #23).
//!
//! The policy artifact is a hand-authored fixture (`provenance.kind = "fixture"`); these tests
//! qualify the Lattice operation boundary, not model quality.

mod common;
#[path = "support/typed_policy.rs"]
mod fixture;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::StepResult;
use lattice::engine::value::ContentHash;
use lattice::engine::value::Value;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::Workflow;
use lattice::workflow::decision::Adapter;
use lattice::workflow::decision::Response;
use lattice::workflow::decision::admission::Refusal;
use lattice::workflow::decision::binding::Invocation;
use lattice::workflow::decision::ledger::NodeRecord;
use lattice::workflow::decision::ledger::ReplayRefusal;
use lattice::workflow::decision::validation::Outcome;
use lattice::workflow::decision::validation::Source;
use pretty_assertions::assert_eq;
use serde_json::json;

type Cache = HashMap<ContentHash, HashMap<PortId, Value>>;

const SLOW_ADAPTER_MILLIS: u64 = 2_000;
const SHORT_DEADLINE_MILLIS: u64 = 20;
const CANCEL_AFTER_MILLIS: u64 = 40;
const TINY_RECEIPT_BYTES: u64 = 64;
const PARALLEL_SLOTS: u32 = 4;

/// Counts invocations and answers with a scripted response or a delegate.
struct Counting {
    calls: AtomicU64,
    behavior: Behavior,
}

enum Behavior {
    Delegate(Arc<dyn Adapter>),
    Fixed(Response),
    Slow(Response),
    Eligible(Vec<String>),
}

impl Counting {
    fn new(behavior: Behavior) -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicU64::new(0),
            behavior,
        })
    }

    fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Adapter for Counting {
    fn invoke(&self, invocation: &Invocation) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match &self.behavior {
            Behavior::Delegate(inner) => inner.invoke(invocation),
            Behavior::Fixed(response) => response.clone(),
            Behavior::Slow(response) => {
                std::thread::sleep(std::time::Duration::from_millis(SLOW_ADAPTER_MILLIS));
                response.clone()
            }
            Behavior::Eligible(eligible) => decide_with_eligible(invocation, eligible.clone()),
        }
    }
}

/// A tampering backend: decides with another eligible set than the declared candidates.
fn decide_with_eligible(invocation: &Invocation, eligible: Vec<String>) -> Response {
    let artifact = fixture::artifact();
    let admitted = animus_frozen_policy::admit_policy(&artifact, &invocation.binding.artifact);
    let request = animus_frozen_policy::DecisionRequest {
        binding: invocation.binding.clone(),
        features: invocation.features.clone(),
        eligible,
        fallback: None,
        max_operations: invocation.max_operation_count,
    };
    Response::Receipt(animus_frozen_policy::decide(admitted.as_ref(), &request).receipt.encode().into_bytes())
}

fn case(id: &str) -> fixture::Case {
    fixture::cases().cases.into_iter().find(|case| case.id == id).expect("case exists")
}

struct Run {
    workflow: Workflow,
    nodes: fixture::RoutingNodes,
    results: Vec<StepResult>,
    decisions: Vec<NodeRecord>,
}

async fn run_case(id: &str, runtime: Option<lattice::workflow::decision::Runtime>) -> Run {
    run_case_with(id, runtime, &HashMap::new(), |executor| executor).await
}

async fn run_case_with(
    id: &str,
    runtime: Option<lattice::workflow::decision::Runtime>,
    cache: &Cache,
    configure: impl FnOnce(Executor) -> Executor,
) -> Run {
    let (workflow, nodes) = fixture::routing_workflow(&case(id).lines);
    let executor = match runtime {
        Some(runtime) => Executor::new().with_typed_policy(runtime),
        None => Executor::new(),
    };
    let mut executor = configure(executor);
    let (_run_id, results) = executor.run(&workflow, cache).await;
    let decisions = executor.take_policy_decisions();
    Run {
        workflow,
        nodes,
        results,
        decisions,
    }
}

fn status(results: &[StepResult], node: NodeId) -> &NodeStatus {
    &results.iter().find(|step| step.node_id == node).expect("node has a result").status
}

fn single_decision(run: &Run) -> &lattice::workflow::decision::ledger::Record {
    assert_eq!(run.decisions.len(), 1, "exactly one decision per bound node execution");
    assert_eq!(run.decisions[0].node, run.nodes.route);
    &run.decisions[0].record
}

fn routed(run: &Run) -> Option<String> {
    fixture::routed_label(&run.results, &run.nodes).expect("at most one continuation ran")
}

fn profile_with(edit: impl FnOnce(&mut serde_json::Value)) -> serde_json::Value {
    let mut profile = fixture::profile_json();
    edit(&mut profile);
    profile
}

fn refusal(profile: &serde_json::Value) -> Refusal {
    let bytes = serde_json::to_vec(profile).expect("profile encodes");
    lattice::workflow::decision::admission::admit(&bytes).expect_err("profile is refused")
}

fn invalid(reason: &str) -> Outcome {
    Outcome::Invalid {
        reason: reason.to_owned(),
    }
}

// r[verify lattice.typed_policy.profile.scenario.valid]
#[test]
fn profile_admits_with_a_stable_canonical_identity() {
    let admitted = fixture::admit(&fixture::profile_json());
    let respaced = serde_json::to_vec_pretty(&fixture::profile_json()).expect("profile encodes");
    let again = lattice::workflow::decision::admission::admit(&respaced).expect("respaced profile admits");

    assert!(admitted.identity.starts_with("b3:"));
    assert_eq!(admitted.identity, again.identity);
    assert_eq!(admitted.profile.labels, fixture::LABELS.map(str::to_owned).to_vec());
    let revised = fixture::admit(&profile_with(|profile| profile["version"] = json!(2)));
    assert_ne!(revised.identity, admitted.identity);
}

// r[verify lattice.typed_policy.profile.scenario.invalid]
// r[verify lattice.typed_policy.stateless.scenario.refused]
#[test]
fn unbounded_stateful_or_malformed_profiles_are_refused() {
    let cases: Vec<(serde_json::Value, Refusal)> = vec![
        (profile_with(|profile| profile["stateful"] = json!(true)), Refusal::Stateful),
        (profile_with(|profile| profile["shape"] = json!("rank")), Refusal::Shape),
        (
            profile_with(|profile| profile["schema"] = json!("lattice.typed-policy-profile.v0")),
            Refusal::Schema,
        ),
        (profile_with(|profile| profile["limits"]["deadline_millis"] = json!(0)), Refusal::Limits),
        (profile_with(|profile| profile["limits"]["max_operation_count"] = json!(2_000_000)), Refusal::Limits),
        (profile_with(|profile| profile["labels"] = json!(["page", "page"])), Refusal::Labels),
        (profile_with(|profile| profile["labels"] = json!([])), Refusal::Labels),
        (profile_with(|profile| profile["labels"] = json!(["page", "ticket", "has space"])), Refusal::Labels),
        (profile_with(|profile| profile["unexpected"] = json!(1)), Refusal::Malformed),
        (
            profile_with(|profile| profile["fallback"] = json!({"kind": "label", "label": "escalate"})),
            Refusal::Fallback,
        ),
        (profile_with(|profile| profile["cohort"]["artifact"] = json!("log-triage-latest")), Refusal::Cohort),
        (profile_with(|profile| profile["cohort"]["runtime_crate"] = json!("other-runtime")), Refusal::Cohort),
        (profile_with(|profile| profile["input"]["features"][0]["min"] = json!(60)), Refusal::Input),
        (profile_with(|profile| profile["result"]["max_receipt_bytes"] = json!(1_000_000)), Refusal::Result),
        (profile_with(|profile| profile["binding"]["node_label"] = json!(" ")), Refusal::Selector),
    ];
    for (profile, expected) in cases {
        assert_eq!(refusal(&profile), expected);
    }
    let oversized = vec![b' '; 70_000];
    assert_eq!(lattice::workflow::decision::admission::admit(&oversized), Err(Refusal::Size));
}

#[test]
fn projection_refuses_missing_malformed_oversized_and_out_of_range_inputs() {
    let input = fixture::admit(&fixture::profile_json()).profile.input;
    let project = |value: Option<Value>| lattice::workflow::decision::binding::project(&input, value.as_ref());
    let fault = lattice::workflow::decision::binding::ProjectionFault::OutOfRange;
    let features =
        json!({"features": {"error_count": 0, "warn_count": 1, "max_latency_ms": 12_000, "critical_errors": 0}});

    assert_eq!(project(None), Err(lattice::workflow::decision::binding::ProjectionFault::Missing));
    assert_eq!(
        project(Some(Value::from_string("not json".to_owned()))),
        Err(lattice::workflow::decision::binding::ProjectionFault::Malformed)
    );
    assert_eq!(
        project(Some(Value::from_json(json!({"features": {}})))),
        Err(lattice::workflow::decision::binding::ProjectionFault::MissingFeature)
    );
    assert_eq!(project(Some(Value::from_json(features))), Err(fault));
    let oversized = format!("{{\"pad\":\"{}\"}}", "x".repeat(5_000));
    assert_eq!(
        project(Some(Value::from_string(oversized))),
        Err(lattice::workflow::decision::binding::ProjectionFault::Oversized)
    );
}

// r[verify lattice.typed_policy.route.scenario.accepted]
#[tokio::test]
async fn accepted_label_routes_only_through_the_declared_branch() {
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let run = run_case("payments-r1-02", Some(fixture::runtime(&fixture::profile_json(), adapter.clone()))).await;
    let record = single_decision(&run);

    assert_eq!(adapter.calls(), 1);
    assert_eq!(routed(&run).as_deref(), Some("page"));
    assert!(matches!(&record.outcome, Outcome::Accepted { label, .. } if label == "page"));
    assert_eq!(record.routed_by, Some(Source::Policy));
    assert_eq!(record.fallback_reason, None);
    assert_eq!(record.usage.reserved_operation_count, 16);
    assert!(record.usage.observed_inference_count.is_some_and(|count| count <= 16));
    assert!(record.receipt.is_some());
    for continuation in &run.nodes.continuations[1..] {
        assert!(matches!(status(&run.results, *continuation), NodeStatus::Skipped));
    }
    let admitted = fixture::admit(&fixture::profile_json());
    assert_eq!(record.profile, admitted.identity);
    assert_eq!(record.candidates, fixture::LABELS.map(str::to_owned).to_vec());
    assert!(record.projected_input.as_deref().is_some_and(|identity| identity.starts_with("b3:")));
}

// r[verify lattice.typed_policy.route.scenario.disabled]
#[tokio::test]
async fn disabled_or_unbound_profiles_leave_runs_byte_identical() {
    let disabled = profile_with(|profile| profile["enabled"] = json!(false));
    let other_workflow = profile_with(|profile| profile["binding"]["workflow"] = json!("another-workflow"));
    let baseline = run_case("search-r1-03", None).await;
    for profile in [disabled, other_workflow] {
        let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
        let run = run_case("search-r1-03", Some(fixture::runtime(&profile, adapter.clone()))).await;
        assert_eq!(adapter.calls(), 0);
        assert!(run.decisions.is_empty());
        assert_eq!(detail_bytes(&run), detail_bytes(&baseline));
        let hashes = |run: &Run| run.results.iter().map(|step| step.input_hash).collect::<Vec<_>>();
        assert_eq!(hashes(&run), hashes(&baseline));
    }
}

fn detail_bytes(run: &Run) -> Vec<u8> {
    let record = lattice::run::history::detail_record_from_results(
        lattice::engine::executor::RunId(0),
        &run.workflow,
        &run.results,
    );
    serde_json::to_vec(&record).expect("details encode")
}

// r[verify lattice.typed_policy.route.scenario.undeclared]
#[tokio::test]
async fn labels_outside_the_declared_branches_or_non_condition_nodes_fail_before_invocation() {
    let undeclared = profile_with(|profile| profile["labels"] = json!(["page", "ticket", "escalate"]));
    let not_condition = profile_with(|profile| profile["binding"]["node_label"] = json!(fixture::FEATURES_LABEL));
    for profile in [undeclared, not_condition] {
        let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
        let run = run_case("payments-r1-02", Some(fixture::runtime(&profile, adapter.clone()))).await;
        assert_eq!(adapter.calls(), 0);
        assert!(run.decisions.is_empty());
        assert_eq!(routed(&run), None);
        assert!(run.results.iter().any(|step| matches!(
            &step.status,
            NodeStatus::Failed { error } if error.starts_with("typed policy binding refused")
        )));
    }
}

async fn fallback_outcome(behavior: Behavior, profile: &serde_json::Value, case_id: &str) -> (Run, u64) {
    let adapter = Counting::new(behavior);
    let run = run_case(case_id, Some(fixture::runtime(profile, adapter.clone()))).await;
    let calls = adapter.calls();
    (run, calls)
}

// r[verify lattice.typed_policy.validation.scenario.invalid]
// r[verify lattice.typed_policy.fallback.scenario.timeout]
#[tokio::test]
async fn negative_responses_take_the_declared_bounded_fallback() {
    let base = fixture::profile_json();
    let candidates_minus_one = vec!["page".to_owned(), "ticket".to_owned()];
    let stale_artifact = {
        let mut artifact: serde_json::Value = serde_json::from_slice(&fixture::artifact()).expect("artifact parses");
        artifact["model"]["nodes"][0]["split"]["threshold"] = json!(18);
        serde_json::to_vec(&artifact).expect("artifact encodes")
    };
    let cases: Vec<(&str, Behavior, serde_json::Value, Outcome)> = vec![
        (
            "malformed",
            Behavior::Fixed(Response::Receipt(b"{not json".to_vec())),
            base.clone(),
            invalid("unverified-receipt"),
        ),
        (
            "oversized",
            Behavior::Delegate(fixture::leaf_adapter()),
            profile_with(|profile| profile["result"]["max_receipt_bytes"] = json!(TINY_RECEIPT_BYTES)),
            invalid("oversized-receipt"),
        ),
        (
            "missing candidate",
            Behavior::Eligible(candidates_minus_one),
            base.clone(),
            invalid("candidate-set"),
        ),
        (
            "undeclared label",
            Behavior::Eligible(fixture::LABELS.map(str::to_owned).to_vec()),
            profile_with(|profile| profile["labels"] = json!(["ticket", "ignore"])),
            invalid("candidate-set"),
        ),
        (
            "stale artifact",
            Behavior::Delegate(Arc::new(lattice::animus_agent::frozen::LeafAdapter::new(stale_artifact))),
            base.clone(),
            invalid("incompatible-artifact"),
        ),
        (
            "stale feature schema",
            Behavior::Delegate(fixture::leaf_adapter()),
            profile_with(|profile| profile["cohort"]["feature_schema"] = json!("lattice.log-triage-features.v0")),
            invalid("stale-binding"),
        ),
        (
            "stale model",
            Behavior::Delegate(fixture::leaf_adapter()),
            profile_with(|profile| {
                profile["cohort"]["model"] =
                    json!("b3:0000000000000000000000000000000000000000000000000000000000000000");
            }),
            invalid("stale-artifact"),
        ),
        (
            "stale runtime",
            Behavior::Delegate(fixture::leaf_adapter()),
            profile_with(|profile| profile["cohort"]["runtime_version"] = json!("0.0.9")),
            invalid("stale-runtime"),
        ),
        (
            "timeout",
            Behavior::Slow(Response::Unavailable),
            profile_with(|profile| profile["limits"]["deadline_millis"] = json!(SHORT_DEADLINE_MILLIS)),
            Outcome::TimedOut {
                reason: "deadline".to_owned(),
            },
        ),
        (
            "operation budget",
            Behavior::Delegate(fixture::leaf_adapter()),
            profile_with(|profile| profile["limits"]["max_operation_count"] = json!(2)),
            Outcome::TimedOut {
                reason: "operation-budget".to_owned(),
            },
        ),
        ("backend unavailable", Behavior::Fixed(Response::Unavailable), base.clone(), Outcome::Unavailable {
            reason: "backend-unavailable".to_owned(),
        }),
        (
            "artifact unavailable",
            Behavior::Delegate(Arc::new(lattice::animus_agent::frozen::LeafAdapter::without_artifact())),
            base.clone(),
            Outcome::Unavailable {
                reason: "artifact-unavailable".to_owned(),
            },
        ),
    ];
    for (name, behavior, profile, expected) in cases {
        let (run, calls) = fallback_outcome(behavior, &profile, "payments-r1-02").await;
        let record = single_decision(&run);
        assert_eq!(calls, 1, "{name}");
        assert_eq!(record.outcome, expected, "{name}");
        assert_eq!(record.routed_by, Some(Source::Fallback), "{name}");
        assert_eq!(record.fallback_reason.as_deref(), Some(expected.code().as_str()), "{name}");
        assert_eq!(routed(&run).as_deref(), Some("page"), "{name}: the rule hint routes ERROR to page");
    }
}

#[tokio::test]
async fn abstention_falls_back_and_label_or_terminal_fallbacks_are_honored() {
    let (abstained, _) =
        fallback_outcome(Behavior::Delegate(fixture::leaf_adapter()), &fixture::profile_json(), "payments-r1-03").await;
    assert_eq!(single_decision(&abstained).outcome, Outcome::Abstained);
    assert_eq!(routed(&abstained).as_deref(), Some("ticket"));

    let label = profile_with(|profile| profile["fallback"] = json!({"kind": "label", "label": "ignore"}));
    let (labelled, _) = fallback_outcome(Behavior::Fixed(Response::Unavailable), &label, "payments-r1-02").await;
    assert_eq!(routed(&labelled).as_deref(), Some("ignore"));

    let terminal = profile_with(|profile| profile["fallback"] = json!({"kind": "terminal"}));
    let (ended, _) = fallback_outcome(Behavior::Fixed(Response::Unavailable), &terminal, "payments-r1-02").await;
    let record = single_decision(&ended);
    assert_eq!(routed(&ended), None);
    assert_eq!(record.routed_label, None);
    assert_eq!(record.fallback_reason.as_deref(), Some("unavailable:backend-unavailable"));
    assert!(matches!(
        status(&ended.results, ended.nodes.route),
        NodeStatus::Failed { error } if error == "typed policy terminal outcome: unavailable:backend-unavailable"
    ));
}

#[tokio::test]
async fn cancellation_is_terminal_before_or_during_invocation() {
    let before = Arc::new(AtomicBool::new(false));
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let runtime = fixture::runtime(&fixture::profile_json(), adapter.clone());
    let (workflow, nodes) = fixture::routing_workflow(&case("payments-r1-02").lines);
    let route = workflow.graph.node(nodes.route).expect("route node").clone();
    let executor = Executor::new().with_typed_policy(runtime).with_cancellation(before.clone());
    before.store(true, Ordering::SeqCst);
    let step = executor.execute_node(&workflow, route.id, &condition_inputs(&route)).await;
    assert_eq!(adapter.calls(), 0);
    assert!(matches!(step.status, NodeStatus::Failed { ref error } if error.ends_with(": cancelled")));
    assert_eq!(executor.take_policy_decisions()[0].record.outcome, Outcome::Cancelled);

    let during = Arc::new(AtomicBool::new(false));
    let slow = Counting::new(Behavior::Slow(Response::Unavailable));
    let profile = profile_with(|profile| profile["limits"]["deadline_millis"] = json!(SLOW_ADAPTER_MILLIS));
    let executor = Executor::new()
        .with_typed_policy(fixture::runtime(&profile, slow.clone()))
        .with_cancellation(during.clone());
    let flag = during.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(CANCEL_AFTER_MILLIS));
        flag.store(true, Ordering::SeqCst);
    });
    let step = executor.execute_node(&workflow, route.id, &condition_inputs(&route)).await;
    canceller.join().expect("canceller joins");
    assert_eq!(slow.calls(), 1);
    assert!(matches!(step.status, NodeStatus::Failed { ref error } if error.ends_with(": cancelled")));
    assert_eq!(executor.take_policy_decisions()[0].record.outcome, Outcome::Cancelled);
}

fn condition_inputs(route: &lattice::engine::workflow::Node) -> HashMap<PortId, Value> {
    let features =
        r#"{"features":{"error_count":1,"warn_count":0,"max_latency_ms":0,"critical_errors":1},"rule":{"page":true}}"#;
    HashMap::from([(route.input_ports[0].id, Value::from_string(features.to_owned()))])
}

// r[verify lattice.typed_policy.record.scenario.persisted]
// r[verify lattice.typed_policy.replay.scenario.persisted]
// r[verify lattice.typed_policy.replay.scenario.changed]
#[tokio::test]
async fn headless_runs_persist_decisions_that_replay_without_adapter_work() {
    let (store, _directory) = common::temp_store();
    let (workflow, _nodes) = fixture::routing_workflow(&case("auth-r2-01").lines);
    store.save_workflow(&workflow).expect("workflow saves");
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let run_id = lattice::engine::executor::RunId(7);
    let authority = common::authority::TestRunContextProvider::new();
    let summary = lattice::headless::run_saved_workflow_with_typed_policy(
        lattice::headless::TypedPolicyRunRequest {
            store: &store,
            workflow_name: fixture::WORKFLOW_NAME,
            run_id,
            authority_provider: &authority,
        },
        fixture::runtime(&fixture::profile_json(), adapter.clone()),
    )
    .await
    .expect("typed policy run completes");
    assert_eq!(summary.failures, 0);
    assert_eq!(adapter.calls(), 1);

    let details = store.load_run_details(run_id).expect("details load").expect("details exist");
    let encoded = serde_json::to_value(&details).expect("details encode");
    let decided = details.nodes.iter().filter(|node| node.policy_decision.is_some()).collect::<Vec<_>>();
    assert_eq!(decided.len(), 1);
    assert_eq!(decided[0].node_label, fixture::ROUTE_LABEL);
    let decided_count = encoded["nodes"]
        .as_array()
        .expect("node array")
        .iter()
        .filter(|node| node.get("policy_decision").is_some())
        .count();
    assert_eq!(decided_count, 1, "nodes without a decision serialize no policy field");
    let record = decided[0].policy_decision.clone().expect("decision persists");
    assert_eq!(record.workflow_outcome, Some(lattice::workflow::decision::ledger::RunOutcome::Completed));

    let labels = fixture::LABELS.map(str::to_owned).to_vec();
    let admitted = fixture::admit(&fixture::profile_json());
    let replayed = lattice::workflow::decision::ledger::replay(&record, &admitted, &labels);
    assert_eq!(replayed, Ok("page".to_owned()));
    assert_eq!(adapter.calls(), 1, "replay starts no adapter work");

    let revised = fixture::admit(&profile_with(|profile| profile["version"] = json!(2)));
    assert_eq!(
        lattice::workflow::decision::ledger::replay(&record, &revised, &labels),
        Err(ReplayRefusal::ProfileChanged)
    );
    let reloaded = store.load_run_details(run_id).expect("details reload").expect("details exist");
    assert_eq!(reloaded, details, "a profile change never rewrites the prior decision");

    let mut tampered = record.clone();
    tampered.outcome = Outcome::Accepted {
        label: "ignore".to_owned(),
        score: 0,
    };
    tampered.routed_label = Some("ignore".to_owned());
    assert_eq!(
        lattice::workflow::decision::ledger::replay(&tampered, &admitted, &labels),
        Err(ReplayRefusal::Receipt)
    );
}

#[tokio::test]
async fn cached_decisions_reuse_outputs_and_profile_changes_miss_the_cache() {
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let first = run_case("search-r1-02", Some(fixture::runtime(&fixture::profile_json(), adapter.clone()))).await;
    assert_eq!(adapter.calls(), 1);
    let cache = first
        .results
        .iter()
        .filter_map(|step| match &step.status {
            NodeStatus::Completed { outputs, .. } => Some((step.input_hash, outputs.clone())),
            _ => None,
        })
        .collect::<Cache>();

    let cached = run_case_with(
        "search-r1-02",
        Some(fixture::runtime(&fixture::profile_json(), adapter.clone())),
        &cache,
        |executor| executor,
    )
    .await;
    assert_eq!(adapter.calls(), 1, "a cached decision starts no adapter work");
    assert!(matches!(status(&cached.results, cached.nodes.route), NodeStatus::Completed { cached: true, .. }));

    let revised = profile_with(|profile| profile["version"] = json!(2));
    let missed =
        run_case_with("search-r1-02", Some(fixture::runtime(&revised, adapter.clone())), &cache, |executor| executor)
            .await;
    assert_eq!(adapter.calls(), 2, "a changed profile identity misses the cache");
    assert!(matches!(status(&missed.results, missed.nodes.route), NodeStatus::Completed { cached: false, .. }));
}

#[tokio::test]
async fn ready_frontier_still_admits_the_routed_continuation() {
    let lanes = [
        lattice::engine::frontier::PURE_LANE,
        lattice::engine::frontier::PROCESS_LANE,
        lattice::engine::frontier::NETWORK_LANE,
        lattice::engine::frontier::FILESYSTEM_LANE,
        lattice::engine::frontier::IMAGE_LANE,
        lattice::engine::frontier::AGENT_LANE,
        lattice::engine::frontier::OPERATION_LANE,
        lattice::engine::frontier::SUBWORKFLOW_LANE,
        lattice::engine::frontier::APPROVAL_LANE,
    ];
    let policy = lattice_core::scheduling::CapacityPolicy {
        total_slots: PARALLEL_SLOTS,
        lane_slots: lanes.iter().map(|lane| ((*lane).to_owned(), PARALLEL_SLOTS)).collect::<BTreeMap<_, _>>(),
    };
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let (workflow, nodes) = fixture::routing_workflow(&case("search-r1-05").lines);
    let mut executor = Executor::new()
        .with_typed_policy(fixture::runtime(&fixture::profile_json(), adapter.clone()))
        .with_frontier_policy(policy)
        .expect("parallel frontier policy admits");
    let (_run_id, results) = executor.run(&workflow, &HashMap::new()).await;
    let order = executor.observed_completion_order().to_vec();
    let route_position = order.iter().position(|node| *node == nodes.route).expect("route completes");
    let ticket_position = order.iter().position(|node| *node == nodes.continuations[1]).expect("ticket completes");

    assert_eq!(fixture::routed_label(&results, &nodes), Ok(Some("ticket".to_owned())));
    assert!(route_position < ticket_position, "the continuation waits for frontier admission after the decision");
    assert_eq!(adapter.calls(), 1);
}

// The decision is a workflow branch only: product authority still governs the routed effect.
#[tokio::test]
async fn an_accepted_label_cannot_cross_a_denied_authority_boundary() {
    let mut workflow = Workflow::new(fixture::WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let route = workflow.add_condition_node(fixture::ROUTE_LABEL, lattice::control_flow::ConditionNodeConfig {
        branches: fixture::LABELS
            .iter()
            .map(|label| lattice::control_flow::ConditionBranch {
                label: (*label).to_owned(),
                predicate: lattice::control_flow::ConditionPredicate::Always,
            })
            .collect(),
    });
    let page = workflow.add_shell_node("page on-call", "cat > /dev/null; echo page");
    fixture::connect(&mut workflow, (route, "page"), (page, "stdin"));
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let mut executor = Executor::new().with_typed_policy(fixture::runtime(&fixture::profile_json(), adapter.clone()));
    let features = json!({"features": {"error_count": 1, "warn_count": 0, "max_latency_ms": 0, "critical_errors": 1}});
    let inputs = BTreeMap::from([("value".to_owned(), Value::from_json(features))]);
    let context = denied_context();
    let (_run_id, results) = executor
        .run_authorized_with_named_inputs(&workflow, &HashMap::new(), &inputs, &context)
        .await
        .expect("inputs bind");
    let decisions = executor.take_policy_decisions();

    assert!(matches!(&decisions[0].record.outcome, Outcome::Accepted { label, .. } if label == "page"));
    assert!(
        matches!(status(&results, page), NodeStatus::Failed { .. }),
        "denied authority still refuses the effect"
    );
}

// An upstream effect without a settled outcome never reaches the policy.
#[tokio::test]
async fn unsettled_upstream_effects_never_reach_the_policy() {
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let (workflow, nodes) = fixture::routing_workflow(&case("payments-r1-02").lines);
    let mut executor = Executor::new().with_typed_policy(fixture::runtime(&fixture::profile_json(), adapter.clone()));
    let context = denied_context();
    let (_run_id, results) = executor
        .run_authorized_with_named_inputs(&workflow, &HashMap::new(), &BTreeMap::new(), &context)
        .await
        .expect("no inputs to bind");

    assert!(matches!(status(&results, nodes.source), NodeStatus::Failed { .. }));
    assert!(!matches!(status(&results, nodes.route), NodeStatus::Completed { .. }));
    assert_eq!(adapter.calls(), 0);
    assert!(executor.take_policy_decisions().is_empty());
}

// An accepted workflow label is not a protocol label: exact-label admission still refuses it.
#[tokio::test]
async fn an_accepted_label_cannot_advance_an_exact_label_protocol_cursor() {
    let adapter = Counting::new(Behavior::Delegate(fixture::leaf_adapter()));
    let run = run_case("payments-r1-02", Some(fixture::runtime(&fixture::profile_json(), adapter))).await;
    assert!(matches!(&single_decision(&run).outcome, Outcome::Accepted { label, .. } if label == "page"));

    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::choice()));
    let placements = common::protocol::native_placements(&cohort);
    let plan = lattice::engine::protocol::plan_session(cohort, placements, common::protocol::identity("typed-policy"))
        .expect("session plans");
    let session = lattice::engine::protocol::create_session(plan).expect("session creates");
    let before = session.clone();
    let policy_label = lattice::engine::protocol::LabelId(u32::MAX);
    let refused = lattice::engine::protocol::plan_action(&session, lattice::engine::protocol::ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: policy_label,
        payload_reference: common::protocol::identity("policy-choice"),
    });
    assert_eq!(refused, Err(lattice::engine::protocol::Error::Issue(lattice::engine::protocol::Issue::WrongLabel)));
    assert_eq!(session, before);
}

fn denied_context() -> lattice::engine::effect::execution::RunContext {
    let shell = lattice::engine::effect::dispatch::Shell::new(
        Box::new(DeniedAuthority),
        Box::new(NoScope),
        Box::new(lattice::engine::effect::execution::UnavailableAttempts),
        Box::new(lattice::engine::effect::runtime::ConfiguredRoots::new(BTreeMap::new())),
    );
    lattice::engine::effect::execution::RunContext::new(
        lattice::engine::effect::authority::RunIdentity::new("test-run/typed-policy").expect("run identity"),
        lattice::engine::effect::authority::AuthorityIdentity::new("test-denied-authority")
            .expect("authority identity"),
        lattice::engine::effect::authority::CollaborationScopeBinding::Unscoped,
        Arc::new(shell),
    )
}

struct DeniedAuthority;

impl lattice::engine::effect::dispatch::AuthorityFactsProvider for DeniedAuthority {
    fn load(
        &self,
        _binding: &lattice::engine::effect::authority::IntentBinding,
    ) -> Result<lattice::engine::effect::authority::AuthorizationFacts, lattice::engine::effect::dispatch::Error> {
        Err(lattice::engine::effect::dispatch::Error::AuthorityDenied)
    }
}

struct NoScope;

impl lattice::engine::effect::dispatch::ScopeSnapshotProvider for NoScope {
    fn load_current(
        &self,
        _binding: &lattice::engine::effect::authority::IntentBinding,
    ) -> Result<Option<lattice::engine::effect::authority::ScopeSnapshotFacts>, lattice::engine::effect::dispatch::Error>
    {
        Ok(None)
    }
}
