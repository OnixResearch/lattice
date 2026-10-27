/// Stable protocol event class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessProtocolEventClass {
    /// Command acknowledgement event.
    Acknowledgement,
    /// Runtime lifecycle event.
    Runtime,
}

impl HeadlessProtocolEventClass {
    /// Returns the stable event-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Acknowledgement => EVENT_ACK,
            Self::Runtime => EVENT_RUNTIME,
        }
    }
}

/// Headless protocol event DTO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadlessProtocolEvent {
    sequence: u64,
    class: HeadlessProtocolEventClass,
    kind: crate::RuntimeEventKind,
    request_id: Option<Text>,
    turn_id: Option<Text>,
    model_call_id: Option<Text>,
    tool_call_id: Option<Text>,
    summary: Text,
}

impl HeadlessProtocolEvent {
    /// Builds a protocol event from a runtime event.
    ///
    /// # Errors
    ///
    /// Returns [`HeadlessProtocolError`] when the sequence is invalid or a safe
    /// label derived from the event is unsafe.
    pub fn from_runtime_event(
        sequence: u64,
        event: &crate::RuntimeEvent,
    ) -> Result<Self, HeadlessProtocolError> {
        if sequence < super::MIN_EVENT_SEQUENCE {
            return Err(HeadlessProtocolError::new(
                None,
                HeadlessProtocolErrorKind::RuntimeFailure,
                EVENT_SEQUENCE_FIELD,
            ));
        }
        let projection = RuntimeProjection::from_event(event)?;
        Ok(Self {
            sequence,
            class: HeadlessProtocolEventClass::Runtime,
            kind: event.kind(),
            request_id: None,
            turn_id: projection.turn_id,
            model_call_id: projection.model_call_id,
            tool_call_id: projection.tool_call_id,
            summary: projection.summary,
        })
    }

    /// Returns the protocol sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the protocol event class.
    #[must_use]
    pub const fn class(&self) -> HeadlessProtocolEventClass {
        self.class
    }

    /// Returns the runtime event kind.
    #[must_use]
    pub const fn kind(&self) -> crate::RuntimeEventKind {
        self.kind
    }

    /// Returns the optional request identity.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Returns the optional turn correlation identity.
    #[must_use]
    pub fn turn_id(&self) -> Option<&str> {
        self.turn_id.as_deref()
    }

    /// Returns the optional model-call correlation identity.
    #[must_use]
    pub fn model_call_id(&self) -> Option<&str> {
        self.model_call_id.as_deref()
    }

    /// Returns the optional tool-call correlation identity.
    #[must_use]
    pub fn tool_call_id(&self) -> Option<&str> {
        self.tool_call_id.as_deref()
    }

