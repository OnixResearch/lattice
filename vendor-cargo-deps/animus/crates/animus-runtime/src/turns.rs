#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FieldName {
    value: &'static str,
}

impl FieldName {
    const fn new(value: &'static str) -> Self {
        Self { value }
    }

    const fn as_str(self) -> &'static str {
        self.value
    }
}

const SUMMARY_FIELD: FieldName = FieldName::new("summary");
const DECISION_KIND_FIELD: FieldName = FieldName::new("kind");
const MODEL_PROMPT_FIELD: FieldName = FieldName::new("model_prompt");
const TOOL_NAME_FIELD: FieldName = FieldName::new("tool_name");
const TOOL_INPUT_FIELD: FieldName = FieldName::new("tool_input");
const RESPONSE_MESSAGE_FIELD: FieldName = FieldName::new("response_message");
const COMPLETION_REQUEST_FIELD: FieldName = FieldName::new("completion_request");
const MODEL_KIND: &str = "model";
const TOOL_KIND: &str = "tool";
const RESPOND_KIND: &str = "respond";
const COMPLETION_CANDIDATE_KIND: &str = "completion_candidate";
const EXACT_COMPLETION_REQUEST: &str = "COMPLETE";
const STOP_KIND: &str = "stop";
const NUL_CHARACTER: char = '\0';

/// Runtime-owned input made available to any loop fabric for one turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeLoopTurn {
    summary: String,
    last_model_message: Option<String>,
    last_tool_output: Option<String>,
}

impl RuntimeLoopTurn {
    /// Builds a neutral runtime loop turn.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeLoopContractError`] when `summary` is empty, too long,
    /// or contains invalid control data for an adapter-safe boundary.
    pub fn new(
        summary: &str,
        last_model_message: Option<&str>,
        last_tool_output: Option<&str>,
    ) -> Result<Self, RuntimeLoopContractError> {
        validate_required_text(SUMMARY_FIELD, summary)?;
        validate_optional_text(MODEL_PROMPT_FIELD, last_model_message)?;
        validate_optional_text(TOOL_INPUT_FIELD, last_tool_output)?;
        Ok(Self {
            summary: String::from(summary),
            last_model_message: last_model_message.map(String::from),
            last_tool_output: last_tool_output.map(String::from),
        })
    }

    /// Returns the deterministic runtime summary for this turn.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the previous model message, if a model call completed.
    #[must_use]
    pub fn last_model_message(&self) -> Option<&str> {
        self.last_model_message.as_deref()
    }

    /// Returns the previous tool output, if a tool call completed.
    #[must_use]
    pub fn last_tool_output(&self) -> Option<&str> {
        self.last_tool_output.as_deref()
    }
}

/// Borrowed neutral decision draft produced by a loop fabric before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeLoopDecisionDraft<'a> {
    /// Decision kind label: `model`, `tool`, `respond`, `completion_candidate`, or `stop`.
    pub kind: &'a str,
    /// Model prompt for `model` decisions.
    pub model_prompt: Option<&'a str>,
    /// Tool name for `tool` decisions.
    pub tool_name: Option<&'a str>,
    /// Tool input for `tool` decisions.
    pub tool_input: Option<&'a str>,
    /// Final response message for `respond` decisions.
    pub response_message: Option<&'a str>,
    /// Exact advisory request for `completion_candidate` decisions.
    pub completion_request: Option<&'a str>,
}

impl<'a> RuntimeLoopDecisionDraft<'a> {
    /// Builds a model-dispatch decision draft.
    #[must_use]
    pub const fn model(prompt: &'a str) -> Self {
        Self {
            kind: MODEL_KIND,
            model_prompt: Some(prompt),
            tool_name: None,
            tool_input: None,
            response_message: None,
            completion_request: None,
        }
    }

    /// Builds a tool-dispatch decision draft.
    #[must_use]
    pub const fn tool(name: &'a str, input: &'a str) -> Self {
        Self {
            kind: TOOL_KIND,
            model_prompt: None,
            tool_name: Some(name),
            tool_input: Some(input),
            response_message: None,
            completion_request: None,
        }
    }

    /// Builds a final-response decision draft.
    #[must_use]
    pub const fn respond(message: &'a str) -> Self {
        Self {
            kind: RESPOND_KIND,
            model_prompt: None,
            tool_name: None,
            tool_input: None,
            response_message: Some(message),
            completion_request: None,
        }
    }

