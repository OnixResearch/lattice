//! Pure-core fixtures for the opt-in interaction-net worker profile.
//!
//! The receipts under `tests/fixtures/pure-inet/` are actual output of the
//! pinned Noble executable for the committed program and input artifacts;
//! negative cases mutate those real receipts rather than inventing a format.

use lattice::inet as profile;
use lattice::pure_inet_profile as host;
use profile::Error;

const FIXTURES: &str = "tests/fixtures/pure-inet";

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("{FIXTURES}/{name}")).expect("fixture bytes")
}

fn lines(name: &str) -> Vec<serde_json::Value> {
    String::from_utf8(fixture(name))
        .expect("fixture text")
        .lines()
        .map(|line| serde_json::from_str(line).expect("fixture line"))
        .collect()
}

fn encode(lines: &[serde_json::Value]) -> Vec<u8> {
    lines.iter().map(|line| format!("{line}\n")).collect::<String>().into_bytes()
}

fn expectation() -> profile::report::Expectation<'static> {
    let input_bytes = fixture("program.ngraph").len() + fixture("inputs.vectors").len();
    profile::report::Expectation {
        source_revision: profile::report::PRODUCER_SOURCE_REVISION,
        input_bytes: u64::try_from(input_bytes).expect("input length"),
        ceilings: profile::route::CEILINGS,
    }
}

fn normalize(lines: &[serde_json::Value], exit: i32) -> profile::report::Normalized {
    profile::report::normalize(&encode(lines), Some(exit), &expectation())
}

/// The independent comparator: Rust wrapping arithmetic over the same values.
fn reference(first: i64, second: i64) -> i64 {
    first.wrapping_add(second).wrapping_mul(first.wrapping_sub(second)).wrapping_add(i64::MIN)
}

// r[verify lattice.pure_inet_profile.purity.scenario.exact_integer]
#[test]
fn committed_program_and_inputs_are_canonical_and_match_the_reference() {
    let program = profile::text::parse_program(&fixture("program.ngraph")).expect("program admits");
    assert_eq!(profile::text::render_program(&program).into_bytes(), fixture("program.ngraph"));
    let inputs = profile::text::parse_inputs(&fixture("inputs.vectors")).expect("inputs admit");
    assert_eq!(profile::text::render_inputs(inputs).into_bytes(), fixture("inputs.vectors"));
    assert_eq!((inputs.first, inputs.second), (i64::MAX, -9_007_199_254_740_993));
    assert_eq!(program.evaluate(inputs), Ok(reference(inputs.first, inputs.second)));
}

fn program_with(lines: &[&str]) -> Vec<u8> {
    let subject = "a".repeat(64);
    format!("ngraph-ir/0 ngraph-i64/0\nsubject authored {subject} closure -\n{}\n", lines.join("\n")).into_bytes()
}

const SLOTS: [&str; 2] = ["slot input i64", "slot input i64"];

fn body(nodes: &[&str]) -> Vec<u8> {
    let mut all = SLOTS.to_vec();
    all.extend_from_slice(nodes);
    program_with(&all)
}

