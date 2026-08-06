/// Host event subscription filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostEventSubscription {
    correlation_id: String,
    selector: Option<HostSelector>,
    kinds: Vec<HostEventKind>,
}

impl HostEventSubscription {
    /// Validates an event subscription.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the correlation id or kind labels are invalid.
    pub fn new(
        correlation_id: &str,
        selector: Option<&HostSelector>,
        kinds: &[&str],
    ) -> Result<Self, HostControlError> {
        if kinds.is_empty() {
            return Err(HostControlError::InvalidLayout);
        }
        Ok(Self {
            correlation_id: required_text(CORRELATION_FIELD, correlation_id)?,
            selector: selector.cloned(),
            kinds: kinds
                .iter()
                .map(|kind| HostEventKind::parse(kind))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Returns true when an event is admitted by this subscription.
    #[must_use]
    pub fn admits(&self, event: &HostEvent) -> bool {
        event.correlation_id == self.correlation_id
            && self.kinds.contains(&event.kind)
            && self
                .selector
                .as_ref()
                .is_none_or(|selector| event.matches_selector(selector))
    }
}

/// Filters host events in original order.
#[must_use]
pub fn filter_host_events(
    subscription: &HostEventSubscription,
    events: &[HostEvent],
) -> Vec<HostEvent> {
    events
        .iter()
        .filter(|event| subscription.admits(event))
        .cloned()
        .collect()
}

/// One-shot wait request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostWaitRequest {
    correlation_id: String,
    selector: HostSelector,
    expected_status: HostSemanticStatus,
    timeout_ticks: u32,
}

impl HostWaitRequest {
    /// Validates a one-shot wait request.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when correlation id, status, or timeout is invalid.
    pub fn new(
        correlation_id: &str,
        selector: HostSelector,
        expected_status: &str,
        timeout_ticks: u32,
    ) -> Result<Self, HostControlError> {
        if timeout_ticks == NO_TIMEOUT_TICKS {
            return Err(HostControlError::InvalidTimeout);
        }
        Ok(Self {
            correlation_id: required_text(CORRELATION_FIELD, correlation_id)?,
            selector,
            expected_status: HostSemanticStatus::parse(expected_status)?,
            timeout_ticks,
        })
    }

    /// Returns finite timeout ticks.
    #[must_use]
    pub const fn timeout_ticks(&self) -> u32 {
        self.timeout_ticks
    }
}

/// Wait outcome kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostWaitOutcomeKind {
    /// Expected event was observed.
    Matched,
    /// Finite timeout elapsed.
    Timeout,
    /// Wait was cancelled.
    Cancelled,
    /// Host returned a typed error.
    HostError,
}

/// Wait outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostWaitOutcome {
    kind: HostWaitOutcomeKind,
}

impl HostWaitOutcome {
    /// Returns wait outcome kind.
    #[must_use]
    pub const fn kind(&self) -> HostWaitOutcomeKind {
        self.kind
    }
}

/// Resolves a wait request over in-memory events.
///
/// # Panics
///
/// Panics if callers bypass wait-request validation before resolution.
#[must_use]
pub fn resolve_host_wait(request: &HostWaitRequest, events: &[HostEvent]) -> HostWaitOutcome {
    assert!(
        !request.correlation_id.is_empty(),
        "wait correlation must be validated"
    );
    assert!(
        request.timeout_ticks > NO_TIMEOUT_TICKS,
        "wait timeout must be finite"
    );
    for event in events {
        if event.correlation_id != request.correlation_id
            || !event.matches_selector(&request.selector)
        {
            continue;
        }
        let kind = match event.kind {
            HostEventKind::Cancelled => HostWaitOutcomeKind::Cancelled,
            HostEventKind::Timeout => HostWaitOutcomeKind::Timeout,
            HostEventKind::HostError => HostWaitOutcomeKind::HostError,
            HostEventKind::AgentStatus if event.status == Some(request.expected_status) => {
                HostWaitOutcomeKind::Matched
            }
            _unmatched => continue,
        };
        return HostWaitOutcome { kind };
    }
    HostWaitOutcome {
        kind: HostWaitOutcomeKind::Timeout,
    }
}
