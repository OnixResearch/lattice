/// Stable contract identity for public tool request DTOs.
pub const TOOL_REQUEST_CONTRACT_IDENTITY: &str = crate::tool::TOOL_REQUEST_CONTRACT_IDENTITY;
/// Stable contract identity for public tool response DTOs.
pub const TOOL_RESPONSE_CONTRACT_IDENTITY: &str = crate::tool::TOOL_RESPONSE_CONTRACT_IDENTITY;

/// Request sent from the harness to a tool component.
pub type ToolRequest = crate::tool::Request;

/// Response returned by a tool component.
pub type ToolResponse = crate::tool::Response;
