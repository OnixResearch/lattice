//! Receipts: bounded, identity-bound records that replay a decision without inference, and that
//! refuse tampering.

mod support;

#[test]
// r[verify agent_harness_runtime.frozen_policy.receipt]
fn a_retained_receipt_replays_its_outcome_without_inference() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let decision = animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[2, 200, 10]));
    let retained = decision.receipt.encode();
    assert!(retained.len() <= animus_frozen_policy::MAX_RECEIPT_BYTES);
    let replayed = animus_frozen_policy::verify_receipt(retained.as_bytes())?;
    assert_eq!(replayed, decision.receipt);
    assert_eq!(
        replayed.body.outcome,
        animus_frozen_policy::ReceiptOutcome::Selected
    );
    assert_eq!(replayed.body.action.as_deref(), Some("proceed"));
    assert_eq!(replayed.body.schema, animus_frozen_policy::RECEIPT_SCHEMA);
    assert_eq!(
        replayed.body.runtime_crate,
        animus_frozen_policy::RUNTIME_CRATE
    );
    assert!(animus_frozen_policy::is_b3_identity(&replayed.identity));
    let mut refused = support::request(&tree, &[2, 200, 10]);
    refused.max_operations = 0;
    let refusal = animus_frozen_policy::decide(Ok(&tree), &refused);
    assert!(matches!(
        refusal.outcome,
        animus_frozen_policy::Outcome::Refused { .. }
    ));
    assert_eq!(
        animus_frozen_policy::verify_receipt(refusal.receipt.encode().as_bytes())?
            .body
            .failure
            .as_deref(),
        Some("operation-budget")
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.receipt]
fn receipt_identities_bind_input_eligible_set_and_artifact() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let base =
        animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[2, 200, 10])).receipt;
    let mut reordered = support::request(&tree, &[2, 200, 10]);
    reordered.eligible.reverse();
    let reordered = animus_frozen_policy::decide(Ok(&tree), &reordered).receipt;
    assert_eq!(
        reordered.body.action_set, base.body.action_set,
        "the eligible-set identity ignores label order"
    );
    assert_ne!(
        reordered.identity, base.identity,
        "the receipt keeps the requested order"
    );
    let other_input =
        animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[2, 201, 10])).receipt;
    assert_ne!(other_input.body.input, base.body.input);
    assert_eq!(other_input.body.action, base.body.action);
    assert_ne!(other_input.identity, base.identity);
    let retrained = support::admitted_edit(support::TREE, |value| {
        value["model"]["nodes"][3]["leaf"]["scores"][0] = serde_json::json!(101);
    })?;
    let after_change =
        animus_frozen_policy::decide(Ok(&retrained), &support::request(&retrained, &[2, 200, 10]))
            .receipt;
    assert_ne!(after_change.body.artifact, base.body.artifact);
    assert_ne!(after_change.body.model, base.body.model);
    assert_ne!(after_change.identity, base.identity);
    assert_eq!(
        animus_frozen_policy::verify_receipt(base.encode().as_bytes())?,
        base,
        "a model change leaves an earlier receipt valid and unchanged"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.receipt]
fn tampered_receipts_are_refused() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let receipt =
        animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[2, 200, 10])).receipt;
    let base: serde_json::Value = serde_json::from_str(&receipt.encode())?;
    let cases: [(support::JsonEdit, animus_frozen_policy::ReceiptError); 9] = [
        (
            |value| value["body"]["action"] = serde_json::json!("escalate"),
            animus_frozen_policy::ReceiptError::Identity,
        ),
        (
            |value| value["identity"] = serde_json::json!(format!("b3:{}", "0".repeat(64))),
            animus_frozen_policy::ReceiptError::Identity,
        ),
        (
            |value| value["body"]["extra"] = serde_json::json!(true),
            animus_frozen_policy::ReceiptError::Malformed,
        ),
        (
            |value| value["body"]["schema"] = serde_json::json!("animus.frozen-policy-receipt.v2"),
            animus_frozen_policy::ReceiptError::Schema,
        ),
        (
            |value| value["body"]["features"] = serde_json::json!(vec![0; 65]),
            animus_frozen_policy::ReceiptError::Bounds,
        ),
        (
            |value| value["body"]["input"] = serde_json::json!("blake3:abc"),
            animus_frozen_policy::ReceiptError::Bounds,
        ),
        (
            |value| value["body"]["action"] = serde_json::json!("invented"),
            animus_frozen_policy::ReceiptError::Inconsistent,
        ),
        (
            |value| value["body"]["failure"] = serde_json::json!("abstained"),
            animus_frozen_policy::ReceiptError::Inconsistent,
        ),
        (
            |value| value["body"]["outcome"] = serde_json::json!("refused"),
            animus_frozen_policy::ReceiptError::Inconsistent,
        ),
    ];
    for (edit, expected) in cases {
        let mut value = base.clone();
        edit(&mut value);
        let bytes = serde_json::to_vec(&value)?;
        assert_eq!(
            animus_frozen_policy::verify_receipt(&bytes),
            Err(expected),
            "{value}"
        );
    }
    assert_eq!(
        animus_frozen_policy::verify_receipt(&[]),
        Err(animus_frozen_policy::ReceiptError::Size)
    );
    let oversized_bytes = vec![b' '; animus_frozen_policy::MAX_RECEIPT_BYTES.saturating_add(1)];
    assert_eq!(
        animus_frozen_policy::verify_receipt(&oversized_bytes),
        Err(animus_frozen_policy::ReceiptError::Size)
    );
    Ok(())
}
