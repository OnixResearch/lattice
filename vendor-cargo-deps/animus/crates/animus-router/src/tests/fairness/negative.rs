use super::super::*;

#[test]
fn round_robin_selection_denies_stale_cursor_when_policy_requires() {
    let projected = signals(crate::SignalDraft {
        task_shape: "simple",
        risk_band: "low",
        contract_pressure: "low",
        ..signal_draft()
    });
    let recipes = [recipe(RecipeFixture {
        id: RECIPE_SINGLE,
        family: "single",
        roles: &["primary"],
    })];
    let catalog = alloc::vec![entry(EntryFixture {
        label: MODEL_PRIMARY,
        backend: BACKEND_LOCAL,
        role: "primary",
        local: true,
    })];
    let snapshot = rotation_snapshot(RotationSnapshotFixture {
        policy: "role_candidates",
        stale_cursor_policy: "deny",
        role: "primary",
        position: ZERO_COUNT,
        cursor_hash: OTHER_SAFE_HASH,
        catalog_hash: SAFE_HASH,
    });

    let router_budget = budget();
    assert!(!recipes.is_empty());
    assert!(!catalog.is_empty());
    assert_eq!(
        crate::select_recipe_with_rotation(crate::RotatingSelectionRequest {
            signals: &projected,
            recipes: &recipes,
            catalog: &catalog,
            budget: &router_budget,
            rotation: &snapshot,
        }),
        Err(crate::SelectionDenial::StaleRotationCursor)
    );
}

#[test]
fn round_robin_cursor_and_receipt_validation_fail_closed() {
    let malformed_cursor = crate::RouterRotationSnapshot::new(crate::RouterRotationSnapshotDraft {
        policy: "role_candidates",
        stale_cursor_policy: "reset",
        catalog_hash: SAFE_HASH,
        cursors: &[crate::RouterRotationCursorDraft {
            role: BAD_VALUE,
            position: ZERO_COUNT,
            catalog_hash: Some(SAFE_HASH),
        }],
    });
    let receipt_leak = crate::RouterRotationReceipt::new(crate::RouterRotationReceiptDraft {
        recipe_id: RECIPE_SINGLE,
        route_family: "single",
        role: "primary",
        policy: "role_candidates",
        cursor_class: "matched",
        selected_model: "provider payload body",
        candidate_count: MAX_CANDIDATES_ONE,
        catalog_hash: SAFE_HASH,
        next_cursor_hash: SAFE_HASH,
        redactions: &["trace_bounded"],
    });

    assert!(matches!(
        malformed_cursor,
        Err(crate::AdapterContractError::UnsupportedRouterValue { .. })
    ));
    assert!(matches!(
        receipt_leak,
        Err(crate::RouterRotationError::ReceiptLeak(
            crate::ReceiptLeak::ProviderPayload
        ))
    ));
}

#[test]
fn round_robin_rotation_cannot_bypass_privacy_filtering() {
    let projected = signals(crate::SignalDraft {
        task_shape: "simple",
        risk_band: "low",
        contract_pressure: "low",
        privacy_band: "local_only",
        ..signal_draft()
    });
    let recipes = [recipe(RecipeFixture {
        id: RECIPE_SINGLE,
        family: "single",
        roles: &["primary"],
    })];
    let catalog = super::data::primary_mixed_privacy_catalog();
    let snapshot = rotation_snapshot(RotationSnapshotFixture {
        policy: "role_candidates",
        stale_cursor_policy: "reset",
        role: "primary",
        position: MAX_CANDIDATES_ONE,
        cursor_hash: SAFE_HASH,
        catalog_hash: SAFE_HASH,
    });
    let router_budget = budget();
    let selected = crate::select_recipe_with_rotation(crate::RotatingSelectionRequest {
        signals: &projected,
        recipes: &recipes,
        catalog: &catalog,
        budget: &router_budget,
        rotation: &snapshot,
    })
    .expect("local-only rotation should keep local candidate");

    assert_eq!(catalog.len(), super::data::PRIMARY_ROTATION_CATALOG_COUNT);
    assert_eq!(selected.selected_models()[0], MODEL_PRIMARY);
    assert_eq!(
        selected.rotation_decisions()[0].candidate_count(),
        MAX_CANDIDATES_ONE
    );
}

#[test]
fn router_receipt_rejects_uppercase_policy_hash() {
    let error = crate::Receipt::new(crate::ReceiptDraft {
        recipe_id: RECIPE_REMOM,
        signal_buckets: &["reasoning"],
        selected_models: &[MODEL_WORKER],
        worker_count: WORKER_COUNT,
        success_count: SUCCESS_COUNT,
        quorum_class: "met",
        fallback_policy: "deny",
        policy_hash: super::data::UPPERCASE_POLICY_HASH,
        duration_bucket: "bounded",
        redactions: &["prompt_omitted"],
    });

    assert!(error.is_err());
    let Err(crate::ValidationError::Contract(crate::AdapterContractError::InvalidEvidenceHash {
        field,
        reason,
    })) = error
    else {
        panic!("uppercase policy hash must fail with an invalid evidence hash");
    };

    assert_eq!(field, "router_effect_receipt_hash");
    assert_eq!(reason, "effect hash must contain lowercase hex only");
}
