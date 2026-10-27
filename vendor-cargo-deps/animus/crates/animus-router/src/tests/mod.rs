mod bounded;
mod fairness;
mod fallback;
mod negative;
mod positive;

pub(super) const RECIPE_SINGLE: &str = "recipe:single";
pub(super) const RECIPE_CONFIDENCE: &str = "recipe:confidence";
pub(super) const RECIPE_RATINGS: &str = "recipe:ratings";
pub(super) const RECIPE_REMOM: &str = "recipe:remom";
pub(super) const RECIPE_FUSION: &str = "recipe:fusion";
pub(super) const RECIPE_WORKFLOW: &str = "recipe:workflow";
pub(super) const BAD_VALUE: &str = "surprise";
pub(super) const MODEL_PRIMARY: &str = "model:primary";
pub(super) const MODEL_PRIMARY_ALT: &str = "model:primary:alt";
pub(super) const MODEL_ESCALATION: &str = "model:escalation";
pub(super) const MODEL_RATER: &str = "model:rater";
pub(super) const MODEL_SYNTHESIZER: &str = "model:synthesizer";
pub(super) const MODEL_JUDGE: &str = "model:judge";
pub(super) const MODEL_FINALIZER: &str = "model:finalizer";
pub(super) const MODEL_WORKER: &str = "model:worker";
pub(super) const MODEL_FALLBACK_LOCAL: &str = "model:fallback:local";
pub(super) const MODEL_FALLBACK_REMOTE: &str = "model:fallback:remote";
pub(super) const BACKEND_LOCAL: &str = "backend:local";
pub(super) const BACKEND_REMOTE: &str = "backend:remote";
pub(super) const SAFE_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
pub(super) const OTHER_SAFE_HASH: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
pub(super) const ZERO_COUNT: u32 = 0;
pub(super) const MAX_CANDIDATES_ONE: u32 = 1;
pub(super) const MAX_CANDIDATES_TWO: u32 = 2;
pub(super) const MAX_CANDIDATES_THREE: u32 = 3;
pub(super) const MAX_PARALLELISM_ONE: u32 = 1;
pub(super) const MAX_PARALLELISM_THREE: u32 = 3;
pub(super) const QUORUM_ONE: u32 = 1;
pub(super) const QUORUM_TWO: u32 = 2;
pub(super) const SYNTHESIS_NONE: u32 = 0;
pub(super) const SYNTHESIS_ONE: u32 = 1;
pub(super) const REPAIR_NONE: u32 = 0;
pub(super) const REPAIR_ONE: u32 = 1;
pub(super) const WORKER_COUNT: u32 = 3;
pub(super) const SUCCESS_COUNT: u32 = 2;
pub(super) const SIMPLE_PROMPT_CHARS: u32 = 500;
pub(super) const REVIEW_PROMPT_CHARS: u32 = 4_000;
pub(super) const REASONING_PROMPT_CHARS: u32 = 9_000;
pub(super) const STRICT_PROMPT_CHARS: u32 = 25_000;
pub(super) const OVERSIZED_BOUNDED_PROMPT_CHARS: u32 = 120_000;
pub(super) const LOW_CONTEXT_PER_MILLE: u16 = 100;
pub(super) const HIGH_CONTEXT_PER_MILLE: u16 = 800;
pub(super) const STRICT_CONTEXT_PER_MILLE: u16 = 950;
pub(super) const INVALID_CONTEXT_PER_MILLE: u16 = 1_001;
pub(super) const TOOL_COUNT_NONE: u32 = 0;
pub(super) const TOOL_COUNT_ONE: u32 = 1;
pub(super) const TOOL_COUNT_TWO: u32 = 2;

#[derive(Clone, Copy)]
pub(super) struct BoundsFixture {
    pub(super) max_candidates: u32,
    pub(super) max_parallelism: u32,
    pub(super) min_success_quorum: u32,
    pub(super) timeout_class: crate::limit::DeadlineClass,
    pub(super) max_synthesis_attempts: u32,
    pub(super) max_repair_attempts: u32,
    pub(super) fallback_policy: crate::limit::FallbackPolicy,
}

#[derive(Clone)]
pub(super) struct SelectedPlanFixture<'a> {
    pub(super) recipe_id: &'a str,
    pub(super) family: crate::RouteFamily,
    pub(super) selected_models: &'a [&'a str],
    pub(super) output_contract: crate::OutputContract,
    pub(super) bounds: crate::Bounds,
    pub(super) privacy: crate::PrivacyBand,
    pub(super) fallback_eligibility: crate::FallbackEligibility,
}