// r[verify lattice.pure_inet_profile.purity.scenario.invalid_program]
// r[verify lattice.pure_inet_profile.purity.scenario.denied]
#[test]
fn programs_outside_the_closed_acyclic_vocabulary_are_refused_before_dispatch() {
    let valid = ["%0 = input 0", "%1 = input 1", "%2 = i64.add %0 %1", "out %2"];
    assert!(profile::text::parse_program(&body(&valid)).is_ok());
    let cases: [(&[&str], Error); 12] = [
        (
            &[
                "%0 = input 0",
                "%1 = input 1",
                "%2 = i64.add %0 %3",
                "%3 = i64.mul %0 %1",
                "out %2",
            ],
            Error::CyclicOperand,
        ),
        (&["%0 = input 0", "%1 = const i64 1", "out %1"], Error::WrongInputArity),
        (&["%0 = input 0", "%1 = input 1", "%2 = input 0", "out %2"], Error::WrongInputArity),
        (&["%0 = input 0", "%1 = input 1", "%2 = i64.eq %0 %1", "out %2"], Error::UnsupportedProgramLine),
        (&["%0 = input 0", "%1 = input 1", "%2 = f32.add %0 %1", "out %2"], Error::UnsupportedProgramLine),
        (&["%0 = input 0", "%1 = input 1", "%2 = input 2", "out %2"], Error::UnsupportedProgramLine),
        (
            &[
                "%0 = input 0",
                "%1 = input 1",
                "fact %0 integral input-validated",
                "out %1",
            ],
            Error::MalformedProgramText,
        ),
        (&["%0 = input 0", "%1 = input 1", "out %1", "out %0"], Error::MalformedProgramText),
        (
            &["%0 = input 0", "%1 = input 1", "%2 = const i64 +1", "out %2"],
            Error::Core(host::PureProfileError::NonCanonicalInteger),
        ),
        (
            &["%0 = input 0", "%1 = input 1", "%2 = const i64 -0", "out %2"],
            Error::Core(host::PureProfileError::NonCanonicalInteger),
        ),
        (
            &[
                "%0 = input 0",
                "%1 = input 1",
                "%2 = const i64 9223372036854775808",
                "out %2",
            ],
            Error::Core(host::PureProfileError::IntegerOutOfRange),
        ),
        (&["%0 = input 0", "%2 = input 1", "out %0"], Error::MalformedProgramText),
    ];
    for (nodes, expected) in cases {
        assert_eq!(profile::text::parse_program(&body(nodes)), Err(expected), "{nodes:?}");
    }
    let extra_slot = program_with(&[
        "slot input i64",
        "slot input i64",
        "slot input i64",
        "%0 = input 0",
        "out %0",
    ]);
    assert_eq!(profile::text::parse_program(&extra_slot), Err(Error::MalformedProgramText));
    let parameter = program_with(&["slot input i64", "slot param i64", "%0 = input 0", "out %0"]);
    assert_eq!(profile::text::parse_program(&parameter), Err(Error::UnsupportedProgramLine));
    for noise in [b"  ".as_slice(), b"\r", b" # comment"] {
        let mut bytes = body(&valid);
        bytes.truncate(bytes.len() - 1);
        bytes.extend_from_slice(noise);
        bytes.push(b'\n');
        assert!(profile::text::parse_program(&bytes).is_err());
    }
    let wrong_profile = String::from_utf8(body(&valid)).expect("utf8").replace("ngraph-i64/0", "ngraph-f32/0");
    assert_eq!(profile::text::parse_program(wrong_profile.as_bytes()), Err(Error::MalformedProgramText));
}

#[test]
fn inputs_require_two_canonical_decimal_strings() {
    let json = profile::text::inputs_to_json(host::PureInputs {
        first: i64::MIN,
        second: 9_007_199_254_740_993,
    });
    assert_eq!(json["inputs"], serde_json::json!(["-9223372036854775808", "9007199254740993"]));
    assert!(profile::text::inputs_from_json(&json).is_ok());
    let schema = profile::text::INPUTS_JSON_SCHEMA;
    assert_eq!(
        profile::text::inputs_from_json(&serde_json::json!({"schema": schema, "inputs": [1, "2"]})),
        Err(Error::Core(host::PureProfileError::NonStringInteger))
    );
    assert_eq!(
        profile::text::inputs_from_json(&serde_json::json!({"schema": schema, "inputs": ["1", "2", "3"]})),
        Err(Error::MalformedInputsText)
    );
    assert_eq!(
        profile::text::inputs_from_json(&serde_json::json!({"schema": schema, "inputs": ["01", "2"]})),
        Err(Error::Core(host::PureProfileError::NonCanonicalInteger))
    );
    assert!(
        profile::text::inputs_from_json(&serde_json::json!({"schema": schema, "inputs": ["1", "2"], "x": 1})).is_err()
    );
    for text in [
        "noble-ngraph-vectors/0\ntypes i64 i64\nv 1\n",
        "noble-ngraph-vectors/0\ntypes i64 i64\nv 1 2 3\n",
        "noble-ngraph-vectors/0\ntypes i64 bool\nv 1 true\n",
        "noble-ngraph-vectors/0\ntypes i64 i64\nv 1 2\nv 3 4\n",
        "noble-ngraph-vectors/0\ntypes i64 i64\nv +1 2\n",
        "noble-ngraph-vectors/0\ntypes i64 i64\nv 1 2",
    ] {
        assert!(profile::text::parse_inputs(text.as_bytes()).is_err(), "{text:?}");
    }
}

// r[verify lattice.pure_inet_profile.results.scenario.complete]
// r[verify lattice.pure_inet_profile.semantic_count]
#[test]
fn real_complete_receipt_is_identity_bound_and_census_checked() {
    let normalized = normalize(&lines("complete.jsonl"), 0);
    let completion = normalized.completion.expect("complete");
    assert_eq!(completion.result, reference(i64::MAX, -9_007_199_254_740_993));
    assert_eq!(normalized.outcome, host::PureEvaluationOutcome::Complete {
        result: completion.result
    });
    assert_eq!(completion.counters.interactions, 10);
    assert_eq!(completion.counters.interactions, completion.counters.expected_interactions);
    assert!(completion.counters.retained_bytes_high_water > completion.counters.retained_bytes);
    assert_eq!(completion.interactions.assurance, profile::count::Assurance::Observed);
    assert!(completion.interactions.is_bridge_bound());
    assert_eq!(completion.interactions.value, "10");
    assert_eq!(completion.interactions.unit, profile::count::UNIT);
}