    /// Builds an advisory completion-candidate decision draft.
    #[must_use]
    pub const fn completion_candidate(request: &'a str) -> Self {
        Self {
            kind: COMPLETION_CANDIDATE_KIND,
            model_prompt: None,
            tool_name: None,
            tool_input: None,
            response_message: None,
            completion_request: Some(request),
        }
    }

    /// Builds a stop decision draft.
    #[must_use]
    pub const fn stop() -> Self {
        Self {
            kind: STOP_KIND,
            model_prompt: None,
            tool_name: None,
            tool_input: None,
            response_message: None,
            completion_request: None,
        }
    }
}

/// Runtime-owned advisory completion request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCompletionCandidate {
    request: String,
}

impl RuntimeCompletionCandidate {
    /// Returns the exact admitted request spelling.
    #[must_use]
    pub fn request(&self) -> &str {
        self.request.as_str()
    }
}

/// Runtime-owned action selected by a loop fabric after validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeLoopDecision {
    /// Send a request to a model adapter.
    CallModel {
        /// Validated model request.
        request: animus_adapters::ModelRequest,
    },
    /// Send a request to a tool adapter.
    CallTool {
        /// Validated tool request.
        request: animus_adapters::ToolRequest,
    },
    /// Complete the runtime loop with a final model response.
    Respond {
        /// Validated final response.
        response: animus_adapters::ModelResponse,
    },
    /// Request host-owned completion evaluation without a final response.
    CompletionCandidate {
        /// Validated advisory candidate.
        candidate: RuntimeCompletionCandidate,
    },
    /// Stop without producing a final response.
    Stop,
}

impl RuntimeLoopDecision {
    /// Validates a borrowed neutral decision draft.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeLoopContractError`] for empty required fields,
    /// unsupported decision kinds, or adapter DTO validation failures.
    pub fn from_draft(draft: RuntimeLoopDecisionDraft<'_>) -> Result<Self, RuntimeLoopContractError> {
        match normalized_kind(draft.kind)? {
            MODEL_KIND => from_model_draft(draft),
            TOOL_KIND => from_tool_draft(draft),
            RESPOND_KIND => from_respond_draft(draft),
            COMPLETION_CANDIDATE_KIND => from_completion_candidate_draft(draft),
            STOP_KIND => Ok(Self::Stop),
            other => Err(RuntimeLoopContractError::UnsupportedDecisionKind {
                field: DECISION_KIND_FIELD.as_str(),
                value: String::from(other),
            }),
        }
    }

    /// Returns the stable decision kind label.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::CallModel { .. } => MODEL_KIND,
            Self::CallTool { .. } => TOOL_KIND,
            Self::Respond { .. } => RESPOND_KIND,
            Self::CompletionCandidate { .. } => COMPLETION_CANDIDATE_KIND,
            Self::Stop => STOP_KIND,
        }
    }
}

#[cfg(feature = "steel")]
impl From<animus_steel::LoopDecision> for RuntimeLoopDecision {
    fn from(decision: animus_steel::LoopDecision) -> Self {
        match decision {
            animus_steel::LoopDecision::CallModel { request } => Self::CallModel { request },
            animus_steel::LoopDecision::CallTool { request } => Self::CallTool { request },
            animus_steel::LoopDecision::Respond { response } => Self::Respond { response },
            animus_steel::LoopDecision::CompletionCandidate { candidate } => Self::CompletionCandidate {
                candidate: RuntimeCompletionCandidate {
                    request: candidate.request().to_string(),
                },
            },
            animus_steel::LoopDecision::Stop => Self::Stop,
        }
    }
}

#[cfg(feature = "steel")]
impl TryFrom<&RuntimeLoopTurn> for animus_steel::LoopTurn {
    type Error = animus_steel::LoopError;

    fn try_from(turn: &RuntimeLoopTurn) -> Result<Self, Self::Error> {
        Self::new(turn.summary(), turn.last_model_message(), turn.last_tool_output())
    }
}

/// Error returned when loop-fabric input or output fails neutral validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeLoopContractError {
    /// A required field contained no user-visible characters.
    EmptyField {
        /// Field name.
        field: &'static str,
    },
    /// A field exceeded the adapter-safe text limit.
    FieldTooLong {
        /// Field name.
        field: &'static str,
        /// Maximum accepted character count.
        max_chars: usize,
        /// Actual character count.
        actual_chars: usize,
    },
    /// A field contained a NUL character.
    NulCharacter {
        /// Field name.
        field: &'static str,
        /// Byte index where the NUL character starts.
        byte_index: usize,
    },
    /// A fabric returned an unsupported decision kind.
    UnsupportedDecisionKind {
        /// Field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: String,
    },
    /// Legacy raw tool-call payload was supplied where current shape is required.
    CurrentShapeRequired {
        /// Field name.
        field: &'static str,
    },
    /// Adapter DTO validation rejected the decision payload.
    AdapterContract {
        /// Field name.
        field: &'static str,
        /// Safe adapter validation message.
        message: String,
    },
}

