use super::Collection;
use super::HashInput;
use super::InteropError;
use super::MCP_CAPABILITY_FIELD;
use super::MCP_RECEIPT_RESOURCE_FIELD;
use super::MCP_RESPONSE_HASH_FIELD;
use super::MCP_SERVER_LABEL_FIELD;
use super::MCP_TOOL_NAME_FIELD;
use super::MCP_TRUNCATION_BOUNDED;
use super::MCP_TRUNCATION_FIELD;
use super::MCP_TRUNCATION_NONE;
use super::MCP_TRUNCATION_REJECTED;
use super::RECEIPT_LEAK_MARKERS;
use super::SCHEMA_FIELD;
use super::STATUS_FIELD;
use super::Text;
use super::hash_parts;
use super::parse_value;
use super::required_text;
use super::text_collection;
use super::validate_hash;

/// Truncation state recorded in safe MCP bridge receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpReceiptTruncationState {
    /// No truncation occurred.
    None,
    /// Visible result was bounded and truncated by host policy.
    Bounded,
    /// Oversized result was rejected instead of copied into context or receipts.
    OversizedRejected,
}

impl McpReceiptTruncationState {
    /// Parses a stable truncation-state label.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when the label is unsupported.
    pub fn parse(value: &str) -> Result<Self, InteropError> {
        parse_value(MCP_TRUNCATION_FIELD, value, Self::from_normalized)
    }

    /// Returns the stable label for this truncation state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => MCP_TRUNCATION_NONE,
            Self::Bounded => MCP_TRUNCATION_BOUNDED,
            Self::OversizedRejected => MCP_TRUNCATION_REJECTED,
        }
    }

    const fn from_normalized(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"none" => Some(Self::None),
            b"bounded" => Some(Self::Bounded),
            b"oversized_rejected" => Some(Self::OversizedRejected),
            _unsupported => None,
        }
    }
}

/// Borrowed safe MCP bridge receipt metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpReceiptDraft<'a> {
    /// Host-redacted server label.
    pub server_label: &'a str,
    /// Animus tool name.
    pub tool_name: &'a str,
    /// Strict schema identity.
    pub schema_identity: &'a str,
    /// Status class label.
    pub status: &'a str,
    /// Capability labels included for auditability.
    pub capability_labels: &'a [&'a str],
    /// Redacted resource identities, never raw URIs or paths.
    pub redacted_resource_ids: &'a [&'a str],
    /// Truncation-state label.
    pub truncation_state: &'a str,
    /// BLAKE3 hash of bounded response metadata or redacted failure class.
    pub response_hash: &'a str,
}

// r[impl agent_harness_runtime.claude_code_mcp.receipts]
/// Safe MCP bridge receipt with bounded labels and hashes only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpReceipt {
    server_label: Text,
    tool_name: Text,
    schema_identity: Text,
    status: Text,
    capability_labels: Collection<Text>,
    redacted_resource_ids: Collection<Text>,
    truncation_state: McpReceiptTruncationState,
    response_hash: Text,
    identity_hash: Text,
}

impl McpReceipt {
    /// Builds a safe receipt for an MCP-backed bridge action.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when receipt metadata is malformed, hashes are
    /// invalid, redacted resources are absent, or unsafe/raw payload markers leak.
    pub fn new(draft: &McpReceiptDraft<'_>) -> Result<Self, InteropError> {
        let server_label = safe_receipt_text(MCP_SERVER_LABEL_FIELD, draft.server_label)?;
        let tool_name = safe_receipt_text(MCP_TOOL_NAME_FIELD, draft.tool_name)?;
        let schema_identity = safe_receipt_text(SCHEMA_FIELD, draft.schema_identity)?;
        let status = safe_receipt_text(STATUS_FIELD, draft.status)?;
        let capability_labels = safe_receipt_text_collection(MCP_CAPABILITY_FIELD, draft.capability_labels, true)?;
        let redacted_resource_ids =
            safe_receipt_text_collection(MCP_RECEIPT_RESOURCE_FIELD, draft.redacted_resource_ids, true)?;
        let truncation_state = McpReceiptTruncationState::parse(draft.truncation_state)?;
        validate_hash(HashInput {
            field: MCP_RESPONSE_HASH_FIELD.as_str(),
            value: draft.response_hash,
        })?;
        let response_hash = Text::from(draft.response_hash);
        let identity_hash = hash_parts(&[
            server_label.as_str(),
            tool_name.as_str(),
            schema_identity.as_str(),
            status.as_str(),
            truncation_state.as_str(),
            response_hash.as_str(),
        ]);
        Ok(Self {
            server_label,
            tool_name,
            schema_identity,
            status,
            capability_labels,
            redacted_resource_ids,
            truncation_state,
            response_hash,
            identity_hash,
        })
    }

    /// Returns the server label.
    #[must_use]
    pub fn server_label(&self) -> &str {
        self.server_label.as_str()
    }

    /// Returns the tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns the status class label.
    #[must_use]
    pub fn status(&self) -> &str {
        self.status.as_str()
    }

    /// Returns capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[Text] {
        self.capability_labels.as_slice()
    }

    /// Returns redacted resource identities.
    #[must_use]
    pub fn redacted_resource_ids(&self) -> &[Text] {
        self.redacted_resource_ids.as_slice()
    }

    /// Returns truncation state.
    #[must_use]
    pub const fn truncation_state(&self) -> McpReceiptTruncationState {
        self.truncation_state
    }

    /// Returns the response hash.
    #[must_use]
    pub fn response_hash(&self) -> &str {
        self.response_hash.as_str()
    }

    /// Returns the receipt identity hash.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }
}

/// Validates rendered MCP bridge receipt text for raw transport or sensitive data.
///
/// # Errors
///
/// Returns [`InteropError::ReceiptLeak`] when raw payloads, credentials, full
/// paths, provider payloads, URLs, arguments, or unbounded result markers appear.
// r[impl agent_harness_runtime.claude_code_mcp.receipts.malformed]
pub fn validate_mcp_receipt_text(text: &str) -> Result<(), InteropError> {
    let lowered = text.to_ascii_lowercase();
    if contains_mcp_receipt_leak_marker(lowered.as_str()) {
        return Err(InteropError::ReceiptLeak);
    }
    Ok(())
}

fn safe_receipt_text(field_name: crate::validation::FieldName, value: &str) -> Result<Text, InteropError> {
    let text = required_text(field_name, value)?;
    validate_mcp_receipt_text(text.as_str())?;
    Ok(text)
}

fn safe_receipt_text_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
    required: bool,
) -> Result<Collection<Text>, InteropError> {
    let normalized = text_collection(field_name, values, required)?;
    for value in &normalized {
        validate_mcp_receipt_text(value.as_str())?;
    }
    Ok(normalized)
}

fn contains_mcp_receipt_leak_marker(lowered: &str) -> bool {
    RECEIPT_LEAK_MARKERS.iter().any(|marker| lowered.contains(marker))
}