    /// Returns the safe event summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

struct RuntimeProjection {
    turn_id: Option<Text>,
    model_call_id: Option<Text>,
    tool_call_id: Option<Text>,
    summary: Text,
}

impl RuntimeProjection {
    fn from_event(event: &crate::RuntimeEvent) -> Result<Self, HeadlessProtocolError> {
        match event {
            crate::RuntimeEvent::TurnStarted { turn_index } => {
                Ok(Self::summary_with_turn(event.kind().as_str(), *turn_index))
            }
            crate::RuntimeEvent::LoopDecision {
                turn_index,
                decision,
            } => Ok(Self::summary_with_turn(decision.as_str(), *turn_index)),
            crate::RuntimeEvent::ModelStreamStarted { .. }
            | crate::RuntimeEvent::ModelStreamFrame { .. }
            | crate::RuntimeEvent::ModelStreamEnded { .. }
            | crate::RuntimeEvent::ModelStreamFailed { .. } => Self::from_stream_event(event),
            crate::RuntimeEvent::ProviderRetryProgress { progress } => {
                let delay_ms = progress
                    .delay_ms
                    .map_or_else(|| "none".to_string(), |value| value.to_string());
                let summary = format!(
                    "{} attempt={}/{} delay_ms={} reason={}",
                    progress.status.as_str(),
                    progress.attempt_count,
                    progress.max_attempt_count,
                    delay_ms,
                    progress.reason.as_str()
                );
                Self::summary_checked("provider_retry_progress", summary.as_str())
            }
            crate::RuntimeEvent::ModelRequested { .. } => Ok(Self::summary(MODEL_REQUEST_SUMMARY)),
            crate::RuntimeEvent::ModelResponded { .. } => Ok(Self::summary(MODEL_RESPONSE_SUMMARY)),
            crate::RuntimeEvent::ModelDenied { reason, .. } => {
                Ok(Self::summary_checked("model_denial", reason)?)
            }
            crate::RuntimeEvent::ToolRequested { request } => Ok(Self::summary_with_tool(
                TOOL_REQUEST_SUMMARY,
                request.name(),
            )?),
            crate::RuntimeEvent::ToolResponded { .. } => Ok(Self::summary(TOOL_RESPONSE_SUMMARY)),
            crate::RuntimeEvent::ToolDenied { request, reason } => {
                let mut projection = Self::summary_with_tool(reason, request.name())?;
                projection.summary = safe_text("tool_denial", reason)?;
                Ok(projection)
            }
            crate::RuntimeEvent::FinalResponse { .. } => Ok(Self::summary(FINAL_RESPONSE_SUMMARY)),
            crate::RuntimeEvent::CompletionCandidateRequested { .. } => {
                Ok(Self::summary(COMPLETION_CANDIDATE_SUMMARY))
            }
            crate::RuntimeEvent::Stopped => Ok(Self::summary(STOPPED_SUMMARY)),
            crate::RuntimeEvent::Shutdown { status } => Ok(Self::summary(status.as_str())),
            crate::RuntimeEvent::RuntimeStarted { summary }
            | crate::RuntimeEvent::LoopStarted { summary }
            | crate::RuntimeEvent::ContextReplayed { summary }
            | crate::RuntimeEvent::AdapterOutcomeClassified {
                message: summary, ..
            }
            | crate::RuntimeEvent::AdapterError {
                message: summary, ..
            }
            | crate::RuntimeEvent::FabricError { message: summary }
            | crate::RuntimeEvent::Cancelled { reason: summary }
            | crate::RuntimeEvent::CompactionRequested { summary } => {
                Self::summary_checked(event.kind().as_str(), summary)
            }
            crate::RuntimeEvent::HookBlocked { reason, .. } => {
                Self::summary_checked("hook_blocked", reason)
            }
            crate::RuntimeEvent::HookFailed { message, .. } => {
                Self::summary_checked("hook_failed", message)
            }
            crate::RuntimeEvent::LifecycleHookOutcome { outcome } => Self::summary_checked(
                "lifecycle_hook_outcome",
                outcome.outcome_class().as_str(),
            ),
            crate::RuntimeEvent::SteelPolicyDenied { reason, .. } => {
                Self::summary_checked("steel_policy_denied", reason)
            }
            crate::RuntimeEvent::SandboxEscalation {
                escalation_reason, ..
            } => Self::summary_checked("sandbox_escalation", escalation_reason),
            crate::RuntimeEvent::SessionStateChanged { .. }
            | crate::RuntimeEvent::SessionQueueMutationObserved { .. }
            | crate::RuntimeEvent::SessionCommitObserved { .. }
            | crate::RuntimeEvent::SessionRecoveryObserved { .. }
            | crate::RuntimeEvent::SessionAuthorityProfileObserved { .. } => {
                Self::from_session_event(event)
            }
            crate::RuntimeEvent::MaxTurnsExceeded { max_turns } => {
                Ok(Self::summary(max_turns.to_string().as_str()))
            }
        }
    }

