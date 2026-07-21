/// Policy composition diagnostic under the public facade naming scheme.
pub type PolicyCompositionDiagnostic = crate::profile::CompositionDiagnostic;
/// Policy composition report under the public facade naming scheme.
pub type PolicyCompositionReport = crate::profile::CompositionReport;
/// Policy composition diagnostic kind under the public facade naming scheme.
pub type PolicyCompositionDiagnosticKind = crate::profile::DiagnosticKind;
/// Policy/profile descriptor under the public facade naming scheme.
pub type PolicyProfile = crate::profile::Descriptor;
/// Borrowed policy/profile descriptor draft under the public facade naming scheme.
pub type PolicyProfileDraft<'a> = crate::profile::Draft<'a>;
/// Policy enforcement boundary under the public facade naming scheme.
pub type PolicyEnforcementBoundary = crate::profile::EnforcementBoundary;
/// Policy/profile validation error under the public facade naming scheme.
pub type PolicyProfileError = crate::profile::Error;
/// Policy/profile kind under the public facade naming scheme.
pub type PolicyProfileKind = crate::profile::Kind;
/// Project policy trust mode under the public facade naming scheme.
pub type ProjectPolicyTrustMode = crate::profile::ProjectTrustMode;
/// Policy prompt hygiene issue under the public facade naming scheme.
pub type PolicyPromptHygieneIssue = crate::profile::PromptHygieneIssue;
/// Policy prompt segment under the public facade naming scheme.
pub type PolicyPromptSegment = crate::profile::PromptSegment;
/// Runtime guard requirement under the public facade naming scheme.
pub type RuntimeGuardRequirement = crate::profile::RuntimeGuardRequirement;
/// Policy source scope under the public facade naming scheme.
pub type PolicySourceScope = crate::profile::SourceScope;
/// Policy trust status under the public facade naming scheme.
pub type PolicyTrustStatus = crate::profile::TrustStatus;
/// Composes trusted policy profiles into attributed prompt segments.
#[must_use]
pub fn compose_policy_context(
    profiles: &[PolicyProfile],
    project_trust_mode: ProjectPolicyTrustMode,
) -> PolicyCompositionReport {
    crate::profile::compose_context(profiles, project_trust_mode)
}