fn mutated(mutate: impl Fn(&mut [serde_json::Value])) -> Vec<serde_json::Value> {
    let mut receipt = lines("complete.jsonl");
    mutate(&mut receipt);
    receipt
}

// r[verify lattice.pure_inet_profile.identity.scenario.crossed]
// r[verify lattice.pure_inet_profile.results.scenario.partial]
// r[verify lattice.pure_inet_profile.results.scenario.incomplete]
#[test]
fn crossed_malformed_and_incomplete_receipts_never_supply_a_result() {
    let cases: Vec<(Vec<serde_json::Value>, host::PureEvaluationOutcome, &str)> = vec![
        (
            mutated(|r| r[0]["producer"]["source_revision"] = "0".repeat(40).into()),
            host::PureEvaluationOutcome::Invalid,
            "crossed-producer",
        ),
        (
            mutated(|r| r[0]["producer"]["executable_digest"] = "a".repeat(64).into()),
            host::PureEvaluationOutcome::Invalid,
            "crossed-producer",
        ),
        (
            mutated(|r| r[0]["net_contract"]["rule_table"] = "noble-net-i64-rules/2".into()),
            host::PureEvaluationOutcome::Invalid,
            "crossed-contract",
        ),
        (
            mutated(|r| r[0]["limits"]["interaction_fuel"] = 1.into()),
            host::PureEvaluationOutcome::Invalid,
            "crossed-limits",
        ),
        (
            mutated(|r| r[0]["input_bytes"] = 368.into()),
            host::PureEvaluationOutcome::Invalid,
            "crossed-input-bytes",
        ),
        (
            mutated(|r| r[1]["outputs"] = serde_json::json!([9_205_357_638_345_293_824_u64])),
            host::PureEvaluationOutcome::Invalid,
            "non-string-output",
        ),
        (
            mutated(|r| r[1]["outputs"] = serde_json::json!(["i64:01"])),
            host::PureEvaluationOutcome::Invalid,
            "malformed-output",
        ),
        (
            mutated(|r| r[1]["outputs"] = serde_json::json!(["bool:true"])),
            host::PureEvaluationOutcome::Invalid,
            "malformed-output",
        ),
        (
            mutated(|r| r[1]["outputs"] = serde_json::json!(["i64:1", "i64:2"])),
            host::PureEvaluationOutcome::Invalid,
            "output-arity",
        ),
        (
            mutated(|r| r[1]["interactions"] = 11.into()),
            host::PureEvaluationOutcome::Invalid,
            "interaction-census",
        ),
        (mutated(|r| r[1]["vector"] = 1.into()), host::PureEvaluationOutcome::Invalid, "malformed-row"),
        (lines("complete.jsonl")[..1].to_vec(), host::PureEvaluationOutcome::Partial, "missing-row"),
    ];
    for (receipt, outcome, reason) in cases {
        let normalized = normalize(&receipt, 0);
        assert_eq!((normalized.outcome, normalized.reason), (outcome, reason));
        assert!(normalized.completion.is_none() && normalized.outcome.usable_result().is_none());
    }
    let mut extra = lines("complete.jsonl");
    extra.push(extra[1].clone());
    assert_eq!(normalize(&extra, 0).reason, "row-arity");
    let truncated = &fixture("complete.jsonl")[..100];
    assert_eq!(profile::report::normalize(truncated, Some(0), &expectation()).reason, "malformed-report");
    assert_eq!(
        profile::report::normalize(b"", Some(2), &expectation()).outcome,
        host::PureEvaluationOutcome::Failed
    );
    assert_eq!(normalize(&lines("complete.jsonl"), 1).reason, "malformed-report");
}

