//! Artifact admission: both example kinds are admitted under their canonical identity, and every
//! tampered, unselected, missing, or oversized artifact is refused with a typed error.

mod support;

#[test]
// r[verify agent_harness_runtime.frozen_policy.artifact]
// r[verify agent_harness_runtime.frozen_policy.model]
fn both_examples_are_admitted_under_their_canonical_identity() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let mlp = support::admitted(support::MLP)?;
    assert_eq!(tree.policy_kind(), animus_frozen_policy::TREE_POLICY_KIND);
    assert_eq!(mlp.policy_kind(), animus_frozen_policy::MLP_POLICY_KIND);
    assert_eq!(
        tree.worst_case_operations(),
        2,
        "two splits on the deepest path"
    );
    assert_eq!(mlp.worst_case_operations(), 24, "3x4 plus 4x3 products");
    assert_eq!(tree.actions(), support::ACTIONS);
    assert_eq!(tree.feature_schema(), support::FEATURE_SCHEMA);
    assert_eq!(tree.action_schema(), support::ACTION_SCHEMA);
    assert_eq!(tree.abstain_below(), Some(50));
    assert_eq!(mlp.abstain_below(), None);
    assert_eq!(tree.score_bounds(), (-1000, 1000));
    assert_eq!(tree.provenance_kind(), "fixture");
    assert_ne!(tree.identity(), mlp.identity());
    assert_ne!(tree.model_identity(), mlp.model_identity());
    assert!(animus_frozen_policy::is_b3_identity(tree.identity()));
    assert!(animus_frozen_policy::is_b3_identity(tree.model_identity()));
    let reordered =
        serde_json::to_vec_pretty(&serde_json::from_str::<serde_json::Value>(support::TREE)?)?;
    assert_eq!(
        animus_frozen_policy::policy_identity(&reordered)?,
        tree.identity(),
        "spacing and key order do not change the identity"
    );
    let reprovenanced = support::admitted_edit(support::TREE, |value| {
        value["provenance"]["evaluation"] = serde_json::json!("another evaluation summary");
    })?;
    assert_ne!(reprovenanced.identity(), tree.identity());
    assert_eq!(
        reprovenanced.model_identity(),
        tree.model_identity(),
        "the model identity covers the weights alone"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.artifact]
