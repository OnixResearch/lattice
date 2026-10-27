use std::string::ToString;

const RUNTIME_EVENT_METRIC_NAME: &str = "runtime.events";
const RUNTIME_EVENT_LOG_TARGET: &str = "runtime.event";
const EVENT_KIND_LABEL: &str = "event_kind";
const SEQUENCE_LABEL: &str = "sequence";
const COMPONENT_LABEL: &str = "component";
const FAILURE_CLASS_LABEL: &str = "failure_class";
const TERMINAL_STATUS_LABEL: &str = "terminal_status";
const DECISION_LABEL: &str = "decision";
const HOOK_BOUNDARY_LABEL: &str = "hook_boundary";
const FIRST_EVENT_SEQUENCE: u32 = 0;
const EVENT_SEQUENCE_STEP: u32 = 1;

/// Safe observability projection for a runtime event or report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    metrics: std::vec::Vec<animus_adapters::MetricSample>,
    logs: std::vec::Vec<animus_adapters::LogRecord>,
}

impl Projection {
    /// Builds a projection from already validated records.
    #[must_use]
    pub const fn new(
        metrics: std::vec::Vec<animus_adapters::MetricSample>,
        logs: std::vec::Vec<animus_adapters::LogRecord>,
    ) -> Self {
        Self { metrics, logs }
    }

    /// Returns projected safe metric samples.
    #[must_use]
    pub fn metrics(&self) -> &[animus_adapters::MetricSample] {
        self.metrics.as_slice()
    }

    /// Returns projected safe log records.
    #[must_use]
    pub fn logs(&self) -> &[animus_adapters::LogRecord] {
        self.logs.as_slice()
    }

    fn push(&mut self, mut other: Self) {
        self.metrics.append(&mut other.metrics);
        self.logs.append(&mut other.logs);
    }
}

/// Projects one runtime event into safe metric and log records.
///
/// # Errors
///
/// Returns [`animus_adapters::ObservabilityError`] when generated safe labels fail validation.
pub fn project_runtime_event(
    event: &super::RuntimeEvent,
    sequence: u32,
) -> Result<Projection, animus_adapters::ObservabilityError> {
    let event_kind = event.kind().as_str();
    let sequence_label = sequence.to_string();
    debug_assert!(!event_kind.is_empty());
    debug_assert!(!sequence_label.is_empty());
    let mut labels = vec![
        animus_adapters::MetricLabelDraft::new(EVENT_KIND_LABEL, event_kind),
        animus_adapters::MetricLabelDraft::new(SEQUENCE_LABEL, sequence_label.as_str()),
    ];
    labels.extend(event_specific_labels(event));

    let metric = animus_adapters::MetricSample::new(animus_adapters::MetricSampleDraft {
        name: RUNTIME_EVENT_METRIC_NAME,
        kind: animus_adapters::MetricKind::Counter,
        unit: animus_adapters::MetricUnit::Count,
        value: u64::from(EVENT_SEQUENCE_STEP),
        labels: labels.as_slice(),
    })?;
    let log = animus_adapters::LogRecord::new(animus_adapters::LogRecordDraft::new(
        RUNTIME_EVENT_LOG_TARGET,
        level_for_event(event),
        event_kind,
        labels.as_slice(),
    ))?;
    Ok(Projection::new(vec![metric], vec![log]))
}

/// Projects a complete runtime report into safe metric and log records in event order.
///
/// # Errors
///
/// Returns [`animus_adapters::ObservabilityError`] when a generated record fails validation.
pub fn project_report(
    report: &crate::RunReport,
) -> Result<Projection, animus_adapters::ObservabilityError> {
    let mut projection = Projection::new(std::vec::Vec::new(), std::vec::Vec::new());
    let mut sequence = FIRST_EVENT_SEQUENCE;
    for event in report.events() {
        projection.push(project_runtime_event(event, sequence)?);
        sequence = sequence
            .checked_add(EVENT_SEQUENCE_STEP)
            .ok_or(animus_adapters::ObservabilityError::InvalidLabelCount)?;
    }
    Ok(projection)
}