// r[verify lattice.pure_inet_profile.bounds.scenario.exhausted]
#[test]
fn real_producer_ceiling_refusals_are_exhausted_without_partial_results() {
    for (name, limit) in [
        ("exhausted-fuel.jsonl", "fuel"),
        ("exhausted-retained.jsonl", "retained-bytes"),
        ("exhausted-output.jsonl", "output-bytes"),
        ("exhausted-input.jsonl", "input-bytes"),
    ] {
        let normalized = normalize(&lines(name), 5);
        assert_eq!(normalized.outcome, host::PureEvaluationOutcome::Exhausted, "{name}");
        let exhaustion = normalized.exhaustion.expect("exhaustion");
        assert_eq!(exhaustion.limit, limit);
        assert!(exhaustion.observed > exhaustion.cap);
        assert!(normalized.completion.is_none());
        assert_eq!(normalize(&lines(name), 0).outcome, host::PureEvaluationOutcome::Invalid, "{name} at exit 0");
    }
    let rejected = normalize(&lines("producer-reject.jsonl"), 2);
    assert_eq!((rejected.outcome, rejected.reason), (host::PureEvaluationOutcome::Invalid, "producer-reject"));
}

// r[verify lattice.pure_inet_profile.semantic_count.scenario.observed]
// r[verify lattice.pure_inet_profile.semantic_count.scenario.units]
#[test]
fn receipts_without_theorems_stay_observed_and_crossed_bridges_fail_admission() {
    let without = normalize(&mutated(|r| r[0]["semantic_count"]["theorems"] = serde_json::json!([])), 0);
    let interactions = without.completion.expect("still a complete result").interactions;
    assert_eq!(interactions.assurance, profile::count::Assurance::Observed);
    assert!(!interactions.is_bridge_bound());
    let absent = normalize(
        &mutated(|r| {
            r[0].as_object_mut().expect("header").remove("semantic_count");
        }),
        0,
    );
    assert!(!absent.completion.expect("complete").interactions.is_bridge_bound());
    for mutate in [
        (|r: &mut [serde_json::Value]| r[0]["semantic_count"]["unit"] = "graph-operation".into())
            as fn(&mut [serde_json::Value]),
        |r| r[0]["semantic_count"]["census"] = "ops".into(),
        |r| r[0]["semantic_count"]["proof_source"]["module_blake3"] = "0".repeat(64).into(),
        |r| r[0]["semantic_count"]["theorems"] = serde_json::json!(["NobleFoundation.NetI64.i64_normal_form_unique"]),
        |r| r[0]["semantic_count"]["model_boundary"] = "refinement".into(),
    ] {
        let normalized = normalize(&mutated(mutate), 0);
        assert_eq!((normalized.outcome, normalized.reason), (host::PureEvaluationOutcome::Invalid, "count-binding"));
    }
}

fn evaluation(attempt: &str, receipt: &[serde_json::Value]) -> profile::count::Evaluation {
    let completion = normalize(receipt, 0).completion.expect("complete");
    profile::count::Evaluation {
        computation: "b3:computation".to_string(),
        invocation: format!("b3:{attempt}"),
        attempt: attempt.to_string(),
        result: completion.result,
        interactions: completion.interactions,
    }
}

// r[verify lattice.pure_inet_profile.recompute_integrity.scenario.match]
// r[verify lattice.pure_inet_profile.recompute_integrity.scenario.mismatch]
#[test]
fn recomputation_checks_output_and_count_and_names_integrity_failures() {
    let first = evaluation("attempt-1", &lines("complete.jsonl"));
    let second = evaluation("attempt-2", &lines("complete.jsonl"));
    let Ok(profile::count::Verdict::Consistent(checked)) = profile::count::compare(&first, &second, true) else {
        panic!("matching fresh evaluations are consistent");
    };
    assert_eq!(checked.assurance, profile::count::Assurance::Checked);
    let output = evaluation("attempt-2", &mutated(|r| r[1]["outputs"] = serde_json::json!(["i64:7"])));
    assert_eq!(profile::count::compare(&first, &output, true), Ok(profile::count::Verdict::OutputIntegrityFailure));
    let count = evaluation(
        "attempt-2",
        &mutated(|r| {
            r[1]["interactions"] = 12.into();
            r[1]["expected_interactions"] = 12.into();
        }),
    );
    assert_eq!(
        profile::count::compare(&first, &count, true),
        Ok(profile::count::Verdict::InteractionCountIntegrityFailure)
    );
    assert_eq!(profile::count::compare(&first, &first, true), Err(Error::RecomputationRefused));
    let mut other = second.clone();
    other.computation = "b3:other".to_string();
    assert_eq!(profile::count::compare(&first, &other, true), Err(Error::RecomputationRefused));
    let observed = evaluation("attempt-2", &mutated(|r| r[0]["semantic_count"]["theorems"] = serde_json::json!([])));
    assert_eq!(profile::count::compare(&first, &observed, true), Err(Error::CrossedCountBinding));
    assert_eq!(
        profile::count::compare(&first, &observed, false),
        Ok(profile::count::Verdict::ConsistentObservedCount)
    );
    let mut crossed = second;
    crossed.interactions.unit = "graph-operation".to_string();
    assert_eq!(profile::count::compare(&first, &crossed, true), Err(Error::CrossedCountBinding));
}

