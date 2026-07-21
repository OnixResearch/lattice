/// Runtime phase visible to the headless protocol preflight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessRuntimePhase {
    /// Runtime is ready to accept a new prompt turn.
    Idle,
    /// Runtime is processing a turn.
    ActiveTurn,
}

/// Command acknowledgement status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessResponseStatus {
    /// Command was accepted for host/runtime handling.
    Accepted,
    /// Command was rejected before mutation or dispatch.
    Rejected,
}

impl HeadlessResponseStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => ACCEPTED_STATUS,
            Self::Rejected => REJECTED_STATUS,
        }
    }
}

/// Headless command response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadlessResponse {
    request_id: Option<Text>,
    status: HeadlessResponseStatus,
    error: Option<HeadlessProtocolError>,
}

impl HeadlessResponse {
    fn accepted(request_id: &str) -> Self {
        Self {
            request_id: Some(Text::from(request_id)),
            status: HeadlessResponseStatus::Accepted,
            error: None,
        }
    }

    fn rejected(request_id: Option<&str>, error: HeadlessProtocolError) -> Self {
        Self {
            request_id: request_id.map(Text::from),
            status: HeadlessResponseStatus::Rejected,
            error: Some(error),
        }
    }

    /// Returns the optional request identity.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Returns the response status.
    #[must_use]
    pub const fn status(&self) -> HeadlessResponseStatus {
        self.status
    }

    /// Returns a bounded safe error when rejected.
    #[must_use]
    pub const fn error(&self) -> Option<&HeadlessProtocolError> {
        self.error.as_ref()
    }
}

/// Safe bounded protocol error kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessProtocolErrorKind {
    /// Record parse failure.
    Parse,
    /// Command kind or field was invalid.
    InvalidCommand,
    /// Command is invalid for the current runtime state.
    InvalidState,
    /// Required host-owned service is unavailable.
    UnavailableService,
    /// Operation was cancelled.
    Cancelled,
    /// Runtime reported a bounded failure.
    RuntimeFailure,
}

impl HeadlessProtocolErrorKind {
    /// Returns the stable error-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parse => ERROR_PARSE,
            Self::InvalidCommand => ERROR_INVALID_COMMAND,
            Self::InvalidState => ERROR_INVALID_STATE,
            Self::UnavailableService => ERROR_UNAVAILABLE_SERVICE,
            Self::Cancelled => ERROR_CANCELLED,
            Self::RuntimeFailure => ERROR_RUNTIME_FAILURE,
        }
    }
}

/// Safe bounded protocol error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadlessProtocolError {
    request_id: Option<Text>,
    kind: HeadlessProtocolErrorKind,
    message: Text,
}

impl HeadlessProtocolError {
    /// Builds a safe protocol error.
    ///
    /// # Errors
    ///
    /// Returns [`HeadlessProtocolError`] with parse kind when the message itself
    /// contains unsafe material.
    pub fn bounded(
        request_id: Option<&str>,
        kind: HeadlessProtocolErrorKind,
        message: &str,
    ) -> Result<Self, Self> {
        let request_id = request_id
            .map(|value| safe_text(REQUEST_ID_FIELD, value))
            .transpose()?;
        let message = safe_text(ERROR_MESSAGE_FIELD, message)?;
        Ok(Self {
            request_id,
            kind,
            message,
        })
    }

    fn new(request_id: Option<&str>, kind: HeadlessProtocolErrorKind, message: &str) -> Self {
        Self::bounded(request_id, kind, message).unwrap_or_else(core::convert::identity)
    }

    /// Returns the optional request identity recovered for this error.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Returns the error kind.
    #[must_use]
    pub const fn kind(&self) -> HeadlessProtocolErrorKind {
        self.kind
    }

    /// Returns the bounded safe message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for HeadlessProtocolError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "headless protocol error {}: {}",
            self.kind.as_str(),
            self.message
        )
    }
}

impl std::error::Error for HeadlessProtocolError {}
