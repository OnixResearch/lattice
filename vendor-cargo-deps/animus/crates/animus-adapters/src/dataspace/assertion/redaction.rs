const FIELD: crate::validation::FieldName = crate::validation::FieldName::new("dataspace_assertion_redaction");

const PROMPT_OMITTED: &str = "prompt_omitted";
const PROVIDER_PAYLOAD_OMITTED: &str = "provider_payload_omitted";
const CREDENTIAL_OMITTED: &str = "credential_omitted";
const PATH_REDACTED: &str = "path_redacted";
const BODY_OMITTED: &str = "file_body_omitted";
const TOOL_PAYLOAD_OMITTED: &str = "tool_payload_omitted";
const MODEL_OUTPUT_OMITTED: &str = "model_output_omitted";
const LOGS_BOUNDED: &str = "logs_bounded";

/// Redaction marker proving raw sensitive material was not asserted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    /// Prompt text was omitted.
    PromptOmitted,
    /// Provider payload was omitted.
    ProviderPayloadOmitted,
    /// Credential or secret material was omitted.
    CredentialOmitted,
    /// Full paths were redacted.
    PathRedacted,
    /// File bodies were omitted.
    FileBodyOmitted,
    /// Tool input or output payload was omitted.
    ToolPayloadOmitted,
    /// Model output was omitted.
    ModelOutputOmitted,
    /// Logs were bounded to safe counts or classes.
    LogsBounded,
}

impl Value {
    /// Parses a stable redaction marker label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(FIELD, value, Self::from_normalized)
    }

    fn from_normalized(normalized: &str) -> Option<Self> {
        match normalized {
            PROMPT_OMITTED => Some(Self::PromptOmitted),
            PROVIDER_PAYLOAD_OMITTED => Some(Self::ProviderPayloadOmitted),
            CREDENTIAL_OMITTED => Some(Self::CredentialOmitted),
            PATH_REDACTED => Some(Self::PathRedacted),
            BODY_OMITTED => Some(Self::FileBodyOmitted),
            TOOL_PAYLOAD_OMITTED => Some(Self::ToolPayloadOmitted),
            MODEL_OUTPUT_OMITTED => Some(Self::ModelOutputOmitted),
            LOGS_BOUNDED => Some(Self::LogsBounded),
            _ => None,
        }
    }
}
