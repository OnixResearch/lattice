//! Typed prompt context-reference contracts and pure validation helpers.

#![allow(non_trait_imports)]

mod base;
mod data;
mod family;
mod flow;
mod host;
mod profiles;
mod scan;
#[cfg(test)]
mod tests;

pub use data::{
    ContextLineRange, ContextOversizedPolicy, ContextReference, ContextReferenceBudget,
    ContextReferenceCounts, ContextReferenceError, ContextReferenceFragment, ContextReferenceKind,
    ContextReferencePolicy, ContextReferencePolicyDecision, ContextReferenceProvenance,
    ContextReferenceProvenanceDraft, ContextReferenceStatus, ContextRenderMode,
    ContextResolutionPlan, ContextResolverKind, ContextSourceTrust, ContextTruncationState,
    ResolvedContextCandidate,
};
pub use family::{
    ContextAdapterAvailability, ContextAdapterOutcomeClass, ContextAdapterPolicyDecision,
    ContextAdapterRole, ContextAdapterRoleDraft, ContextAdapterRoleRecord,
    validate_context_resolver_role,
};
/// Validated context adapter family manifest for public prompt contracts.
pub type ContextAdapterFamilyManifest = family::Manifest;
pub use flow::{apply_context_budget, expand_context_reference, plan_context_resolution};
pub use host::{
    ContextResolverSet, DiffContextResolver, DirectoryContextResolver, FileContextResolver,
    ImageContextResolver, UrlContextResolver,
};
/// Prompt profile assembly report.
pub type ProfileAssembly = profiles::Assembly;
/// Prompt profile assembly diagnostic.
pub type ProfileAssemblyDiagnostic = profiles::AssemblyDiagnostic;
/// Prompt profile assembly policy.
pub type ProfileAssemblyPolicy = profiles::AssemblyPolicy;
/// Prompt profile capability label.
pub type ProfileCapability = profiles::Capability;
/// Prompt profile capability binding.
pub type ProfileCapabilityBinding = profiles::CapabilityBinding;
/// Prompt profile capability binding status.
pub type ProfileCapabilityBindingStatus = profiles::BindingStatus;
/// Prompt profile chain decision.
pub type ProfileChainDecision = profiles::ChainDecision;
/// Prompt profile chain plan.
pub type ProfileChainPlan = profiles::ChainPlan;
/// Prompt profile chain request.
pub type ProfileChainRequest<'a> = profiles::ChainRequest<'a>;
/// Prompt profile descriptor.
pub type ProfileDescriptor = profiles::Descriptor;
/// Prompt profile descriptor draft.
pub type ProfileDescriptorDraft<'a> = profiles::DescriptorDraft<'a>;
/// Prompt profile directive diagnostic.
pub type ProfileDirectiveDiagnostic = profiles::Diagnostic;
/// Prompt profile directive issue.
pub type ProfileDirectiveIssue = profiles::Issue;
/// Prompt profile directive report.
pub type ProfileDirectiveReport = profiles::Report;
/// Prompt profile validation error.
pub type ProfileError = profiles::Error;
/// Prompt profile inclusion policy.
pub type ProfileInclusionPolicy = profiles::InclusionPolicy;
/// Prompt profile mode.
pub type ProfileMode = profiles::Mode;
/// Prompt profile mode binding.
pub type ProfileModeBinding = profiles::Binding;
/// Prompt profile mode binding request.
pub type ProfileModeBindingRequest<'a> = profiles::BindingRequest<'a>;
/// Prompt profile omission reason.
pub type ProfileOmittedReason = profiles::OmittedReason;
/// Prompt profile receipt.
pub type ProfileReceipt = profiles::Receipt;
/// Prompt profile receipt redaction marker.
pub type ProfileReceiptRedaction = profiles::ReceiptRedaction;
/// Prompt profile model-visible segment.
pub type ProfileSegment = profiles::Segment;
/// Prompt profile source admission.
pub type ProfileSourceAdmission = profiles::SourceAdmission;
/// Prompt profile source trust decision.
pub type ProfileSourceTrust = profiles::SourceTrust;
/// Prompt profile trust scope.
pub type ProfileTrustScope = profiles::TrustScope;

/// Assembles prompt profiles into model-visible prompt segments.
#[must_use]
pub fn assemble_profiles(
    profiles: &[ProfileDescriptor],
    policy: ProfileAssemblyPolicy,
    supported_chain_targets: &[ProfileMode],
) -> ProfileAssembly {
    profiles::assemble(profiles, policy, supported_chain_targets)
}

/// Resolves profile-requested mode metadata below host and runtime authority.
#[must_use]
pub fn bind_profile_mode(request: ProfileModeBindingRequest<'_>) -> ProfileModeBinding {
    profiles::bind_mode(request)
}

/// Parses first-line profile directives and strips them from body text.
#[must_use]
pub fn parse_profile_directives(
    body: &str,
    supported_chain_targets: &[ProfileMode],
) -> ProfileDirectiveReport {
    profiles::parse(body, supported_chain_targets)
}
pub use scan::parse_context_reference;
