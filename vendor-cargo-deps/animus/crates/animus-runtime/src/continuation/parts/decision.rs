
/// Selected continuation dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationDispatch {
    work_id: Text,
    action: ContinuationAction,
    reason: ContinuationReason,
}

impl ContinuationDispatch {
    /// Returns the selected work ID.
    #[must_use]
    pub fn work_id(&self) -> &str {
        self.work_id.as_str()
    }

    /// Returns the selected action.
    #[must_use]
    pub fn action(&self) -> ContinuationAction {
        self.action
    }

    /// Returns the selection reason.
    #[must_use]
    pub fn reason(&self) -> ContinuationReason {
        self.reason
    }

    /// Returns the selected work ID hash.
    #[must_use]
    pub fn work_id_hash(&self) -> Text {
        blake3_hex(self.work_id.as_bytes())
    }
}

// r[impl agent_harness_runtime.continuation_policy.priority]
/// Pure continuation decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContinuationDecision {
    /// Dispatch a shell-owned follow-up action.
    Dispatch(ContinuationDispatch),
    /// Await host-owned completion evidence because no dispatchable work remains.
    AwaitingCompletionEvidence {
        /// Safe reason for awaiting evidence.
        reason: ContinuationReason,
    },
    /// Finish because no continuation work remains.
    Complete {
        /// Safe reason for completion.
        reason: ContinuationReason,
    },
    /// Cancel continuation work.
    Cancel {
        /// Opaque cancellation work ID.
        work_id: Text,
        /// Safe reason for cancellation.
        reason: ContinuationReason,
    },
}

// r[impl agent_harness_runtime.authoritative_completion.continuation]
impl ContinuationDecision {
    /// Returns the selected action class.
    #[must_use]
    pub fn action(&self) -> ContinuationAction {
        match self {
            Self::Dispatch(dispatch) => dispatch.action(),
            Self::AwaitingCompletionEvidence { .. } => {
                ContinuationAction::AwaitCompletionEvidence
            }
            Self::Complete { .. } => ContinuationAction::Complete,
            Self::Cancel { .. } => ContinuationAction::Cancel,
        }
    }

    /// Returns the selected reason.
    #[must_use]
    pub fn reason(&self) -> ContinuationReason {
        match self {
            Self::Dispatch(dispatch) => dispatch.reason(),
            Self::AwaitingCompletionEvidence { reason }
            | Self::Complete { reason }
            | Self::Cancel { reason, .. } => *reason,
        }
    }

    /// Returns the selected work ID, when the action has one.
    #[must_use]
    pub fn work_id(&self) -> Option<&str> {
        match self {
            Self::Dispatch(dispatch) => Some(dispatch.work_id()),
            Self::Cancel { work_id, .. } => Some(work_id.as_str()),
            Self::AwaitingCompletionEvidence { .. } | Self::Complete { .. } => None,
        }
    }

    /// Returns a BLAKE3 work ID hash or a stable no-work hash.
    #[must_use]
    pub fn pending_work_id_hash(&self) -> Text {
        self.work_id().map_or_else(
            || blake3_hex(EMPTY_WORK_HASH_MARKER.as_bytes()),
            |work_id| blake3_hex(work_id.as_bytes()),
        )
    }
}

/// Dispatch acknowledgement supplied by a shell after attempting dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchAcknowledgement<'a> {
    /// Work ID the shell accepted or rejected.
    pub work_id: &'a str,
    /// Dispatch outcome.
    pub outcome: DispatchAcknowledgementOutcome,
}

/// Shell dispatch acknowledgement outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchAcknowledgementOutcome {
    /// The shell accepted dispatch and work became busy.
    Accepted,
    /// The shell rejected dispatch without clearing queued work.
    Rejected,
}

/// Completion acknowledgement supplied by a shell-owned worker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionAcknowledgement<'a> {
    /// Work ID that completed.
    pub work_id: &'a str,
    /// Action class that completed.
    pub action: ContinuationAction,
}

