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

pub use binding::ModelInvocationIdentitySnapshot;
pub use binding::ModelInvocationTransitionDecision;
pub use binding::ModelInvocationTransitionKind;
pub use binding::ModelInvocationTransitionRequest;
pub use binding::ModelToolCallBinding;
pub use binding::ModelToolCallBindingDraft;
pub use binding::ProfileBoundModelDispatchPlan;
pub use binding::bind_model_dispatch_plan;
pub use binding::evaluate_model_invocation_transition;
pub use binding::validate_model_tool_call_binding;
pub use composition::ModelInvocationAuthority;
pub use composition::ModelInvocationCompositionInput;
pub use composition::ModelInvocationDiagnostic;
pub use composition::ModelInvocationOmissionClass;
pub use composition::ModelInvocationPlan;
pub use composition::ModelResourceCeiling;
pub use composition::ModelSkillCompositionResource;
pub use composition::ModelSkillPlanEntry;
pub use composition::ModelToolDeclaration;
pub use composition::compose_model_invocation_plan;
pub use receipt::ModelInvocationReceipt;
pub use receipt::ModelInvocationReceiptLeak;
pub use receipt::ModelInvocationReceiptSensitiveDraft;
pub use receipt::ModelInvocationSkillReceiptEntry;
pub use receipt::ModelInvocationToolReceiptEntry;
pub use receipt::project_model_invocation_receipt;
pub use resolution::resolve_model_runtime_profile;
pub use types::MODEL_INVOCATION_PLAN_HASH_DOMAIN;
pub use types::MODEL_INVOCATION_RECEIPT_HASH_DOMAIN;
pub use types::MODEL_RUNTIME_PROFILE_HASH_DOMAIN;
pub use types::MODEL_RUNTIME_PROFILE_SCHEMA;
pub use types::MODEL_RUNTIME_RESOLUTION_HASH_DOMAIN;
pub use types::ModelMemoryAccessMode;
pub use types::ModelNamedResourceReference;
pub use types::ModelNamedResourceReferenceDraft;
pub use types::ModelProfileAdmissionStatus;
pub use types::ModelProfileEnablementPolicy;
pub use types::ModelProfileSourceScope;
pub use types::ModelProfileSpecificity;
pub use types::ModelProfileTrustStatus;
pub use types::ModelPromptSourceReference;
pub use types::ModelPromptSourceReferenceDraft;
pub use types::ModelRuntimeBinding;
pub use types::ModelRuntimeBindingDraft;
pub use types::ModelRuntimeBudgetPolicy;
pub use types::ModelRuntimeBudgetPolicyDraft;
pub use types::ModelRuntimeProfile;
pub use types::ModelRuntimeProfileDraft;
pub use types::ModelRuntimeProfileError;
pub use types::ModelRuntimeProfileIssue;
pub use types::ModelRuntimeProfileResolution;
pub use types::ModelRuntimeProfileSet;
pub use types::ModelRuntimeProfileSetDraft;
pub use types::ModelRuntimeTarget;
pub use types::ModelRuntimeTargetDraft;
pub use types::ModelSkillReference;
pub use types::ModelSkillReferenceDraft;
pub use types::ModelSkillResourcePosture;
pub use types::ModelToolset;
pub use types::ModelToolsetDraft;
pub use types::NativeToolCallingMode;
pub use types::ResolvedModelRuntimeProfile;
pub use validation::blake3_model_runtime_profile_identity;

#[cfg(test)]
pub(crate) mod tests;