fn event_specific_labels(
    event: &super::RuntimeEvent,
) -> std::vec::Vec<animus_adapters::MetricLabelDraft<'static>> {
    match event {
        super::RuntimeEvent::LoopDecision { decision, .. } => {
            vec![animus_adapters::MetricLabelDraft::new(
                DECISION_LABEL,
                decision.as_str(),
            )]
        }
        super::RuntimeEvent::AdapterOutcomeClassified {
            component, class, ..
        } => vec![
            animus_adapters::MetricLabelDraft::new(COMPONENT_LABEL, component.as_str()),
            animus_adapters::MetricLabelDraft::new(FAILURE_CLASS_LABEL, class.as_str()),
        ],
        super::RuntimeEvent::AdapterError {
            component,
            message: _,
        } => vec![animus_adapters::MetricLabelDraft::new(
            COMPONENT_LABEL,
            component.as_str(),
        )],
        super::RuntimeEvent::HookBlocked { boundary, .. }
        | super::RuntimeEvent::HookFailed { boundary, .. } => {
            vec![animus_adapters::MetricLabelDraft::new(
                HOOK_BOUNDARY_LABEL,
                boundary.as_str(),
            )]
        }
        super::RuntimeEvent::Shutdown { status } => {
            vec![animus_adapters::MetricLabelDraft::new(
                TERMINAL_STATUS_LABEL,
                status.as_str(),
            )]
        }
        super::RuntimeEvent::ModelRequested { .. }
        | super::RuntimeEvent::ModelResponded { .. }
        | super::RuntimeEvent::ModelDenied { .. }
        | super::RuntimeEvent::ToolRequested { .. }
        | super::RuntimeEvent::ToolResponded { .. }
        | super::RuntimeEvent::ToolDenied { .. }
        | super::RuntimeEvent::FinalResponse { .. }
        | super::RuntimeEvent::CompletionCandidateRequested { .. }
        | super::RuntimeEvent::RuntimeStarted { .. }
        | super::RuntimeEvent::LoopStarted { .. }
        | super::RuntimeEvent::ContextReplayed { .. }
        | super::RuntimeEvent::TurnStarted { .. }
        | super::RuntimeEvent::ModelStreamStarted { .. }
        | super::RuntimeEvent::ModelStreamFrame { .. }
        | super::RuntimeEvent::ModelStreamEnded { .. }
        | super::RuntimeEvent::ModelStreamFailed { .. }
        | super::RuntimeEvent::ProviderRetryProgress { .. }
        | super::RuntimeEvent::Stopped
        | super::RuntimeEvent::MaxTurnsExceeded { .. }
        | super::RuntimeEvent::FabricError { .. }
        | super::RuntimeEvent::LifecycleHookOutcome { .. }
        | super::RuntimeEvent::SteelPolicyDenied { .. }
        | super::RuntimeEvent::SandboxEscalation { .. }
        | super::RuntimeEvent::Cancelled { .. }
        | super::RuntimeEvent::CompactionRequested { .. }
        | super::RuntimeEvent::SessionStateChanged { .. }
        | super::RuntimeEvent::SessionQueueMutationObserved { .. }
        | super::RuntimeEvent::SessionCommitObserved { .. }
        | super::RuntimeEvent::SessionRecoveryObserved { .. }
        | super::RuntimeEvent::SessionAuthorityProfileObserved { .. } => std::vec::Vec::new(),
    }
}

