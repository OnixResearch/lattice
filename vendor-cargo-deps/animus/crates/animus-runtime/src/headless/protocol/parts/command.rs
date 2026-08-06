use super::{Collection, Text};

const REQUEST_ID_FIELD: &str = "request_id";
const COMMAND_KIND_FIELD: &str = "command_kind";
const COMMAND_PAYLOAD_FIELD: &str = "command_payload";
const QUEUEING_FIELD: &str = "queueing";
const ERROR_MESSAGE_FIELD: &str = "error_message";
const EVENT_SEQUENCE_FIELD: &str = "protocol_event_sequence";
const RECORD_FIELD_COUNT: usize = 4;
const FIELD_SEPARATOR: char = '|';
const LF_DELIMITER: char = '\n';
const CR_TRAILER: char = '\r';
const TURN_ID_PREFIX: &str = "turn-";
const TOOL_ID_PREFIX: &str = "tool:";
const MODEL_REQUEST_SUMMARY: &str = "model_request";
const MODEL_RESPONSE_SUMMARY: &str = "model_response";
const TOOL_REQUEST_SUMMARY: &str = "tool_request";
const TOOL_RESPONSE_SUMMARY: &str = "tool_response";
const FINAL_RESPONSE_SUMMARY: &str = "final_response";
const COMPLETION_CANDIDATE_SUMMARY: &str = "completion_candidate";
const STOPPED_SUMMARY: &str = "stopped";
const MALFORMED_RECORD_SUMMARY: &str = "malformed_record";
const AMBIGUOUS_ACTIVE_PROMPT_SUMMARY: &str = "ambiguous_active_prompt";
const ACCEPTED_STATUS: &str = "accepted";
const REJECTED_STATUS: &str = "rejected";
const COMMAND_PROMPT: &str = "prompt";
const COMMAND_STEER: &str = "steer";
const COMMAND_FOLLOW_UP: &str = "follow_up";
const COMMAND_ABORT: &str = "abort";
const COMMAND_GET_STATE: &str = "get_state";
const COMMAND_GET_MESSAGES: &str = "get_messages";
const COMMAND_SELECT_MODEL: &str = "select_model";
const COMMAND_LEDGER: &str = "ledger";
const COMMAND_SESSION: &str = "session";
const COMMAND_COMPACT: &str = "compact";
const COMMAND_SHUTDOWN: &str = "shutdown";
const QUEUEING_NEW_TURN: &str = "new_turn";
const QUEUEING_STEER: &str = "steer";
const QUEUEING_FOLLOW_UP: &str = "follow_up";
const ERROR_PARSE: &str = "parse";
const ERROR_INVALID_COMMAND: &str = "invalid_command";
const ERROR_INVALID_STATE: &str = "invalid_state";
const ERROR_UNAVAILABLE_SERVICE: &str = "unavailable_service";
const ERROR_CANCELLED: &str = "cancelled";
const ERROR_RUNTIME_FAILURE: &str = "runtime_failure";
const EVENT_ACK: &str = "ack";
const EVENT_RUNTIME: &str = "runtime";
const LEAK_RAW_PROMPT: &str = "raw_prompt:";
const LEAK_PROVIDER_PAYLOAD: &str = "provider payload";
const LEAK_CREDENTIAL: &str = "secret=";
const LEAK_AUTHORIZATION: &str = "authorization:";
const LEAK_FULL_PATH: &str = "/home/";
const LEAK_UNBOUNDED_OUTPUT: &str = "unbounded_output:";

/// Borrowed headless command fields before protocol validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessCommandDraft<'a> {
    /// Request correlation identity supplied by the client.
    pub request_id: &'a str,
    /// Command kind label.
    pub kind: &'a str,
    /// Optional queueing semantics for prompt-like commands.
    pub queueing: Option<&'a str>,
    /// Bounded command payload label or summary.
    pub payload: &'a str,
}

/// Validated headless protocol command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadlessCommand {
    request_id: Text,
    kind: HeadlessCommandKind,
    queueing: Option<HeadlessQueueing>,
    payload: Text,
}

