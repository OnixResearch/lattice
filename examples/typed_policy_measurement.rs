//! Pre-declared comparison for the optional typed policy operation (Lattice #23).
//!
//! Runs every `tests/fixtures/typed-policy/cases.json` case through the extended log-triage
//! workflow under the deterministic rule (profile disabled) and the fast frozen policy (profile
//! enabled), records the present reasoning-model arm as unavailable when no model endpoint is
//! configured, and prints one JSON report. Thresholds are fixed in
//! `.cairn/changes/add-typed-policy-operation/design.md`; this harness only evaluates them.
//!
//! ```sh
//! cargo run --locked --example typed_policy_measurement > measurement.json
//! ```

#[path = "../tests/support/typed_policy.rs"]
mod fixture;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Instant;

use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::workflow::decision::validation::Outcome;
use serde_json::json;

const REPETITIONS: usize = 5;
const STAGE_ITERATIONS: usize = 50;
const T2_MARGIN: f64 = 0.10;
const T3_COVERAGE: f64 = 0.80;
const T4_INVALID: f64 = 0.05;
const T4_FALLBACK: f64 = 0.20;
const T5_STAGE_MICROS: f64 = 1_000.0;
const T5_LATENCY_RATIO: f64 = 1.10;

#[derive(Debug, Clone, Default)]
struct CaseResult {
    routed: Option<String>,
    success: bool,
    more_than_one: bool,
    outside_declared: bool,
    outcome: Option<String>,
    is_accepted: bool,
    is_invalid: bool,
    is_fallback: bool,
    is_abstained: bool,
    inference_ops: u64,
    normalization_ops: u64,
    latencies_micros: Vec<u128>,
    feature_json: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cases = fixture::cases();
    let profile = fixture::profile_json();
    let admitted = fixture::admit(&profile);
    let mut rows = Vec::new();
    let mut by_arm: BTreeMap<&str, Vec<(fixture::Case, CaseResult)>> = BTreeMap::new();
    for case in &cases.cases {
        for arm in ["deterministic", "fast_policy"] {
            let result = run_arm(arm, case, &profile).await;
            rows.push(json!({
                "case": case.id, "family": case.family, "expected": case.expected, "arm": arm,
                "routed": result.routed, "success": result.success, "outcome": result.outcome,
                "inference_operations": result.inference_ops, "normalization_operations": result.normalization_ops,
                "median_workflow_micros": median(&result.latencies_micros),
            }));
            by_arm.entry(arm).or_default().push((case.clone(), result));
        }
    }
    let held_out = |family: &str| cases.held_out_families.iter().any(|held| held == family);
    let summary = |arm: &str, is_held_out: bool| {
        let selected = by_arm[arm].iter().filter(|(case, _)| held_out(&case.family) == is_held_out).collect::<Vec<_>>();
        summarize(&selected)
    };
    let stage = stage_overhead(&by_arm["fast_policy"], &admitted);
    let deterministic = summary("deterministic", true);
    let fast = summary("fast_policy", true);
    let reasoning = reasoning_arm();
    let thresholds = evaluate(&deterministic, &fast, &stage, &by_arm["fast_policy"]);
    let report = json!({
        "schema": "lattice.typed-policy-measurement.v1",
        "task": "log-triage incident routing",
        "profile_identity": admitted.identity,
        "artifact": admitted.profile.cohort.artifact,
        "model": admitted.profile.cohort.model,
        "runtime": format!("{} {}", admitted.profile.cohort.runtime_crate, admitted.profile.cohort.runtime_version),
        "repetitions": REPETITIONS,
        "arms": {
            "deterministic": {"held_out": deterministic, "authoring": summary("deterministic", false)},
            "reasoning_model": reasoning,
            "fast_policy": {"held_out": fast, "authoring": summary("fast_policy", false), "policy_stage": stage},
        },
        "thresholds": thresholds.0,
        "verdict": thresholds.1,
        "non_claims": [
            "the fixture artifact is hand-authored, not trained or distilled",
            "the rubric and the fixture tree were written by the same change; agreement measures plumbing, not generalization",
            "latency is wall-clock on a shared, heavily loaded host and is not a performance guarantee",
            "no improvement claim is made from a fixture artifact",
        ],
        "cases": rows,
    });
    println!("{}", serde_json::to_string_pretty(&report).expect("report encodes"));
}

