use super::{
    Collection, HashInput, InteropError, McpDescriptor, SCHEMA_FIELD, STATUS_FIELD, Text,
    hash_parts, parse_value, required_text, text_collection, validate_hash,
};

mod admit;
mod audit;
mod flow;
mod shape;

pub use admit::{
    McpAdvertisementDecision, McpAdvertisementInput, McpServerTrust, plan_mcp_advertisement,
};
pub use audit::{
    McpReceipt, McpReceiptDraft, McpReceiptTruncationState, validate_mcp_receipt_text,
};
pub use flow::{
    McpArgumentStatus, McpDispatchDecision, McpDispatchRequest, McpMutationStatus,
    McpTransportDecision, McpTransportFailureClass, McpTransportState, evaluate_mcp_dispatch,
    evaluate_mcp_transport,
};
pub use shape::{McpToolMapping, McpToolMappingDraft};

const MCP_SERVER_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_server_label");
const MCP_TOOL_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_tool_name");
const MCP_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_capability_label");
const MCP_SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_source_metadata");
const MCP_RESULT_BOUNDS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_result_bounds");
const MCP_RESOURCE_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_resource_policy_ref");
const MCP_ABILITY_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_ability_policy_ref");
const MCP_RECEIPT_RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_redacted_resource_id");
const MCP_TRUNCATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_truncation_state");
const MCP_RESPONSE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_response_hash");

const MCP_TRUNCATION_NONE: &str = "none";
const MCP_TRUNCATION_BOUNDED: &str = "bounded";
const MCP_TRUNCATION_REJECTED: &str = "oversized_rejected";
const MCP_MAPPING_DESCRIPTION: &str =
    "MCP-backed tool mapped into Animus-owned tool contracts before dispatch.";
const MCP_MAPPING_PROMPT_SNIPPET: &str = "Use this MCP-backed tool only after Animus argument, capability, mutation, and result checks pass.";
const MCP_MAPPING_GUIDELINE: &str =
    "MCP transport, process management, discovery, network, and credentials remain host-owned.";
const MCP_MAPPING_GUIDELINES: &[&str] = &[MCP_MAPPING_GUIDELINE];
const RECEIPT_RAW_HTTP_MARKER: &str = "http://";
const RECEIPT_RAW_HTTPS_MARKER: &str = "https://";
const RECEIPT_API_KEY_MARKER: &str = "api_key";
const RECEIPT_SECRET_MARKER: &str = "secret";
const RECEIPT_CREDENTIAL_MARKER: &str = "credential";
const RECEIPT_RAW_PAYLOAD_MARKER: &str = "raw_payload";
const RECEIPT_TRANSPORT_PAYLOAD_MARKER: &str = "transport_payload";
const RECEIPT_PROVIDER_PAYLOAD_MARKER: &str = "provider_payload";
const RECEIPT_UNBOUNDED_RESULT_MARKER: &str = "unbounded_external_result";
const RECEIPT_HOME_PATH_MARKER: &str = "/home/";
const RECEIPT_ROOT_PATH_MARKER: &str = " path=/";
const RECEIPT_ARGS_MARKER: &str = "args=";
const RECEIPT_LEAK_MARKERS: &[&str] = &[
    RECEIPT_RAW_HTTP_MARKER,
    RECEIPT_RAW_HTTPS_MARKER,
    RECEIPT_API_KEY_MARKER,
    RECEIPT_SECRET_MARKER,
    RECEIPT_CREDENTIAL_MARKER,
    RECEIPT_RAW_PAYLOAD_MARKER,
    RECEIPT_TRANSPORT_PAYLOAD_MARKER,
    RECEIPT_PROVIDER_PAYLOAD_MARKER,
    RECEIPT_UNBOUNDED_RESULT_MARKER,
    RECEIPT_HOME_PATH_MARKER,
    RECEIPT_ROOT_PATH_MARKER,
    RECEIPT_ARGS_MARKER,
];

fn required_optional_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, InteropError> {
    match value {
        Some(text) => required_text(field_name, text),
        None => Err(InteropError::Contract(
            crate::AdapterContractError::EmptyField {
                field: field_name.as_str(),
            },
        )),
    }
}

fn borrowed_texts(values: &[Text]) -> Collection<&str> {
    values.iter().map(alloc::string::String::as_str).collect()
}