impl HeadlessCommand {
    /// Validates a borrowed command without dispatching runtime work.
    ///
    /// # Errors
    ///
    /// Returns [`HeadlessProtocolError`] when the request ID, kind, queueing
    /// label, or payload is empty, unsupported, or unsafe.
    pub fn validate(draft: &HeadlessCommandDraft<'_>) -> Result<Self, HeadlessProtocolError> {
        let request_id = safe_text(REQUEST_ID_FIELD, draft.request_id)?;
        let kind = HeadlessCommandKind::parse(draft.kind)?;
        let queueing = draft.queueing.map(HeadlessQueueing::parse).transpose()?;
        let payload = safe_text(COMMAND_PAYLOAD_FIELD, draft.payload)?;
        Ok(Self {
            request_id,
            kind,
            queueing,
            payload,
        })
    }

    /// Returns the request correlation identity.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.request_id.as_str()
    }

    /// Returns the command kind.
    #[must_use]
    pub const fn kind(&self) -> HeadlessCommandKind {
        self.kind
    }

    /// Returns optional queueing semantics.
    #[must_use]
    pub const fn queueing(&self) -> Option<HeadlessQueueing> {
        self.queueing
    }

    /// Returns the bounded payload label or summary.
    #[must_use]
    pub fn payload(&self) -> &str {
        self.payload.as_str()
    }
}

/// Headless command kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessCommandKind {
    /// Start a prompt turn when idle.
    Prompt,
    /// Steer an active turn.
    Steer,
    /// Queue a follow-up after the active turn.
    FollowUp,
    /// Abort active work.
    Abort,
    /// Inspect protocol/runtime state.
    GetState,
    /// Inspect bounded message metadata.
    GetMessages,
    /// Select a model or route label.
    SelectModel,
    /// Operate on ledger state through host-owned services.
    Ledger,
    /// Operate on session state through host-owned services.
    Session,
    /// Request compaction through host-owned services.
    Compact,
    /// Request shutdown.
    Shutdown,
}

impl HeadlessCommandKind {
    fn parse(value: &str) -> Result<Self, HeadlessProtocolError> {
        match value {
            COMMAND_PROMPT => Ok(Self::Prompt),
            COMMAND_STEER => Ok(Self::Steer),
            COMMAND_FOLLOW_UP => Ok(Self::FollowUp),
            COMMAND_ABORT => Ok(Self::Abort),
            COMMAND_GET_STATE => Ok(Self::GetState),
            COMMAND_GET_MESSAGES => Ok(Self::GetMessages),
            COMMAND_SELECT_MODEL => Ok(Self::SelectModel),
            COMMAND_LEDGER => Ok(Self::Ledger),
            COMMAND_SESSION => Ok(Self::Session),
            COMMAND_COMPACT => Ok(Self::Compact),
            COMMAND_SHUTDOWN => Ok(Self::Shutdown),
            _unsupported => Err(HeadlessProtocolError::new(
                None,
                HeadlessProtocolErrorKind::InvalidCommand,
                COMMAND_KIND_FIELD,
            )),
        }
    }

    /// Returns the stable command-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => COMMAND_PROMPT,
            Self::Steer => COMMAND_STEER,
            Self::FollowUp => COMMAND_FOLLOW_UP,
            Self::Abort => COMMAND_ABORT,
            Self::GetState => COMMAND_GET_STATE,
            Self::GetMessages => COMMAND_GET_MESSAGES,
            Self::SelectModel => COMMAND_SELECT_MODEL,
            Self::Ledger => COMMAND_LEDGER,
            Self::Session => COMMAND_SESSION,
            Self::Compact => COMMAND_COMPACT,
            Self::Shutdown => COMMAND_SHUTDOWN,
        }
    }

    const fn is_prompt_like(self) -> bool {
        matches!(self, Self::Prompt | Self::Steer | Self::FollowUp)
    }
}

/// Prompt queueing semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessQueueing {
    /// Start a new turn only when idle.
    NewTurn,
    /// Steer the active turn.
    Steer,
    /// Queue a follow-up after the active turn.
    FollowUp,
}

impl HeadlessQueueing {
    fn parse(value: &str) -> Result<Self, HeadlessProtocolError> {
        match value {
            QUEUEING_NEW_TURN => Ok(Self::NewTurn),
            QUEUEING_STEER => Ok(Self::Steer),
            QUEUEING_FOLLOW_UP => Ok(Self::FollowUp),
            _unsupported => Err(HeadlessProtocolError::new(
                None,
                HeadlessProtocolErrorKind::InvalidCommand,
                QUEUEING_FIELD,
            )),
        }
    }

    /// Returns the stable queueing label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NewTurn => QUEUEING_NEW_TURN,
            Self::Steer => QUEUEING_STEER,
            Self::FollowUp => QUEUEING_FOLLOW_UP,
        }
    }
}