async fn run_arm(arm: &str, case: &fixture::Case, profile: &serde_json::Value) -> CaseResult {
    let mut result = CaseResult::default();
    for repetition in 0..REPETITIONS {
        let (workflow, nodes) = fixture::routing_workflow(&case.lines);
        let mut executor = if arm == "fast_policy" {
            Executor::new().with_typed_policy(fixture::runtime(profile, fixture::leaf_adapter()))
        } else {
            Executor::new()
        };
        let started = Instant::now();
        let (_run_id, results) = executor.run(&workflow, &HashMap::new()).await;
        result.latencies_micros.push(started.elapsed().as_micros());
        if repetition > 0 {
            continue;
        }
        let routed = fixture::routed_label(&results, &nodes);
        result.more_than_one = routed.is_err();
        result.routed = routed.unwrap_or(None);
        result.outside_declared = result.routed.as_deref().is_some_and(|label| !fixture::LABELS.contains(&label));
        let has_failure = results.iter().any(|step| matches!(step.status, NodeStatus::Failed { .. }));
        result.success = !has_failure && result.routed.as_deref() == Some(case.expected.as_str());
        result.feature_json =
            results.iter().find(|step| step.node_id == nodes.features).and_then(|step| match &step.status {
                NodeStatus::Completed { outputs, .. } => outputs.values().find_map(|value| match &value.data {
                    lattice::engine::ValueData::String(text) if text.contains("\"features\"") => Some(text.clone()),
                    _ => None,
                }),
                _ => None,
            });
        if let Some(decision) = executor.take_policy_decisions().into_iter().next() {
            let record = decision.record;
            result.outcome = Some(record.outcome.code());
            result.is_accepted = matches!(record.outcome, Outcome::Accepted { .. });
            result.is_invalid = matches!(record.outcome, Outcome::Invalid { .. });
            result.is_abstained = matches!(record.outcome, Outcome::Abstained);
            result.is_fallback = record.routed_by == Some(lattice::workflow::decision::validation::Source::Fallback);
            result.inference_ops = record.usage.observed_inference_count.unwrap_or(0);
            result.normalization_ops = record.usage.observed_normalization_count.unwrap_or(0);
        }
    }
    result
}

fn summarize(selected: &[&(fixture::Case, CaseResult)]) -> serde_json::Value {
    let count = selected.len();
    let rate = |predicate: &dyn Fn(&CaseResult) -> bool| {
        ratio(selected.iter().filter(|(_, result)| predicate(result)).count(), count)
    };
    let medians = selected.iter().map(|(_, result)| median(&result.latencies_micros)).collect::<Vec<_>>();
    json!({
        "cases": count,
        "whole_workflow_success_rate": rate(&|result| result.success),
        "successes": selected.iter().filter(|(_, result)| result.success).count(),
        "coverage": rate(&|result| result.is_accepted),
        "invalid_output_rate": rate(&|result| result.is_invalid),
        "fallback_rate": rate(&|result| result.is_fallback),
        "abstentions": selected.iter().filter(|(_, result)| result.is_abstained).count(),
        "routed_outside_declared": selected.iter().filter(|(_, result)| result.outside_declared).count(),
        "runs_with_more_than_one_continuation": selected.iter().filter(|(_, result)| result.more_than_one).count(),
        "median_of_case_median_workflow_micros": median(&medians),
        "p95_of_case_median_workflow_micros": percentile(&medians, 95),
        "inference_operations_total": selected.iter().map(|(_, result)| result.inference_ops).sum::<u64>(),
        "normalization_operations_total": selected.iter().map(|(_, result)| result.normalization_ops).sum::<u64>(),
        "failures": selected.iter().filter(|(_, result)| !result.success).map(|(case, result)| json!({
            "case": case.id, "expected": case.expected, "routed": result.routed, "outcome": result.outcome,
        })).collect::<Vec<_>>(),
    })
}

