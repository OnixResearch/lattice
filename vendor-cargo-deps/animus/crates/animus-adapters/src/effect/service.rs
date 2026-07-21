/// Returns a fail-closed denial when no confirmation broker is injected.
#[must_use]
pub fn default_deny_confirmation(
    request: &super::ConfirmationRequest,
) -> super::ServiceOutcome<super::ConfirmationDecision> {
    let receipt = super::Receipt::new(super::ReceiptDraft::new(
        super::ServiceKind::Confirmation,
        request.capability(),
        super::OutcomeKind::Denied,
        super::Reason::NotInjected,
    ));
    super::ServiceOutcome::denied(
        Some(super::ConfirmationDecision::Denied {
            reason: super::Reason::NotInjected,
        }),
        receipt,
    )
}

/// Model/provider dispatch service contract.
pub trait ModelProviderService:
    crate::ComponentAdapter<crate::ModelRequest, super::ServiceOutcome<crate::ModelResponse>>
{
}

impl<T> ModelProviderService for T where
    T: crate::ComponentAdapter<crate::ModelRequest, super::ServiceOutcome<crate::ModelResponse>>
{
}

/// Tool execution service contract.
pub trait ToolExecutionService:
    crate::ComponentAdapter<crate::ToolRequest, super::ServiceOutcome<crate::ToolResponse>>
{
}

impl<T> ToolExecutionService for T where
    T: crate::ComponentAdapter<crate::ToolRequest, super::ServiceOutcome<crate::ToolResponse>>
{
}

/// Host-owned effect service contract.
pub trait HostService:
    crate::ComponentAdapter<super::HostRequest, super::ServiceOutcome<super::Receipt>>
{
}

impl<T> HostService for T where
    T: crate::ComponentAdapter<super::HostRequest, super::ServiceOutcome<super::Receipt>>
{
}

/// Capability decision service contract.
pub trait CapabilityDecisionService:
    crate::ComponentAdapter<
        super::CapabilityDecisionRequest,
        super::ServiceOutcome<super::CapabilityDecision>,
    >
{
}

impl<T> CapabilityDecisionService for T where
    T: crate::ComponentAdapter<
            super::CapabilityDecisionRequest,
            super::ServiceOutcome<super::CapabilityDecision>,
        >
{
}

/// Cancellation service contract.
pub trait CancellationService:
    crate::ComponentAdapter<
        super::CancellationCheck,
        super::ServiceOutcome<super::CancellationDecision>,
    >
{
}

impl<T> CancellationService for T where
    T: crate::ComponentAdapter<
            super::CancellationCheck,
            super::ServiceOutcome<super::CancellationDecision>,
        >
{
}

/// Confirmation broker service contract.
pub trait ConfirmationBroker:
    crate::ComponentAdapter<
        super::ConfirmationRequest,
        super::ServiceOutcome<super::ConfirmationDecision>,
    >
{
}

impl<T> ConfirmationBroker for T where
    T: crate::ComponentAdapter<
            super::ConfirmationRequest,
            super::ServiceOutcome<super::ConfirmationDecision>,
        >
{
}

/// Runtime policy service contract.
pub trait RuntimePolicyService:
    crate::ComponentAdapter<
        super::RuntimePolicyRequest,
        super::ServiceOutcome<super::RuntimePolicyDecision>,
    >
{
}

impl<T> RuntimePolicyService for T where
    T: crate::ComponentAdapter<
            super::RuntimePolicyRequest,
            super::ServiceOutcome<super::RuntimePolicyDecision>,
        >
{
}
