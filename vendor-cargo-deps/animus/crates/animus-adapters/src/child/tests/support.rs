use super::super::*;

pub(super) const CHILD_ID: &str = "child-review";
pub(super) const PARENT_ID: &str = "parent-turn";
pub(super) const ROLE: &str = "reviewer";
pub(super) const TASK: &str = "Review adapter contract changes.";
pub(super) const RECIPE_ID: &str = "goose.recipe.workflow";
pub(super) const PROFILE_ID: &str = "review-profile";
pub(super) const PARAM_TOPIC: &str = "topic=adapter contracts";
pub(super) const TOOL_READ: &str = "tool.read";
pub(super) const TOOL_WRITE: &str = "tool.write";
pub(super) const SECRET_READ: &str = "secret.read";
pub(super) const WORKSPACE_READONLY: &str = "readonly-workspace";
pub(super) const MODEL_PROFILE: &str = "standard-reasoning";
pub(super) const TIMEOUT: &str = "standard";
pub(super) const SUMMARY_SCHEMA: &str = "review-summary";
pub(super) const CONTENT_HASH: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const MAX_TURNS: u32 = 4;
pub(super) const ZERO_TURNS: u32 = 0;
pub(super) const DETAIL_CHARS: u32 = 10;
pub(super) const MAX_DETAIL_CHARS: u32 = 20;
pub(super) const EXCESSIVE_DETAIL_CHARS: u32 = 21;
pub(super) const EVENT_KIND_FIELD: &str = "subagent_event_kind";
pub(super) const BAD_EVENT_KIND: &str = "teleport";
pub(super) const FAILURE_EVIDENCE_REF: &str = "subagent-failure:b3:abc";
pub(super) const EVENT_STARTED: &str = "started";
pub(super) const EVENT_AWAITING_PARENT: &str = "awaiting-parent";
pub(super) const AGGREGATE_SUMMARY_REF: &str = "aggregate-summary";
pub(super) const CHILD_OK: &str = "child-ok";
pub(super) const CHILD_TIMEOUT: &str = "child-timeout";
pub(super) const CHILD_CONFLICT: &str = "child-conflict";

pub(super) fn admitted_recipe() -> crate::RecipeTrustReceipt {
    crate::RecipeTrustReceipt::new(&crate::RecipeProvenanceDraft {
        source_kind: "project",
        source_label: "config/recipe-workflows.ncl",
        content_hash: CONTENT_HASH,
        admitted: true,
    })
    .expect("recipe trust should parse")
}

pub(super) fn pending_recipe() -> crate::RecipeTrustReceipt {
    crate::RecipeTrustReceipt::new(&crate::RecipeProvenanceDraft {
        source_kind: "project",
        source_label: "config/recipe-workflows.ncl",
        content_hash: CONTENT_HASH,
        admitted: false,
    })
    .expect("recipe trust should parse")
}

pub(super) fn valid_delegation(recipe: &crate::RecipeTrustReceipt) -> SubagentDelegationDraft<'_> {
    SubagentDelegationDraft {
        id: CHILD_ID,
        role: ROLE,
        task: TASK,
        recipe_id: Some(RECIPE_ID),
        profile_id: Some(PROFILE_ID),
        parameter_bindings: &[PARAM_TOPIC],
        tool_ceilings: &[TOOL_READ],
        workspace_scope: WORKSPACE_READONLY,
        model_profile: MODEL_PROFILE,
        timeout_class: TIMEOUT,
        max_turns: MAX_TURNS,
        return_mode: RETURN_STATUS_PLUS_SUMMARY,
        summary_schema: SUMMARY_SCHEMA,
        delegation_source: SOURCE_USER,
        recursive: false,
        recipe_trust: Some(recipe),
        automation_admitted: false,
        external: None,
    }
}