/// Projection + adapter invocation + validation, outside the workflow, over the same inputs.
fn stage_overhead(
    results: &[(fixture::Case, CaseResult)],
    admitted: &lattice::workflow::decision::admission::Admitted,
) -> serde_json::Value {
    let adapter = fixture::leaf_adapter();
    let mut samples = Vec::new();
    for (_, result) in results {
        let Some(features) = result.feature_json.as_ref() else {
            continue;
        };
        let value = lattice::engine::value::Value::from_json(json!(features));
        for _ in 0..STAGE_ITERATIONS {
            let started = Instant::now();
            if let Ok(projected) = lattice::workflow::decision::binding::project(&admitted.profile.input, Some(&value))
            {
                let invocation = lattice::workflow::decision::binding::invocation(
                    admitted,
                    lattice::engine::workflow::NodeId(0),
                    projected,
                );
                let response = adapter.invoke(&invocation);
                let _checked =
                    lattice::workflow::decision::validation::validate(&admitted.profile, &invocation, &response);
            }
            samples.push(started.elapsed().as_micros());
        }
    }
    json!({
        "samples": samples.len(),
        "median_micros": median(&samples),
        "p95_micros": percentile(&samples, 95),
        "includes": ["projection", "artifact admission", "inference", "receipt encoding", "Lattice validation"],
    })
}

fn reasoning_arm() -> serde_json::Value {
    json!({
        "status": "not_applicable",
        "reason": "examples/workflows/log-triage.ncl contains only shell nodes (log source, grep filter, awk render); no Agent or reasoning-model node makes a routing decision in that workflow today, and the routing step exists only as the deterministic Condition this harness adds",
    })
}

fn evaluate(
    deterministic: &serde_json::Value,
    fast: &serde_json::Value,
    stage: &serde_json::Value,
    all_fast: &[(fixture::Case, CaseResult)],
) -> (serde_json::Value, &'static str) {
    let number = |value: &serde_json::Value, key: &str| value[key].as_f64().unwrap_or(f64::NAN);
    let t1 = all_fast.iter().all(|(_, result)| !result.outside_declared && !result.more_than_one);
    let t2 =
        number(fast, "whole_workflow_success_rate") >= number(deterministic, "whole_workflow_success_rate") + T2_MARGIN;
    let t3 = number(fast, "coverage") >= T3_COVERAGE;
    let t4 = number(fast, "invalid_output_rate") <= T4_INVALID && number(fast, "fallback_rate") <= T4_FALLBACK;
    let latency_ratio = number(fast, "median_of_case_median_workflow_micros")
        / number(deterministic, "median_of_case_median_workflow_micros");
    let t5 = number(stage, "median_micros") <= T5_STAGE_MICROS && latency_ratio <= T5_LATENCY_RATIO;
    let verdict = match (t1, t2 && t3 && t4 && t5) {
        (false, _) => "no-go",
        (true, true) => "plumbing-qualified-comparison-incomplete",
        (true, false) => "no-improvement-established",
    };
    (
        json!({
            "T1_safety": {"pass": t1, "note": "replay adapter calls are checked by tests/test_typed_policy.rs"},
            "T2_benefit": {"pass": t2, "required_margin": T2_MARGIN},
            "T3_coverage": {"pass": t3, "minimum": T3_COVERAGE},
            "T4_validity": {"pass": t4, "max_invalid": T4_INVALID, "max_fallback": T4_FALLBACK},
            "T5_cost": {"pass": t5, "max_stage_micros": T5_STAGE_MICROS, "max_latency_ratio": T5_LATENCY_RATIO, "observed_latency_ratio": latency_ratio},
        }),
        verdict,
    )
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return f64::NAN;
    }
    numerator as f64 / denominator as f64
}

fn median(samples: &[u128]) -> u128 {
    percentile(samples, 50)
}

fn percentile(samples: &[u128], percent: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    match sorted.len() {
        0 => 0,
        length => sorted[(length - 1) * percent / 100],
    }
}
