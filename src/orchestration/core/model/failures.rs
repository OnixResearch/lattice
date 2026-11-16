#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    InvalidSchemaVersion,
    EmptyDefinitionId,
    EmptyObjective,
    EmptyWorkflowName,
    InvalidContentHash,
    WorkflowRevisionMismatch,
    MissingApprover,
    DuplicateApprover,
    EmptyTriggerId,
    DuplicateTriggerId,
    MissingBudgetBound,
    BudgetExceeded,
    BudgetOverflow,
    DuplicateReservation,
    MissingReservation,
    InvalidLifecycleTransition,
    TerminalStateMutation,
    DispatchMismatch,
    ProposalMismatch,
    StaleApproval,
    UnauthorizedActor,
    CancellationRequired,
    RetryDisabled,
    RetryLimitExceeded,
    RetryEvidenceMissing,
    SuccessorValidationFailed,
    InvalidIdentity,
    ReplayMismatch,
    ReplayHeadMismatch,
    TriggerNotBound,
    DuplicateTrigger,
    SerializationFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for Diagnostic {}

pub(in crate::orchestration::core) fn identity_error(
    error: crate::orchestration::identity::AdmissionError,
) -> Diagnostic {
    diagnostic(DiagnosticCode::InvalidIdentity, error.to_string())
}

pub(in crate::orchestration::core) fn invalid_transition(message: impl Into<String>) -> Diagnostic {
    diagnostic(DiagnosticCode::InvalidLifecycleTransition, message)
}

pub(in crate::orchestration::core) fn diagnostic(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        message: message.into(),
    }
}
