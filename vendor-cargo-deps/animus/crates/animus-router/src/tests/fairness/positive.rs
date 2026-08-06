use super::super::*;

#[test]
fn round_robin_selection_rotates_per_role_and_wraps_cursor() {
    let recipes = [recipe(RecipeFixture {
        id: RECIPE_SINGLE,
        family: "single",
        roles: &["primary"],
    })];
    let catalog = super::data::primary_rotation_catalog();
    let projected = signals(crate::SignalDraft {
        task_shape: "simple",
        risk_band: "low",
        contract_pressure: "low",
        ..signal_draft()
    });
    let snapshot = rotation_snapshot(RotationSnapshotFixture {
        policy: "role_candidates",
        stale_cursor_policy: "reset",
        role: "primary",
        position: MAX_CANDIDATES_THREE,
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
    .expect("rotation selection should pass");

    assert_eq!(catalog.len(), super::data::PRIMARY_ROTATION_CATALOG_COUNT);
    assert_eq!(selected.selected_models()[0], MODEL_PRIMARY_ALT);
    assert_eq!(selected.rotation_decisions().len(), super::data::EXPECTED_ROTATION_DECISIONS);
    assert_eq!(selected.rotation_decisions()[0].cursor_class(), crate::RouterRotationCursorClass::Wrapped);
    assert_eq!(selected.rotation_decisions()[0].next_cursor().position(), ZERO_COUNT);
}

#[test]
fn round_robin_materializes_rotated_fallback_chain_and_receipt() {
    let catalog = super::data::fallback_rotation_catalog();
    let plan = super::data::fallback_rotation_plan();
    let snapshot = super::data::fallback_rotation_snapshot();
    let execution = crate::materialize_execution_plan_with_rotation(&plan, &catalog, &budget(), &snapshot)
        .expect("rotated execution plan should materialize");
    let receipt = crate::RouterRotationReceipt::from_decision(
        execution.recipe_id(),
        execution.family(),
        &execution.rotation_decisions()[0],
        &["trace_bounded"],
    )
    .expect("rotation receipt should validate");

    assert_eq!(catalog.len(), super::data::FALLBACK_ROTATION_CATALOG_COUNT);
    assert_eq!(execution.fallback_chain().steps()[0].model_label(), MODEL_FALLBACK_REMOTE);
    assert_eq!(execution.rotation_decisions()[0].selected_model(), MODEL_FALLBACK_REMOTE);
    assert_eq!(receipt.selected_model(), MODEL_FALLBACK_REMOTE);
    assert_eq!(receipt.identity_hash().len(), crate::SafeHash::HEX_CHARS);
}
