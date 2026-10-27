//! Pure request-context contribution reduction and injected skill-turn loading.
//!
//! Existing prompt, skill, memory, project-context, MCP, and runtime owners
//! retain source admission. This module selects already-admitted metadata,
//! binds one exact model invocation, and keeps request-local bodies out of
//! durable evidence and storage.

mod core;
mod data;
mod evidence;
mod host;

/// Compatibility alias for the original request-boundary function.
pub use core::bind_skill_provider as bind_skill_provider_request;
pub use core::{
    admit_loaded_skill_body, admit_skill_turn_completion, advance_contribution_lifetime,
    begin_skill_turn, bind_skill_provider, binding_from_model_invocation, blake3_payload_identity,
    contribution_from_owner, expire_unconsumed_skill_turn, observe_skill_provider_response,
    reduce_contributions, reduce_required_contributions, validate_contribution,
    validate_invocation_binding, validate_model_invocation_binding, validate_selected_payloads,
};
pub use data::{
    AuthorityIdentity, Contribution, ContributionBinding, ContributionDraft, ContributionIdentity,
    ContributionIssue, ContributionLifetime, ContributionLifetimeState, ContributionOmission,
    ContributionOmissionClass, ContributionPayloadView, ContributionPlan, ContributionPlanIdentity,
    ContributionPolicy, ContributionPosture, ContributionReceipt, ContributionReceiptEntry,
    ContributionReceiptSensitiveDraft, ContributionReductionInput, ContributionSensitivity,
    ContributionSource, ContributionTerminalObservation, ContributionTrust, LoadedSkillBody,
    LogicalTurnIdentity, ModelInvocationIdentity, ModelProfileIdentity, ModelTargetIdentity,
    PayloadIdentity, PolicyIdentity, PromptPlanIdentity, ProviderRequestIdentity,
    ProviderResponseIdentity, ProviderTerminalOutcome, ReceiptIdentity, RetentionSink,
    SessionIdentity, SkillBodyLoad, SkillBodyLoadRequest, SkillContributionPolicy,
    SkillManifestIdentity, SkillProviderObservation, SkillProviderRequestObservation,
    SkillProviderResponseObservation, SkillSelectionBinding, SkillSelectionIdentity,
    SkillSelectionResult, SkillTurn, SkillTurnState,
};
pub use evidence::{
    admit_payload_retention, project_contribution_receipt, validate_contribution_receipt,
};
pub use host::{LoadedSkillTurn, SkillBodyService, SkillBodyServiceError, load_skill_turn};

#[cfg(test)]
mod tests;
