//! Data-only evaluation contracts for Goose MCP Apps-style UI metadata.

mod classify;
mod data;
mod matrix;
mod norm;

pub use data::Descriptor;
pub use data::DescriptorDraft;
pub use data::LifecyclePolicy;
pub use data::PolicyAdmission;
pub use data::PolicyAdmissionDraft;
pub use data::SizePolicy;
pub use matrix::AdoptionDecision;
pub use matrix::Evaluation;
pub use matrix::EvaluationAxis;
pub use matrix::EvaluationError;
pub use matrix::MCP_APP_EVALUATION_AXES;
pub use matrix::RiskReason;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const RESOURCE_URI_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_resource_uri");
const MIME_PROFILE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_mime_profile");
const TITLE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_title");
const CSP_DOMAIN_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_csp_domain");
const PERMISSION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_permission");
const HOST_CONTEXT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_host_context");
const CALLBACK_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_callback");
const SIZE_POLICY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("mcp_app_size_policy");
const LIFECYCLE_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("mcp_app_lifecycle_policy");

const SIZE_POLICY_BOUNDED: &str = "bounded";
const SIZE_POLICY_HOST_FIT: &str = "host-fit";
const LIFECYCLE_POLICY_PASSIVE: &str = "passive";
const LIFECYCLE_POLICY_BOUNDED_CALLBACKS: &str = "bounded-callbacks";
const LIFECYCLE_POLICY_UNBOUNDED_CALLBACKS: &str = "unbounded-callbacks";
const WILDCARD_DOMAIN_MARKER: char = '*';
const INSECURE_HTTP_PREFIX: &str = "http://";
const CAMERA_PERMISSION: &str = "camera";
const MICROPHONE_PERMISSION: &str = "microphone";
const GEOLOCATION_PERMISSION: &str = "geolocation";
const CLIPBOARD_WRITE_PERMISSION: &str = "clipboard-write";
const SCREEN_CAPTURE_PERMISSION: &str = "screen-capture";
const DIRECT_CHAT_CALLBACK: &str = "direct-chat-message";
const APPEND_MODEL_MESSAGE_CALLBACK: &str = "append-model-message";
const AMBIENT_HOST_CONTROL_CONTEXT: &str = "ambient-host-control";
const HOST_COMMAND_CONTEXT: &str = "host-command";
const SHELL_CONTEXT: &str = "shell";
const MAX_CSP_DOMAIN_ITEMS: usize = 16;
const MAX_PERMISSION_ITEMS: usize = 16;
const MAX_HOST_CONTEXT_ITEMS: usize = 16;
const MAX_CALLBACK_ITEMS: usize = 16;

const _: () = assert!(MAX_CSP_DOMAIN_ITEMS > 0);
const _: () = assert!(MAX_PERMISSION_ITEMS > 0);
const _: () = assert!(MAX_HOST_CONTEXT_ITEMS > 0);
const _: () = assert!(MAX_CALLBACK_ITEMS > 0);

/// Descriptor-side effects that MCP App metadata validation explicitly does not perform.
pub const PROHIBITED_MCP_APP_VALIDATION_EFFECTS: &[&str] = &[
    "HTML rendering",
    "JavaScript execution",
    "remote resource loading",
    "browser permission requests",
    "chat callback dispatch",
    "host control execution",
];

#[cfg(test)]
mod tests;