fn identities_use_the_animus_evaluation_framing_byte_for_byte() {
    // Reference values from `animus_eval_core::hash_identity` over the same domains and values.
    let cases: [(&str, &[&str], &str); 3] = [
        (
            "animus.frozen-policy.v1",
            &["{}"],
            "b3:c05b29345a5c9e42bb45d0bd5efdfef9871b0420d7e32865fde5d7afc36c9578",
        ),
        (
            "d",
            &["a", "bc", ""],
            "b3:49238c32c05c72d9f82930c255680a55b3bf74dce38531105b9dde3dba786ca8",
        ),
        (
            "",
            &[],
            "b3:71e0a99173564931c0b8acc52d2685a8e39c64dc52e3d02390fdac2a12b155cb",
        ),
    ];
    for (domain, values, expected) in cases {
        assert_eq!(
            animus_frozen_policy::domain_identity(domain, values),
            expected
        );
    }
    assert!(!animus_frozen_policy::is_b3_identity("b3:ABC"));
    assert!(!animus_frozen_policy::is_safe_identity("has spaces"));
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.artifact]
fn a_selected_identity_admits_only_its_own_bytes() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let (changed, _) = support::edited(support::TREE, |value| {
        value["model"]["nodes"][2]["leaf"]["scores"][2] = serde_json::json!(91);
    })?;
    assert_eq!(
        animus_frozen_policy::admit_policy(&changed, tree.identity()),
        Err(animus_frozen_policy::AdmissionError::Identity),
        "one changed leaf score under the selected identity"
    );
    assert_eq!(
        animus_frozen_policy::admit_policy(support::MLP.as_bytes(), tree.identity()),
        Err(animus_frozen_policy::AdmissionError::Identity)
    );
    assert_eq!(
        animus_frozen_policy::admit_policy(&[], tree.identity()),
        Err(animus_frozen_policy::AdmissionError::Unavailable)
    );
    let oversized_bytes = vec![b' '; animus_frozen_policy::MAX_ARTIFACT_BYTES.saturating_add(1)];
    assert_eq!(
        animus_frozen_policy::admit_policy(&oversized_bytes, tree.identity()),
        Err(animus_frozen_policy::AdmissionError::TooLarge)
    );
    assert_eq!(
        animus_frozen_policy::admit_policy(b"{\"schema\":", tree.identity()),
        Err(animus_frozen_policy::AdmissionError::Malformed)
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.artifact]
fn tampered_contracts_are_refused_before_any_decision() -> Result<(), support::Failure> {
    let cases: [support::Refusal; 13] = [
        (
            |value| value["unexpected"] = serde_json::json!(1),
            animus_frozen_policy::AdmissionError::Malformed,
        ),
        (
            |value| value["model"]["kind"] = serde_json::json!("transformer-v1"),
            animus_frozen_policy::AdmissionError::Malformed,
        ),
        (
            |value| value["model"]["nodes"][0]["split"]["extra"] = serde_json::json!(1),
            animus_frozen_policy::AdmissionError::Malformed,
        ),
        (
            |value| value["schema"] = serde_json::json!("animus.frozen-policy.v2"),
            animus_frozen_policy::AdmissionError::Schema,
        ),
        (
            |value| value["runtime"]["crate"] = serde_json::json!("animus-campaign-manager-eval"),
            animus_frozen_policy::AdmissionError::Runtime,
        ),
        (
            |value| value["runtime"]["artifact_api"] = serde_json::json!(2),
            animus_frozen_policy::AdmissionError::Runtime,
        ),
        (
            |value| value["feature_schema"]["normalization"] = serde_json::json!("z-score-v1"),
            animus_frozen_policy::AdmissionError::FeatureSchema,
        ),
        (
            |value| value["feature_schema"]["features"][1]["name"] = serde_json::json!("attempts"),
            animus_frozen_policy::AdmissionError::FeatureSchema,
        ),
        (
            |value| value["feature_schema"]["features"][0]["max"] = serde_json::json!(0),
            animus_frozen_policy::AdmissionError::FeatureSchema,
        ),
        (
            |value| value["action_schema"]["actions"][2] = serde_json::json!("proceed"),
            animus_frozen_policy::AdmissionError::ActionSchema,
        ),
        (
            |value| value["action_schema"]["actions"] = serde_json::json!([]),
            animus_frozen_policy::AdmissionError::ActionSchema,
        ),
        (
            |value| value["provenance"]["kind"] = serde_json::json!("online"),
            animus_frozen_policy::AdmissionError::Provenance,
        ),
        (
            |value| value["provenance"]["non_claims"] = serde_json::json!([]),
            animus_frozen_policy::AdmissionError::Provenance,
        ),
    ];
    let refused = support::refusals(support::TREE, &cases)?;
    assert_eq!(refused, 13, "every tampered contract is refused");
    assert!(
        support::admitted(support::TREE).is_ok(),
        "the untampered artifact is admitted"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.model]
fn malformed_trees_and_bounds_are_refused() -> Result<(), support::Failure> {
    let cases: [support::Refusal; 10] = [
        (
            |value| value["model"]["nodes"][1]["split"]["below"] = serde_json::json!(0),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["nodes"][1]["split"]["below"] = serde_json::json!(2),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["nodes"][1]["split"]["above"] = serde_json::json!(9),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["nodes"][0]["split"]["feature"] = serde_json::json!(3),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["nodes"][0]["split"]["threshold"] = serde_json::json!(1001),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["nodes"][2]["leaf"]["scores"] = serde_json::json!([0, 10]),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["nodes"][2]["leaf"]["scores"][0] = serde_json::json!(1_000_001),
            animus_frozen_policy::AdmissionError::Parameters,
        ),
        (
            |value| value["bounds"]["score_min"] = serde_json::json!(1000),
            animus_frozen_policy::AdmissionError::Bounds,
        ),
        (
            |value| value["bounds"]["abstain_below"] = serde_json::json!(2000),
            animus_frozen_policy::AdmissionError::Bounds,
        ),
        (
            |value| value["bounds"]["operation_budget"] = serde_json::json!(1),
            animus_frozen_policy::AdmissionError::Bounds,
        ),
    ];
    let refused = support::refusals(support::TREE, &cases)?;
    assert_eq!(refused, 10, "every malformed tree is refused");
    assert!(
        support::admitted(support::TREE).is_ok(),
        "the well-formed tree is admitted"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.model]
fn malformed_mlps_are_refused() -> Result<(), support::Failure> {
    let cases: [support::Refusal; 6] = [
        (
            |value| value["model"]["layers"][1]["activation"] = serde_json::json!("relu"),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["layers"][0]["activation"] = serde_json::json!("tanh"),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["layers"][0]["weights"][0] = serde_json::json!([1, 2]),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["layers"][1]["biases"] = serde_json::json!([0, 0]),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["layers"] = serde_json::json!([]),
            animus_frozen_policy::AdmissionError::Model,
        ),
        (
            |value| value["model"]["layers"][0]["weights"][0][0] = serde_json::json!(-1_000_001),
            animus_frozen_policy::AdmissionError::Parameters,
        ),
    ];
    let refused = support::refusals(support::MLP, &cases)?;
    assert_eq!(refused, 6, "every malformed MLP is refused");
    assert!(
        support::admitted(support::MLP).is_ok(),
        "the well-formed MLP is admitted"
    );
    Ok(())
}