impl core::fmt::Display for RuntimeLoopContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::FieldTooLong {
                field,
                max_chars,
                actual_chars,
            } => write!(formatter, "{field} must be at most {max_chars} characters; got {actual_chars}"),
            Self::NulCharacter { field, byte_index } => {
                write!(formatter, "{field} contains a NUL character at byte index {byte_index}")
            }
            Self::UnsupportedDecisionKind { field, value } => {
                write!(formatter, "{field} contains unsupported decision kind {value}")
            }
            Self::CurrentShapeRequired { field } => {
                write!(formatter, "{field} requires current structured tool-call payload")
            }
            Self::AdapterContract { field, message } => {
                write!(formatter, "{field} failed adapter validation: {message}")
            }
        }
    }
}

impl std::error::Error for RuntimeLoopContractError {}

fn from_model_draft(draft: RuntimeLoopDecisionDraft<'_>) -> Result<RuntimeLoopDecision, RuntimeLoopContractError> {
    let prompt = required_payload(MODEL_PROMPT_FIELD, draft.model_prompt)?;
    animus_adapters::ModelRequest::new(prompt)
        .map(|request| RuntimeLoopDecision::CallModel { request })
        .map_err(|error| adapter_error(MODEL_PROMPT_FIELD, &error))
}

fn from_tool_draft(draft: RuntimeLoopDecisionDraft<'_>) -> Result<RuntimeLoopDecision, RuntimeLoopContractError> {
    let _name = required_payload(TOOL_NAME_FIELD, draft.tool_name)?;
    let _input = required_payload(TOOL_INPUT_FIELD, draft.tool_input)?;
    Err(RuntimeLoopContractError::CurrentShapeRequired {
        field: TOOL_INPUT_FIELD.as_str(),
    })
}

fn from_respond_draft(draft: RuntimeLoopDecisionDraft<'_>) -> Result<RuntimeLoopDecision, RuntimeLoopContractError> {
    let message = required_payload(RESPONSE_MESSAGE_FIELD, draft.response_message)?;
    animus_adapters::ModelResponse::new(message)
        .map(|response| RuntimeLoopDecision::Respond { response })
        .map_err(|error| adapter_error(RESPONSE_MESSAGE_FIELD, &error))
}

fn from_completion_candidate_draft(
    draft: RuntimeLoopDecisionDraft<'_>,
) -> Result<RuntimeLoopDecision, RuntimeLoopContractError> {
    let request = required_payload(COMPLETION_REQUEST_FIELD, draft.completion_request)?;
    if request != EXACT_COMPLETION_REQUEST {
        return Err(RuntimeLoopContractError::UnsupportedDecisionKind {
            field: COMPLETION_REQUEST_FIELD.as_str(),
            value: String::from(request),
        });
    }
    Ok(RuntimeLoopDecision::CompletionCandidate {
        candidate: RuntimeCompletionCandidate {
            request: String::from(request),
        },
    })
}

fn normalized_kind(kind: &str) -> Result<&str, RuntimeLoopContractError> {
    validate_required_text(DECISION_KIND_FIELD, kind)?;
    Ok(kind.trim())
}

fn required_payload(field: FieldName, value: Option<&str>) -> Result<&str, RuntimeLoopContractError> {
    let Some(value) = value else {
        return Err(RuntimeLoopContractError::EmptyField { field: field.as_str() });
    };
    validate_required_text(field, value)?;
    Ok(value)
}

fn adapter_error(field: FieldName, error: &animus_adapters::AdapterContractError) -> RuntimeLoopContractError {
    RuntimeLoopContractError::AdapterContract {
        field: field.as_str(),
        message: error.to_string(),
    }
}

fn validate_required_text(field: FieldName, value: &str) -> Result<(), RuntimeLoopContractError> {
    assert!(!field.as_str().is_empty(), "field name must not be empty");
    if value.trim().is_empty() {
        return Err(RuntimeLoopContractError::EmptyField { field: field.as_str() });
    }
    validate_optional_text(field, Some(value))
}

