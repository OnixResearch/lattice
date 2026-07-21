#![cfg_attr(animus_octet, allow(tigerstyle::module_file_count))]

mod approval;
mod audit;
mod authority;
mod exec_policy;
mod input;
mod permit;
mod result;
#[allow(unknown_lints, excessive_file_length)]
mod sandbox;
mod service;
mod taxonomy;

#[cfg(test)]
mod tests;

pub use approval::{
    ConfirmationActionKind, ConfirmationBinding, ConfirmationBindingDraft, ConfirmationBrokerClass,
    ConfirmationDecision, ConfirmationEvaluation, ConfirmationExpiry, ConfirmationExpiryClass,
    ConfirmationIssue, ConfirmationLevel, ConfirmationReceipt, ConfirmationReceiptDraft,
    ConfirmationReceiptLeak, ConfirmationRequest, ConfirmationRequestDraft, ConfirmationStatus,
    ConfirmationTick, ConfirmationTimingClass, evaluate_confirmation_decision,
    evaluate_confirmation_outcome, validate_confirmation_receipt_text,
};
pub use audit::{
    Receipt, ReceiptDraft, SAFE_HASH_HEX_CHARS, SAFE_RECEIPT_CONTRACT_IDENTITY, SafeHash,
};
pub use authority::{
    AuthorizationContext, AuthorizationIssue, AuthorizationLeak, AuthorizationReceipt,
    AuthorizationStatus, CapabilityPolicyContract, CapabilityRequest, CaveatSummary, GuardedAction,
    LocalCaveat, LocalCaveatDecision, LocalPolicyLayer, SandboxConstraint, SandboxState, UcanGrant,
    authorize_capability, validate_authorization_receipt_text,
};
pub use input::{CancellationCheck, CapabilityDecisionRequest, HostRequest, RuntimePolicyRequest};
/// Exec-policy command intent.
pub type ExecPolicyCommandIntent = exec_policy::CommandIntent;
/// Exec-policy command intent draft.
pub type ExecPolicyCommandIntentDraft<'a> = exec_policy::CommandIntentDraft<'a>;
/// Exec-policy decision.
pub type ExecPolicyDecision = exec_policy::Decision;
/// Exec-policy evaluation result.
pub type ExecPolicyEvaluation = exec_policy::Evaluation;
/// Exec-policy validation error.
pub type ExecPolicyError = exec_policy::Error;
/// Exec-policy request.
pub type ExecPolicyRequest<'a> = exec_policy::Request<'a>;
/// Exec-policy rule.
pub type ExecPolicyRule = exec_policy::Rule;
/// Exec-policy rule draft.
pub type ExecPolicyRuleDraft<'a> = exec_policy::RuleDraft<'a>;
/// Exec-policy matched-rule evidence.
pub type ExecPolicyRuleEvidence = exec_policy::RuleEvidence;
/// Exec-policy sandbox context.
pub type ExecPolicySandboxContext = exec_policy::SandboxContext;
/// Exec-policy sandbox context draft.
pub type ExecPolicySandboxContextDraft<'a> = exec_policy::SandboxContextDraft<'a>;
/// Exec-policy shell analysis result.
pub type ExecPolicyShellAnalysis = exec_policy::Analysis;
/// Exec-policy shell analysis kind.
pub type ExecPolicyShellAnalysisKind = exec_policy::AnalysisKind;

/// Evaluates exec policy for a sandbox escalation request.
#[must_use]
pub fn evaluate_exec_policy(request: ExecPolicyRequest<'_>) -> ExecPolicyEvaluation {
    exec_policy::evaluate(request)
}

