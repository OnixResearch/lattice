mod envelope;
mod gate;

type Reason = super::Reason;
type ServiceOutcome<T> = super::ServiceOutcome<T>;

pub use envelope::{
    ConfirmationBinding, ConfirmationBindingDraft, ConfirmationRequest, ConfirmationRequestDraft,
};
pub use gate::{
    ConfirmationActionKind, ConfirmationBrokerClass, ConfirmationExpiry, ConfirmationExpiryClass,
    ConfirmationIssue, ConfirmationLevel, ConfirmationReceipt, ConfirmationReceiptDraft,
    ConfirmationReceiptLeak, ConfirmationStatus, ConfirmationTick, ConfirmationTimingClass,
    evaluate_confirmation_decision, evaluate_confirmation_outcome,
    validate_confirmation_receipt_text,
};

/// Decision returned by an injected confirmation broker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfirmationDecision {
    /// Protected action is approved for the exact bound request.
    Approved {
        /// Approval binding echoed by the broker.
        binding: ConfirmationBinding,
    },
    /// Protected action is denied.
    Denied {
        /// Reason class for denial.
        reason: super::Reason,
    },
    /// Broker returned no actionable decision.
    NoDecision,
}

/// Pure confirmation evaluation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationEvaluation {
    status: ConfirmationStatus,
    issue: ConfirmationIssue,
    receipt: ConfirmationReceipt,
}

impl ConfirmationEvaluation {
    /// Returns final confirmation status.
    #[must_use]
    pub const fn status(&self) -> ConfirmationStatus {
        self.status
    }

    /// Returns issue code.
    #[must_use]
    pub const fn issue(&self) -> ConfirmationIssue {
        self.issue
    }

    /// Returns true only when the protected action may dispatch.
    #[must_use]
    pub const fn dispatch_allowed(&self) -> bool {
        matches!(self.status, ConfirmationStatus::Approved)
            && matches!(self.issue, ConfirmationIssue::None)
    }

    /// Returns the safe confirmation receipt.
    #[must_use]
    pub const fn receipt(&self) -> &ConfirmationReceipt {
        &self.receipt
    }
}