// r[verify lattice.pure_inet_profile.semantic_limit.scenario.contradiction]
// r[verify lattice.pure_inet_profile.semantic_limit.scenario.independent]
#[test]
fn checked_count_fixes_the_exact_semantic_limit_without_waiving_other_bounds() {
    let first = evaluation("attempt-1", &lines("complete.jsonl"));
    let Ok(profile::count::Verdict::Consistent(checked)) =
        profile::count::compare(&first, &evaluation("attempt-2", &lines("complete.jsonl")), true)
    else {
        panic!("consistent baseline");
    };
    let baseline = profile::count::Evaluation {
        interactions: checked,
        ..first.clone()
    };
    let rerun = |name: &str, exit: i32, fuel: u64| (normalize(&lines(name), exit), fuel);
    for ((outcome, fuel), expected) in [
        (rerun("complete.jsonl", 0, 10), profile::count::LimitVerdict::CompletedAtCount),
        (rerun("exhausted-fuel.jsonl", 5, 9), profile::count::LimitVerdict::ExpectedExhaustion),
        (rerun("exhausted-retained.jsonl", 5, 10), profile::count::LimitVerdict::IndependentBound),
        (rerun("exhausted-fuel.jsonl", 5, 10), profile::count::LimitVerdict::InteractionCountIntegrityFailure),
        (rerun("producer-reject.jsonl", 2, 10), profile::count::LimitVerdict::Retained),
    ] {
        let verdict = profile::count::limit_consequence(&baseline, &profile::count::Rerun {
            computation: &baseline.computation,
            fuel_interactions: fuel,
            outcome: &outcome,
        });
        assert_eq!(verdict, Ok(expected));
    }
    let complete = normalize(&lines("complete.jsonl"), 0);
    let observed_only = profile::count::limit_consequence(&first, &profile::count::Rerun {
        computation: &first.computation,
        fuel_interactions: 10,
        outcome: &complete,
    });
    assert_eq!(observed_only, Err(Error::CrossedCountBinding));
}

// r[verify lattice.pure_inet_profile.recovery.scenario.recompute]
// r[verify lattice.pure_inet_profile.ownership.scenario.stale]
#[test]
fn recomputation_requires_reconciliation_lineage_and_budget() {
    let budget = profile::lineage::Budget {
        maximum_attempts: 2,
        consumed_attempts: 1,
    };
    let started = profile::lineage::Predecessor {
        invocation: "b3:first",
        attempt: "attempt-1",
        state: lattice::operation::RunState::Started,
        is_reconciled: false,
    };
    assert_eq!(
        profile::lineage::admit_recomputation(&started, "attempt-2", budget),
        Err(Error::RecomputationRefused)
    );
    let reconciled = profile::lineage::Predecessor {
        is_reconciled: true,
        ..started
    };
    let admitted = profile::lineage::admit_recomputation(&reconciled, "attempt-2", budget).expect("admitted");
    assert_eq!((admitted.predecessor_invocation.as_str(), admitted.remaining_attempts), ("b3:first", 0));
    assert_eq!(
        profile::lineage::admit_recomputation(&reconciled, "attempt-1", budget),
        Err(Error::RecomputationRefused)
    );
    let spent = profile::lineage::Budget {
        maximum_attempts: 2,
        consumed_attempts: 2,
    };
    assert_eq!(
        profile::lineage::admit_recomputation(&reconciled, "attempt-3", spent),
        Err(Error::RecomputationRefused)
    );
    let unknown = profile::lineage::Predecessor {
        state: lattice::operation::RunState::Unknown,
        ..started
    };
    assert!(profile::lineage::admit_recomputation(&unknown, "attempt-2", budget).is_ok());

    let current = "c".repeat(64);
    let document = serde_json::json!({
        "evaluation": "complete",
        "authoritative": true,
        "result": "-5",
        "binding": {"consumer_binding": current},
    });
    assert_eq!(profile::lineage::admit_for_consumer(&document, &current), Ok(-5));
    assert_eq!(
        profile::lineage::admit_for_consumer(&document, &"d".repeat(64)),
        Err(Error::SupersededConsumerBinding)
    );
    let mut diagnostic = document;
    diagnostic["authoritative"] = false.into();
    assert_eq!(profile::lineage::admit_for_consumer(&diagnostic, &current), Err(Error::RecomputationRefused));
}