/// Analyzes a shell wrapper without evaluating policy.
#[must_use]
pub fn analyze_exec_policy_shell_wrapper(
    command: &ExecPolicyCommandIntent,
) -> ExecPolicyShellAnalysis {
    exec_policy::analyze_wrapper(command)
}
/// Permission prompt decision.
pub type PermissionPromptDecision = permit::Decision;
/// Permission prompt decision draft.
pub type PermissionPromptDecisionDraft<'a> = permit::DecisionDraft<'a>;
/// Prompt decision outcome.
pub type PromptDecisionOutcome = permit::DecisionOutcome;
/// Permission decision reason.
pub type PermissionDecisionReason = permit::DecisionReason;
/// Permission decision status.
pub type PermissionDecisionStatus = permit::DecisionStatus;
/// Permission kernel receipt.
pub type PermissionKernelReceipt = permit::KernelReceipt;
/// Permission kernel receipt draft.
pub type PermissionKernelReceiptDraft<'a> = permit::KernelReceiptDraft<'a>;
/// Permission mode-default reason.
pub type PermissionModeDefaultReason = permit::ModeDefaultReason;
/// Permission mode-default request.
pub type PermissionModeDefaultRequest = permit::ModeDefaultRequest;
/// Permission mode-default resolution.
pub type PermissionModeDefaultResolution = permit::ModeDefaultResolution;
/// Permission UX mode.
pub type PermissionMode = permit::PermissionMode;
/// Permission pattern match kind.
pub type PermissionPatternMatchKind = permit::PatternMatchKind;
/// Permission pattern rule.
pub type PermissionPatternRule = permit::PatternRule;
/// Permission pattern rule draft.
pub type PermissionPatternRuleDraft<'a> = permit::PatternRuleDraft<'a>;
/// Permission policy validation error.
pub type PermissionPolicyError = permit::Error;
/// Permission local outcome.
pub type PermissionOutcome = permit::Outcome;
/// Permission policy descriptor.
pub type PermissionPolicy = permit::Policy;
/// Permission policy draft.
pub type PermissionPolicyDraft<'a> = permit::PolicyDraft<'a>;
/// Permission prompt request.
pub type PermissionPromptRequest = permit::PromptRequest;
/// Permission prompt request draft.
pub type PermissionPromptRequestDraft<'a> = permit::PromptRequestDraft<'a>;
/// Permission prompt status.
pub type PermissionPromptStatus = permit::PromptStatus;
/// Prompt summary policy.
pub type PromptSummaryPolicy = permit::PromptSummaryPolicy;
/// Repeated permission action input.
pub type PermissionRepeatedActionInput<'a> = permit::RepeatedActionInput<'a>;
/// Repeated permission action policy.
pub type PermissionRepeatedActionPolicy = permit::RepeatedActionPolicy;
/// Repeated permission action resolution.
pub type PermissionRepeatedActionResolution = permit::RepeatedActionResolution;
/// Repeated permission action status.
pub type PermissionRepeatedActionStatus = permit::RepeatedActionStatus;
/// Permission safe receipt.
pub type PermissionReceipt = permit::Receipt;
/// Permission receipt draft.
pub type PermissionReceiptDraft<'a> = permit::ReceiptDraft<'a>;
/// Permission receipt leak class.
pub type PermissionReceiptLeak = permit::ReceiptLeak;
/// Permission resolution.
pub type PermissionResolution = permit::Resolution;
/// Permission resolution request.
pub type PermissionResolutionRequest<'a> = permit::ResolutionRequest<'a>;
/// Permission scoped session allowlist entry.
pub type PermissionSessionAllowlistEntry = permit::SessionAllowlistEntry;
/// Permission scoped session allowlist entry draft.
pub type PermissionSessionAllowlistEntryDraft<'a> = permit::SessionAllowlistEntryDraft<'a>;
/// Permission scoped session allowlist status.
pub type PermissionSessionAllowlistStatus = permit::SessionAllowlistStatus;
/// Permission scoped session allowlist validation request.
pub type PermissionSessionAllowlistValidation<'a> = permit::SessionAllowlistValidation<'a>;
/// Permission trust scope.
pub type PermissionTrustScope = permit::TrustScope;
/// Permission trust status.
pub type PermissionTrustStatus = permit::TrustStatus;

/// Resolves permission policy after authority checks.
#[must_use]
pub fn resolve_permission(request: PermissionResolutionRequest<'_>) -> PermissionResolution {
    permit::resolve(request)
}

/// Resolves advisory permission-mode defaults below stronger authority gates.
#[must_use]
pub fn resolve_permission_mode_default(
    request: PermissionModeDefaultRequest,
) -> PermissionModeDefaultResolution {
    permit::resolve_mode_default(request)
}

/// Detects repeated permission action identities without dispatching adapters.
#[must_use]
pub fn detect_permission_repeated_action(
    input: PermissionRepeatedActionInput<'_>,
) -> PermissionRepeatedActionResolution {
    permit::detect_repeated_action(input)
}

/// Validates a scoped session allowlist token against the current request.
#[must_use]
pub fn validate_permission_session_allowlist(
    request: PermissionSessionAllowlistValidation<'_>,
) -> PermissionSessionAllowlistStatus {
    permit::validate_session_allowlist(request)
}

/// Validates rendered permission receipt text for unsafe sentinels.
///
/// # Errors
///
/// Returns [`PermissionReceiptLeak`] when unsafe material is present.
pub fn validate_permission_receipt_text(text: &str) -> Result<(), PermissionReceiptLeak> {
    permit::validate_text(text)
}
pub use result::{OutcomeKind, Reason, ServiceOutcome};
pub use sandbox::{
    SandboxConstraintClass, SandboxCowDisposition, SandboxDescriptor, SandboxDescriptorDraft,
    SandboxDeterminismPolicy, SandboxDispatchDecision, SandboxDispatchIssue,
    SandboxDispatchReceipt, SandboxDispatchReceiptDraft, SandboxDispatchStatus,
    SandboxEnforcementRequirement, SandboxEnvironmentPolicy, SandboxFilesystemMode,
    SandboxHttpPolicy, SandboxNetworkPolicy, SandboxPlatformCapability, SandboxPlatformSupport,
    SandboxPlatformSupportDraft, SandboxPolicy, SandboxPolicyDraft, SandboxProcessProfile,
    SandboxReceiptLeak, SandboxWorkdirMode, evaluate_sandbox_dispatch,
    validate_sandbox_receipt_text,
};
pub use service::{
    CancellationService, CapabilityDecisionService, ConfirmationBroker, HostService,
    ModelProviderService, RuntimePolicyService, ToolExecutionService, default_deny_confirmation,
};
pub use taxonomy::{CapabilityClass, RedactionMarker, ServiceKind};

/// Capability decision returned by an injected policy service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDecision {
    /// Capability is allowed.
    Allow,
    /// Capability is denied.
    Deny {
        /// Reason class for denial.
        reason: Reason,
    },
}

/// Cancellation decision returned by an injected cancellation service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancellationDecision {
    /// Continue evaluating the turn.
    Continue,
    /// Cancel before running later effects.
    Cancel {
        /// Reason class for cancellation.
        reason: Reason,
    },
}

/// Runtime-policy decision returned by an injected runtime-policy service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePolicyDecision {
    /// Runtime policy allows the request.
    Allow,
    /// Runtime policy denies the request.
    Deny {
        /// Reason class for denial.
        reason: Reason,
    },
}
