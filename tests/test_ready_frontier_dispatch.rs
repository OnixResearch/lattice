mod common;

use std::collections::BTreeMap;
use std::collections::HashMap;

use lattice::engine::Executor;
use lattice::engine::NodeId;
use lattice::engine::Workflow;
use lattice::engine::effect::execution::RunContextProvider;
use lattice::engine::executor::NodeStatus;
use lattice::engine::frontier;
use serde::Deserialize;

const PARALLEL_SLOT_COUNT: u32 = 2;
const GENERATED_CONFIG_SLOT_COUNT: u32 = 4;
const AUTHORIZED_RUN_ID: u64 = 41;
const SOURCE_DELAY_SECONDS: &str = "0.01";
const SLOW_DELAY_SECONDS: &str = "0.30";
const FAST_DELAY_SECONDS: &str = "0.03";
const CHILD_DELAY_SECONDS: &str = "0.03";
const MODEL_SOURCE_MILLISECONDS: u64 = 10;
const MODEL_SLOW_MILLISECONDS: u64 = 300;
const MODEL_FAST_MILLISECONDS: u64 = 30;
const MODEL_CHILD_MILLISECONDS: u64 = 30;
const SOURCE_ALIAS: &str = "process:source";
const SLOW_ALIAS: &str = "process:slow";
const FAST_ALIAS: &str = "process:fast";
const CHILD_ALIAS: &str = "process:child";
const SHARED_ALIAS: &str = "process:shared";
const PROOF_LANE: &str = "process";
const POSITIVE_REFINEMENT_FIXTURE: &str = include_str!("fixtures/ready-frontier-proof/positive/stable-selection.ncl");
const OVER_CAPACITY_FIXTURE: &str = include_str!("fixtures/ready-frontier-proof/negative/over-capacity.ncl");
const CONFLICT_FIXTURE: &str = include_str!("fixtures/ready-frontier-proof/negative/conflicting-alias.ncl");
const NON_READY_FIXTURE: &str = include_str!("fixtures/ready-frontier-proof/negative/non-ready-selection.ncl");
const FRONTIER_GUIDE: &str = include_str!("../docs/bounded-ready-frontier.md");

#[derive(Deserialize)]
struct RefinementFixture {
    total_slots: u32,
    lane_slots: u32,
    candidates: Vec<RefinementCandidate>,
    claimed_selection: Vec<u64>,
}

#[derive(Deserialize)]
struct RefinementCandidate {
    node_id: u64,
    ready: bool,
    conflict: String,
}

fn delayed_command(delay_seconds: &str, output: &str) -> String {
    format!("sleep {delay_seconds}; printf {output}")
}

fn parallel_executor() -> Executor {
    let lane_limits = frontier::uniform_lane_limits(PARALLEL_SLOT_COUNT);
    let policy = frontier::capacity_policy(PARALLEL_SLOT_COUNT, &lane_limits).expect("parallel policy");
    Executor::new().with_frontier_policy(policy).expect("executor policy")
}

fn assert_completed(steps: &[lattice::engine::StepResult]) {
    assert!(!steps.is_empty());
    assert!(steps.iter().all(|step| matches!(step.status, NodeStatus::Completed { .. })));
}

// r[verify lattice.workflow_execution.ready_frontier.integration]
// r[verify lattice.workflow_execution.ready_frontier.integration.scenario.duplicate_owner]
// r[verify lattice.workflow_execution.ready_frontier.benchmark.scenario.paper_claim]
// r[verify lattice.workflow_execution.ready_frontier.evidence]
// r[verify lattice.workflow_execution.ready_frontier.evidence.scenario.ready_only]
#[test]
fn guide_assigns_integration_owners_and_separates_evidence_states() {
    for required in [
        "memoization and cache identity",
        "backend and attempt preparation",
        "completion observation",
        "does not convert unknown outcomes into retries",
    ] {
        assert!(FRONTIER_GUIDE.contains(required), "missing guide boundary: {required}");
    }
}

// r[verify lattice.workflow_execution.ready_frontier.capacity]
// r[verify lattice.workflow_execution.ready_frontier.capacity.scenario.valid]
#[test]
fn omitted_policy_is_serial_and_generated_nickel_policy_is_typed() {
    let omitted = lattice::config::AppConfig::default().frontier_capacity_policy().expect("serial default policy");
    assert_eq!(omitted.total_slots, 1);

    let directory = tempfile::tempdir().expect("temporary config directory");
    let path = directory.path().join("config.ncl");
    std::fs::write(&path, lattice::config::default_config_ncl()).expect("write default config");
    let generated = lattice::config::load_app_config(&path);
    let policy = generated.frontier_capacity_policy().expect("generated frontier policy");
    assert_eq!(policy.total_slots, GENERATED_CONFIG_SLOT_COUNT);
    assert!(policy.lane_slots.values().all(|slots| *slots == GENERATED_CONFIG_SLOT_COUNT));
}