#[derive(Clone, Copy)]
pub(super) struct RecipeFixture<'a> {
    pub(super) id: &'a str,
    pub(super) family: &'a str,
    pub(super) roles: &'a [&'a str],
}

#[derive(Clone, Copy)]
pub(super) struct EntryFixture<'a> {
    pub(super) label: &'a str,
    pub(super) backend: &'a str,
    pub(super) role: &'a str,
    pub(super) local: bool,
}

pub(super) fn recipe(fixture: RecipeFixture<'_>) -> crate::Recipe {
    crate::Recipe::new(crate::RecipeDraft {
        id: fixture.id,
        family: fixture.family,
        roles: fixture.roles,
        output_contract: "json",
        max_candidates: MAX_CANDIDATES_THREE,
        max_parallelism: MAX_PARALLELISM_THREE,
        min_success_quorum: QUORUM_ONE,
        timeout_class: "standard",
        max_synthesis_attempts: SYNTHESIS_ONE,
        max_repair_attempts: Some(REPAIR_ONE),
        fallback_policy: "best_valid_evidence",
    })
    .expect("valid recipe should parse")
}

pub(super) fn entry(fixture: EntryFixture<'_>) -> crate::CatalogEntry {
    crate::CatalogEntry::new(crate::CatalogEntryDraft {
        label: fixture.label,
        backend: fixture.backend,
        role: fixture.role,
        local: fixture.local,
    })
    .expect("valid catalog entry should parse")
}

pub(super) fn catalog() -> alloc::vec::Vec<crate::CatalogEntry> {
    alloc::vec![
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
            label: MODEL_RATER,
            backend: BACKEND_REMOTE,
            role: "rater",
            local: false,
        }),
        entry(EntryFixture {
            label: MODEL_SYNTHESIZER,
            backend: BACKEND_REMOTE,
            role: "synthesizer",
            local: false,
        }),
        entry(EntryFixture {
            label: MODEL_JUDGE,
            backend: BACKEND_REMOTE,
            role: "judge",
            local: false,
        }),
        entry(EntryFixture {
            label: MODEL_FINALIZER,
            backend: BACKEND_REMOTE,
            role: "finalizer",
            local: false,
        }),
        entry(EntryFixture {
            label: MODEL_WORKER,
            backend: BACKEND_REMOTE,
            role: "worker",
            local: false,
        }),
        entry(EntryFixture {
            label: MODEL_FALLBACK_LOCAL,
            backend: BACKEND_LOCAL,
            role: "fallback",
            local: true,
        }),
    ]
}

pub(super) fn budget() -> crate::Budget {
    crate::Budget::new(MAX_CANDIDATES_THREE, &[BACKEND_LOCAL, BACKEND_REMOTE])
        .expect("valid budget should parse")
}

pub(super) fn signal_draft() -> crate::SignalDraft<'static> {
    crate::SignalDraft {
        task_shape: "reasoning",
        risk_band: "high",
        contract_pressure: "strict",
        context_pressure: "medium",
        latency_band: "medium",
        cost_band: "medium",
        privacy_band: "remote_allowed",
        fallback_eligibility: "eligible",
    }
}

pub(super) fn signals(draft: crate::SignalDraft<'_>) -> crate::Signals {
    crate::Signals::project(draft).expect("valid signals should parse")
}

pub(super) fn plan_bounds(fixture: BoundsFixture) -> crate::Bounds {
    crate::limit::test_bounds(fixture)
}

pub(super) fn selected_plan(fixture: SelectedPlanFixture<'_>) -> crate::SelectedPlan {
    crate::choose::test_selected_plan(fixture)
}

#[derive(Clone, Copy)]
pub(super) struct RotationSnapshotFixture {
    pub(super) policy: &'static str,
    pub(super) stale_cursor_policy: &'static str,
    pub(super) role: &'static str,
    pub(super) position: u32,
    pub(super) cursor_hash: &'static str,
    pub(super) catalog_hash: &'static str,
}

pub(super) fn rotation_snapshot(fixture: RotationSnapshotFixture) -> crate::RouterRotationSnapshot {
    let cursor = crate::RouterRotationCursorDraft {
        role: fixture.role,
        position: fixture.position,
        catalog_hash: Some(fixture.cursor_hash),
    };
    crate::RouterRotationSnapshot::new(crate::RouterRotationSnapshotDraft {
        policy: fixture.policy,
        stale_cursor_policy: fixture.stale_cursor_policy,
        catalog_hash: fixture.catalog_hash,
        cursors: core::slice::from_ref(&cursor),
    })
    .expect("rotation snapshot should validate")
}