    fn from_session_event(event: &crate::RuntimeEvent) -> Result<Self, HeadlessProtocolError> {
        match event {
            crate::RuntimeEvent::SessionStateChanged {
                status,
                queue_count,
                ..
            } => Self::summary_checked(
                "session_state",
                format!("{status} queue_count={queue_count}").as_str(),
            ),
            crate::RuntimeEvent::SessionQueueMutationObserved { mutation } => {
                Self::summary_checked(
                    "session_queue_mutation",
                    format!(
                        "{} {} generation={}",
                        mutation.operation.as_str(),
                        mutation.status,
                        mutation.generation.value()
                    )
                    .as_str(),
                )
            }
            crate::RuntimeEvent::SessionCommitObserved {
                message_kind,
                durable_cursor,
                ..
            } => Self::summary_checked(
                "session_commit",
                format!("{message_kind} durable_cursor={durable_cursor}").as_str(),
            ),
            crate::RuntimeEvent::SessionRecoveryObserved { status, .. } => {
                Self::summary_checked("session_recovery", status)
            }
            crate::RuntimeEvent::SessionAuthorityProfileObserved {
                status,
                changed_components,
                ..
            } => Self::summary_checked(
                "session_authority_profile",
                format!("{status} changed_count={}", changed_components.len()).as_str(),
            ),
            _other => Err(HeadlessProtocolError::new(
                None,
                HeadlessProtocolErrorKind::RuntimeFailure,
                "session_event",
            )),
        }
    }

    fn from_stream_event(event: &crate::RuntimeEvent) -> Result<Self, HeadlessProtocolError> {
        match event {
            crate::RuntimeEvent::ModelStreamStarted { call_id } => Self::summary_with_model(
                event.kind().as_str(),
                call_id.as_str(),
            ),
            crate::RuntimeEvent::ModelStreamFrame { frame } => {
                let model_call_id = Some(safe_text("model_call_id", frame.call_id().as_str())?);
                let tool_call_id = frame
                    .tool_call_id()
                    .map(|tool_call_id| safe_text("tool_call_id", tool_call_id.as_str()))
                    .transpose()?;
                Ok(Self {
                    turn_id: None,
                    model_call_id,
                    tool_call_id,
                    summary: Text::from(frame.kind().as_str()),
                })
            }
            crate::RuntimeEvent::ModelStreamEnded { call_id, status } => {
                Self::summary_with_model(status.as_str(), call_id.as_str())
            }
            crate::RuntimeEvent::ModelStreamFailed { call_id, reason } => Ok(Self {
                turn_id: None,
                model_call_id: Some(safe_text("model_call_id", call_id.as_str())?),
                tool_call_id: None,
                summary: safe_text("model_stream_failure", reason)?,
            }),
            _other => Err(HeadlessProtocolError::bounded(
                None,
                HeadlessProtocolErrorKind::RuntimeFailure,
                "stream projection requires a model-stream event",
            )
            .unwrap_or_else(core::convert::identity)),
        }
    }

    fn summary(summary: &str) -> Self {
        Self {
            turn_id: None,
            model_call_id: None,
            tool_call_id: None,
            summary: Text::from(summary),
        }
    }

    fn summary_checked(field: &'static str, summary: &str) -> Result<Self, HeadlessProtocolError> {
        Ok(Self {
            turn_id: None,
            model_call_id: None,
            tool_call_id: None,
            summary: safe_text(field, summary)?,
        })
    }

    fn summary_with_turn(summary: &str, turn_index: usize) -> Self {
        Self {
            turn_id: Some(Text::from(format!("{TURN_ID_PREFIX}{turn_index}"))),
            model_call_id: None,
            tool_call_id: None,
            summary: Text::from(summary),
        }
    }

    fn summary_with_model(summary: &str, call_id: &str) -> Result<Self, HeadlessProtocolError> {
        Ok(Self {
            turn_id: None,
            model_call_id: Some(safe_text("model_call_id", call_id)?),
            tool_call_id: None,
            summary: safe_text("model_summary", summary)?,
        })
    }

    fn summary_with_tool(summary: &str, tool_name: &str) -> Result<Self, HeadlessProtocolError> {
        Ok(Self {
            turn_id: None,
            model_call_id: None,
            tool_call_id: Some(safe_text(
                "tool_call_id",
                format!("{TOOL_ID_PREFIX}{tool_name}").as_str(),
            )?),
            summary: safe_text("tool_summary", summary)?,
        })
    }
}