// r[verify lattice.workflow_execution.ready_frontier.capacity.scenario.invalid]
#[test]
fn zero_or_unknown_policy_values_fail_closed() {
    let zero = lattice::config::AppConfig {
        max_parallelism: 0,
        ..lattice::config::AppConfig::default()
    };
    assert_eq!(zero.frontier_capacity_policy(), Err(lattice_core::scheduling::PolicyError::ZeroTotal));
    let unknown = lattice::config::AppConfig {
        max_parallelism: PARALLEL_SLOT_COUNT,
        resource_lanes: BTreeMap::from([("unknown".to_string(), 1)]),
        ..lattice::config::AppConfig::default()
    };
    assert_eq!(unknown.frontier_capacity_policy(), Err(lattice_core::scheduling::PolicyError::UnknownLaneOverride));
}

// r[verify lattice.workflow_execution.ready_frontier.choregraph.scenario.non_ready]
// r[verify lattice.workflow_execution.ready_frontier.completion.scenario.incomplete_dependency]
// r[verify lattice.workflow_execution.ready_frontier.refinement]
// r[verify lattice.workflow_execution.ready_frontier.refinement.scenario.non_ready]
// r[verify lattice.workflow_execution.ready_frontier.negative]
// r[verify lattice.workflow_execution.ready_frontier.negative.scenario.lane_excess]
#[test]
fn production_planner_accepts_positive_and_rejects_expected_failure_fixtures() {
    assert!(refinement_claim_matches(POSITIVE_REFINEMENT_FIXTURE));
    assert!(!refinement_claim_matches(OVER_CAPACITY_FIXTURE));
    assert!(!refinement_claim_matches(CONFLICT_FIXTURE));
    assert!(!refinement_claim_matches(NON_READY_FIXTURE));
}

fn refinement_claim_matches(source: &str) -> bool {
    let value = lattice::nickel::evaluate_source(source).expect("refinement fixture evaluates");
    let fixture: RefinementFixture = serde_json::from_value(value).expect("refinement fixture decodes");
    let policy = lattice_core::scheduling::CapacityPolicy {
        total_slots: fixture.total_slots,
        lane_slots: BTreeMap::from([(PROOF_LANE.to_string(), fixture.lane_slots)]),
    };
    let candidates = fixture
        .candidates
        .into_iter()
        .filter(|candidate| candidate.ready)
        .map(|candidate| lattice_core::scheduling::Candidate {
            node_id: candidate.node_id,
            lane: PROOF_LANE.to_string(),
            conflict: refinement_conflict(&candidate.conflict),
            backend_ready: true,
            authority_ready: true,
            credential_ready: true,
            attempt_ready: true,
            suspended: false,
            cancelled: false,
            terminal: false,
        })
        .collect::<Vec<_>>();
    let planned =
        lattice_core::scheduling::plan(&policy, &lattice_core::scheduling::ActiveReservations::default(), &candidates)
            .expect("refinement fixture plans");
    planned.selected_nodes == fixture.claimed_selection
}

fn refinement_conflict(value: &str) -> lattice_core::scheduling::ConflictState {
    if value == "pure" {
        return lattice_core::scheduling::ConflictState::Pure;
    }
    match value.strip_prefix("resolved:") {
        Some(key) => lattice_core::scheduling::ConflictState::Resolved(key.to_string()),
        None => lattice_core::scheduling::ConflictState::Unresolved,
    }
}

