#![allow(assertion_density, excessive_file_length)]

use super::*;

#[derive(Clone, Copy)]
struct SelectCase<'a> {
    task_shape: &'a str,
    risk_band: &'a str,
    contract_pressure: &'a str,
    expected: crate::RouteFamily,
}

#[test]
fn recipe_contracts_accept_supported_families() {
    let cases = [
        RecipeFixture {
            id: RECIPE_SINGLE,
            family: "single",
            roles: &["primary"],
        },
        RecipeFixture {
            id: RECIPE_CONFIDENCE,
            family: "confidence",
            roles: &["primary", "escalation"],
        },
        RecipeFixture {
            id: RECIPE_RATINGS,
            family: "ratings",
            roles: &["primary", "rater"],
        },
        RecipeFixture {
            id: RECIPE_REMOM,
            family: "remom",
            roles: &["worker", "synthesizer"],
        },
        RecipeFixture {
            id: RECIPE_FUSION,
            family: "fusion",
            roles: &["judge", "finalizer"],
        },
        RecipeFixture {
            id: RECIPE_WORKFLOW,
            family: "workflow",
            roles: &["worker", "finalizer"],
        },
    ];

    for case in cases {
        let checked = recipe(case);
        assert!(!checked.roles().is_empty());
        assert_eq!(checked.output_contract(), crate::OutputContract::Json);
    }
}

#[test]
fn signal_projection_uses_only_bounded_labels() {
    let projected = signals(signal_draft());
    let rendered = alloc::format!("{:?}", projected.bucket_labels());

    assert_eq!(projected.task_shape(), crate::TaskShape::Reasoning);
    assert_eq!(projected.contract_pressure(), crate::SignalBand::Strict);
    assert!(!rendered.contains("prompt:"));
    assert!(!rendered.contains("provider payload"));
    assert!(!rendered.contains("credential"));
}

#[test]
fn deterministic_selection_covers_recipe_families() {
    let recipes = supported_recipes();
    let catalog = catalog();
    let budget = budget();
    let cases = [
        SelectCase {
            task_shape: "simple",
            risk_band: "low",
            contract_pressure: "low",
            expected: crate::RouteFamily::Single,
        },
        SelectCase {
            task_shape: "reasoning",
            risk_band: "medium",
            contract_pressure: "low",
            expected: crate::RouteFamily::Confidence,
        },
        SelectCase {
            task_shape: "review",
            risk_band: "high",
            contract_pressure: "medium",
            expected: crate::RouteFamily::Ratings,
        },
        SelectCase {
            task_shape: "reasoning",
            risk_band: "high",
            contract_pressure: "strict",
            expected: crate::RouteFamily::Remom,
        },
        SelectCase {
            task_shape: "reasoning",
            risk_band: "strict",
            contract_pressure: "high",
            expected: crate::RouteFamily::Fusion,
        },
        SelectCase {
            task_shape: "workflow",
            risk_band: "medium",
            contract_pressure: "high",
            expected: crate::RouteFamily::Workflow,
        },
    ];

    assert_eq!(cases.len(), 6);
    assert_eq!(recipes.len(), cases.len());
    for case in cases {
        assert_selects(case, &recipes, &catalog, &budget);
    }
}

pub(crate) fn supported_recipes() -> alloc::vec::Vec<crate::Recipe> {
    alloc::vec![
        recipe(RecipeFixture {
            id: RECIPE_SINGLE,
            family: "single",
            roles: &["primary"],
        }),
        recipe(RecipeFixture {
            id: RECIPE_CONFIDENCE,
            family: "confidence",
            roles: &["primary", "escalation"],
        }),
        recipe(RecipeFixture {
            id: RECIPE_RATINGS,
            family: "ratings",
            roles: &["primary", "rater"],
        }),
        recipe(RecipeFixture {
            id: RECIPE_REMOM,
            family: "remom",
            roles: &["worker", "synthesizer"],
        }),
        recipe(RecipeFixture {
            id: RECIPE_FUSION,
            family: "fusion",
            roles: &["judge", "finalizer"],
        }),
        recipe(RecipeFixture {
            id: RECIPE_WORKFLOW,
            family: "workflow",
            roles: &["worker", "finalizer"],
        }),
    ]
}

#[test]
fn output_contract_validation_fails_closed_or_falls_back() {
    let malformed = result(false, "deny", false);
    let fallback = result(false, "best_valid_evidence", true);
    let success = result(true, "deny", false);

    assert_eq!(
        malformed.status(),
        crate::RouteResultStatus::MalformedOutput
    );
    assert_eq!(
        fallback.status(),
        crate::RouteResultStatus::BestValidEvidence
    );
    assert_eq!(success.status(), crate::RouteResultStatus::Success);
    assert_eq!(success.status().as_str(), "success");
}

