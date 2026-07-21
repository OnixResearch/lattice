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
pub(super) const MODEL_ESCALATION: &str = "model:escalation";
pub(super) const MODEL_RATER: &str = "model:rater";
pub(super) const MODEL_SYNTHESIZER: &str = "model:synthesizer";
pub(super) const MODEL_JUDGE: &str = "model:judge";
pub(super) const MODEL_FINALIZER: &str = "model:finalizer";
pub(super) const MODEL_WORKER: &str = "model:worker";
pub(super) const BACKEND_LOCAL: &str = "backend:local";
pub(super) const BACKEND_REMOTE: &str = "backend:remote";
pub(super) const SAFE_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
pub(super) const ZERO_COUNT: u32 = 0;
pub(super) const MAX_CANDIDATES_ONE: u32 = 1;
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

pub(super) fn recipe(fixture: RecipeFixture<'_>) -> super::Recipe {
    super::Recipe::new(super::RecipeDraft {
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

pub(super) fn entry(fixture: EntryFixture<'_>) -> super::CatalogEntry {
    super::CatalogEntry::new(super::CatalogEntryDraft {
        label: fixture.label,
        backend: fixture.backend,
        role: fixture.role,
        local: fixture.local,
    })
    .expect("valid catalog entry should parse")
}

pub(super) fn catalog() -> alloc::vec::Vec<super::CatalogEntry> {
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
    ]
}

pub(super) fn budget() -> super::Budget {
    super::Budget::new(MAX_CANDIDATES_THREE, &[BACKEND_LOCAL, BACKEND_REMOTE])
        .expect("valid budget should parse")
}

pub(super) fn signal_draft() -> super::SignalDraft<'static> {
    super::SignalDraft {
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

pub(super) fn signals(draft: super::SignalDraft<'_>) -> super::Signals {
    super::Signals::project(draft).expect("valid signals should parse")
}
