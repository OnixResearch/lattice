/// Agent-definition ability class.
pub type AgentDefinitionAbilityKind = crate::actor::AbilityKind;
/// Agent-definition activation result.
pub type AgentDefinitionActivation = crate::actor::Activation;
/// Agent-definition activation issue.
pub type AgentDefinitionActivationIssue = crate::actor::ActivationIssue;
/// Agent-definition activation policy.
pub type AgentDefinitionActivationPolicy<'a> = crate::actor::ActivationPolicy<'a>;
/// Agent-definition activation request.
pub type AgentDefinitionActivationRequest<'a> = crate::actor::ActivationRequest<'a>;
/// Agent-definition activation status.
pub type AgentDefinitionActivationStatus = crate::actor::ActivationStatus;
/// Agent-definition project admission input.
pub type AgentDefinitionAdmission<'a> = crate::actor::Admission<'a>;
/// Agent-definition project admission status.
pub type AgentDefinitionAdmissionStatus = crate::actor::AdmissionStatus;
/// Agent-definition authority boundary.
pub type AgentDefinitionAuthorityBoundary = crate::actor::AuthorityBoundary;
/// Agent-definition denied escalation.
pub type AgentDefinitionDeniedEscalation = crate::actor::DeniedEscalation;
/// Agent-definition discovery diagnostic.
pub type AgentDefinitionDiagnostic = crate::actor::Diagnostic;
/// Agent-definition discovery issue.
pub type AgentDefinitionDiscoveryIssue = crate::actor::DiscoveryIssue;
/// Agent-definition discovery report.
pub type AgentDefinitionDiscoveryReport = crate::actor::DiscoveryReport;
/// Agent-definition manifest draft.
pub type AgentDefinitionDraft<'a> = crate::actor::Draft<'a>;
/// Agent-definition validation error.
pub type AgentDefinitionError = crate::actor::Error;
/// Validated agent-definition manifest.
pub type AgentDefinitionManifest = crate::actor::Manifest;
/// Agent-definition safe receipt.
pub type AgentDefinitionReceipt = crate::actor::Receipt;
/// Agent-definition receipt draft.
pub type AgentDefinitionReceiptDraft<'a> = crate::actor::ReceiptDraft<'a>;
/// Agent-definition receipt leak class.
pub type AgentDefinitionReceiptLeak = crate::actor::ReceiptLeak;
/// Agent-definition source scope.
pub type AgentDefinitionSourceScope = crate::actor::SourceScope;
/// Agent-definition trust status.
pub type AgentDefinitionTrustStatus = crate::actor::TrustStatus;

/// Evaluates an agent-definition activation request.
#[must_use]
pub fn activate_agent_definition(
    request: AgentDefinitionActivationRequest<'_>,
) -> AgentDefinitionActivation {
    crate::actor::activate(request)
}

/// Discovers agent definitions deterministically from loaded manifests.
#[must_use]
pub fn discover_agent_definitions(
    manifests: &[AgentDefinitionManifest],
) -> AgentDefinitionDiscoveryReport {
    crate::actor::discover(manifests)
}

/// Validates rendered agent-definition receipt text for unsafe sentinels.
///
/// # Errors
///
/// Returns [`AgentDefinitionReceiptLeak`] when unsafe material is present.
pub fn validate_agent_definition_receipt_text(
    text: &str,
) -> Result<(), AgentDefinitionReceiptLeak> {
    crate::actor::validate_text(text)
}