fn validate_optional_text(field: FieldName, value: Option<&str>) -> Result<(), RuntimeLoopContractError> {
    assert!(!field.as_str().is_empty(), "field name must not be empty");
    const {
        assert!(
            animus_adapters::MAX_ADAPTER_FIELD_CHARS >= animus_adapters::MIN_ADAPTER_FIELD_CHARS,
            "adapter field bounds must be ordered"
        );
    };
    let Some(value) = value else {
        return Ok(());
    };
    let actual_chars = value.chars().count();
    if actual_chars > animus_adapters::MAX_ADAPTER_FIELD_CHARS {
        return Err(RuntimeLoopContractError::FieldTooLong {
            field: field.as_str(),
            max_chars: animus_adapters::MAX_ADAPTER_FIELD_CHARS,
            actual_chars,
        });
    }
    if let Some((byte_index, _character)) =
        value.char_indices().find(|(_byte_index, character)| *character == NUL_CHARACTER)
    {
        return Err(RuntimeLoopContractError::NulCharacter {
            field: field.as_str(),
            byte_index,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARY: &str = "intent: build reliable systems";
    const MODEL_MESSAGE: &str = "model answer";
    const TOOL_NAME: &str = "search";
    const TOOL_INPUT: &str = "query";
    const BLANK_TEXT: &str = "   ";
    const UNKNOWN_KIND: &str = "ambient-provider";
    const OVERSIZED_EXTRA_CHAR: usize = 1;

    #[test]
    fn neutral_turn_accepts_valid_inputs() {
        let turn =
            RuntimeLoopTurn::new(SUMMARY, Some(MODEL_MESSAGE), Some(TOOL_INPUT)).expect("valid turn should build");

        assert_eq!(turn.summary(), SUMMARY);
        assert_eq!(turn.last_model_message(), Some(MODEL_MESSAGE));
        assert_eq!(turn.last_tool_output(), Some(TOOL_INPUT));
    }

    #[test]
    fn neutral_decision_accepts_model_response_candidate_and_stop_variants() {
        let decisions = [
            RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::model(SUMMARY))
                .expect("model decision should build"),
            RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::respond(MODEL_MESSAGE))
                .expect("response decision should build"),
            RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::completion_candidate(EXACT_COMPLETION_REQUEST))
                .expect("candidate decision should build"),
            RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::stop()).expect("stop decision should build"),
        ];

        assert_eq!(decisions[0].kind(), MODEL_KIND);
        assert_eq!(decisions[1].kind(), RESPOND_KIND);
        assert_eq!(decisions[2].kind(), COMPLETION_CANDIDATE_KIND);
        assert_eq!(decisions[3].kind(), STOP_KIND);
    }

    #[test]
    fn neutral_decision_rejects_legacy_raw_tool_payload() {
        // r[verify agent_harness_runtime.tool_calling_breaking.current_shape.rejects_legacy]
        let error = RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::tool(TOOL_NAME, TOOL_INPUT))
            .expect_err("legacy raw tool payload should fail");

        assert!(matches!(error, RuntimeLoopContractError::CurrentShapeRequired { .. }));
    }

    #[test]
    fn neutral_decision_rejects_empty_payloads_unknown_kinds_and_oversized_text() {
        let empty = RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::model(BLANK_TEXT))
            .expect_err("empty prompt should fail");
        let kind_rejection = RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft {
            kind: UNKNOWN_KIND,
            model_prompt: None,
            tool_name: None,
            tool_input: None,
            response_message: None,
            completion_request: None,
        })
        .expect_err("unsupported kind should fail");
        let response_payload =
            "x".repeat(animus_adapters::MAX_ADAPTER_FIELD_CHARS.saturating_add(OVERSIZED_EXTRA_CHAR));
        let too_long = RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::respond(response_payload.as_str()))
            .expect_err("oversized response should fail");
        let incidental =
            RuntimeLoopDecision::from_draft(RuntimeLoopDecisionDraft::completion_candidate("not COMPLETE"))
                .expect_err("incidental marker must fail");

        assert!(matches!(empty, RuntimeLoopContractError::EmptyField { .. }));
        assert!(matches!(kind_rejection, RuntimeLoopContractError::UnsupportedDecisionKind { .. }));
        assert!(matches!(
            too_long,
            RuntimeLoopContractError::AdapterContract { .. } | RuntimeLoopContractError::FieldTooLong { .. }
        ));
        assert!(matches!(incidental, RuntimeLoopContractError::UnsupportedDecisionKind { .. }));
    }
}
