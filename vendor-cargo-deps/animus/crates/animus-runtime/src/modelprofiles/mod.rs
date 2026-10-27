//! Pure per-model runtime profile validation, resolution, composition, binding, and evidence.
//!
//! Hosts load Nickel exports, provider metadata, prompt sources, skills, memory
//! policies, tool catalogs, policy decisions, and executor readiness before calling this module.
//! The module performs no filesystem, environment, provider, process, network,
//! credential, clock, terminal, persistence, or executor effects.

mod binding;
mod composition;
mod receipt;
mod resolution;
mod types;
mod validation;

pub use binding::{
    ModelDialectTransitionPlan, ModelDialectTransitionRequest, ModelInvocationIdentitySnapshot,
    ModelInvocationTransitionDecision, ModelInvocationTransitionKind,
    ModelInvocationTransitionRequest, ModelToolCallBinding, ModelToolCallBindingDraft,
    ModelTransportDialectAdmissionDraft, ProfileBoundModelDispatchPlan, bind_model_dispatch_plan,
    evaluate_model_dialect_transition, evaluate_model_invocation_transition,
    validate_model_tool_call_binding,
};
pub use composition::{
    ModelInvocationAuthority, ModelInvocationCompositionInput, ModelInvocationDiagnostic,
    ModelInvocationOmissionClass, ModelInvocationPlan, ModelResourceCeiling,
    ModelSkillCompositionResource, ModelSkillPlanEntry, ModelToolDeclaration,
    compose_model_invocation_plan, select_steel_skill_modules,
};
pub use receipt::{
    ModelInvocationReceipt, ModelInvocationReceiptLeak, ModelInvocationReceiptSensitiveDraft,
    ModelInvocationSkillReceiptEntry, ModelInvocationToolReceiptEntry,
    project_model_invocation_receipt,
};
pub use resolution::resolve_model_runtime_profile;
pub use types::{
    MODEL_INVOCATION_PLAN_HASH_DOMAIN, MODEL_INVOCATION_RECEIPT_HASH_DOMAIN,
    MODEL_RUNTIME_PROFILE_HASH_DOMAIN, MODEL_RUNTIME_PROFILE_SCHEMA,
    MODEL_RUNTIME_RESOLUTION_HASH_DOMAIN, ModelMemoryAccessMode, ModelNamedResourceReference,
    ModelNamedResourceReferenceDraft, ModelProfileAdmissionStatus, ModelProfileEnablementPolicy,
    ModelProfileSourceScope, ModelProfileSpecificity, ModelProfileTrustStatus,
    ModelPromptSourceReference, ModelPromptSourceReferenceDraft, ModelRuntimeBinding,
    ModelRuntimeBindingDraft, ModelRuntimeBudgetPolicy, ModelRuntimeBudgetPolicyDraft,
    ModelRuntimeProfile, ModelRuntimeProfileDraft, ModelRuntimeProfileError,
    ModelRuntimeProfileIssue, ModelRuntimeProfileResolution, ModelRuntimeProfileSet,
    ModelRuntimeProfileSetDraft, ModelRuntimeTarget, ModelRuntimeTargetDraft, ModelSkillReference,
    ModelSkillReferenceDraft, ModelSkillResourcePosture, ModelToolset, ModelToolsetDraft,
    NativeToolCallingMode, ResolvedModelRuntimeProfile,
};
pub use validation::blake3_model_runtime_profile_identity;

#[cfg(test)]
pub(crate) mod tests;
