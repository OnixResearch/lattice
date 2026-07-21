/// Machine-readable automation event kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomationEventKind {
    /// Thread start event.
    ThreadStarted,
    /// Turn start event.
    TurnStarted,
    /// Item lifecycle event.
    ItemCreated,
    /// Prompt assembly event.
    PromptAssembly,
    /// Model dispatch event.
    ModelDispatch,
    /// Model stream start event.
    ModelStreamStart,
    /// Model stream frame event.
    ModelStreamFrame,
    /// Model stream end event.
    ModelStreamEnd,
    /// Model stream failure event.
    ModelStreamFailure,
    /// Tool dispatch event.
    ToolDispatch,
    /// Policy decision event.
    PolicyDecision,
    /// Background work event.
    BackgroundWork,
    /// Validation event.
    Validation,
    /// Receipt event.
    Receipt,
    /// Final response event.
    FinalResponse,
    /// Hook outcome event.
    HookOutcome,
    /// Usage accounting event.
    Usage,
    /// Terminal completion event containing the final response body.
    Completion,
    /// Terminal failure event.
    Failure,
}

impl AutomationEventKind {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            super::super::KIND_THREAD_STARTED => Some(Self::ThreadStarted),
            super::super::KIND_TURN_STARTED => Some(Self::TurnStarted),
            super::super::KIND_ITEM_CREATED => Some(Self::ItemCreated),
            super::super::KIND_PROMPT_ASSEMBLY => Some(Self::PromptAssembly),
            super::super::KIND_MODEL_DISPATCH => Some(Self::ModelDispatch),
            super::super::KIND_MODEL_STREAM_START => Some(Self::ModelStreamStart),
            super::super::KIND_MODEL_STREAM_FRAME => Some(Self::ModelStreamFrame),
            super::super::KIND_MODEL_STREAM_END => Some(Self::ModelStreamEnd),
            super::super::KIND_MODEL_STREAM_FAILURE => Some(Self::ModelStreamFailure),
            super::super::KIND_TOOL_DISPATCH => Some(Self::ToolDispatch),
            super::super::KIND_POLICY_DECISION => Some(Self::PolicyDecision),
            super::super::KIND_BACKGROUND_WORK => Some(Self::BackgroundWork),
            super::super::KIND_VALIDATION => Some(Self::Validation),
            super::super::KIND_RECEIPT => Some(Self::Receipt),
            super::super::KIND_FINAL_RESPONSE => Some(Self::FinalResponse),
            super::super::KIND_HOOK_OUTCOME => Some(Self::HookOutcome),
            super::super::KIND_USAGE => Some(Self::Usage),
            super::super::KIND_COMPLETION => Some(Self::Completion),
            super::super::KIND_FAILURE => Some(Self::Failure),
            _unsupported => None,
        }
    }

    /// Returns the stable event-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ThreadStarted => super::super::KIND_THREAD_STARTED,
            Self::TurnStarted => super::super::KIND_TURN_STARTED,
            Self::ItemCreated => super::super::KIND_ITEM_CREATED,
            Self::PromptAssembly => super::super::KIND_PROMPT_ASSEMBLY,
            Self::ModelDispatch => super::super::KIND_MODEL_DISPATCH,
            Self::ModelStreamStart => super::super::KIND_MODEL_STREAM_START,
            Self::ModelStreamFrame => super::super::KIND_MODEL_STREAM_FRAME,
            Self::ModelStreamEnd => super::super::KIND_MODEL_STREAM_END,
            Self::ModelStreamFailure => super::super::KIND_MODEL_STREAM_FAILURE,
            Self::ToolDispatch => super::super::KIND_TOOL_DISPATCH,
            Self::PolicyDecision => super::super::KIND_POLICY_DECISION,
            Self::BackgroundWork => super::super::KIND_BACKGROUND_WORK,
            Self::Validation => super::super::KIND_VALIDATION,
            Self::Receipt => super::super::KIND_RECEIPT,
            Self::FinalResponse => super::super::KIND_FINAL_RESPONSE,
            Self::HookOutcome => super::super::KIND_HOOK_OUTCOME,
            Self::Usage => super::super::KIND_USAGE,
            Self::Completion => super::super::KIND_COMPLETION,
            Self::Failure => super::super::KIND_FAILURE,
        }
    }
}

/// Safe event-body redaction marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventRedaction {
    /// Body is safe to display in bounded form.
    Visible,
    /// Body was redacted before projection.
    Redacted,
    /// Projection should rely on hash and metadata only.
    HashOnly,
}

impl EventRedaction {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            super::super::REDACTION_VISIBLE => Some(Self::Visible),
            super::super::REDACTION_REDACTED => Some(Self::Redacted),
            super::super::REDACTION_HASH_ONLY => Some(Self::HashOnly),
            _unsupported => None,
        }
    }

    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Visible => super::super::REDACTION_VISIBLE,
            Self::Redacted => super::super::REDACTION_REDACTED,
            Self::HashOnly => super::super::REDACTION_HASH_ONLY,
        }
    }
}
