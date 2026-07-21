const QUEUE_CAPABILITY_EMPTY_MASK: u8 = 0;
const QUEUE_CAPABILITY_WAIT_BIT: u8 = 0;
const QUEUE_CAPABILITY_STREAM_BIT: u8 = 1;
const QUEUE_CAPABILITY_POLL_BIT: u8 = 2;
const QUEUE_CAPABILITY_SEND_INPUT_BIT: u8 = 3;
const QUEUE_CAPABILITY_WAIT_MASK: u8 = 1_u8 << QUEUE_CAPABILITY_WAIT_BIT;
const QUEUE_CAPABILITY_STREAM_MASK: u8 = 1_u8 << QUEUE_CAPABILITY_STREAM_BIT;
const QUEUE_CAPABILITY_POLL_MASK: u8 = 1_u8 << QUEUE_CAPABILITY_POLL_BIT;
const QUEUE_CAPABILITY_SEND_INPUT_MASK: u8 = 1_u8 << QUEUE_CAPABILITY_SEND_INPUT_BIT;
const QUEUE_CAPABILITY_ALL_MASK: u8 = QUEUE_CAPABILITY_WAIT_MASK
    | QUEUE_CAPABILITY_STREAM_MASK
    | QUEUE_CAPABILITY_POLL_MASK
    | QUEUE_CAPABILITY_SEND_INPUT_MASK;
const QUEUE_CAPABILITY_WAIT_STREAM_POLL_MASK: u8 =
    QUEUE_CAPABILITY_WAIT_MASK | QUEUE_CAPABILITY_STREAM_MASK | QUEUE_CAPABILITY_POLL_MASK;

/// Queue capabilities advertised by a host adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueCapabilities {
    mask: u8,
}

impl QueueCapabilities {
    /// Builds an empty capability set for queue interaction modes.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            mask: QUEUE_CAPABILITY_EMPTY_MASK,
        }
    }

    /// Builds a capability set with all interaction modes enabled.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            mask: QUEUE_CAPABILITY_ALL_MASK,
        }
    }

    /// Builds a capability set for wait, stream, and poll interactions.
    #[must_use]
    pub const fn wait_stream_poll() -> Self {
        Self {
            mask: QUEUE_CAPABILITY_WAIT_STREAM_POLL_MASK,
        }
    }

    /// Returns a copy with one capability enabled.
    #[must_use]
    pub const fn with(self, capability: QueueCapabilityKind) -> Self {
        Self {
            mask: self.mask | capability.mask(),
        }
    }

    /// Returns whether wait is supported.
    #[must_use]
    pub const fn wait(self) -> bool {
        self.supports(QueueCapabilityKind::Wait)
    }

    /// Returns whether stream is supported.
    #[must_use]
    pub const fn stream(self) -> bool {
        self.supports(QueueCapabilityKind::Stream)
    }

    /// Returns whether poll is supported.
    #[must_use]
    pub const fn poll(self) -> bool {
        self.supports(QueueCapabilityKind::Poll)
    }

    /// Returns whether send-input is supported.
    #[must_use]
    pub const fn send_input(self) -> bool {
        self.supports(QueueCapabilityKind::SendInput)
    }

    /// Returns whether a specific capability is supported.
    #[must_use]
    pub const fn supports(self, capability: QueueCapabilityKind) -> bool {
        self.mask & capability.mask() != QUEUE_CAPABILITY_EMPTY_MASK
    }
}

/// Optional queue capability requested by an interaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueCapabilityKind {
    /// Wait for task completion.
    Wait,
    /// Stream task progress.
    Stream,
    /// Poll task status.
    Poll,
    /// Send input to a task.
    SendInput,
}

impl QueueCapabilityKind {
    const fn mask(self) -> u8 {
        match self {
            Self::Wait => QUEUE_CAPABILITY_WAIT_MASK,
            Self::Stream => QUEUE_CAPABILITY_STREAM_MASK,
            Self::Poll => QUEUE_CAPABILITY_POLL_MASK,
            Self::SendInput => QUEUE_CAPABILITY_SEND_INPUT_MASK,
        }
    }
}

/// Queue interaction mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueInteractionKind {
    /// Wait for completion.
    Wait,
    /// Stream status or log progress.
    Stream,
    /// Poll current state.
    Poll,
    /// Send input through a backend-supported channel.
    SendInput,
}

impl QueueInteractionKind {
    const fn capability(self) -> QueueCapabilityKind {
        match self {
            Self::Wait => QueueCapabilityKind::Wait,
            Self::Stream => QueueCapabilityKind::Stream,
            Self::Poll => QueueCapabilityKind::Poll,
            Self::SendInput => QueueCapabilityKind::SendInput,
        }
    }
}

