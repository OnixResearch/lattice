//! Typed failures: stale bindings, malformed and out-of-range inputs, the operation budget,
//! out-of-range scores, and unavailable or incompatible artifacts, with the declared fallback
//! rule.

mod support;

/// One edit of a binding under test.
type BindingEdit = fn(&mut animus_frozen_policy::PolicyBinding);

#[test]
// r[verify agent_harness_runtime.frozen_policy.failure]
fn stale_bindings_fail_before_inference() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let mlp = support::admitted(support::MLP)?;
    let cases: [(BindingEdit, animus_frozen_policy::StaleField); 3] = [
        (
            |binding| binding.artifact = String::from("b3:0000"),
            animus_frozen_policy::StaleField::Artifact,
        ),
        (
            |binding| binding.feature_schema = String::from("animus.other-features.v1"),
            animus_frozen_policy::StaleField::FeatureSchema,
        ),
        (
            |binding| binding.action_schema = String::from("animus.other-actions.v1"),
            animus_frozen_policy::StaleField::ActionSchema,
        ),
    ];
    for (edit, field) in cases {
        let mut request = support::request_with_fallback(&tree, &[2, 200, 10], "retry");
        edit(&mut request.binding);
        let decision = animus_frozen_policy::decide(Ok(&tree), &request);
        assert_eq!(
            decision.outcome,
            support::fallback("retry", animus_frozen_policy::Failure::Stale(field))
        );
        assert!(decision.receipt.body.scores.is_empty());
        assert_eq!(decision.receipt.body.cost.inference_operations, 0);
    }
    assert_eq!(
        animus_frozen_policy::decide(Ok(&mlp), &support::request(&tree, &[2, 200, 10])).outcome,
        animus_frozen_policy::Outcome::Refused {
            failure: animus_frozen_policy::Failure::Stale(
                animus_frozen_policy::StaleField::Artifact
            ),
        },
        "a profile pinned to one artifact never runs another"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.failure]
fn inputs_the_artifact_refuses_fall_back() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let cases: [(support::RequestEdit, animus_frozen_policy::Failure); 4] = [
        (
            |request| request.features = vec![1, 2],
            animus_frozen_policy::Failure::MalformedInput(
                animus_frozen_policy::InputFault::FeatureCount,
            ),
        ),
        (
            |request| request.eligible = support::labels(&["retry", "invented"]),
            animus_frozen_policy::Failure::MalformedInput(
                animus_frozen_policy::InputFault::UnknownAction,
            ),
        ),
        (
            |request| request.features = vec![11, 0, 0],
            animus_frozen_policy::Failure::InputOutOfRange { feature: 0 },
        ),
        (
            |request| request.features = vec![0, 0, -1],
            animus_frozen_policy::Failure::InputOutOfRange { feature: 2 },
        ),
    ];
    for (edit, failure) in cases {
        let mut request = support::request_with_fallback(&tree, &[2, 200, 10], "retry");
        edit(&mut request);
        let decision = animus_frozen_policy::decide(Ok(&tree), &request);
        assert_eq!(decision.outcome, support::fallback("retry", failure));
        assert_eq!(decision.receipt.body.cost.inference_operations, 0);
        assert!(decision.receipt.body.scores.is_empty());
    }
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.failure]
fn malformed_request_shapes_never_fall_back() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let cases: [(support::RequestEdit, animus_frozen_policy::InputFault); 4] = [
        (
            |request| request.eligible = Vec::new(),
            animus_frozen_policy::InputFault::NoEligibleAction,
        ),
        (
            |request| request.eligible = support::labels(&["retry", "retry"]),
            animus_frozen_policy::InputFault::DuplicateAction,
        ),
        (
            |request| request.fallback = Some(String::from("escalate-to-human")),
            animus_frozen_policy::InputFault::FallbackNotEligible,
        ),
        (
            |request| request.binding.artifact = String::from("has spaces"),
            animus_frozen_policy::InputFault::Binding,
        ),
    ];
    for (edit, fault) in cases {
        let mut request = support::request_with_fallback(&tree, &[2, 200, 10], "retry");
        edit(&mut request);
        let body = animus_frozen_policy::decide(Ok(&tree), &request)
            .receipt
            .body;
        assert_eq!(body.outcome, animus_frozen_policy::ReceiptOutcome::Refused);
        assert_eq!(body.failure.as_deref(), Some("malformed-input"));
        assert_eq!(body.failure_detail.as_deref(), Some(fault.code()));
        assert!(body.eligible.is_empty());
        assert!(body.features.is_empty());
    }
    let mut unsafe_binding = support::request(&tree, &[2, 200, 10]);
    unsafe_binding.binding.artifact = String::from("has spaces");
    let body = animus_frozen_policy::decide(Ok(&tree), &unsafe_binding)
        .receipt
        .body;
    assert_eq!(
        body.binding,
        animus_frozen_policy::PolicyBinding {
            action_schema: String::new(),
            artifact: String::new(),
            feature_schema: String::new(),
        },
        "an unsafe binding is never copied into the receipt"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.failure]
fn budget_and_range_failures_are_typed() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let mut over_budget = support::request(&tree, &[2, 200, 10]);
    over_budget.max_operations = 1;
    assert_eq!(
        animus_frozen_policy::decide(Ok(&tree), &over_budget).outcome,
        animus_frozen_policy::Outcome::Refused {
            failure: animus_frozen_policy::Failure::OperationBudget,
        },
        "two worst-case splits exceed a one-operation deadline"
    );
    let narrow = support::admitted_edit(support::MLP, |value| {
        value["bounds"]["score_max"] = serde_json::json!(300);
    })?;
    assert_eq!(
        animus_frozen_policy::decide(Ok(&narrow), &support::request(&narrow, &[5, 800, 250]))
            .outcome,
        animus_frozen_policy::Outcome::Refused {
            failure: animus_frozen_policy::Failure::ScoreOutOfRange,
        },
        "retry's 380 is out of range, never clamped to 300"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.failure]
fn unavailable_and_incompatible_artifacts_fall_back() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let request = support::request_with_fallback(&tree, &[2, 200, 10], "escalate");
    let unavailable = animus_frozen_policy::decide(
        Err(&animus_frozen_policy::AdmissionError::Unavailable),
        &request,
    );
    assert_eq!(
        unavailable.outcome,
        support::fallback("escalate", animus_frozen_policy::Failure::Unavailable)
    );
    assert_eq!(unavailable.receipt.body.artifact, None);
    let incompatible = animus_frozen_policy::decide(
        Err(&animus_frozen_policy::AdmissionError::Identity),
        &request,
    );
    assert_eq!(
        incompatible.outcome,
        support::fallback(
            "escalate",
            animus_frozen_policy::Failure::IncompatibleArtifact(
                animus_frozen_policy::AdmissionError::Identity
            ),
        )
    );
    assert_eq!(
        incompatible.receipt.body.failure_detail.as_deref(),
        Some("identity")
    );
    Ok(())
}
