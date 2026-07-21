/// Candidate classes that can be selected for continuation dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationCandidate {
    /// Replay user input that arrived while another prompt was busy.
    QueuedPrompt,
    /// Continue an explicit loop or workflow step.
    LoopContinuation,
    /// Run validation, auto-test, or other bounded follow-up work.
    ValidationFollowUp,
}

impl ContinuationCandidate {
    #[must_use]
    fn action(self) -> ContinuationAction {
        match self {
            Self::QueuedPrompt => ContinuationAction::ReplayQueuedPrompt,
            Self::LoopContinuation => ContinuationAction::ContinueLoop,
            Self::ValidationFollowUp => ContinuationAction::RunValidationFollowUp,
        }
    }

    #[must_use]
    fn reason(self) -> ContinuationReason {
        match self {
            Self::QueuedPrompt => ContinuationReason::QueuedUserInput,
            Self::LoopContinuation => ContinuationReason::ActiveLoopWork,
            Self::ValidationFollowUp => ContinuationReason::ValidationFollowUp,
        }
    }

    #[must_use]
    fn as_str(self) -> &'static str {
        match self {
            Self::QueuedPrompt => QUEUED_PROMPT_LABEL,
            Self::LoopContinuation => LOOP_CONTINUATION_LABEL,
            Self::ValidationFollowUp => VALIDATION_FOLLOW_UP_LABEL,
        }
    }
}

/// Typed work class stored in continuation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationWorkKind {
    /// User prompt queued while the runtime was busy.
    QueuedPrompt,
    /// Active loop or workflow continuation.
    LoopContinuation,
    /// Validation or auto-test follow-up.
    ValidationFollowUp,
    /// Work currently dispatched to a shell-owned prompt path.
    BusyPrompt,
    /// Cancellation request that preempts normal continuation work.
    Cancellation,
}

impl ContinuationWorkKind {
    /// Returns the stable work-kind label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QueuedPrompt => QUEUED_PROMPT_LABEL,
            Self::LoopContinuation => LOOP_CONTINUATION_LABEL,
            Self::ValidationFollowUp => VALIDATION_FOLLOW_UP_LABEL,
            Self::BusyPrompt => BUSY_PROMPT_LABEL,
            Self::Cancellation => CANCELLATION_LABEL,
        }
    }
}

/// Dispatch action selected by the pure continuation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationAction {
    /// Replay queued user input.
    ReplayQueuedPrompt,
    /// Continue loop work.
    ContinueLoop,
    /// Run a validation or auto-test follow-up.
    RunValidationFollowUp,
    /// No continuation work remains.
    Complete,
    /// Cancel current continuation work.
    Cancel,
}

impl ContinuationAction {
    /// Returns the stable action label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReplayQueuedPrompt => REPLAY_QUEUED_PROMPT_LABEL,
            Self::ContinueLoop => CONTINUE_LOOP_LABEL,
            Self::RunValidationFollowUp => RUN_VALIDATION_FOLLOW_UP_LABEL,
            Self::Complete => COMPLETE_LABEL,
            Self::Cancel => CANCEL_LABEL,
        }
    }

    fn parse(value: &str) -> Result<Self, ContinuationError> {
        match value {
            REPLAY_QUEUED_PROMPT_LABEL => Ok(Self::ReplayQueuedPrompt),
            CONTINUE_LOOP_LABEL => Ok(Self::ContinueLoop),
            RUN_VALIDATION_FOLLOW_UP_LABEL => Ok(Self::RunValidationFollowUp),
            COMPLETE_LABEL => Ok(Self::Complete),
            CANCEL_LABEL => Ok(Self::Cancel),
            _ => Err(ContinuationError::UnsupportedValue {
                field: FIELD_SELECTED_ACTION,
                value: Text::from(value),
            }),
        }
    }
}

/// Reason selected by the pure continuation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationReason {
    /// Queued user input has priority.
    QueuedUserInput,
    /// Active loop work is eligible.
    ActiveLoopWork,
    /// Validation or auto-test follow-up is eligible.
    ValidationFollowUp,
    /// No continuation work remains.
    NoPendingWork,
    /// Cancellation was requested.
    CancellationRequested,
}

impl ContinuationReason {
    /// Returns the stable reason label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QueuedUserInput => QUEUED_USER_INPUT_REASON,
            Self::ActiveLoopWork => ACTIVE_LOOP_WORK_REASON,
            Self::ValidationFollowUp => VALIDATION_FOLLOW_UP_REASON,
            Self::NoPendingWork => NO_PENDING_WORK_REASON,
            Self::CancellationRequested => CANCELLATION_REQUESTED_REASON,
        }
    }
}

/// Safe class of continuation state for receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationStateClass {
    /// No busy, pending-dispatch, or cancellation state is present.
    Idle,
    /// A shell-owned prompt dispatch is busy.
    Busy,
    /// A dispatch decision is waiting for shell acknowledgement.
    PendingDispatch,
    /// A cancellation request is present.
    Cancelled,
}

impl ContinuationStateClass {
    /// Returns the stable state-class label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => IDLE_LABEL,
            Self::Busy => BUSY_LABEL,
            Self::PendingDispatch => PENDING_DISPATCH_LABEL,
            Self::Cancelled => CANCELLED_LABEL,
        }
    }

    fn parse(value: &str) -> Result<Self, ContinuationError> {
        match value {
            IDLE_LABEL => Ok(Self::Idle),
            BUSY_LABEL => Ok(Self::Busy),
            PENDING_DISPATCH_LABEL => Ok(Self::PendingDispatch),
            CANCELLED_LABEL => Ok(Self::Cancelled),
            _ => Err(ContinuationError::UnsupportedValue {
                field: FIELD_PRIOR_STATE_CLASS,
                value: Text::from(value),
            }),
        }
    }
}

/// Pending continuation work with an opaque safe identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingContinuationWork {
    work_id: Text,
    kind: ContinuationWorkKind,
}

impl PendingContinuationWork {
    /// Builds typed pending continuation work.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `work_id` is empty, unsafe, or looks
    /// like leaked prompt/provider/tool/transcript data.
    pub fn new(work_id: &str, kind: ContinuationWorkKind) -> Result<Self, ContinuationError> {
        Ok(Self {
            work_id: validate_work_id(FIELD_WORK_ID, work_id)?,
            kind,
        })
    }

    /// Returns the opaque work identifier.
    #[must_use]
    pub fn work_id(&self) -> &str {
        self.work_id.as_str()
    }

    /// Returns the typed work kind.
    #[must_use]
    pub fn kind(&self) -> ContinuationWorkKind {
        self.kind
    }

    /// Returns the BLAKE3 work identifier hash.
    #[must_use]
    pub fn work_id_hash(&self) -> Text {
        blake3_hex(self.work_id.as_bytes())
    }
}