/// Borrowed interaction request input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueInteractionDraft<'a> {
    /// Interaction mode.
    pub kind: QueueInteractionKind,
    /// Target task ID.
    pub task_id: &'a str,
    /// Safe timeout policy label.
    pub timeout_policy: &'a str,
    /// Capabilities advertised by the host.
    pub capabilities: QueueCapabilities,
    /// Optional BLAKE3 input hash for send-input requests.
    pub input_hash: Option<&'a str>,
}

/// Validated queue wait, stream, poll, or send-input request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueInteractionRequest {
    kind: QueueInteractionKind,
    task_id: String,
    timeout_policy: String,
    input_hash: Option<crate::SafeHash>,
}

impl QueueInteractionRequest {
    /// Validates an interaction request against advertised host capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError::UnsupportedCapability`] when the host did
    /// not advertise the requested mode.
    pub fn new(draft: QueueInteractionDraft<'_>) -> Result<Self, QueueContractError> {
        validate_capability(draft.kind, draft.capabilities)?;
        let input_hash = draft.input_hash.map(crate::SafeHash::new).transpose()?;
        if draft.kind == QueueInteractionKind::SendInput && input_hash.is_none() {
            return Err(QueueContractError::MissingInputHash);
        }
        let task_id = crate::validation::normalized_adapter_field(TASK_ID_FIELD, draft.task_id)?;
        let timeout_policy = crate::validation::normalized_adapter_field(
            TIMEOUT_POLICY_FIELD,
            draft.timeout_policy,
        )?;
        Ok(Self {
            kind: draft.kind,
            task_id,
            timeout_policy,
            input_hash,
        })
    }

    /// Returns the interaction kind.
    #[must_use]
    pub const fn kind(&self) -> QueueInteractionKind {
        self.kind
    }

    /// Returns the task ID.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns the timeout policy label.
    #[must_use]
    pub fn timeout_policy(&self) -> &str {
        self.timeout_policy.as_str()
    }

    /// Returns the optional safe input hash.
    #[must_use]
    pub const fn input_hash(&self) -> Option<&crate::SafeHash> {
        self.input_hash.as_ref()
    }
}

/// Borrowed log request input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueLogRequestDraft<'a> {
    /// Task ID to inspect.
    pub task_id: &'a str,
    /// Requested model-visible bounds.
    pub bounds: QueueLogBounds,
}

/// Validated request to read one bounded task log viewport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueLogRequest {
    task_id: String,
    bounds: QueueLogBounds,
}

impl QueueLogRequest {
    /// Validates a bounded log request.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError`] when the task ID is not adapter-safe.
    pub fn new(draft: QueueLogRequestDraft<'_>) -> Result<Self, QueueContractError> {
        let task_id = crate::validation::normalized_adapter_field(TASK_ID_FIELD, draft.task_id)?;
        Ok(Self {
            task_id,
            bounds: draft.bounds,
        })
    }

    /// Returns the task ID.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns requested bounds.
    #[must_use]
    pub const fn bounds(&self) -> QueueLogBounds {
        self.bounds
    }
}

/// Request variants sent to an injected background queue service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueServiceRequest {
    /// Enqueue a task.
    Enqueue(QueueEnqueueRequest),
    /// Apply or validate a control operation.
    Control(QueueControlRequest),
    /// Wait, stream, poll, or send input.
    Interaction(QueueInteractionRequest),
    /// Read a bounded log viewport.
    ReadLog(QueueLogRequest),
    /// List queue groups and group status.
    ListGroups,
}

/// Response variants returned by an injected background queue service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueServiceResponse {
    /// A task was enqueued or observed.
    Task(QueueTaskObservation),
    /// A control request was accepted by the host queue service.
    ControlAccepted(QueueOperationKind),
    /// An interaction request was accepted by the host queue service.
    InteractionAccepted(QueueInteractionKind),
    /// A bounded log viewport was returned.
    Log(QueueLogViewport),
    /// Group statuses were returned.
    Groups(Vec<QueueGroupStatus>),
}

/// Host-injected background queue service contract.
pub trait BackgroundQueueService:
    crate::ComponentAdapter<QueueServiceRequest, crate::ServiceOutcome<QueueServiceResponse>>
{
}

impl<T> BackgroundQueueService for T where
    T: crate::ComponentAdapter<QueueServiceRequest, crate::ServiceOutcome<QueueServiceResponse>>
{
}
