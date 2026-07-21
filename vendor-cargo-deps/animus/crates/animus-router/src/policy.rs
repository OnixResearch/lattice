//! Checked-in router policy bridge.

/// Checked-in generated router recipe policy JSON.
///
/// The JSON is produced from `policy/model-router/recipe-policy.ncl` by
/// `scripts/cairn/check-router-policy`; the router crate includes it as static
/// data and never evaluates Nickel at runtime.
pub const ROUTER_RECIPE_POLICY_JSON: &str =
    include_str!("../../../policy/model-router/generated/router-recipe-policy.json");

pub(crate) const FAMILY_SINGLE: &str = "single";
pub(crate) const FAMILY_CONFIDENCE: &str = "confidence";
pub(crate) const FAMILY_RATINGS: &str = "ratings";
pub(crate) const FAMILY_REMOM: &str = "remom";
pub(crate) const FAMILY_FUSION: &str = "fusion";
pub(crate) const FAMILY_WORKFLOW: &str = "workflow";

pub(crate) const ROLE_PRIMARY: &str = "primary";
pub(crate) const ROLE_ESCALATION: &str = "escalation";
pub(crate) const ROLE_RATER: &str = "rater";
pub(crate) const ROLE_SYNTHESIZER: &str = "synthesizer";
pub(crate) const ROLE_JUDGE: &str = "judge";
pub(crate) const ROLE_FINALIZER: &str = "finalizer";
pub(crate) const ROLE_WORKER: &str = "worker";
pub(crate) const ROLE_REPAIR: &str = "repair";
pub(crate) const ROLE_FALLBACK: &str = "fallback";

pub(crate) const CONTRACT_TEXT: &str = "text";
pub(crate) const CONTRACT_JSON: &str = "json";
pub(crate) const CONTRACT_TOOL_CALL: &str = "tool_call";
pub(crate) const CONTRACT_ARTIFACT: &str = "artifact";

pub(crate) const DEADLINE_SHORT: &str = "short";
pub(crate) const DEADLINE_STANDARD: &str = "standard";
pub(crate) const DEADLINE_EXTENDED: &str = "extended";

pub(crate) const FALLBACK_DENY: &str = "deny";
pub(crate) const FALLBACK_BEST_VALID_EVIDENCE: &str = "best_valid_evidence";
pub(crate) const FALLBACK_SAFE_SINGLE: &str = "safe_single";
