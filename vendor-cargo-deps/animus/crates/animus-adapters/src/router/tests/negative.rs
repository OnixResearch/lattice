use super::*;

#[derive(Clone)]
struct BoundsCase {
    max_candidates: u32,
    min_success_quorum: u32,
    max_repair_attempts: Option<u32>,
    expected: super::super::ValidationError,
}

#[test]
fn recipe_contracts_reject_invalid_shapes() {
    assert!(matches!(
        super::super::Recipe::new(super::super::RecipeDraft {
            id: RECIPE_SINGLE,
            family: "single",
            roles: &["primary", "primary"],
            output_contract: "json",
            max_candidates: MAX_CANDIDATES_ONE,
            max_parallelism: MAX_PARALLELISM_ONE,
            min_success_quorum: QUORUM_ONE,
            timeout_class: "standard",
            max_synthesis_attempts: SYNTHESIS_NONE,
            max_repair_attempts: Some(REPAIR_NONE),
            fallback_policy: "deny",
        }),
        Err(super::super::ValidationError::DuplicateRole { .. })
    ));
    assert!(matches!(
        super::super::Role::parse(BAD_VALUE),
        Err(crate::AdapterContractError::UnsupportedRouterValue { .. })
    ));
    assert!(matches!(
        super::super::OutputContract::parse(BAD_VALUE),
        Err(crate::AdapterContractError::UnsupportedRouterValue { .. })
    ));
    assert!(matches!(
        super::super::RouteFamily::parse(BAD_VALUE),
        Err(crate::AdapterContractError::UnsupportedRouterValue { .. })
    ));
}

#[test]
fn signal_projection_rejects_unsupported_labels() {
    let error = super::super::Signals::project(super::super::SignalDraft {
        task_shape: BAD_VALUE,
        ..signal_draft()
    })
    .expect_err("unsupported task shape should fail");

    assert!(matches!(
        error,
        crate::AdapterContractError::UnsupportedRouterValue { .. }
    ));
}

#[test]
fn deterministic_selection_denies_unsafe_eligibility() {
    let local_only = signals(super::super::SignalDraft {
        risk_band: "strict",
        privacy_band: "local_only",
        ..signal_draft()
    });
    let recipes = [recipe(RecipeFixture {
        id: RECIPE_FUSION,
        family: "fusion",
        roles: &["judge", "finalizer"],
    })];
    let catalog = catalog();

    assert_eq!(
        super::super::select_recipe(&local_only, &recipes, &catalog, &budget()),
        Err(super::super::SelectionDenial::NoEligibleModel)
    );
}

#[test]
fn execution_bounds_reject_invalid_plans() {
    let cases = [
        BoundsCase {
            max_candidates: ZERO_COUNT,
            min_success_quorum: QUORUM_ONE,
            max_repair_attempts: Some(REPAIR_NONE),
            expected: super::super::ValidationError::UnboundedFanOut,
        },
        BoundsCase {
            max_candidates: MAX_CANDIDATES_ONE,
            min_success_quorum: ZERO_COUNT,
            max_repair_attempts: Some(REPAIR_NONE),
            expected: super::super::ValidationError::ZeroQuorum,
        },
        BoundsCase {
            max_candidates: MAX_CANDIDATES_ONE,
            min_success_quorum: QUORUM_TWO,
            max_repair_attempts: Some(REPAIR_NONE),
            expected: super::super::ValidationError::ImpossibleQuorum,
        },
        BoundsCase {
            max_candidates: MAX_CANDIDATES_ONE,
            min_success_quorum: QUORUM_ONE,
            max_repair_attempts: None,
            expected: super::super::ValidationError::UnlimitedRepair,
        },
    ];

    assert_eq!(cases.len(), 4);
    assert!(cases.iter().any(|case| case.max_candidates == ZERO_COUNT));
    for case in &cases {
        assert_bounds_error(case);
    }
}

#[test]
fn router_receipt_leak_fixtures_fail_closed() {
    let cases = [
        ("raw prompt: hello", super::super::ReceiptLeak::RawPrompt),
        (
            "provider json { }",
            super::super::ReceiptLeak::ProviderPayload,
        ),
        (
            "worker output body",
            super::super::ReceiptLeak::WorkerOutput,
        ),
        (
            "judge rationale body",
            super::super::ReceiptLeak::JudgeRationale,
        ),
        (
            "hidden reasoning text",
            super::super::ReceiptLeak::HiddenReasoning,
        ),
        ("api_key=secret", super::super::ReceiptLeak::Credential),
        ("/home/user/private", super::super::ReceiptLeak::FullPath),
        (
            "unbounded trace follows",
            super::super::ReceiptLeak::UnboundedTrace,
        ),
    ];

    for (text, expected) in cases {
        assert_eq!(super::super::validate_receipt_text(text), Err(expected));
    }
    assert!(
        super::super::validate_receipt_text("recipe=remom quorum=met redactions=trace_bounded")
            .is_ok()
    );
}

fn assert_bounds_error(case: &BoundsCase) {
    assert_eq!(
        super::super::Bounds::new(super::super::BoundsDraft {
            max_candidates: case.max_candidates,
            max_parallelism: MAX_PARALLELISM_ONE,
            min_success_quorum: case.min_success_quorum,
            timeout_class: "standard",
            max_synthesis_attempts: SYNTHESIS_NONE,
            max_repair_attempts: case.max_repair_attempts,
            fallback_policy: "deny",
        }),
        Err(case.expected.clone())
    );
}
