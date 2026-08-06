/// Injected tool executor availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutorStatus {
    /// A host executor is available.
    Available,
    /// No executor was injected for the known tool.
    Missing,
    /// The executor service exists but is unavailable.
    Unavailable,
}

/// Capability policy result for the tool call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityStatus {
    /// Active policy admits the call.
    Allowed,
    /// Active policy denies the call.
    Denied,
}

/// Output boundary status before dispatch finalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputBoundaryStatus {
    /// Output handling is bounded.
    Bounded,
    /// Executor output reference or handle is unsafe.
    UnsafeReference,
    /// Executor output is oversized without a bounded contract.
    UnboundedOversized,
}

/// Typed tool dispatch or execution failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureClass {
    /// Tool definition or arguments were invalid.
    InvalidInput,
    /// No executor was injected for the known tool.
    MissingExecutor,
    /// The injected executor service was unavailable.
    UnavailableService,
    /// Capability policy denied the call.
    DeniedCapability,
    /// Approval policy denied the call.
    ApprovalDenied,
    /// The call was cancelled.
    Cancelled,
    /// The call timed out.
    TimedOut,
    /// The executor returned malformed output.
    MalformedOutput,
    /// Output reference was unsafe.
    UnsafeOutputReference,
    /// Output was oversized and not bounded.
    UnboundedOversizedOutput,
    /// Declared mutation targets conflicted.
    MutationConflict,
    /// Fallback was requested and denied.
    FallbackDenied,
    /// Execution failed after dispatch.
    ExecutionFailure,
}

/// Borrowed dispatch readiness inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchRequest<'a> {
    /// Known validated definition, when lookup succeeded.
    pub definition: Option<&'a super::Definition>,
    /// Injected executor status.
    pub executor_status: ExecutorStatus,
    /// Capability decision status.
    pub capability_status: CapabilityStatus,
    /// Output-boundary status.
    pub output_boundary_status: OutputBoundaryStatus,
}

/// Tool dispatch readiness status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchStatus {
    /// Dispatch may call the injected executor.
    Ready,
    /// Dispatch failed before executor fallback.
    Failed,
}

/// Pure dispatch decision with explicit no-fallback evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchDecision {
    status: DispatchStatus,
    failure_class: Option<FailureClass>,
    fallback_attempted: bool,
}

impl DispatchDecision {
    /// Returns dispatch status.
    #[must_use]
    pub const fn status(self) -> DispatchStatus {
        self.status
    }

    /// Returns the failure class, if dispatch failed.
    #[must_use]
    pub const fn failure_class(self) -> Option<FailureClass> {
        self.failure_class
    }

    /// Returns whether a fallback process, shell, or provider was attempted.
    #[must_use]
    pub const fn fallback_attempted(self) -> bool {
        self.fallback_attempted
    }
}

/// Evaluates tool dispatch preconditions without executing tools.
#[must_use]
pub const fn evaluate_dispatch(request: DispatchRequest<'_>) -> DispatchDecision {
    match dispatch_failure(request) {
        Some(failure_class) => failed_dispatch(failure_class),
        None => ready_dispatch(),
    }
}

const fn dispatch_failure(request: DispatchRequest<'_>) -> Option<FailureClass> {
    if request.definition.is_none() {
        return Some(FailureClass::InvalidInput);
    }
    match (request.capability_status, request.executor_status, request.output_boundary_status) {
        (CapabilityStatus::Denied, _, _) => Some(FailureClass::DeniedCapability),
        (_, ExecutorStatus::Missing, _) => Some(FailureClass::MissingExecutor),
        (_, ExecutorStatus::Unavailable, _) => Some(FailureClass::UnavailableService),
        (_, _, OutputBoundaryStatus::UnsafeReference) => Some(FailureClass::UnsafeOutputReference),
        (_, _, OutputBoundaryStatus::UnboundedOversized) => Some(FailureClass::UnboundedOversizedOutput),
        _ready => None,
    }
}

const fn failed_dispatch(failure_class: FailureClass) -> DispatchDecision {
    DispatchDecision {
        status: DispatchStatus::Failed,
        failure_class: Some(failure_class),
        fallback_attempted: false,
    }
}

const fn ready_dispatch() -> DispatchDecision {
    DispatchDecision {
        status: DispatchStatus::Ready,
        failure_class: None,
        fallback_attempted: false,
    }
}