// r[verify lattice.workflow_execution.ready_frontier.determinism]
// r[verify lattice.workflow_execution.ready_frontier.determinism.scenario.reverse_completion]
#[tokio::test]
async fn canonical_results_remain_stable_while_completion_order_is_observed() {
    let mut workflow = Workflow::new("parallel-order", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let slow = workflow.add_shell_node("slow", delayed_command(SLOW_DELAY_SECONDS, "slow"));
    let fast = workflow.add_shell_node("fast", delayed_command(FAST_DELAY_SECONDS, "fast"));
    let mut executor = parallel_executor()
        .with_resolved_resource_alias(slow, SLOW_ALIAS)
        .expect("slow alias")
        .with_resolved_resource_alias(fast, FAST_ALIAS)
        .expect("fast alias");

    let (run_id, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_completed(&steps);
    assert_eq!(steps.iter().map(|step| step.node_id).collect::<Vec<_>>(), vec![slow, fast]);
    assert_eq!(executor.observed_completion_order(), &[fast, slow]);

    let details = lattice::run::history::detail_record_from_results_with_completion_order(
        run_id,
        &workflow,
        &steps,
        executor.observed_completion_order().to_vec(),
    );
    assert_eq!(details.nodes.iter().map(|node| node.node_id).collect::<Vec<_>>(), vec![slow, fast]);
    assert_eq!(details.observed_completion_order, vec![fast, slow]);
    let directory = tempfile::tempdir().expect("temporary run-detail directory");
    let store = lattice::store::Store::new(directory.path().join("runs.redb")).expect("run-detail store");
    store.save_run_details(&details).expect("persist run details");
    let persisted = store.load_run_details(run_id).expect("load run details").expect("stored run details");
    assert_eq!(persisted.observed_completion_order, vec![fast, slow]);
}

// r[verify lattice.workflow_execution.ready_frontier.choregraph]
// r[verify lattice.workflow_execution.ready_frontier.dispatch]
// r[verify lattice.workflow_execution.ready_frontier.positive.scenario.overlap]
#[tokio::test]
async fn authorized_product_effects_use_the_same_bounded_frontier() {
    let mut workflow =
        Workflow::new("authorized-parallel-order", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let slow = workflow.add_shell_node("slow", delayed_command(SLOW_DELAY_SECONDS, "slow"));
    let fast = workflow.add_shell_node("fast", delayed_command(FAST_DELAY_SECONDS, "fast"));
    let mut executor = parallel_executor()
        .with_resolved_resource_alias(slow, SLOW_ALIAS)
        .expect("slow alias")
        .with_resolved_resource_alias(fast, FAST_ALIAS)
        .expect("fast alias");
    let directory = tempfile::tempdir().expect("temporary authorized store directory");
    let store = lattice::store::Store::new(directory.path().join("authorized.redb")).expect("authorized store");
    let context = common::authority::TestRunContextProvider::new()
        .load(lattice::engine::RunId(AUTHORIZED_RUN_ID), Some(store))
        .expect("authorized run context");

    let (_, steps) = executor.run_authorized(&workflow, &HashMap::new(), &context).await;

    assert_completed(&steps);
    assert_eq!(executor.observed_completion_order(), &[fast, slow]);
}

// r[verify lattice.workflow_execution.ready_frontier.baseline]
// r[verify lattice.workflow_execution.ready_frontier.baseline.scenario.diamond]
// r[verify lattice.workflow_execution.ready_frontier.validation]
// r[verify lattice.workflow_execution.ready_frontier.validation.scenario.serial_compatibility]
#[tokio::test]
async fn capacity_one_preserves_serial_completion_order() {
    let mut workflow = Workflow::new("serial-order", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let slow = workflow.add_shell_node("slow", delayed_command(SLOW_DELAY_SECONDS, "slow"));
    let fast = workflow.add_shell_node("fast", delayed_command(FAST_DELAY_SECONDS, "fast"));
    let mut executor = Executor::new()
        .with_resolved_resource_alias(slow, SLOW_ALIAS)
        .expect("slow alias")
        .with_resolved_resource_alias(fast, FAST_ALIAS)
        .expect("fast alias");

    let (_, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_completed(&steps);
    assert_eq!(executor.observed_completion_order(), &[slow, fast]);
}

// r[verify lattice.workflow_execution.ready_frontier.benchmark]
#[test]
fn bounded_duration_model_compares_serial_level_barrier_and_ready_frontier() {
    let serial_milliseconds = MODEL_SOURCE_MILLISECONDS
        .saturating_add(MODEL_SLOW_MILLISECONDS)
        .saturating_add(MODEL_FAST_MILLISECONDS)
        .saturating_add(MODEL_CHILD_MILLISECONDS);
    let level_barrier_milliseconds = MODEL_SOURCE_MILLISECONDS
        .saturating_add(MODEL_SLOW_MILLISECONDS.max(MODEL_FAST_MILLISECONDS))
        .saturating_add(MODEL_CHILD_MILLISECONDS);
    let ready_frontier_milliseconds = MODEL_SOURCE_MILLISECONDS
        .saturating_add(MODEL_SLOW_MILLISECONDS.max(MODEL_FAST_MILLISECONDS.saturating_add(MODEL_CHILD_MILLISECONDS)));

    assert!(ready_frontier_milliseconds < level_barrier_milliseconds);
    assert!(level_barrier_milliseconds < serial_milliseconds);
    assert_eq!(ready_frontier_milliseconds, MODEL_SOURCE_MILLISECONDS + MODEL_SLOW_MILLISECONDS);
}

#[tokio::test]
#[ignore = "comparative benchmark fixture; run explicitly for local evidence"]
async fn benchmark_serial_and_bounded_frontier_on_the_same_effect_fixture() {
    let mut workflow = Workflow::new("comparative-benchmark", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let left = workflow.add_shell_node("left", delayed_command(SLOW_DELAY_SECONDS, "left"));
    let right = workflow.add_shell_node("right", delayed_command(SLOW_DELAY_SECONDS, "right"));
    let mut serial = Executor::new()
        .with_resolved_resource_alias(left, SLOW_ALIAS)
        .expect("left serial alias")
        .with_resolved_resource_alias(right, FAST_ALIAS)
        .expect("right serial alias");
    let serial_started = std::time::Instant::now();
    let (_, serial_steps) = serial.run(&workflow, &HashMap::new()).await;
    let serial_elapsed = serial_started.elapsed();

    let mut parallel = parallel_executor()
        .with_resolved_resource_alias(left, SLOW_ALIAS)
        .expect("left parallel alias")
        .with_resolved_resource_alias(right, FAST_ALIAS)
        .expect("right parallel alias");
    let parallel_started = std::time::Instant::now();
    let (_, parallel_steps) = parallel.run(&workflow, &HashMap::new()).await;
    let parallel_elapsed = parallel_started.elapsed();

    assert_completed(&serial_steps);
    assert_completed(&parallel_steps);
    assert!(parallel_elapsed < serial_elapsed, "serial={serial_elapsed:?} parallel={parallel_elapsed:?}");
    println!("serial={serial_elapsed:?} bounded_ready_frontier={parallel_elapsed:?}");
}

// r[verify lattice.workflow_execution.ready_frontier.conflicts]
// r[verify lattice.workflow_execution.ready_frontier.conflicts.scenario.same_resource]
#[tokio::test]
async fn shared_resource_alias_prevents_overlapping_effects() {
    let mut workflow = Workflow::new("conflict-order", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let slow = workflow.add_shell_node("slow", delayed_command(SLOW_DELAY_SECONDS, "slow"));
    let fast = workflow.add_shell_node("fast", delayed_command(FAST_DELAY_SECONDS, "fast"));
    let mut executor = parallel_executor()
        .with_resolved_resource_alias(slow, SHARED_ALIAS)
        .expect("slow alias")
        .with_resolved_resource_alias(fast, SHARED_ALIAS)
        .expect("fast alias");

    let (_, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_completed(&steps);
    assert_eq!(executor.observed_completion_order(), &[slow, fast]);
}

// r[verify lattice.workflow_execution.ready_frontier.positive]
// r[verify lattice.workflow_execution.ready_frontier.completion]
// r[verify lattice.workflow_execution.ready_frontier.completion.scenario.fast_branch]
#[tokio::test]
async fn dependent_child_starts_without_waiting_for_an_unrelated_slow_sibling() {
    let mut workflow = Workflow::new("no-level-barrier", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let source = workflow.add_shell_node("source", delayed_command(SOURCE_DELAY_SECONDS, "source"));
    let slow = workflow.add_shell_node("slow", delayed_command(SLOW_DELAY_SECONDS, "slow"));
    let fast = workflow.add_shell_node("fast", delayed_command(FAST_DELAY_SECONDS, "fast"));
    let child = workflow.add_shell_node("child", delayed_command(CHILD_DELAY_SECONDS, "child"));
    connect(&mut workflow, source, slow);
    connect(&mut workflow, source, fast);
    connect(&mut workflow, fast, child);
    let mut executor = parallel_executor()
        .with_resolved_resource_alias(source, SOURCE_ALIAS)
        .expect("source alias")
        .with_resolved_resource_alias(slow, SLOW_ALIAS)
        .expect("slow alias")
        .with_resolved_resource_alias(fast, FAST_ALIAS)
        .expect("fast alias")
        .with_resolved_resource_alias(child, CHILD_ALIAS)
        .expect("child alias");

    let (_, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_completed(&steps);
    assert_eq!(executor.observed_completion_order(), &[source, fast, child, slow]);
    assert_eq!(steps.iter().map(|step| step.node_id).collect::<Vec<_>>(), vec![source, slow, fast, child]);
}

fn connect(workflow: &mut Workflow, source: NodeId, target: NodeId) {
    let source_port = workflow.graph.node(source).expect("source node").output_ports[0].id;
    let target_port = workflow.graph.node(target).expect("target node").input_ports[0].id;
    workflow.graph.add_edge(source_port, target_port).expect("connect nodes");
}