#[test]
fn budget_ledger_tracks_and_fallback_decides() {
    let bounds = crate::limit::test_bounds(BoundsFixture {
        max_candidates: MAX_CANDIDATES_THREE,
        max_parallelism: MAX_PARALLELISM_THREE,
        min_success_quorum: QUORUM_ONE,
        timeout_class: crate::limit::DeadlineClass::Standard,
        max_synthesis_attempts: SYNTHESIS_ONE,
        max_repair_attempts: REPAIR_ONE,
        fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
    });
    let ledger = crate::BudgetLedger::from(&bounds);
    let chain =
        crate::evidence::test_fallback_chain(alloc::vec![crate::evidence::test_fallback_step(
            crate::evidence::FallbackStepFixture {
                role: crate::Role::Fallback,
                model_label: MODEL_FALLBACK_LOCAL,
                backend_label: BACKEND_LOCAL,
                is_local: true,
            }
        ),]);

    assert_eq!(ledger.remaining_candidates(), MAX_CANDIDATES_THREE);
    assert_eq!(
        ledger.consume_candidate().unwrap().remaining_candidates(),
        MAX_CANDIDATES_TWO
    );
    assert_eq!(
        crate::evaluate_fallback(
            &crate::StepFailure::new(MODEL_PRIMARY, true),
            &ledger,
            &chain,
            crate::FallbackPolicy::BestValidEvidence,
        ),
        crate::FallbackDecision::Repair(REPAIR_ONE)
    );
    assert_eq!(
        crate::evaluate_fallback(
            &crate::StepFailure::new(MODEL_PRIMARY, false),
            &ledger,
            &chain,
            crate::FallbackPolicy::BestValidEvidence,
        ),
        crate::FallbackDecision::FallbackTo(crate::evidence::test_fallback_step(
            crate::evidence::FallbackStepFixture {
                role: crate::Role::Fallback,
                model_label: MODEL_FALLBACK_LOCAL,
                backend_label: BACKEND_LOCAL,
                is_local: true,
            }
        ))
    );
}

#[test]
fn safe_receipts_compute_identity_and_omit_raw_traces() {
    let receipt = crate::Receipt::new(crate::ReceiptDraft {
        recipe_id: RECIPE_REMOM,
        signal_buckets: &["reasoning", "high", "strict"],
        selected_models: &[MODEL_WORKER, MODEL_SYNTHESIZER],
        worker_count: WORKER_COUNT,
        success_count: SUCCESS_COUNT,
        quorum_class: "met",
        fallback_policy: "best_valid_evidence",
        policy_hash: SAFE_HASH,
        duration_bucket: "bounded",
        redactions: &[
            "prompt_omitted",
            "provider_payload_omitted",
            "worker_output_omitted",
            "judge_rationale_omitted",
            "hidden_reasoning_omitted",
            "credential_omitted",
            "path_redacted",
            "trace_bounded",
        ],
    })
    .expect("safe receipt should parse");
    let rendered = alloc::format!("{receipt:?}");

    assert_eq!(receipt.recipe_id(), RECIPE_REMOM);
    assert_eq!(receipt.worker_count(), WORKER_COUNT);
    assert_eq!(receipt.quorum_class(), crate::QuorumClass::Met);
    assert_eq!(receipt.identity_hash().len(), crate::SafeHash::HEX_CHARS);
    assert!(!rendered.contains("prompt:"));
    assert!(!rendered.contains("provider payload"));
}

fn assert_selects(
    case: SelectCase<'_>,
    recipes: &[crate::Recipe],
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
) {
    let projected = signals(crate::SignalDraft {
        task_shape: case.task_shape,
        risk_band: case.risk_band,
        contract_pressure: case.contract_pressure,
        ..signal_draft()
    });
    let selected = crate::select_recipe(&projected, recipes, catalog, budget)
        .expect("selection should succeed");

    assert_eq!(selected.family(), case.expected);
}

fn result(final_output_valid: bool, fallback_policy: &str, evidence: bool) -> crate::RouteResult {
    crate::RouteResult::new(crate::outcome::ResultDraft {
        output_contract: "json",
        fallback_policy,
        final_output_valid,
        repair_attempts_used: REPAIR_ONE,
        best_valid_evidence_available: evidence,
    })
    .expect("route result should classify")
}
