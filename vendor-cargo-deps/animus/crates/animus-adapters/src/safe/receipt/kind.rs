/// Status class recorded by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Status {
    /// The observed operation completed.
    Completed,
    /// The observed operation was blocked before unsafe work happened.
    Blocked,
    /// The observed operation was denied by policy or authorization.
    Denied,
    /// The observed operation failed at a typed adapter boundary.
    Failed,
    /// The observed operation was accepted only after redaction.
    Redacted,
}

impl Status {
    /// Returns a stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Blocked => "blocked",
            Self::Denied => "denied",
            Self::Failed => "failed",
            Self::Redacted => "redacted",
        }
    }
}

/// Issue code recorded by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IssueCode {
    /// No issue was observed.
    None,
    /// Model dispatch evidence.
    ModelDispatch,
    /// Tool execution evidence.
    ToolExecution,
    /// Authorization denial evidence.
    AuthorizationDenied,
    /// Runtime block evidence.
    RuntimeBlocked,
    /// Final response evidence.
    FinalResponse,
    /// Adapter error evidence.
    AdapterError,
    /// Provenance observation evidence.
    ProvenanceObservation,
    /// Unsafe material was rejected or redacted.
    LeakRejected,
}

impl IssueCode {
    /// Returns a stable issue-code label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ModelDispatch => "model_dispatch",
            Self::ToolExecution => "tool_execution",
            Self::AuthorizationDenied => "authorization_denied",
            Self::RuntimeBlocked => "runtime_blocked",
            Self::FinalResponse => "final_response",
            Self::AdapterError => "adapter_error",
            Self::ProvenanceObservation => "provenance_observation",
            Self::LeakRejected => "leak_rejected",
        }
    }
}

/// Effect or mutation class observed by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Operation {
    /// Model or provider dispatch.
    ModelDispatch,
    /// Tool execution.
    ToolExecution,
    /// Authorization or capability decision.
    AuthorizationDecision,
    /// Runtime policy block.
    RuntimeBlock,
    /// Final response emission.
    FinalResponse,
    /// Adapter boundary failure.
    AdapterBoundary,
    /// Provenance observation.
    ProvenanceObservation,
    /// Host-owned mutation.
    Mutation,
}

impl Operation {
    /// Returns a stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelDispatch => "model_dispatch",
            Self::ToolExecution => "tool_execution",
            Self::AuthorizationDecision => "authorization_decision",
            Self::RuntimeBlock => "runtime_block",
            Self::FinalResponse => "final_response",
            Self::AdapterBoundary => "adapter_boundary",
            Self::ProvenanceObservation => "provenance_observation",
            Self::Mutation => "mutation",
        }
    }
}

/// Redaction marker recorded by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Redaction {
    /// Raw prompt text was omitted.
    PromptOmitted,
    /// Provider JSON or response bodies were omitted.
    ProviderPayloadOmitted,
    /// Credential-looking values were omitted.
    CredentialOmitted,
    /// Absolute paths were omitted or reduced to safe labels.
    AbsolutePathOmitted,
    /// Raw tool input was omitted.
    ToolInputOmitted,
    /// Raw tool output was omitted.
    ToolOutputOmitted,
    /// Raw model output was omitted.
    ModelOutputOmitted,
    /// Output was bounded to counts, labels, or hashes.
    OutputBounded,
    /// Payload identity was represented by BLAKE3 hash only.
    PayloadHashOnly,
}

impl Redaction {
    /// Returns a stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PromptOmitted => "prompt_omitted",
            Self::ProviderPayloadOmitted => "provider_payload_omitted",
            Self::CredentialOmitted => "credential_omitted",
            Self::AbsolutePathOmitted => "absolute_path_omitted",
            Self::ToolInputOmitted => "tool_input_omitted",
            Self::ToolOutputOmitted => "tool_output_omitted",
            Self::ModelOutputOmitted => "model_output_omitted",
            Self::OutputBounded => "output_bounded",
            Self::PayloadHashOnly => "payload_hash_only",
        }
    }
}

/// Unsafe material class rejected by safe receipt validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeakClass {
    /// Raw prompt text was detected.
    RawPrompt,
    /// Provider JSON was detected.
    ProviderJson,
    /// Credential-looking text was detected.
    Credential,
    /// Absolute path text was detected.
    AbsolutePath,
    /// Raw tool input was detected.
    RawToolInput,
    /// Raw tool output was detected.
    RawToolOutput,
    /// Raw model output was detected.
    RawModelOutput,
    /// Unbounded or oversized output was detected.
    UnboundedOutput,
}

impl LeakClass {
    /// Returns a stable leak-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::ProviderJson => "provider_json",
            Self::Credential => "credential",
            Self::AbsolutePath => "absolute_path",
            Self::RawToolInput => "raw_tool_input",
            Self::RawToolOutput => "raw_tool_output",
            Self::RawModelOutput => "raw_model_output",
            Self::UnboundedOutput => "unbounded_output",
        }
    }
}
