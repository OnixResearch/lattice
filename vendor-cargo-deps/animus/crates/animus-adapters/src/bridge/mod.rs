//! Data-only ACP/MCP interop descriptors, pass-through planning, and safe receipts.

#![allow(non_trait_imports)]

const ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("interop_id");
const TRANSPORT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_transport");
const COMMAND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_command_label");
const ENDPOINT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_endpoint_label");
const TOOL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("interop_tool");
const RESOURCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_resource");
const PROMPT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_prompt");
const SCHEMA_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_schema");
const CONTRACT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_result_contract");
const TIMEOUT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("interop_timeout");
const ENV_KEY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("env_key");
const PROVENANCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("interop_provenance_hash");
const PROFILE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("acp_model_profile");
const PERMISSION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("acp_permission_mode");
const SESSION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("acp_session_correlation");
const POLICY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("acp_passthrough_policy");
const LIMITATION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("acp_limitation");
const OPERATION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("interop_operation");
const STATUS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("receipt_status");
const REDACTION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("receipt_redaction");

const TRANSPORT_STDIO: &str = "stdio";
const TRANSPORT_STREAMABLE_HTTP: &str = "streamable_http";
const PERMISSION_READ_ONLY: &str = "read_only";
const PERMISSION_ASK: &str = "ask";
const PERMISSION_UNRESTRICTED: &str = "unrestricted";
const SESSION_NEW: &str = "new_session";
const SESSION_JOIN_BY_LABEL: &str = "join_by_label";
const POLICY_DENY: &str = "deny";
const POLICY_READ_ONLY: &str = "read_only";
const POLICY_INTERSECTION: &str = "intersection";
const OP_DISCOVERY: &str = "discovery";
const OP_SESSION_START: &str = "session_start";
const OP_PASS_THROUGH: &str = "pass_through";
const OP_DENIAL: &str = "denial";
const OP_TIMEOUT: &str = "timeout";
const OP_UNAVAILABLE_BINARY: &str = "unavailable_binary";
const OP_MALFORMED_SCHEMA: &str = "malformed_schema";
const OP_NETWORK_POLICY: &str = "network_policy";
const HASH_SEPARATOR: &str = "|";
const BLAKE3_HEX_CHARS: usize = 64;
const ASCII_HEX_RADIX: u32 = 16;
const ENV_VALUE_SEPARATOR: char = '=';
const RAW_URL_MARKER: &str = "://";
const LABEL_SPACE: char = ' ';
const MIN_COUNT: u32 = 0;

/// Descriptor-side effects that interop validation explicitly does not perform.
pub const PROHIBITED_INTEROP_EFFECTS: &[&str] = &[
    "stdio process launch",
    "HTTP probing",
    "ambient binary discovery",
    "credential lookup",
    "external agent contact",
    "tool execution",
];

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Error raised while validating ACP/MCP interop contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteropError {
    /// Generic adapter contract validation failed.
    Contract(crate::AdapterContractError),
    /// An enum-like interop field contained an unsupported value.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: Text,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
    /// A named collection contained a duplicate value.
    DuplicateValue {
        /// Boundary collection field name.
        field: &'static str,
        /// Duplicated normalized value.
        value: Text,
    },
    /// A descriptor provided an environment value instead of a key name.
    EnvironmentValueLeak,
    /// A command or endpoint included raw launch/network material instead of a label.
    UnsafeRuntimeLabel,
    /// A descriptor omitted required timeout metadata.
    MissingTimeout,
    /// An ACP descriptor attempted an unsafe permission-mode downgrade.
    UnsafePermissionMapping,
    /// An ACP descriptor omitted session correlation policy.
    MissingSessionCorrelation,
    /// A BLAKE3 hash field was malformed.
    InvalidHash {
        /// Boundary field name.
        field: &'static str,
    },
    /// Receipt text contained raw interop material.
    ReceiptLeak,
}

impl From<crate::AdapterContractError> for InteropError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

mod boundary;
mod labels;
mod provider;
mod record;
mod route;
mod server;

#[cfg(test)]
mod bridgecases;
#[cfg(test)]
mod tests;

pub use boundary::McpAdvertisementDecision;
pub use boundary::McpAdvertisementInput;
pub use boundary::McpArgumentStatus;
pub use boundary::McpDispatchDecision;
pub use boundary::McpDispatchRequest;
pub use boundary::McpMutationStatus;
pub use boundary::McpReceipt;
pub use boundary::McpReceiptDraft;
pub use boundary::McpReceiptTruncationState;
pub use boundary::McpServerTrust;
pub use boundary::McpToolMapping;
pub use boundary::McpToolMappingDraft;
pub use boundary::McpTransportDecision;
pub use boundary::McpTransportFailureClass;
pub use boundary::McpTransportState;
pub use boundary::evaluate_mcp_dispatch;
pub use boundary::evaluate_mcp_transport;
pub use boundary::plan_mcp_advertisement;
pub use boundary::validate_mcp_receipt_text;
pub use labels::AcpPermissionMode;
pub use labels::AcpSessionCorrelation;
pub use labels::AcpToolPassThroughPolicy;
pub use labels::InteropOperationKind;
pub use labels::McpTransportKind;
pub use provider::AcpDescriptor;
pub use provider::AcpDescriptorDraft;
pub use record::InteropReceipt;
pub use record::InteropReceiptDraft;
pub use record::validate_interop_receipt_text;
pub use route::ToolPassThroughInput;
pub use route::ToolPassThroughResolution;
pub use route::resolve_tool_pass_through;
pub use server::McpDescriptor;
pub use server::McpDescriptorDraft;
fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, InteropError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or(InteropError::UnsupportedValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

fn required_text(field_name: crate::validation::FieldName, value: &str) -> Result<Text, InteropError> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(InteropError::from)
}

fn text_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
    required: bool,
) -> Result<Collection<Text>, InteropError> {
    if required && values.is_empty() {
        return Err(InteropError::EmptyCollection {
            field: field_name.as_str(),
        });
    }
    let mut normalized: Collection<Text> = Collection::with_capacity(values.len());
    for value in values {
        let text = required_text(field_name, value)?;
        if normalized.iter().any(|existing| existing == &text) {
            return Err(InteropError::DuplicateValue {
                field: field_name.as_str(),
                value: text,
            });
        }
        normalized.push(text);
    }
    Ok(normalized)
}

fn contains_text(values: &[Text], needle: &str) -> bool {
    values.iter().any(|value| value.as_str() == needle)
}

#[derive(Clone, Copy)]
struct HashInput<'a> {
    field: &'static str,
    value: &'a str,
}

fn validate_hash(input: HashInput<'_>) -> Result<(), InteropError> {
    let hash_len = input.value.chars().count();
    if hash_len != BLAKE3_HEX_CHARS {
        return Err(InteropError::InvalidHash { field: input.field });
    }
    if input
        .value
        .chars()
        .any(|character| character.to_digit(ASCII_HEX_RADIX).is_none() || character.is_uppercase())
    {
        return Err(InteropError::InvalidHash { field: input.field });
    }
    Ok(())
}

fn hash_parts(parts: &[&str]) -> Text {
    let mut text = Text::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}
