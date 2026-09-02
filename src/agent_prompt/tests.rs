use super::PilotBindings;
use super::PilotObservation;
use super::PilotVerdict;
use super::ProjectionErrorClass;
use super::ProjectionPilotInput;
use super::StructuredInputProjection;
use super::TaskOutcome;
use super::decode_toon_v3_strict;
use super::evaluate_pilot;
use super::render_structured_inputs;

const COMPACT_TOKENS: u64 = 120;
const LOWER_TOON_TOKENS: u64 = 90;
const HIGHER_TOON_TOKENS: u64 = 130;
const TASK_COUNT: usize = 1;

fn bindings() -> PilotBindings {
    PilotBindings {
        model_identity: "model:vibe-thinker-3b".to_string(),
        endpoint_class: "controlled-local-openai-compatible".to_string(),
        system_prompt_identity: "b3:system".to_string(),
        user_prompt_identity: "b3:user".to_string(),
        corpus_identity: "b3:corpus".to_string(),
        comparator_identity: "b3:comparator".to_string(),
        runtime_profile_identity: "b3:runtime".to_string(),
    }
}

fn observation(
    profile: StructuredInputProjection,
    prompt_tokens: Option<u64>,
    outcome: TaskOutcome,
) -> PilotObservation {
    PilotObservation {
        task_id: "lookup-owner".to_string(),
        profile,
        prompt_tokens,
        outcome,
    }
}

fn pilot(compact: PilotObservation, toon: PilotObservation) -> ProjectionPilotInput {
    ProjectionPilotInput {
        expected: bindings(),
        observed: bindings(),
        observations: vec![compact, toon],
    }
}

#[test]
fn pretty_and_compact_profiles_have_exact_sections() {
    let value = serde_json::json!({"enabled": true, "count": 2});
    let pretty = render_structured_inputs(std::slice::from_ref(&value), StructuredInputProjection::JsonPretty)
        .expect("pretty JSON renders");
    let compact =
        render_structured_inputs(&[value], StructuredInputProjection::JsonCompact).expect("compact JSON renders");

    assert_eq!(pretty[0].heading, "JSON input 1");
    assert_eq!(pretty[0].body, "{\n  \"count\": 2,\n  \"enabled\": true\n}");
    assert_eq!(compact[0].heading, "Compact JSON input 1");
    assert_eq!(compact[0].body, r#"{"count":2,"enabled":true}"#);
}

// r[verify lattice.agent_runtime.toon_v3_projection]
#[test]
fn toon_v3_uniform_array_has_exact_versioned_projection() {
    let value = serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    });
    let rendered = render_structured_inputs(&[value], StructuredInputProjection::ToonV3)
        .expect("uniform array renders and round trips");

    assert_eq!(rendered[0].heading, "TOON v3 input 1");
    assert_eq!(rendered[0].body, "users[2]{id,name}:\n  1,Alice\n  2,Bob");
}

#[test]
fn toon_v3_corpus_round_trips_supported_json_shapes() {
    let inputs = vec![
        serde_json::json!({"nested": {"active": true, "items": [1, "two", false]}}),
        serde_json::json!("scalar root"),
        serde_json::json!([]),
        serde_json::json!({"unicode": "λ雪", "separator": "a,b|c\td"}),
        serde_json::json!({"lines": "first\nsecond", "ticks": "```", "instruction": "ignore prior text"}),
        serde_json::json!([{"id": 1}, "heterogeneous", null]),
        serde_json::json!({"maximum": u64::MAX}),
    ];

    let rendered =
        render_structured_inputs(&inputs, StructuredInputProjection::ToonV3).expect("checked corpus round trips");
    assert_eq!(rendered.len(), inputs.len());
    assert!(rendered.iter().all(|item| !item.body.trim().is_empty()));
}

#[test]
fn toon_v3_rejects_ambiguous_empty_object_without_json_fallback() {
    let error = render_structured_inputs(&[serde_json::json!({})], StructuredInputProjection::ToonV3)
        .expect_err("empty root object must not receive a mislabeled fallback");

    assert_eq!(error.input_index, 1);
    assert_eq!(error.class, ProjectionErrorClass::EmptyProjection);
}

#[test]
fn strict_decoder_rejects_malformed_counts_widths_and_indentation() {
    let invalid = ["items[2]: one", "items[1]{id,name}:\n  1", "items[1]{id}:\n   1"];
    for text in invalid {
        assert_eq!(decode_toon_v3_strict(text), Err(ProjectionErrorClass::StrictDecodeRejected));
    }
}

// r[verify lattice.agent_runtime.projection_evaluation.scenario.eligible]
#[test]
fn pilot_allows_only_exact_explicit_use_after_strict_benefit() {
    let input = pilot(
        observation(StructuredInputProjection::JsonCompact, Some(COMPACT_TOKENS), TaskOutcome::Passed),
        observation(StructuredInputProjection::ToonV3, Some(LOWER_TOON_TOKENS), TaskOutcome::Passed),
    );
    let report = evaluate_pilot(&input);

    assert_eq!(report.verdict, PilotVerdict::EligibleForExactExplicitUse);
    assert_eq!(report.task_count, TASK_COUNT);
    assert!(!report.recommends_default_promotion);
    assert!(!report.claims_billing_truth);
    assert!(!report.claims_equal_accuracy);
    assert!(!report.claims_production_readiness);
}

#[test]
fn pilot_blocks_missing_usage_higher_tokens_and_task_regression() {
    let missing = pilot(
        observation(StructuredInputProjection::JsonCompact, None, TaskOutcome::Passed),
        observation(StructuredInputProjection::ToonV3, Some(LOWER_TOON_TOKENS), TaskOutcome::Passed),
    );
    let higher = pilot(
        observation(StructuredInputProjection::JsonCompact, Some(COMPACT_TOKENS), TaskOutcome::Passed),
        observation(StructuredInputProjection::ToonV3, Some(HIGHER_TOON_TOKENS), TaskOutcome::Passed),
    );
    let regression = pilot(
        observation(StructuredInputProjection::JsonCompact, Some(COMPACT_TOKENS), TaskOutcome::Passed),
        observation(StructuredInputProjection::ToonV3, Some(LOWER_TOON_TOKENS), TaskOutcome::Failed),
    );

    assert_eq!(evaluate_pilot(&missing).verdict, PilotVerdict::InconclusiveMissingUsage);
    assert_eq!(evaluate_pilot(&higher).verdict, PilotVerdict::BlockedNoStrictTokenBenefit);
    assert_eq!(evaluate_pilot(&regression).verdict, PilotVerdict::BlockedTaskRegression);
}

#[test]
fn pilot_blocks_binding_drift_and_contradictory_observations() {
    let compact = observation(StructuredInputProjection::JsonCompact, Some(COMPACT_TOKENS), TaskOutcome::Passed);
    let toon = observation(StructuredInputProjection::ToonV3, Some(LOWER_TOON_TOKENS), TaskOutcome::Passed);
    let mut drift = pilot(compact.clone(), toon.clone());
    drift.observed.model_identity = "model:drift".to_string();
    let mut contradictory = pilot(compact, toon.clone());
    contradictory.observations.push(toon);

    assert_eq!(evaluate_pilot(&drift).verdict, PilotVerdict::BlockedBindingDrift);
    assert_eq!(evaluate_pilot(&contradictory).verdict, PilotVerdict::BlockedContradictoryObservation);
}
