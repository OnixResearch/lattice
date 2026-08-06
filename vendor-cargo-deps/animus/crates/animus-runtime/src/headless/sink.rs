/// Final output selected from terminal automation events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalOutput {
    content: super::Text,
}

impl FinalOutput {
    /// Returns the final output content.
    #[must_use]
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

/// Borrowed completion receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionReceiptDraft<'a> {
    /// Machine-readable completion status label.
    pub status: &'a str,
    /// Safe evidence references for the completion outcome.
    pub evidence_references: &'a [&'a str],
    /// Optional final response body for successful runs.
    pub final_response: Option<&'a str>,
}

/// Validated noninteractive completion receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionReceipt {
    status: CompletionStatus,
    evidence_references: super::Collection<super::Text>,
    final_response: Option<super::Text>,
}

impl CompletionReceipt {
    /// Validates a noninteractive completion receipt.
    ///
    /// # Errors
    ///
    /// Returns [`super::AutomationError`] when status is unsupported, evidence
    /// is missing or duplicated, or a success receipt lacks a final response.
    pub fn validate(draft: &CompletionReceiptDraft<'_>) -> Result<Self, super::AutomationError> {
        let status = CompletionStatus::parse(draft.status).ok_or_else(|| {
            super::unsupported(super::TextField {
                field: super::COMPLETION_STATUS_FIELD,
                value: draft.status,
            })
        })?;
        let evidence_references =
            super::required_unique_texts(super::EVIDENCE_REFERENCE_FIELD, draft.evidence_references)?;
        let final_response = match status {
            CompletionStatus::Success => Some(super::required_optional_text(super::OptionalTextField {
                field: super::FINAL_RESPONSE_FIELD,
                value: draft.final_response,
            })?),
            _terminal_failure => super::optional_text(super::OptionalTextField {
                field: super::FINAL_RESPONSE_FIELD,
                value: draft.final_response,
            }),
        };
        Ok(Self {
            status,
            evidence_references,
            final_response,
        })
    }

    /// Returns the completion status.
    #[must_use]
    pub const fn status(&self) -> CompletionStatus {
        self.status
    }

    /// Returns safe evidence references.
    #[must_use]
    pub fn evidence_references(&self) -> &[super::Text] {
        self.evidence_references.as_slice()
    }

    /// Returns optional final response body.
    #[must_use]
    pub fn final_response(&self) -> Option<&str> {
        self.final_response.as_deref()
    }
}

/// Noninteractive completion status class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionStatus {
    /// Run completed successfully.
    Success,
    /// Run was denied by policy.
    Denied,
    /// Run requires an interactive prompt decision.
    PromptRequired,
    /// Run requires missing input.
    InputRequired,
    /// Validation failed before or after execution.
    ValidationFailed,
    /// Required service was unavailable.
    ServiceUnavailable,
    /// Run was cancelled.
    Cancelled,
    /// Execution failed.
    ExecutionFailed,
}

impl CompletionStatus {
    fn parse(value: &str) -> Option<Self> {
        match value {
            super::STATUS_SUCCESS => Some(Self::Success),
            super::STATUS_DENIED => Some(Self::Denied),
            super::STATUS_PROMPT_REQUIRED => Some(Self::PromptRequired),
            super::STATUS_INPUT_REQUIRED => Some(Self::InputRequired),
            super::STATUS_VALIDATION_FAILED => Some(Self::ValidationFailed),
            super::STATUS_SERVICE_UNAVAILABLE => Some(Self::ServiceUnavailable),
            super::STATUS_CANCELLED => Some(Self::Cancelled),
            super::STATUS_EXECUTION_FAILED => Some(Self::ExecutionFailed),
            _unsupported => None,
        }
    }

    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => super::STATUS_SUCCESS,
            Self::Denied => super::STATUS_DENIED,
            Self::PromptRequired => super::STATUS_PROMPT_REQUIRED,
            Self::InputRequired => super::STATUS_INPUT_REQUIRED,
            Self::ValidationFailed => super::STATUS_VALIDATION_FAILED,
            Self::ServiceUnavailable => super::STATUS_SERVICE_UNAVAILABLE,
            Self::Cancelled => super::STATUS_CANCELLED,
            Self::ExecutionFailed => super::STATUS_EXECUTION_FAILED,
        }
    }

    /// Returns whether guarded host actions may continue after this status.
    #[must_use]
    pub const fn allows_guarded_dispatch(self) -> bool {
        matches!(self, Self::Success)
    }
}

/// Builds a prompt-required completion for unresolved noninteractive permission asks.
///
/// # Errors
///
/// Returns [`super::AutomationError`] when the evidence reference is empty.
pub fn prompt_required_completion(evidence_reference: &str) -> Result<CompletionReceipt, super::AutomationError> {
    CompletionReceipt::validate(&CompletionReceiptDraft {
        status: super::STATUS_PROMPT_REQUIRED,
        evidence_references: &[evidence_reference],
        final_response: None,
    })
}

/// Selects final output from the terminal completion event.
///
/// # Errors
///
/// Returns [`super::AutomationError`] when the stream has no completion event or
/// ends in failure instead of completion.
///
/// # Panics
///
/// Panics if terminal-event counting exceeds the supplied stream length, which
/// would indicate corrupted iteration state.
pub fn select_final_output(stream: &[super::AutomationEnvelope]) -> Result<FinalOutput, super::AutomationError> {
    let mut final_content: Option<&str> = None;
    let mut terminal_events_seen = 0usize;
    for envelope in stream {
        match envelope.kind() {
            super::AutomationEventKind::Completion | super::AutomationEventKind::FinalResponse => {
                terminal_events_seen += 1;
                final_content = Some(envelope.body());
            }
            super::AutomationEventKind::Failure => {
                assert!(terminal_events_seen <= stream.len());
                return Err(super::AutomationError::RunFailed {
                    reason: super::Text::from(envelope.body()),
                });
            }
            _progress => {}
        }
    }
    assert!(terminal_events_seen <= stream.len());
    assert_eq!(final_content.is_some(), terminal_events_seen > 0);
    final_content
        .map(|content| FinalOutput {
            content: super::Text::from(content),
        })
        .ok_or(super::AutomationError::MissingTerminalResponse)
}
