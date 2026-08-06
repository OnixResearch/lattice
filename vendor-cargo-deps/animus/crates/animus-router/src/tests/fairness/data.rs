use super::super::*;

pub(super) const EXPECTED_ROTATION_DECISIONS: usize = 1;
pub(super) const PRIMARY_ROTATION_CATALOG_COUNT: usize = 2;
pub(super) const FALLBACK_ROTATION_CATALOG_COUNT: usize = 4;
pub(super) const UPPERCASE_POLICY_HASH: &str = "A000000000000000000000000000000000000000000000000000000000000000";

pub(super) fn primary_rotation_catalog() -> alloc::vec::Vec<crate::CatalogEntry> {
    let catalog = alloc::vec![
        entry(EntryFixture {
            label: MODEL_PRIMARY,
            backend: BACKEND_LOCAL,
            role: "primary",
            local: true,
        }),
        entry(EntryFixture {
            label: MODEL_PRIMARY_ALT,
            backend: BACKEND_LOCAL,
            role: "primary",
            local: true,
        }),
    ];
    assert_eq!(catalog.len(), PRIMARY_ROTATION_CATALOG_COUNT);
    assert_eq!(catalog[0].label(), MODEL_PRIMARY);
    catalog
}

pub(super) fn primary_mixed_privacy_catalog() -> alloc::vec::Vec<crate::CatalogEntry> {
    let catalog = alloc::vec![
        entry(EntryFixture {
            label: MODEL_PRIMARY,
            backend: BACKEND_LOCAL,
            role: "primary",
            local: true,
        }),
        entry(EntryFixture {
            label: MODEL_PRIMARY_ALT,
            backend: BACKEND_REMOTE,
            role: "primary",
            local: false,
        }),
    ];
    assert_eq!(catalog.len(), PRIMARY_ROTATION_CATALOG_COUNT);
    assert_eq!(catalog[0].backend(), BACKEND_LOCAL);
    catalog
}

pub(super) fn fallback_rotation_catalog() -> alloc::vec::Vec<crate::CatalogEntry> {
    let catalog = alloc::vec![
        entry(EntryFixture {
            label: MODEL_PRIMARY,
            backend: BACKEND_LOCAL,
            role: "primary",
            local: true,
        }),
        entry(EntryFixture {
            label: MODEL_ESCALATION,
            backend: BACKEND_REMOTE,
            role: "escalation",
            local: false,
        }),
        entry(EntryFixture {
            label: MODEL_FALLBACK_LOCAL,
            backend: BACKEND_LOCAL,
            role: "fallback",
            local: true,
        }),
        entry(EntryFixture {
            label: MODEL_FALLBACK_REMOTE,
            backend: BACKEND_REMOTE,
            role: "fallback",
            local: false,
        }),
    ];
    assert_eq!(catalog.len(), FALLBACK_ROTATION_CATALOG_COUNT);
    assert_eq!(catalog[0].label(), MODEL_PRIMARY);
    catalog
}

pub(super) fn fallback_rotation_plan() -> crate::SelectedPlan {
    let plan = selected_plan(SelectedPlanFixture {
        recipe_id: RECIPE_CONFIDENCE,
        family: crate::RouteFamily::Confidence,
        selected_models: &[MODEL_PRIMARY, MODEL_ESCALATION],
        output_contract: crate::OutputContract::Json,
        bounds: plan_bounds(BoundsFixture {
            max_candidates: MAX_CANDIDATES_THREE,
            max_parallelism: MAX_PARALLELISM_THREE,
            min_success_quorum: QUORUM_ONE,
            timeout_class: crate::DeadlineClass::Standard,
            max_synthesis_attempts: SYNTHESIS_ONE,
            max_repair_attempts: REPAIR_ONE,
            fallback_policy: crate::FallbackPolicy::BestValidEvidence,
        }),
        privacy: crate::PrivacyBand::RemoteAllowed,
        fallback_eligibility: crate::FallbackEligibility::Eligible,
    });
    assert_eq!(plan.family(), crate::RouteFamily::Confidence);
    assert_eq!(plan.selected_models(), &[MODEL_PRIMARY, MODEL_ESCALATION]);
    plan
}

pub(super) fn fallback_rotation_snapshot() -> crate::RouterRotationSnapshot {
    rotation_snapshot(RotationSnapshotFixture {
        policy: "role_candidates_and_fallback",
        stale_cursor_policy: "reset",
        role: "fallback",
        position: MAX_CANDIDATES_ONE,
        cursor_hash: SAFE_HASH,
        catalog_hash: SAFE_HASH,
    })
}
