/// Semantic host status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostSemanticStatus {
    /// Idle state.
    Idle,
    /// Running state.
    Running,
    /// Waiting state.
    Waiting,
    /// Done state.
    Done,
    /// Failed state.
    Failed,
}

impl HostSemanticStatus {
    /// Parses a semantic status label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(STATUS_FIELD, value, |normalized| match normalized {
            STATUS_IDLE => Some(Self::Idle),
            STATUS_RUNNING => Some(Self::Running),
            STATUS_WAITING => Some(Self::Waiting),
            STATUS_DONE => Some(Self::Done),
            STATUS_FAILED => Some(Self::Failed),
            _unsupported => None,
        })
    }
}

/// Host event kind shared by subscriptions and waits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostEventKind {
    /// Agent status changed.
    AgentStatus,
    /// Output matched a host-supplied detector.
    OutputMatch,
    /// Pane lifecycle changed.
    PaneLifecycle,
    /// Wait or stream was cancelled.
    Cancelled,
    /// Wait timed out.
    Timeout,
    /// Host returned a typed error.
    HostError,
}

impl HostEventKind {
    /// Parses an event-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(EVENT_KIND_FIELD, value, |normalized| match normalized {
            EVENT_AGENT_STATUS => Some(Self::AgentStatus),
            EVENT_OUTPUT_MATCH => Some(Self::OutputMatch),
            EVENT_PANE_LIFECYCLE => Some(Self::PaneLifecycle),
            EVENT_CANCELLED => Some(Self::Cancelled),
            EVENT_TIMEOUT => Some(Self::Timeout),
            EVENT_HOST_ERROR => Some(Self::HostError),
            _unsupported => None,
        })
    }

    /// Returns the stable event-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AgentStatus => EVENT_AGENT_STATUS,
            Self::OutputMatch => EVENT_OUTPUT_MATCH,
            Self::PaneLifecycle => EVENT_PANE_LIFECYCLE,
            Self::Cancelled => EVENT_CANCELLED,
            Self::Timeout => EVENT_TIMEOUT,
            Self::HostError => EVENT_HOST_ERROR,
        }
    }
}

/// Host event DTO.
// r[impl agent_harness_runtime.host_control.events_waits]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostEvent {
    kind: HostEventKind,
    correlation_id: String,
    selector: Option<HostSelector>,
    status: Option<HostSemanticStatus>,
    message: Option<String>,
    error: Option<HostTypedError>,
}

struct HostEventParts<'a> {
    kind: HostEventKind,
    correlation_id: &'a str,
    selector: Option<HostSelector>,
    status: Option<HostSemanticStatus>,
    message: Option<&'a str>,
    error: Option<HostTypedError>,
}

impl HostEvent {
    /// Builds an agent-status event.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the correlation id is invalid.
    pub fn agent_status(
        correlation_id: &str,
        selector: HostSelector,
        status: HostSemanticStatus,
    ) -> Result<Self, HostControlError> {
        Self::new(HostEventParts {
            kind: HostEventKind::AgentStatus,
            correlation_id,
            selector: Some(selector),
            status: Some(status),
            message: None,
            error: None,
        })
    }

    /// Builds an output-match event.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when fields are invalid.
    pub fn output_match(
        correlation_id: &str,
        selector: HostSelector,
        message: &str,
    ) -> Result<Self, HostControlError> {
        Self::new(HostEventParts {
            kind: HostEventKind::OutputMatch,
            correlation_id,
            selector: Some(selector),
            status: None,
            message: Some(message),
            error: None,
        })
    }

    /// Builds a pane-lifecycle event.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when fields are invalid.
    pub fn pane_lifecycle(
        correlation_id: &str,
        selector: HostSelector,
        message: &str,
    ) -> Result<Self, HostControlError> {
        Self::new(HostEventParts {
            kind: HostEventKind::PaneLifecycle,
            correlation_id,
            selector: Some(selector),
            status: None,
            message: Some(message),
            error: None,
        })
    }

    /// Builds a cancellation event.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the correlation id is invalid.
    pub fn cancelled(
        correlation_id: &str,
        selector: HostSelector,
    ) -> Result<Self, HostControlError> {
        Self::new(HostEventParts {
            kind: HostEventKind::Cancelled,
            correlation_id,
            selector: Some(selector),
            status: None,
            message: None,
            error: None,
        })
    }

    /// Builds a timeout event.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the correlation id is invalid.
    pub fn timeout(correlation_id: &str, selector: HostSelector) -> Result<Self, HostControlError> {
        Self::new(HostEventParts {
            kind: HostEventKind::Timeout,
            correlation_id,
            selector: Some(selector),
            status: None,
            message: None,
            error: None,
        })
    }

    /// Builds a host-error event.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when fields are invalid.
    pub fn host_error(
        correlation_id: &str,
        selector: HostSelector,
        error: HostTypedError,
    ) -> Result<Self, HostControlError> {
        Self::new(HostEventParts {
            kind: HostEventKind::HostError,
            correlation_id,
            selector: Some(selector),
            status: None,
            message: None,
            error: Some(error),
        })
    }

    /// Returns event kind.
    #[must_use]
    pub const fn kind(&self) -> HostEventKind {
        self.kind
    }

    /// Returns correlation identity.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    /// Returns semantic status when present.
    #[must_use]
    pub const fn status(&self) -> Option<HostSemanticStatus> {
        self.status
    }

    /// Returns optional event message.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns optional typed host error.
    #[must_use]
    pub const fn error(&self) -> Option<&HostTypedError> {
        self.error.as_ref()
    }

    fn new(parts: HostEventParts<'_>) -> Result<Self, HostControlError> {
        Ok(Self {
            kind: parts.kind,
            correlation_id: required_text(CORRELATION_FIELD, parts.correlation_id)?,
            selector: parts.selector,
            status: parts.status,
            message: optional_text(ERROR_MESSAGE_FIELD, parts.message)?,
            error: parts.error,
        })
    }

    fn matches_selector(&self, selector: &HostSelector) -> bool {
        self.selector.as_ref().is_some_and(|candidate| {
            candidate.host_id == selector.host_id && candidate.resource_id == selector.resource_id
        })
    }
}