fn level_for_event(event: &super::RuntimeEvent) -> animus_adapters::LogLevel {
    match event {
        super::RuntimeEvent::AdapterError { .. }
        | super::RuntimeEvent::FabricError { .. }
        | super::RuntimeEvent::HookFailed { .. }
        | super::RuntimeEvent::SteelPolicyDenied { .. }
        | super::RuntimeEvent::Cancelled { .. } => animus_adapters::LogLevel::Error,
        super::RuntimeEvent::ModelDenied { .. }
        | super::RuntimeEvent::ToolDenied { .. }
        | super::RuntimeEvent::HookBlocked { .. }
        | super::RuntimeEvent::ModelStreamFailed { .. }
        | super::RuntimeEvent::MaxTurnsExceeded { .. }
        | super::RuntimeEvent::AdapterOutcomeClassified { .. } => {
            animus_adapters::LogLevel::Warning
        }
        super::RuntimeEvent::RuntimeStarted { .. }
        | super::RuntimeEvent::LoopStarted { .. }
        | super::RuntimeEvent::ContextReplayed { .. }
        | super::RuntimeEvent::TurnStarted { .. }
        | super::RuntimeEvent::LoopDecision { .. }
        | super::RuntimeEvent::ModelRequested { .. }
        | super::RuntimeEvent::ModelStreamStarted { .. }
        | super::RuntimeEvent::ModelStreamFrame { .. }
        | super::RuntimeEvent::ModelStreamEnded { .. }
        | super::RuntimeEvent::ProviderRetryProgress { .. }
        | super::RuntimeEvent::ModelResponded { .. }
        | super::RuntimeEvent::ToolRequested { .. }
        | super::RuntimeEvent::ToolResponded { .. }
        | super::RuntimeEvent::FinalResponse { .. }
        | super::RuntimeEvent::CompletionCandidateRequested { .. }
        | super::RuntimeEvent::Stopped
        | super::RuntimeEvent::LifecycleHookOutcome { .. }
        | super::RuntimeEvent::SandboxEscalation { .. }
        | super::RuntimeEvent::CompactionRequested { .. }
        | super::RuntimeEvent::SessionStateChanged { .. }
        | super::RuntimeEvent::SessionQueueMutationObserved { .. }
        | super::RuntimeEvent::SessionCommitObserved { .. }
        | super::RuntimeEvent::SessionRecoveryObserved { .. }
        | super::RuntimeEvent::SessionAuthorityProfileObserved { .. }
        | super::RuntimeEvent::Shutdown { .. } => animus_adapters::LogLevel::Info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_SEQUENCE: u32 = 0;
    const FIRST_SEQUENCE_INDEX: usize = 0;
    const SECOND_SEQUENCE_INDEX: usize = 1;
    const EXPECTED_SINGLE_RECORD_COUNT: usize = 1;
    const RAW_PROMPT: &str = "prompt: include private material";
    const RAW_MODEL_OUTPUT: &str = "model response with private material";
    const FAILURE_MESSAGE: &str = "/home/example/private.txt";

    #[test]
    fn projection_uses_safe_runtime_event_labels() -> Result<(), animus_adapters::ObservabilityError>
    {
        let event = crate::RuntimeEvent::LoopDecision {
            turn_index: FIRST_SEQUENCE_INDEX,
            decision: crate::LoopDecisionKind::Model,
        };
        let projection = project_runtime_event(&event, FIRST_SEQUENCE)?;

        assert_eq!(projection.metrics().len(), EXPECTED_SINGLE_RECORD_COUNT);
        assert_eq!(projection.logs().len(), EXPECTED_SINGLE_RECORD_COUNT);
        assert_eq!(
            projection.metrics()[FIRST_SEQUENCE_INDEX].name(),
            RUNTIME_EVENT_METRIC_NAME
        );
        assert_eq!(
            projection.logs()[FIRST_SEQUENCE_INDEX].message(),
            "loop_decision"
        );
        assert!(
            projection.metrics()[FIRST_SEQUENCE_INDEX]
                .labels()
                .iter()
                .any(|label| label.key() == DECISION_LABEL && label.value() == "model")
        );
        Ok(())
    }

    #[test]
    fn projection_omits_raw_prompts_model_outputs_and_paths()
    -> Result<(), animus_adapters::ObservabilityError> {
        let request = animus_adapters::ModelRequest::new(RAW_PROMPT)
            .expect("raw prompt request should be valid in-process data");
        let response = animus_adapters::ModelResponse::new(RAW_MODEL_OUTPUT)
            .expect("raw response should be valid in-process data");
        let run_result = crate::RunReport::new(
            vec![
                crate::RuntimeEvent::ModelRequested { request },
                crate::RuntimeEvent::ModelResponded { response },
                crate::RuntimeEvent::AdapterError {
                    component: crate::AdapterComponent::Model,
                    message: FAILURE_MESSAGE.to_string(),
                },
            ],
            crate::RuntimeStatus::Stopped,
        );

        let projection = project_report(&run_result)?;
        let rendered = format!("{projection:?}");
        assert!(!rendered.contains(RAW_PROMPT));
        assert!(!rendered.contains(RAW_MODEL_OUTPUT));
        assert!(!rendered.contains(FAILURE_MESSAGE));
        assert_eq!(projection.metrics().len(), run_result.events().len());
        assert_eq!(
            projection.metrics()[SECOND_SEQUENCE_INDEX].labels()[SECOND_SEQUENCE_INDEX].value(),
            SECOND_SEQUENCE_INDEX.to_string()
        );
        Ok(())
    }
}
