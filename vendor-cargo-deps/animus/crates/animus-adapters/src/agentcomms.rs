//! Transport-neutral agent-comms DTOs, projections, and safe receipts.
//!
//! This module keeps route admission, bounded frame metadata, projection, and
//! receipt hygiene reusable across host transports such as Iroh, local IPC,
//! WebSocket, room-bus, MCP, and future dataspace adapters.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        tigerstyle::underscore_in_module_filename
    )
)]

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::validation::FieldName;
#[cfg_attr(animus_octet, allow(tigerstyle::non_trait_imports))]
use crate::{
    AdapterContractError, Assertion, AssertionRedaction, AssertionStatus, HostControlEnvelope,
    HostEvent, HostSelector, HostSemanticStatus, HostTypedError, OrchestrationEvent, SafeHash,
    Scope, SubagentEvent, Subscription, normalized_adapter_field,
};

const ROUTE_LABEL_FIELD: FieldName = FieldName::new("agent_comms_route_label");
const TRANSPORT_FAMILY_FIELD: FieldName = FieldName::new("agent_comms_transport_family");
const ENDPOINT_HASH_FIELD: FieldName = FieldName::new("agent_comms_endpoint_hash");
const POLICY_HASH_FIELD: FieldName = FieldName::new("agent_comms_policy_hash");
const ADMISSION_FIELD: FieldName = FieldName::new("agent_comms_admission");
const FRAME_ID_FIELD: FieldName = FieldName::new("agent_comms_frame_id");
const FRAME_KIND_FIELD: FieldName = FieldName::new("agent_comms_frame_kind");
const PAYLOAD_HASH_FIELD: FieldName = FieldName::new("agent_comms_payload_hash");
const REDACTION_FIELD: FieldName = FieldName::new("agent_comms_redaction");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("agent_comms_receipt_status");

const ROUTE_HASH_SCHEMA: &str = "animus.agent-comms.route.v1";
const FRAME_HASH_SCHEMA: &str = "animus.agent-comms.frame.v1";
const RECEIPT_HASH_SCHEMA: &str = "animus.agent-comms.receipt.v1";

const ROUTE_METADATA_ONLY: &str = "route_metadata_only";
const COMMAND_PAYLOAD_OMITTED: &str = "command_payload_omitted";
const EVENT_PAYLOAD_OMITTED: &str = "event_payload_omitted";
const SUBAGENT_PAYLOAD_OMITTED: &str = "subagent_payload_omitted";
const ORCHESTRATION_EVENT_WITH_TARGET_ITEM_COUNT: u32 = 2;
const ORCHESTRATION_EVENT_WITHOUT_TARGET_ITEM_COUNT: u32 = 1;

const COMMAND_SUMMARY_PARTS: u32 = 6;
const EVENT_SUMMARY_PARTS: u32 = 5;
const SUBAGENT_SUMMARY_PARTS: u32 = 5;

const NO_ENDPOINT_HASH: &str = "no-endpoint-hash";
const NO_LABEL: &str = "no-label";
const NO_SELECTOR: &str = "no-selector";
const NO_STATUS: &str = "no-status";
const NO_ERROR_CODE: &str = "no-error-code";
const NO_TARGET_OWNER: &str = "no-target-owner";
const NO_EVIDENCE: &str = "no-evidence";
const NO_MESSAGE: &str = "no-message";

const IROH_FAMILY: &str = "iroh";
const LOCAL_IPC_FAMILY: &str = "local_ipc";
const WEBSOCKET_FAMILY: &str = "websocket";
const MATRIX_FAMILY: &str = "matrix";
const MCP_FAMILY: &str = "mcp";
const DATASPACE_FAMILY: &str = "dataspace";

const ADMITTED: &str = "admitted";
const DENIED: &str = "denied";
const PROMPT_REQUIRED: &str = "prompt_required";
const MISSING: &str = "missing";

const ROUTE_METADATA_KIND: &str = "route_metadata";
const DATASPACE_ASSERTION_KIND: &str = "dataspace_assertion";
const ORCHESTRATION_EVENT_KIND: &str = "orchestration_event";
const HEADLESS_COMMAND_KIND: &str = "headless_command";
const HEADLESS_EVENT_KIND: &str = "headless_event";
const SUBAGENT_LIFECYCLE_KIND: &str = "subagent_lifecycle";

const ASSERTION_RED_ACTION_PROMPT_OMITTED: &str = "prompt_omitted";
const ASSERTION_RED_ACTION_PROVIDER_PAYLOAD_OMITTED: &str = "provider_payload_omitted";
const ASSERTION_RED_ACTION_CREDENTIAL_OMITTED: &str = "credential_omitted";
const ASSERTION_RED_ACTION_PATH_REDACTED: &str = "path_redacted";
const ASSERTION_RED_ACTION_FILE_BODY_OMITTED: &str = "file_body_omitted";
const ASSERTION_RED_ACTION_TOOL_PAYLOAD_OMITTED: &str = "tool_payload_omitted";
const ASSERTION_RED_ACTION_MODEL_OUTPUT_OMITTED: &str = "model_output_omitted";
const ASSERTION_RED_ACTION_LOGS_BOUNDED: &str = "logs_bounded";

const RECEIPT_STATUS_ADMITTED: &str = "admitted";
const RECEIPT_STATUS_DENIED: &str = "denied";
const RECEIPT_STATUS_COMPLETED: &str = "completed";
const RECEIPT_STATUS_CANCELLED: &str = "cancelled";
const RECEIPT_STATUS_TIMEOUT: &str = "timeout";
const RECEIPT_STATUS_UNAVAILABLE: &str = "unavailable";
const RECEIPT_STATUS_FAILED: &str = "failed";

const UNSAFE_MARKERS: &[&str] = &[
    "iroh://",
    "ticket:",
    "nodeaddr:",
    "relay=",
    "socket:",
    "http://",
    "https://",
    "authorization:",
    "raw_ticket",
    "private_key",
    "secret",
    "raw_prompt:",
    "provider_payload:",
    "credential:",
    "file_body:",
    "tool_input:",
    "tool_output:",
    "model_output:",
    "unbounded_log:",
    "api_key=",
    "sk-",
];

const _: () = assert!(COMMAND_SUMMARY_PARTS > 0);
const _: () = assert!(EVENT_SUMMARY_PARTS > 0);
const _: () = assert!(SUBAGENT_SUMMARY_PARTS > 0);

/// Transport-family label for an agent-comms route.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCommsTransportFamily {
    /// Iroh-backed ticketed transport.
    Iroh,
    /// Local IPC transport.
    LocalIpc,
    /// WebSocket transport.
    WebSocket,
    /// Matrix transport.
    Matrix,
    /// MCP transport.
    Mcp,
    /// Dataspace transport.
    Dataspace,
}

impl AgentCommsTransportFamily {
    /// Parses a stable transport-family label.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the family label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, AgentCommsError> {
        parse_value(
            TRANSPORT_FAMILY_FIELD,
            value,
            |normalized| match normalized {
                IROH_FAMILY => Some(Self::Iroh),
                LOCAL_IPC_FAMILY => Some(Self::LocalIpc),
                WEBSOCKET_FAMILY => Some(Self::WebSocket),
                MATRIX_FAMILY => Some(Self::Matrix),
                MCP_FAMILY => Some(Self::Mcp),
                DATASPACE_FAMILY => Some(Self::Dataspace),
                _unsupported => None,
            },
        )
    }

    /// Returns the stable transport-family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Iroh => IROH_FAMILY,
            Self::LocalIpc => LOCAL_IPC_FAMILY,
            Self::WebSocket => WEBSOCKET_FAMILY,
            Self::Matrix => MATRIX_FAMILY,
            Self::Mcp => MCP_FAMILY,
            Self::Dataspace => DATASPACE_FAMILY,
        }
    }
}

/// Admission posture supplied by a host route.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCommsAdmission {
    /// Route is admitted.
    Admitted,
    /// Route is denied.
    Denied,
    /// Route requires out-of-band prompt admission.
    PromptRequired,
    /// Admission evidence is missing.
    Missing,
}

impl AgentCommsAdmission {
    /// Parses a stable admission label.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the admission label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, AgentCommsError> {
        parse_value(ADMISSION_FIELD, value, |normalized| match normalized {
            ADMITTED => Some(Self::Admitted),
            DENIED => Some(Self::Denied),
            PROMPT_REQUIRED => Some(Self::PromptRequired),
            MISSING => Some(Self::Missing),
            _unsupported => None,
        })
    }

    /// Returns the stable admission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => ADMITTED,
            Self::Denied => DENIED,
            Self::PromptRequired => PROMPT_REQUIRED,
            Self::Missing => MISSING,
        }
    }

    /// Returns whether the route may perform effects.
    #[must_use]
    pub const fn allows_effects(self) -> bool {
        matches!(self, Self::Admitted)
    }
}

/// Frame family projected over an agent-comms route.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCommsFrameKind {
    /// Route-admission metadata receipt.
    RouteMetadata,
    /// Redacted dataspace assertion projection.
    DataspaceAssertion,
    /// Host orchestration event projection.
    OrchestrationEvent,
    /// Headless command projection.
    HeadlessCommand,
    /// Headless event projection.
    HeadlessEvent,
    /// Subagent lifecycle projection.
    SubagentLifecycle,
}

impl AgentCommsFrameKind {
    /// Parses a stable frame-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the frame-kind label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, AgentCommsError> {
        parse_value(FRAME_KIND_FIELD, value, |normalized| match normalized {
            ROUTE_METADATA_KIND => Some(Self::RouteMetadata),
            DATASPACE_ASSERTION_KIND => Some(Self::DataspaceAssertion),
            ORCHESTRATION_EVENT_KIND => Some(Self::OrchestrationEvent),
            HEADLESS_COMMAND_KIND => Some(Self::HeadlessCommand),
            HEADLESS_EVENT_KIND => Some(Self::HeadlessEvent),
            SUBAGENT_LIFECYCLE_KIND => Some(Self::SubagentLifecycle),
            _unsupported => None,
        })
    }

    /// Returns the stable frame-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RouteMetadata => ROUTE_METADATA_KIND,
            Self::DataspaceAssertion => DATASPACE_ASSERTION_KIND,
            Self::OrchestrationEvent => ORCHESTRATION_EVENT_KIND,
            Self::HeadlessCommand => HEADLESS_COMMAND_KIND,
            Self::HeadlessEvent => HEADLESS_EVENT_KIND,
            Self::SubagentLifecycle => SUBAGENT_LIFECYCLE_KIND,
        }
    }
}

/// Safe redaction marker used by frames and receipts.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCommsRedaction {
    /// Prompt text was omitted.
    PromptOmitted,
    /// Provider payload was omitted.
    ProviderPayloadOmitted,
    /// Credential material was omitted.
    CredentialOmitted,
    /// Host path was redacted.
    PathRedacted,
    /// File body was omitted.
    FileBodyOmitted,
    /// Tool payload was omitted.
    ToolPayloadOmitted,
    /// Model output was omitted.
    ModelOutputOmitted,
    /// Logs were bounded.
    LogsBounded,
    /// Headless command payload was omitted.
    CommandPayloadOmitted,
    /// Headless event payload was omitted.
    EventPayloadOmitted,
    /// Subagent lifecycle payload was omitted.
    SubagentPayloadOmitted,
    /// Route metadata only was captured.
    RouteMetadataOnly,
}

impl AgentCommsRedaction {
    /// Parses a stable redaction marker.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the redaction marker is unsupported.
    pub fn parse(value: &str) -> Result<Self, AgentCommsError> {
        parse_value(REDACTION_FIELD, value, |normalized| match normalized {
            ASSERTION_RED_ACTION_PROMPT_OMITTED => Some(Self::PromptOmitted),
            ASSERTION_RED_ACTION_PROVIDER_PAYLOAD_OMITTED => Some(Self::ProviderPayloadOmitted),
            ASSERTION_RED_ACTION_CREDENTIAL_OMITTED => Some(Self::CredentialOmitted),
            ASSERTION_RED_ACTION_PATH_REDACTED => Some(Self::PathRedacted),
            ASSERTION_RED_ACTION_FILE_BODY_OMITTED => Some(Self::FileBodyOmitted),
            ASSERTION_RED_ACTION_TOOL_PAYLOAD_OMITTED => Some(Self::ToolPayloadOmitted),
            ASSERTION_RED_ACTION_MODEL_OUTPUT_OMITTED => Some(Self::ModelOutputOmitted),
            ASSERTION_RED_ACTION_LOGS_BOUNDED => Some(Self::LogsBounded),
            COMMAND_PAYLOAD_OMITTED => Some(Self::CommandPayloadOmitted),
            EVENT_PAYLOAD_OMITTED => Some(Self::EventPayloadOmitted),
            SUBAGENT_PAYLOAD_OMITTED => Some(Self::SubagentPayloadOmitted),
            ROUTE_METADATA_ONLY => Some(Self::RouteMetadataOnly),
            _unsupported => None,
        })
    }

    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PromptOmitted => ASSERTION_RED_ACTION_PROMPT_OMITTED,
            Self::ProviderPayloadOmitted => ASSERTION_RED_ACTION_PROVIDER_PAYLOAD_OMITTED,
            Self::CredentialOmitted => ASSERTION_RED_ACTION_CREDENTIAL_OMITTED,
            Self::PathRedacted => ASSERTION_RED_ACTION_PATH_REDACTED,
            Self::FileBodyOmitted => ASSERTION_RED_ACTION_FILE_BODY_OMITTED,
            Self::ToolPayloadOmitted => ASSERTION_RED_ACTION_TOOL_PAYLOAD_OMITTED,
            Self::ModelOutputOmitted => ASSERTION_RED_ACTION_MODEL_OUTPUT_OMITTED,
            Self::LogsBounded => ASSERTION_RED_ACTION_LOGS_BOUNDED,
            Self::CommandPayloadOmitted => COMMAND_PAYLOAD_OMITTED,
            Self::EventPayloadOmitted => EVENT_PAYLOAD_OMITTED,
            Self::SubagentPayloadOmitted => SUBAGENT_PAYLOAD_OMITTED,
            Self::RouteMetadataOnly => ROUTE_METADATA_ONLY,
        }
    }
}

/// Safe status class stored in receipts.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCommsReceiptStatus {
    /// Receipt records admitted work.
    Admitted,
    /// Receipt records denied work.
    Denied,
    /// Receipt records completed work.
    Completed,
    /// Receipt records cancelled work.
    Cancelled,
    /// Receipt records timeout.
    Timeout,
    /// Receipt records unavailable transport.
    Unavailable,
    /// Receipt records a failed outcome.
    Failed,
}

impl AgentCommsReceiptStatus {
    /// Parses a stable receipt status label.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the status label is unsupported.
    pub fn parse(value: &str) -> Result<Self, AgentCommsError> {
        parse_value(RECEIPT_STATUS_FIELD, value, |normalized| match normalized {
            RECEIPT_STATUS_ADMITTED => Some(Self::Admitted),
            RECEIPT_STATUS_DENIED => Some(Self::Denied),
            RECEIPT_STATUS_COMPLETED => Some(Self::Completed),
            RECEIPT_STATUS_CANCELLED => Some(Self::Cancelled),
            RECEIPT_STATUS_TIMEOUT => Some(Self::Timeout),
            RECEIPT_STATUS_UNAVAILABLE => Some(Self::Unavailable),
            RECEIPT_STATUS_FAILED => Some(Self::Failed),
            _unsupported => None,
        })
    }

    /// Returns the stable receipt status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => RECEIPT_STATUS_ADMITTED,
            Self::Denied => RECEIPT_STATUS_DENIED,
            Self::Completed => RECEIPT_STATUS_COMPLETED,
            Self::Cancelled => RECEIPT_STATUS_CANCELLED,
            Self::Timeout => RECEIPT_STATUS_TIMEOUT,
            Self::Unavailable => RECEIPT_STATUS_UNAVAILABLE,
            Self::Failed => RECEIPT_STATUS_FAILED,
        }
    }
}

/// Typed error raised by the neutral agent-comms contract.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentCommsError {
    /// Generic adapter contract validation failed.
    Contract(AdapterContractError),
    /// A label or status value is not supported.
    UnsupportedValue {
        /// Field that failed validation.
        field: &'static str,
        /// Unsupported normalized value.
        value: String,
    },
    /// Route validation found raw transport material or an unsafe label.
    UnsafeRoute {
        /// Safe reason label.
        reason: &'static str,
    },
    /// Frame validation found raw transport material or an unsafe label.
    UnsafeFrame {
        /// Safe reason label.
        reason: &'static str,
    },
    /// Receipt validation found missing policy evidence.
    MissingPolicyEvidence,
    /// Receipt validation found missing redaction evidence.
    MissingRedactionEvidence,
    /// Receipt validation found unsafe raw material.
    ReceiptLeak {
        /// Field that contained unsafe material.
        field: &'static str,
        /// Unsafe marker that triggered rejection.
        marker: &'static str,
    },
    /// Admission was not granted for this route.
    AdmissionRequired,
    /// Route and frame/query/event scopes do not match.
    ScopeMismatch,
    /// Frame exceeded the configured route ceiling.
    FrameTooLarge {
        /// Configured maximum byte count.
        max_bytes: u32,
        /// Actual byte count.
        actual_bytes: u32,
    },
    /// The explicit host transport was not configured or unavailable.
    LiveBackendUnavailable,
    /// Transport returned a safe bounded failure reason.
    Transport {
        /// Safe bounded reason.
        reason: &'static str,
    },
}

impl core::fmt::Display for AgentCommsError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "agent-comms contract error: {error}"),
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "{field} contains unsupported value {value}")
            }
            Self::UnsafeRoute { reason } => write!(formatter, "unsafe agent-comms route: {reason}"),
            Self::UnsafeFrame { reason } => write!(formatter, "unsafe agent-comms frame: {reason}"),
            Self::MissingPolicyEvidence => {
                formatter.write_str("agent-comms policy evidence is missing")
            }
            Self::MissingRedactionEvidence => {
                formatter.write_str("agent-comms frame requires redaction evidence")
            }
            Self::ReceiptLeak { field, marker } => write!(
                formatter,
                "agent-comms receipt {field} leaks marker {marker}"
            ),
            Self::AdmissionRequired => formatter.write_str("agent-comms route requires admission"),
            Self::ScopeMismatch => formatter.write_str("agent-comms route scope mismatch"),
            Self::FrameTooLarge {
                max_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "agent-comms frame is too large: max {max_bytes} bytes, got {actual_bytes}"
            ),
            Self::LiveBackendUnavailable => {
                formatter.write_str("agent-comms transport unavailable")
            }
            Self::Transport { reason } => {
                write!(formatter, "agent-comms transport failed: {reason}")
            }
        }
    }
}

impl core::error::Error for AgentCommsError {}

impl From<AdapterContractError> for AgentCommsError {
    fn from(error: AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

/// Borrowed input used to validate a neutral agent-comms route.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommsRouteDraft<'a> {
    /// Safe route label.
    pub route_label: &'a str,
    /// Transport family label.
    pub transport_family: &'a str,
    /// Explicit tenant/workspace/session scope.
    pub scope: Scope,
    /// Optional BLAKE3 hash of a host-owned endpoint, ticket, or node address.
    pub endpoint_hash: Option<&'a str>,
    /// Optional policy-evidence hash required before effects.
    pub policy_hash: Option<&'a str>,
    /// Admission status label.
    pub admission: &'a str,
    /// Maximum accepted frame size.
    pub max_frame_bytes: u32,
}

/// Validated neutral agent-comms route.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommsRoute {
    route_label: String,
    transport_family: AgentCommsTransportFamily,
    scope: Scope,
    endpoint_hash: Option<SafeHash>,
    policy_hash: SafeHash,
    admission: AgentCommsAdmission,
    max_frame_bytes: u32,
    route_hash: SafeHash,
}

impl AgentCommsRoute {
    /// Validates an agent-comms route and computes stable route hashes.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the route label, family, policy hash,
    /// endpoint hash, or admission data is missing or unsafe.
    #[cfg_attr(animus_octet, allow(tigerstyle::numeric_units))]
    pub fn new(draft: AgentCommsRouteDraft<'_>) -> Result<Self, AgentCommsError> {
        let route_label = required_safe_text(ROUTE_LABEL_FIELD, draft.route_label, |reason| {
            AgentCommsError::UnsafeRoute { reason }
        })?;
        let transport_family = AgentCommsTransportFamily::parse(draft.transport_family)?;
        let policy_hash = match draft.policy_hash {
            Some(value) => {
                validate_no_receipt_leaks(POLICY_HASH_FIELD.as_str(), value)?;
                SafeHash::new(value).map_err(AgentCommsError::from)?
            }
            None => return Err(AgentCommsError::MissingPolicyEvidence),
        };
        if draft.max_frame_bytes == 0 {
            return Err(AgentCommsError::UnsafeRoute {
                reason: "agent-comms max frame bytes must be greater than zero",
            });
        }
        let endpoint_hash = match draft.endpoint_hash {
            Some(value) => {
                validate_no_receipt_leaks(ENDPOINT_HASH_FIELD.as_str(), value)?;
                Some(SafeHash::new(value).map_err(AgentCommsError::from)?)
            }
            None => None,
        };
        let admission = AgentCommsAdmission::parse(draft.admission)?;
        let route_hash = build_route_hash(&RouteHashInput {
            route_label: route_label.as_str(),
            transport_family,
            scope: &draft.scope,
            endpoint_hash: endpoint_hash.as_ref(),
            policy_hash: &policy_hash,
            admission,
            max_frame_bytes: draft.max_frame_bytes,
        });

        Ok(Self {
            route_label,
            transport_family,
            scope: draft.scope,
            endpoint_hash,
            policy_hash,
            admission,
            max_frame_bytes: draft.max_frame_bytes,
            route_hash,
        })
    }

    /// Returns the stable route label.
    #[must_use]
    pub fn route_label(&self) -> &str {
        self.route_label.as_str()
    }

    /// Returns the transport family.
    #[must_use]
    pub const fn transport_family(&self) -> AgentCommsTransportFamily {
        self.transport_family
    }

    /// Returns the scoped tenant/workspace/session identity.
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }

    /// Returns the optional endpoint, ticket, or node-address hash.
    #[must_use]
    pub const fn endpoint_hash(&self) -> Option<&SafeHash> {
        self.endpoint_hash.as_ref()
    }

    /// Returns the policy-evidence hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &SafeHash {
        &self.policy_hash
    }

    /// Returns the admission status.
    #[must_use]
    pub const fn admission(&self) -> AgentCommsAdmission {
        self.admission
    }

    /// Returns the configured maximum frame size.
    #[must_use]
    pub const fn max_frame_bytes(&self) -> u32 {
        self.max_frame_bytes
    }

    /// Returns the stable route hash.
    #[must_use]
    pub const fn route_hash(&self) -> &SafeHash {
        &self.route_hash
    }

    /// Returns whether the route may perform effects.
    #[must_use]
    pub const fn allows_effects(&self) -> bool {
        self.admission.allows_effects()
    }
}

/// Borrowed input used to validate an agent-comms frame.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentCommsFrameDraft<'a> {
    /// Safe frame correlation id.
    pub frame_id: &'a str,
    /// Frame family.
    pub kind: AgentCommsFrameKind,
    /// Validated route used to project the frame.
    pub route: &'a AgentCommsRoute,
    /// BLAKE3 hash of the redacted payload metadata.
    pub payload_hash: &'a str,
    /// Bounded payload byte count.
    pub payload_bytes: u32,
    /// Bounded item count.
    pub item_count: u32,
    /// Redaction evidence labels.
    pub redactions: &'a [&'a str],
}

/// Validated neutral agent-comms frame.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommsFrame {
    frame_id: String,
    kind: AgentCommsFrameKind,
    route_hash: SafeHash,
    payload_hash: SafeHash,
    payload_bytes: u32,
    item_count: u32,
    redactions: Vec<AgentCommsRedaction>,
    frame_hash: SafeHash,
}

impl AgentCommsFrame {
    /// Validates an agent-comms frame and computes a stable frame hash.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the frame id, payload hash, or redaction evidence is missing or unsafe.
    pub fn new(draft: AgentCommsFrameDraft<'_>) -> Result<Self, AgentCommsError> {
        if !draft.route.allows_effects() {
            return Err(AgentCommsError::AdmissionRequired);
        }
        if draft.payload_bytes > draft.route.max_frame_bytes() {
            return Err(AgentCommsError::FrameTooLarge {
                max_bytes: draft.route.max_frame_bytes(),
                actual_bytes: draft.payload_bytes,
            });
        }
        if draft.redactions.is_empty() {
            return Err(AgentCommsError::MissingRedactionEvidence);
        }

        let frame_id = required_safe_text(FRAME_ID_FIELD, draft.frame_id, |reason| {
            AgentCommsError::UnsafeFrame { reason }
        })?;
        validate_no_receipt_leaks(PAYLOAD_HASH_FIELD.as_str(), draft.payload_hash)?;
        let payload_hash = SafeHash::new(draft.payload_hash).map_err(AgentCommsError::from)?;
        let redactions = draft
            .redactions
            .iter()
            .map(|redaction| AgentCommsRedaction::parse(redaction))
            .collect::<Result<Vec<_>, _>>()?;
        let frame_hash = build_frame_hash(&FrameHashInput {
            frame_id: frame_id.as_str(),
            kind: draft.kind,
            route_hash: draft.route.route_hash(),
            policy_hash: draft.route.policy_hash(),
            payload_hash: &payload_hash,
            payload_bytes: draft.payload_bytes,
            item_count: draft.item_count,
            redactions: &redactions,
        });

        Ok(Self {
            frame_id,
            kind: draft.kind,
            route_hash: draft.route.route_hash().clone(),
            payload_hash,
            payload_bytes: draft.payload_bytes,
            item_count: draft.item_count,
            redactions,
            frame_hash,
        })
    }

    /// Returns the safe frame id.
    #[must_use]
    pub fn frame_id(&self) -> &str {
        self.frame_id.as_str()
    }

    /// Returns the frame family.
    #[must_use]
    pub const fn kind(&self) -> AgentCommsFrameKind {
        self.kind
    }

    /// Returns the route hash.
    #[must_use]
    pub const fn route_hash(&self) -> &SafeHash {
        &self.route_hash
    }

    /// Returns the payload metadata hash.
    #[must_use]
    pub const fn payload_hash(&self) -> &SafeHash {
        &self.payload_hash
    }

    /// Returns the bounded payload byte count.
    #[must_use]
    pub const fn payload_bytes(&self) -> u32 {
        self.payload_bytes
    }

    /// Returns the bounded item count.
    #[must_use]
    pub const fn item_count(&self) -> u32 {
        self.item_count
    }

    /// Returns the redaction evidence.
    #[must_use]
    pub fn redactions(&self) -> &[AgentCommsRedaction] {
        self.redactions.as_slice()
    }

    /// Returns the stable frame hash.
    #[must_use]
    pub const fn frame_hash(&self) -> &SafeHash {
        &self.frame_hash
    }
}

/// Borrowed input used to validate an agent-comms receipt.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommsReceiptDraft<'a> {
    /// Safe route label.
    pub route_label: &'a str,
    /// Transport family label.
    pub transport_family: &'a str,
    /// Explicit scope.
    pub scope: Scope,
    /// Optional endpoint hash.
    pub endpoint_hash: Option<&'a str>,
    /// Optional policy hash.
    pub policy_hash: Option<&'a str>,
    /// Admission label.
    pub admission: &'a str,
    /// Maximum frame bytes declared by the route.
    pub max_frame_bytes: u32,
    /// Safe frame id.
    pub frame_id: &'a str,
    /// Frame family label.
    pub frame_kind: &'a str,
    /// Payload metadata hash.
    pub payload_hash: &'a str,
    /// Bounded payload byte count.
    pub payload_bytes: u32,
    /// Bounded item count.
    pub item_count: u32,
    /// Safe receipt status label.
    pub status: &'a str,
    /// Redaction evidence labels.
    pub redactions: &'a [&'a str],
}

/// Validated safe agent-comms receipt.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommsReceipt {
    route_hash: SafeHash,
    frame_hash: SafeHash,
    transport_family: AgentCommsTransportFamily,
    scope_hash: SafeHash,
    admission: AgentCommsAdmission,
    policy_hash: SafeHash,
    frame_kind: AgentCommsFrameKind,
    payload_bytes: u32,
    item_count: u32,
    status: AgentCommsReceiptStatus,
    redactions: Vec<AgentCommsRedaction>,
    receipt_hash: SafeHash,
}

impl AgentCommsReceipt {
    /// Validates a receipt candidate and computes a stable receipt hash.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when a label, hash, redaction, or policy field is unsafe or missing.
    #[cfg_attr(animus_octet, allow(tigerstyle::numeric_units))]
    pub fn new(draft: &AgentCommsReceiptDraft<'_>) -> Result<Self, AgentCommsError> {
        let route_label = required_safe_text(ROUTE_LABEL_FIELD, draft.route_label, |reason| {
            AgentCommsError::UnsafeRoute { reason }
        })?;
        let transport_family = AgentCommsTransportFamily::parse(draft.transport_family)?;
        let policy_hash = match draft.policy_hash {
            Some(value) => {
                validate_no_receipt_leaks(POLICY_HASH_FIELD.as_str(), value)?;
                SafeHash::new(value).map_err(AgentCommsError::from)?
            }
            None => return Err(AgentCommsError::MissingPolicyEvidence),
        };
        let endpoint_hash = match draft.endpoint_hash {
            Some(value) => {
                validate_no_receipt_leaks(ENDPOINT_HASH_FIELD.as_str(), value)?;
                Some(SafeHash::new(value).map_err(AgentCommsError::from)?)
            }
            None => None,
        };
        let admission = AgentCommsAdmission::parse(draft.admission)?;
        if draft.max_frame_bytes == 0 {
            return Err(AgentCommsError::UnsafeRoute {
                reason: "agent-comms max frame bytes must be greater than zero",
            });
        }
        let frame_id = required_safe_text(FRAME_ID_FIELD, draft.frame_id, |reason| {
            AgentCommsError::UnsafeFrame { reason }
        })?;
        let frame_kind = AgentCommsFrameKind::parse(draft.frame_kind)?;
        validate_no_receipt_leaks(PAYLOAD_HASH_FIELD.as_str(), draft.payload_hash)?;
        let payload_hash = SafeHash::new(draft.payload_hash).map_err(AgentCommsError::from)?;
        if draft.redactions.is_empty() {
            return Err(AgentCommsError::MissingRedactionEvidence);
        }
        let redactions = draft
            .redactions
            .iter()
            .map(|redaction| AgentCommsRedaction::parse(redaction))
            .collect::<Result<Vec<_>, _>>()?;
        let route_hash = build_route_hash(&RouteHashInput {
            route_label: route_label.as_str(),
            transport_family,
            scope: &draft.scope,
            endpoint_hash: endpoint_hash.as_ref(),
            policy_hash: &policy_hash,
            admission,
            max_frame_bytes: draft.max_frame_bytes,
        });
        let frame_hash = build_frame_hash(&FrameHashInput {
            frame_id: frame_id.as_str(),
            kind: frame_kind,
            route_hash: &route_hash,
            policy_hash: &policy_hash,
            payload_hash: &payload_hash,
            payload_bytes: draft.payload_bytes,
            item_count: draft.item_count,
            redactions: &redactions,
        });
        let scope_hash = scope_hash(&draft.scope);
        let status = AgentCommsReceiptStatus::parse(draft.status)?;
        let receipt_hash = build_receipt_hash(&ReceiptHashInput {
            route_hash: &route_hash,
            frame_hash: &frame_hash,
            transport_family,
            scope_hash: &scope_hash,
            admission,
            policy_hash: &policy_hash,
            frame_kind,
            payload_bytes: draft.payload_bytes,
            item_count: draft.item_count,
            status,
            redactions: &redactions,
        });

        Ok(Self {
            route_hash,
            frame_hash,
            transport_family,
            scope_hash,
            admission,
            policy_hash,
            frame_kind,
            payload_bytes: draft.payload_bytes,
            item_count: draft.item_count,
            status,
            redactions,
            receipt_hash,
        })
    }

    /// Builds a route-admission receipt with route metadata only.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the route metadata is invalid.
    pub fn from_route(
        route: &AgentCommsRoute,
        status: AgentCommsReceiptStatus,
    ) -> Result<Self, AgentCommsError> {
        let redactions = [AgentCommsRedaction::RouteMetadataOnly.as_str()];
        Self::new(&AgentCommsReceiptDraft {
            route_label: route.route_label(),
            transport_family: route.transport_family().as_str(),
            scope: route.scope().clone(),
            endpoint_hash: route.endpoint_hash().map(SafeHash::as_str),
            policy_hash: Some(route.policy_hash().as_str()),
            admission: route.admission().as_str(),
            max_frame_bytes: route.max_frame_bytes(),
            frame_id: route.route_label(),
            frame_kind: AgentCommsFrameKind::RouteMetadata.as_str(),
            payload_hash: route.policy_hash().as_str(),
            payload_bytes: 0,
            item_count: 0,
            status: status.as_str(),
            redactions: redactions.as_slice(),
        })
    }

    /// Builds a frame receipt from a validated route and frame.
    ///
    /// # Errors
    ///
    /// Returns [`AgentCommsError`] when the route, frame, or status metadata is invalid.
    pub fn from_frame(
        route: &AgentCommsRoute,
        frame: &AgentCommsFrame,
        status: AgentCommsReceiptStatus,
    ) -> Result<Self, AgentCommsError> {
        let redactions = frame
            .redactions()
            .iter()
            .copied()
            .map(AgentCommsRedaction::as_str)
            .collect::<Vec<_>>();
        Self::new(&AgentCommsReceiptDraft {
            route_label: route.route_label(),
            transport_family: route.transport_family().as_str(),
            scope: route.scope().clone(),
            endpoint_hash: route.endpoint_hash().map(SafeHash::as_str),
            policy_hash: Some(route.policy_hash().as_str()),
            admission: route.admission().as_str(),
            max_frame_bytes: route.max_frame_bytes(),
            frame_id: frame.frame_id(),
            frame_kind: frame.kind().as_str(),
            payload_hash: frame.payload_hash().as_str(),
            payload_bytes: frame.payload_bytes(),
            item_count: frame.item_count(),
            status: status.as_str(),
            redactions: redactions.as_slice(),
        })
    }

    /// Returns the route hash.
    #[must_use]
    pub const fn route_hash(&self) -> &SafeHash {
        &self.route_hash
    }

    /// Returns the frame hash.
    #[must_use]
    pub const fn frame_hash(&self) -> &SafeHash {
        &self.frame_hash
    }

    /// Returns the transport family.
    #[must_use]
    pub const fn transport_family(&self) -> AgentCommsTransportFamily {
        self.transport_family
    }

    /// Returns the scoped identity hash.
    #[must_use]
    pub const fn scope_hash(&self) -> &SafeHash {
        &self.scope_hash
    }

    /// Returns the admission posture.
    #[must_use]
    pub const fn admission(&self) -> AgentCommsAdmission {
        self.admission
    }

    /// Returns the policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &SafeHash {
        &self.policy_hash
    }

    /// Returns the frame family.
    #[must_use]
    pub const fn frame_kind(&self) -> AgentCommsFrameKind {
        self.frame_kind
    }

    /// Returns bounded payload bytes.
    #[must_use]
    pub const fn payload_bytes(&self) -> u32 {
        self.payload_bytes
    }

    /// Returns bounded item count.
    #[must_use]
    pub const fn item_count(&self) -> u32 {
        self.item_count
    }

    /// Returns the receipt status.
    #[must_use]
    pub const fn status(&self) -> AgentCommsReceiptStatus {
        self.status
    }

    /// Returns redaction evidence.
    #[must_use]
    pub fn redactions(&self) -> &[AgentCommsRedaction] {
        self.redactions.as_slice()
    }

    /// Returns the stable receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &SafeHash {
        &self.receipt_hash
    }
}

/// Host-injected transport port for agent-comms effects.
pub trait AgentCommsTransport {
    /// Transport failure type.
    type Error;

    /// Publishes a validated dataspace assertion.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host transport cannot publish the assertion.
    fn publish_assertion(
        &mut self,
        route: &AgentCommsRoute,
        assertion: &Assertion,
        frame: &AgentCommsFrame,
    ) -> Result<(), Self::Error>;

    /// Queries assertions through the validated route.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host transport cannot complete the query.
    fn query_assertions(
        &mut self,
        route: &AgentCommsRoute,
        subscription: &Subscription,
    ) -> Result<Vec<Assertion>, Self::Error>;

    /// Publishes a validated orchestration event.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host transport cannot publish the event.
    fn publish_orchestration_event(
        &mut self,
        route: &AgentCommsRoute,
        event: &OrchestrationEvent,
        frame: &AgentCommsFrame,
    ) -> Result<(), Self::Error>;
}

/// Validates that a route may serve the supplied scope.
///
/// # Errors
///
/// Returns [`AgentCommsError`] when admission is missing or the tenant/workspace boundary differs.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
pub fn admit_agent_comms_scope(
    route: &AgentCommsRoute,
    scope: &Scope,
) -> Result<(), AgentCommsError> {
    ensure_admitted(route)?;
    if route.scope().same_tenant_workspace(scope) {
        return Ok(());
    }
    Err(AgentCommsError::ScopeMismatch)
}

/// Builds a safe redacted frame from a dataspace assertion.
///
/// # Errors
///
/// Returns [`AgentCommsError`] when scope, admission, hash, frame-size, or redaction checks fail.
pub fn project_assertion_frame(
    route: &AgentCommsRoute,
    assertion: &Assertion,
) -> Result<AgentCommsFrame, AgentCommsError> {
    admit_agent_comms_scope(route, assertion.scope())?;
    let payload = assertion_payload_summary(assertion);
    let payload_hash = SafeHash::from_blake3_bytes(payload.as_bytes());
    let frame_id = format!("{ASSERTION_FRAME_ID_PREFIX}{}", assertion.stable_id());
    let redactions = assertion_redaction_labels(assertion);
    AgentCommsFrame::new(AgentCommsFrameDraft {
        frame_id: frame_id.as_str(),
        kind: AgentCommsFrameKind::DataspaceAssertion,
        route,
        payload_hash: payload_hash.as_str(),
        payload_bytes: bounded_len_u32(payload.as_str())?,
        item_count: assertion.item_count(),
        redactions: redactions.as_slice(),
    })
}

/// Builds a safe orchestration-event frame.
///
/// # Errors
///
/// Returns [`AgentCommsError`] when scope, admission, hash, frame-size, or redaction checks fail.
pub fn project_orchestration_event_frame(
    route: &AgentCommsRoute,
    event: &OrchestrationEvent,
) -> Result<AgentCommsFrame, AgentCommsError> {
    admit_agent_comms_scope(route, event.scope())?;
    let payload = orchestration_event_payload_summary(event);
    let payload_hash = SafeHash::from_blake3_bytes(payload.as_bytes());
    let frame_id = format!("{EVENT_FRAME_ID_PREFIX}{}", event.stable_id());
    AgentCommsFrame::new(AgentCommsFrameDraft {
        frame_id: frame_id.as_str(),
        kind: AgentCommsFrameKind::OrchestrationEvent,
        route,
        payload_hash: payload_hash.as_str(),
        payload_bytes: bounded_len_u32(payload.as_str())?,
        item_count: orchestration_event_item_count(event),
        redactions: &[EVENT_PAYLOAD_OMITTED],
    })
}

/// Builds a safe headless-command frame.
///
/// # Errors
///
/// Returns [`AgentCommsError`] when admission, hash, or redaction checks fail.
pub fn project_headless_command_frame(
    route: &AgentCommsRoute,
    envelope: &HostControlEnvelope,
) -> Result<AgentCommsFrame, AgentCommsError> {
    ensure_admitted(route)?;
    let payload = host_control_payload_summary(envelope);
    let payload_hash = SafeHash::from_blake3_bytes(payload.as_bytes());
    let frame_id = format!("{COMMAND_FRAME_ID_PREFIX}{}", envelope.correlation_id());
    AgentCommsFrame::new(AgentCommsFrameDraft {
        frame_id: frame_id.as_str(),
        kind: AgentCommsFrameKind::HeadlessCommand,
        route,
        payload_hash: payload_hash.as_str(),
        payload_bytes: bounded_len_u32(payload.as_str())?,
        item_count: COMMAND_SUMMARY_PARTS,
        redactions: &[COMMAND_PAYLOAD_OMITTED],
    })
}

/// Builds a safe headless-event frame.
///
/// # Errors
///
/// Returns [`AgentCommsError`] when admission, hash, or redaction checks fail.
pub fn project_headless_event_frame(
    route: &AgentCommsRoute,
    event: &HostEvent,
) -> Result<AgentCommsFrame, AgentCommsError> {
    ensure_admitted(route)?;
    let payload = host_event_payload_summary(event);
    let payload_hash = SafeHash::from_blake3_bytes(payload.as_bytes());
    let frame_id = format!("{HEADLESS_EVENT_FRAME_ID_PREFIX}{}", event.correlation_id());
    AgentCommsFrame::new(AgentCommsFrameDraft {
        frame_id: frame_id.as_str(),
        kind: AgentCommsFrameKind::HeadlessEvent,
        route,
        payload_hash: payload_hash.as_str(),
        payload_bytes: bounded_len_u32(payload.as_str())?,
        item_count: EVENT_SUMMARY_PARTS,
        redactions: &[EVENT_PAYLOAD_OMITTED],
    })
}

/// Builds a safe subagent-lifecycle frame.
///
/// # Errors
///
/// Returns [`AgentCommsError`] when admission, hash, or redaction checks fail.
pub fn project_subagent_lifecycle_frame(
    route: &AgentCommsRoute,
    event: &SubagentEvent,
) -> Result<AgentCommsFrame, AgentCommsError> {
    ensure_admitted(route)?;
    let payload = subagent_event_payload_summary(event);
    let payload_hash = SafeHash::from_blake3_bytes(payload.as_bytes());
    let frame_id = format!(
        "{SUBAGENT_FRAME_ID_PREFIX}{}:{}",
        event.parent_id(),
        event.child_id()
    );
    AgentCommsFrame::new(AgentCommsFrameDraft {
        frame_id: frame_id.as_str(),
        kind: AgentCommsFrameKind::SubagentLifecycle,
        route,
        payload_hash: payload_hash.as_str(),
        payload_bytes: bounded_len_u32(payload.as_str())?,
        item_count: SUBAGENT_SUMMARY_PARTS,
        redactions: &[SUBAGENT_PAYLOAD_OMITTED],
    })
}

fn ensure_admitted(route: &AgentCommsRoute) -> Result<(), AgentCommsError> {
    if route.admission().allows_effects() {
        return Ok(());
    }
    Err(AgentCommsError::AdmissionRequired)
}

fn parse_value<T>(
    field_name: FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, AgentCommsError> {
    let normalized = required_text(field_name, value)?;
    parser(normalized.as_str()).ok_or_else(|| AgentCommsError::UnsupportedValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

fn required_text(field_name: FieldName, value: &str) -> Result<String, AgentCommsError> {
    normalized_adapter_field(field_name, value).map_err(AgentCommsError::from)
}

fn required_safe_text(
    field_name: FieldName,
    value: &str,
    builder: fn(&'static str) -> AgentCommsError,
) -> Result<String, AgentCommsError> {
    let normalized = required_text(field_name, value)?;
    validate_no_unsafe_markers(normalized.as_str(), builder)?;
    Ok(normalized)
}

fn validate_no_unsafe_markers(
    value: &str,
    builder: fn(&'static str) -> AgentCommsError,
) -> Result<(), AgentCommsError> {
    let lower = value.to_ascii_lowercase();
    for marker in UNSAFE_MARKERS {
        if lower.contains(marker) {
            return Err(builder("unsafe agent-comms material"));
        }
    }
    Ok(())
}

#[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
fn validate_no_receipt_leaks(field: &'static str, value: &str) -> Result<(), AgentCommsError> {
    let lower = value.to_ascii_lowercase();
    for marker in UNSAFE_MARKERS {
        if lower.contains(marker) {
            return Err(AgentCommsError::ReceiptLeak { field, marker });
        }
    }
    Ok(())
}

fn safe_hash_parts(parts: &[&str]) -> SafeHash {
    let canonical = parts.join("\u{1f}");
    SafeHash::from_blake3_bytes(canonical.as_bytes())
}

fn bounded_len_u32(text: &str) -> Result<u32, AgentCommsError> {
    u32::try_from(text.len()).map_err(|_| AgentCommsError::UnsafeFrame {
        reason: "agent-comms payload text exceeds u32",
    })
}

fn scope_summary(scope: &Scope) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        scope.tenant_id(),
        scope.workspace_id(),
        scope.session_id(),
        scope.branch_id(),
        scope.facet_id(),
        scope.actor_kind().as_str(),
        scope.actor_id(),
    )
}

fn selector_summary(selector: &HostSelector) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        selector.host_id(),
        selector.resource_id(),
        selector.kind().as_str(),
        selector.lifetime().as_str(),
        selector.revision(),
    )
}

#[cfg_attr(animus_octet, allow(tigerstyle::assertion_density))]
fn host_control_payload_summary(envelope: &HostControlEnvelope) -> String {
    let command = envelope.command();
    let selector = command
        .selector()
        .map_or_else(|| String::from(NO_SELECTOR), selector_summary);
    let label = command
        .label()
        .map_or_else(|| String::from(NO_LABEL), String::from);
    let accepted = if envelope.accepted_features().is_empty() {
        String::from(NO_LABEL)
    } else {
        envelope.accepted_features().join(",")
    };
    let unknown = if envelope.unknown_features().is_empty() {
        String::from(NO_LABEL)
    } else {
        envelope.unknown_features().join(",")
    };
    format!(
        "{}|{}|{}|{}|{}|{}",
        envelope.correlation_id(),
        command.operation().as_str(),
        selector,
        label,
        accepted,
        unknown,
    )
}

fn host_event_status_label(status: Option<HostSemanticStatus>) -> &'static str {
    match status {
        Some(HostSemanticStatus::Idle) => "idle",
        Some(HostSemanticStatus::Running) => "running",
        Some(HostSemanticStatus::Waiting) => "waiting",
        Some(HostSemanticStatus::Done) => "done",
        Some(HostSemanticStatus::Failed) => "failed",
        None => NO_STATUS,
    }
}

fn host_event_payload_summary(event: &HostEvent) -> String {
    let error_code = event.error().map_or(NO_ERROR_CODE, HostTypedError::code);
    let message_state = if event.message().is_some() {
        "message_bounded"
    } else {
        NO_MESSAGE
    };
    format!(
        "{}|{}|{}|{}|{}",
        event.correlation_id(),
        event.kind().as_str(),
        host_event_status_label(event.status()),
        message_state,
        error_code,
    )
}

fn subagent_event_payload_summary(event: &SubagentEvent) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        event.child_id(),
        event.parent_id(),
        event.kind().as_str(),
        event.status(),
        event.evidence_ref().unwrap_or(NO_EVIDENCE),
    )
}

#[cfg_attr(animus_octet, allow(tigerstyle::assertion_density))]
fn assertion_payload_summary(assertion: &Assertion) -> String {
    let scope = scope_summary(assertion.scope());
    let redactions = assertion_redaction_labels(assertion).join(",");
    let content_hash = assertion
        .content_hash()
        .map_or(NO_ENDPOINT_HASH, SafeHash::as_str);
    let item_count = assertion.item_count().to_string();
    let lease = assertion
        .lease_expires_at()
        .map_or_else(|| String::from(NO_EVIDENCE), |lease| lease.to_string());
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        assertion.stable_id(),
        assertion_kind_label(assertion.kind()),
        assertion_status_label(assertion.status()),
        scope,
        redactions,
        content_hash,
        item_count,
        lease,
    )
}

fn orchestration_event_payload_summary(event: &OrchestrationEvent) -> String {
    let scope = scope_summary(event.scope());
    format!(
        "{}|{}|{}|{}",
        event.stable_id(),
        event.kind().as_str(),
        scope,
        event.target_owner().unwrap_or(NO_TARGET_OWNER),
    )
}

fn assertion_kind_label(kind: crate::AssertionKind) -> &'static str {
    match kind {
        crate::AssertionKind::RuntimeEvent => "runtime_event",
        crate::AssertionKind::ToolState => "tool_state",
        crate::AssertionKind::BackgroundWork => "background_work",
        crate::AssertionKind::SessionSummary => "session_summary",
        crate::AssertionKind::ProvenanceSummary => "provenance_summary",
        crate::AssertionKind::AgentLifecycle => "agent_lifecycle",
        crate::AssertionKind::SubagentLifecycle => "subagent_lifecycle",
        crate::AssertionKind::Cancellation => "cancellation",
        crate::AssertionKind::OwnershipTransfer => "ownership_transfer",
    }
}

fn assertion_status_label(status: AssertionStatus) -> &'static str {
    match status {
        AssertionStatus::Active => "active",
        AssertionStatus::Completed => "completed",
        AssertionStatus::Cancelled => "cancelled",
        AssertionStatus::Expired => "expired",
        AssertionStatus::Failed => "failed",
        AssertionStatus::Blocked => "blocked",
    }
}

fn assertion_redaction_labels(assertion: &Assertion) -> Vec<&'static str> {
    assertion
        .redactions()
        .iter()
        .map(|redaction| match redaction {
            AssertionRedaction::PromptOmitted => AgentCommsRedaction::PromptOmitted.as_str(),
            AssertionRedaction::ProviderPayloadOmitted => {
                AgentCommsRedaction::ProviderPayloadOmitted.as_str()
            }
            AssertionRedaction::CredentialOmitted => {
                AgentCommsRedaction::CredentialOmitted.as_str()
            }
            AssertionRedaction::PathRedacted => AgentCommsRedaction::PathRedacted.as_str(),
            AssertionRedaction::FileBodyOmitted => AgentCommsRedaction::FileBodyOmitted.as_str(),
            AssertionRedaction::ToolPayloadOmitted => {
                AgentCommsRedaction::ToolPayloadOmitted.as_str()
            }
            AssertionRedaction::ModelOutputOmitted => {
                AgentCommsRedaction::ModelOutputOmitted.as_str()
            }
            AssertionRedaction::LogsBounded => AgentCommsRedaction::LogsBounded.as_str(),
        })
        .collect()
}

fn orchestration_event_item_count(event: &OrchestrationEvent) -> u32 {
    if event.target_owner().is_some() {
        return ORCHESTRATION_EVENT_WITH_TARGET_ITEM_COUNT;
    }
    ORCHESTRATION_EVENT_WITHOUT_TARGET_ITEM_COUNT
}

#[derive(Clone, Copy)]
struct RouteHashInput<'a> {
    route_label: &'a str,
    transport_family: AgentCommsTransportFamily,
    scope: &'a Scope,
    endpoint_hash: Option<&'a SafeHash>,
    policy_hash: &'a SafeHash,
    admission: AgentCommsAdmission,
    max_frame_bytes: u32,
}

#[cfg_attr(animus_octet, allow(tigerstyle::assertion_density))]
fn build_route_hash(input: &RouteHashInput<'_>) -> SafeHash {
    let scope_summary = scope_summary(input.scope);
    let endpoint = input
        .endpoint_hash
        .map_or(NO_ENDPOINT_HASH, SafeHash::as_str);
    let max_frame_bytes = input.max_frame_bytes.to_string();
    safe_hash_parts(&[
        ROUTE_HASH_SCHEMA,
        input.route_label,
        input.transport_family.as_str(),
        scope_summary.as_str(),
        endpoint,
        input.policy_hash.as_str(),
        input.admission.as_str(),
        max_frame_bytes.as_str(),
    ])
}

#[derive(Clone, Copy)]
struct FrameHashInput<'a> {
    frame_id: &'a str,
    kind: AgentCommsFrameKind,
    route_hash: &'a SafeHash,
    policy_hash: &'a SafeHash,
    payload_hash: &'a SafeHash,
    payload_bytes: u32,
    item_count: u32,
    redactions: &'a [AgentCommsRedaction],
}

#[cfg_attr(animus_octet, allow(tigerstyle::assertion_density))]
fn build_frame_hash(summary: &FrameHashInput<'_>) -> SafeHash {
    let payload_bytes = summary.payload_bytes.to_string();
    let item_count = summary.item_count.to_string();
    let redactions = summary
        .redactions
        .iter()
        .copied()
        .map(AgentCommsRedaction::as_str)
        .collect::<Vec<_>>()
        .join(",");
    safe_hash_parts(&[
        FRAME_HASH_SCHEMA,
        summary.frame_id,
        summary.kind.as_str(),
        summary.route_hash.as_str(),
        summary.policy_hash.as_str(),
        summary.payload_hash.as_str(),
        payload_bytes.as_str(),
        item_count.as_str(),
        redactions.as_str(),
    ])
}

#[derive(Clone, Copy)]
struct ReceiptHashInput<'a> {
    route_hash: &'a SafeHash,
    frame_hash: &'a SafeHash,
    transport_family: AgentCommsTransportFamily,
    scope_hash: &'a SafeHash,
    admission: AgentCommsAdmission,
    policy_hash: &'a SafeHash,
    frame_kind: AgentCommsFrameKind,
    payload_bytes: u32,
    item_count: u32,
    status: AgentCommsReceiptStatus,
    redactions: &'a [AgentCommsRedaction],
}

#[cfg_attr(animus_octet, allow(tigerstyle::assertion_density))]
fn build_receipt_hash(summary: &ReceiptHashInput<'_>) -> SafeHash {
    let payload_bytes = summary.payload_bytes.to_string();
    let item_count = summary.item_count.to_string();
    let redactions = summary
        .redactions
        .iter()
        .copied()
        .map(AgentCommsRedaction::as_str)
        .collect::<Vec<_>>()
        .join(",");
    safe_hash_parts(&[
        RECEIPT_HASH_SCHEMA,
        summary.route_hash.as_str(),
        summary.frame_hash.as_str(),
        summary.transport_family.as_str(),
        summary.scope_hash.as_str(),
        summary.admission.as_str(),
        summary.policy_hash.as_str(),
        summary.frame_kind.as_str(),
        payload_bytes.as_str(),
        item_count.as_str(),
        summary.status.as_str(),
        redactions.as_str(),
    ])
}

fn scope_hash(scope: &Scope) -> SafeHash {
    let scope_summary = scope_summary(scope);
    safe_hash_parts(&["animus.agent-comms.scope.v1", scope_summary.as_str()])
}

const ASSERTION_FRAME_ID_PREFIX: &str = "agent-comms-frame:assertion:";
const EVENT_FRAME_ID_PREFIX: &str = "agent-comms-frame:orchestration-event:";
const COMMAND_FRAME_ID_PREFIX: &str = "agent-comms-frame:headless-command:";
const HEADLESS_EVENT_FRAME_ID_PREFIX: &str = "agent-comms-frame:headless-event:";
const SUBAGENT_FRAME_ID_PREFIX: &str = "agent-comms-frame:subagent-lifecycle:";

#[cfg(test)]
#[cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::numeric_units)
)]
mod tests {
    use super::*;
    use crate::{
        HostControlEnvelope, HostSelector, HostSelectorDraft, HostTypedError, SelectorLifetime,
        TopologyCommand, TopologyCommandDraft,
    };

    const TENANT: &str = "tenant:local";
    const WORKSPACE: &str = "workspace:main";
    const OTHER_WORKSPACE: &str = "workspace:other";
    const SESSION: &str = "session:main";
    const BRANCH: &str = "branch:main";
    const FACET: &str = "facet:loop";
    const ACTOR_KIND: &str = "agent";
    const ACTOR_ID: &str = "agent:main";
    const HOST_ID: &str = "host:local";
    const RESOURCE_ID: &str = "pane:main";
    const SAFE_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const ROUTE_LABEL: &str = "route:main";
    const ROUTE_LABEL_BAD: &str = "iroh://ticket/raw";
    const FRAME_ID: &str = "frame:main";
    const ASSERTION_ID: &str = "assertion:main";
    const EVENT_ID: &str = "event:main";
    const CHILD_ID: &str = "child:main";
    const PARENT_ID: &str = "parent:main";
    const COMMAND_CORRELATION: &str = "corr:command";
    const EVENT_CORRELATION: &str = "corr:event";
    const STATUS_ACTIVE: &str = "active";
    const STATUS_CANCELLED: &str = "cancelled";
    const COMMAND_LABEL: &str = "rename-pane";
    const COMMAND_KIND: &str = "focus";
    const EVENT_ERROR_CODE: &str = "host-denied";
    const EVENT_ERROR_MESSAGE: &str = "host denied command";
    const REDACTION_PROMPT: &str = "prompt_omitted";
    const REDACTION_PROVIDER: &str = "provider_payload_omitted";
    const REDACTION_CREDENTIAL: &str = "credential_omitted";
    const REDACTION_LOGS: &str = "logs_bounded";
    const MAX_FRAME_BYTES: u32 = 4_096;
    const SMALL_FRAME_BYTES: u32 = 1;
    const HOST_TOPOLOGY_REVISION: u64 = 7;
    const VALIDATED_FRAME_PAYLOAD_BYTES: u32 = 16;
    const OVERSIZED_FRAME_PAYLOAD_BYTES: u32 = 2;
    const EXPECTED_TRANSPORT_FRAME_COUNT: usize = 2;

    type TestResult<T = ()> = Result<T, TestFailure>;

    #[derive(Debug, Eq, PartialEq)]
    struct TestFailure {
        details: String,
    }

    fn test_ok<T, E: core::fmt::Debug>(result: Result<T, E>) -> TestResult<T> {
        result.map_err(|error| TestFailure {
            details: format!("{error:?}"),
        })
    }

    #[derive(Default)]
    struct MemoryTransport {
        frames: Vec<AgentCommsFrame>,
        assertions: Vec<Assertion>,
        unavailable: bool,
    }

    impl AgentCommsTransport for MemoryTransport {
        type Error = AgentCommsError;

        fn publish_assertion(
            &mut self,
            _route: &AgentCommsRoute,
            assertion: &Assertion,
            frame: &AgentCommsFrame,
        ) -> Result<(), Self::Error> {
            if self.unavailable {
                return Err(AgentCommsError::LiveBackendUnavailable);
            }
            self.frames.push(frame.clone());
            self.assertions.push(assertion.clone());
            Ok(())
        }

        fn query_assertions(
            &mut self,
            _route: &AgentCommsRoute,
            subscription: &Subscription,
        ) -> Result<Vec<Assertion>, Self::Error> {
            if self.unavailable {
                return Err(AgentCommsError::LiveBackendUnavailable);
            }
            Ok(self
                .assertions
                .iter()
                .filter(|assertion| {
                    assertion
                        .scope()
                        .same_tenant_workspace(subscription.scope())
                })
                .filter(|assertion| subscription.allows(assertion.kind()))
                .cloned()
                .collect())
        }

        fn publish_orchestration_event(
            &mut self,
            _route: &AgentCommsRoute,
            _event: &OrchestrationEvent,
            frame: &AgentCommsFrame,
        ) -> Result<(), Self::Error> {
            if self.unavailable {
                return Err(AgentCommsError::LiveBackendUnavailable);
            }
            self.frames.push(frame.clone());
            Ok(())
        }
    }

    fn scope(workspace: &str) -> TestResult<Scope> {
        test_ok(Scope::new(crate::ScopeDraft {
            tenant_id: TENANT,
            workspace_id: workspace,
            session_id: SESSION,
            branch_id: BRANCH,
            facet_id: FACET,
            actor_kind: ACTOR_KIND,
            actor_id: ACTOR_ID,
        }))
    }

    #[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
    fn route(
        workspace: &str,
        admission: &str,
        policy_hash: Option<&str>,
    ) -> TestResult<AgentCommsRoute> {
        test_ok(AgentCommsRoute::new(AgentCommsRouteDraft {
            route_label: ROUTE_LABEL,
            transport_family: IROH_FAMILY,
            scope: scope(workspace)?,
            endpoint_hash: Some(SAFE_HASH),
            policy_hash,
            admission,
            max_frame_bytes: MAX_FRAME_BYTES,
        }))
    }

    #[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
    fn assertion_for(workspace: &str, stable_id: &str) -> TestResult<Assertion> {
        test_ok(Assertion::new(crate::AssertionDraft {
            stable_id,
            kind: assertion_kind_label(crate::AssertionKind::RuntimeEvent),
            status: STATUS_ACTIVE,
            scope: scope(workspace)?,
            redactions: &[
                REDACTION_PROMPT,
                REDACTION_PROVIDER,
                REDACTION_CREDENTIAL,
                REDACTION_LOGS,
            ],
            content_hash: Some(SAFE_HASH),
            item_count: 1,
            lease_expires_at: None,
        }))
    }

    fn query(workspace: &str) -> TestResult<Subscription> {
        test_ok(Subscription::new(crate::SubscriptionDraft {
            scope: scope(workspace)?,
            kinds: &[assertion_kind_label(crate::AssertionKind::RuntimeEvent)],
            include_raw_payloads: false,
            cross_workspace: false,
        }))
    }

    fn orchestration_event(workspace: &str) -> TestResult<OrchestrationEvent> {
        test_ok(OrchestrationEvent::new(crate::OrchestrationEventDraft {
            stable_id: EVENT_ID,
            kind: "cancelled",
            scope: scope(workspace)?,
            target_owner: None,
        }))
    }

    #[cfg_attr(animus_octet, allow(tigerstyle::assertion_density))]
    fn host_command() -> TestResult<HostControlEnvelope> {
        let selector = test_ok(HostSelector::validate(HostSelectorDraft {
            host_id: HOST_ID,
            resource_id: RESOURCE_ID,
            kind: "pane",
            lifetime: SelectorLifetime::Live.as_str(),
            revision: HOST_TOPOLOGY_REVISION,
        }))?;
        let snapshot = test_ok(crate::HostTopologySnapshot::validate(
            crate::HostTopologySnapshotDraft {
                host_id: HOST_ID,
                revision: HOST_TOPOLOGY_REVISION,
                selectors: core::slice::from_ref(&selector),
                labels: &[],
                supported_operations: &[COMMAND_KIND],
            },
        ))?;
        let command = test_ok(TopologyCommand::validate(
            &snapshot,
            TopologyCommandDraft {
                operation: COMMAND_KIND,
                selector: Some(&selector),
                label: Some(COMMAND_LABEL),
            },
        ))?;
        test_ok(HostControlEnvelope::new(
            COMMAND_CORRELATION,
            "animus-host-control-v1",
            command,
            &["layout"],
        ))
    }

    fn host_event() -> TestResult<HostEvent> {
        let selector = test_ok(HostSelector::validate(HostSelectorDraft {
            host_id: HOST_ID,
            resource_id: RESOURCE_ID,
            kind: "pane",
            lifetime: SelectorLifetime::Live.as_str(),
            revision: HOST_TOPOLOGY_REVISION,
        }))?;
        let typed_error = test_ok(HostTypedError::new(EVENT_ERROR_CODE, EVENT_ERROR_MESSAGE))?;
        test_ok(HostEvent::host_error(
            EVENT_CORRELATION,
            selector,
            typed_error,
        ))
    }

    fn subagent_event() -> TestResult<SubagentEvent> {
        test_ok(SubagentEvent::validate(&crate::SubagentEventDraft {
            child_id: CHILD_ID,
            parent_id: PARENT_ID,
            kind: "aggregation",
            status: STATUS_CANCELLED,
            evidence_ref: Some("evidence:bounded"),
        }))
    }

    #[test]
    fn route_frame_and_transport_flow_succeeds() -> TestResult {
        let route = route(WORKSPACE, ADMITTED, Some(SAFE_HASH))?;
        let assertion = assertion_for(WORKSPACE, ASSERTION_ID)?;
        let event = orchestration_event(WORKSPACE)?;
        let mut transport = MemoryTransport::default();
        let assertion_frame = test_ok(project_assertion_frame(&route, &assertion))?;
        let event_frame = test_ok(project_orchestration_event_frame(&route, &event))?;
        test_ok(transport.publish_assertion(&route, &assertion, &assertion_frame))?;
        test_ok(transport.publish_orchestration_event(&route, &event, &event_frame))?;
        let subscription = query(WORKSPACE)?;
        let matches = test_ok(transport.query_assertions(&route, &subscription))?;

        assert_eq!(matches.len(), 1);
        assert_eq!(transport.frames.len(), EXPECTED_TRANSPORT_FRAME_COUNT);
        assert_eq!(
            assertion_frame.kind(),
            AgentCommsFrameKind::DataspaceAssertion
        );
        assert_eq!(event_frame.kind(), AgentCommsFrameKind::OrchestrationEvent);
        Ok(())
    }

    #[test]
    fn route_rejects_missing_policy_and_unsupported_family() -> TestResult {
        let missing_policy = AgentCommsRoute::new(AgentCommsRouteDraft {
            route_label: ROUTE_LABEL,
            transport_family: IROH_FAMILY,
            scope: scope(WORKSPACE)?,
            endpoint_hash: Some(SAFE_HASH),
            policy_hash: None,
            admission: ADMITTED,
            max_frame_bytes: MAX_FRAME_BYTES,
        })
        .expect_err("missing policy evidence should fail");
        let bad_family = AgentCommsRoute::new(AgentCommsRouteDraft {
            route_label: ROUTE_LABEL,
            transport_family: "serial_port",
            scope: scope(WORKSPACE)?,
            endpoint_hash: Some(SAFE_HASH),
            policy_hash: Some(SAFE_HASH),
            admission: ADMITTED,
            max_frame_bytes: MAX_FRAME_BYTES,
        })
        .expect_err("unsupported family should fail");

        assert!(matches!(
            missing_policy,
            AgentCommsError::MissingPolicyEvidence
        ));
        assert!(matches!(
            bad_family,
            AgentCommsError::UnsupportedValue { .. }
        ));
        Ok(())
    }

    #[test]
    fn route_fails_closed_without_admission_or_matching_scope() -> TestResult {
        let denied = route(WORKSPACE, DENIED, Some(SAFE_HASH))?;
        let admitted = route(WORKSPACE, ADMITTED, Some(SAFE_HASH))?;
        let other_scope = scope(OTHER_WORKSPACE)?;
        let assertion = assertion_for(WORKSPACE, ASSERTION_ID)?;
        let denied_error = admit_agent_comms_scope(&denied, assertion.scope())
            .expect_err("denied route should fail");
        let scope_error =
            admit_agent_comms_scope(&admitted, &other_scope).expect_err("cross scope should fail");

        assert_eq!(denied_error, AgentCommsError::AdmissionRequired);
        assert_eq!(scope_error, AgentCommsError::ScopeMismatch);
        Ok(())
    }

    #[test]
    fn frame_rejects_missing_redaction_and_invalid_hash() -> TestResult {
        let route = route(WORKSPACE, ADMITTED, Some(SAFE_HASH))?;
        let assertion = assertion_for(WORKSPACE, ASSERTION_ID)?;
        let missing_redactions = AgentCommsFrame::new(AgentCommsFrameDraft {
            frame_id: FRAME_ID,
            kind: AgentCommsFrameKind::DataspaceAssertion,
            route: &route,
            payload_hash: SAFE_HASH,
            payload_bytes: VALIDATED_FRAME_PAYLOAD_BYTES,
            item_count: 1,
            redactions: &[],
        })
        .expect_err("missing redaction evidence should fail");
        let invalid_hash = AgentCommsFrame::new(AgentCommsFrameDraft {
            frame_id: FRAME_ID,
            kind: AgentCommsFrameKind::DataspaceAssertion,
            route: &route,
            payload_hash: "not-a-valid-hash",
            payload_bytes: VALIDATED_FRAME_PAYLOAD_BYTES,
            item_count: 1,
            redactions: &[REDACTION_PROMPT],
        })
        .expect_err("invalid payload hash should fail");
        let oversized = test_ok(AgentCommsFrame::new(AgentCommsFrameDraft {
            frame_id: FRAME_ID,
            kind: AgentCommsFrameKind::DataspaceAssertion,
            route: &route,
            payload_hash: SAFE_HASH,
            payload_bytes: SMALL_FRAME_BYTES,
            item_count: 1,
            redactions: &[REDACTION_PROMPT],
        }))?;
        let tiny_route = test_ok(AgentCommsRoute::new(AgentCommsRouteDraft {
            route_label: ROUTE_LABEL,
            transport_family: IROH_FAMILY,
            scope: scope(WORKSPACE)?,
            endpoint_hash: Some(SAFE_HASH),
            policy_hash: Some(SAFE_HASH),
            admission: ADMITTED,
            max_frame_bytes: SMALL_FRAME_BYTES,
        }))?;
        let too_large = AgentCommsFrame::new(AgentCommsFrameDraft {
            frame_id: FRAME_ID,
            kind: AgentCommsFrameKind::DataspaceAssertion,
            route: &tiny_route,
            payload_hash: assertion.content_hash().map_or(SAFE_HASH, SafeHash::as_str),
            payload_bytes: OVERSIZED_FRAME_PAYLOAD_BYTES,
            item_count: 1,
            redactions: &[REDACTION_PROMPT],
        })
        .expect_err("oversized frame should fail");

        assert_eq!(
            missing_redactions,
            AgentCommsError::MissingRedactionEvidence
        );
        assert!(matches!(invalid_hash, AgentCommsError::Contract(_)));
        assert!(matches!(too_large, AgentCommsError::FrameTooLarge { .. }));
        assert_eq!(oversized.kind(), AgentCommsFrameKind::DataspaceAssertion);
        Ok(())
    }

    #[test]
    fn receipt_rejects_leaks_and_builds_safe_hashes() -> TestResult {
        let route = route(WORKSPACE, ADMITTED, Some(SAFE_HASH))?;
        let command = host_command()?;
        let frame = test_ok(project_headless_command_frame(&route, &command))?;
        let leak = AgentCommsReceipt::new(&AgentCommsReceiptDraft {
            route_label: ROUTE_LABEL_BAD,
            transport_family: IROH_FAMILY,
            scope: scope(WORKSPACE)?,
            endpoint_hash: Some(SAFE_HASH),
            policy_hash: Some(SAFE_HASH),
            admission: ADMITTED,
            max_frame_bytes: MAX_FRAME_BYTES,
            frame_id: FRAME_ID,
            frame_kind: HEADLESS_COMMAND_KIND,
            payload_hash: SAFE_HASH,
            payload_bytes: 1,
            item_count: 1,
            status: RECEIPT_STATUS_COMPLETED,
            redactions: &[ROUTE_METADATA_ONLY],
        })
        .expect_err("raw ticket marker should fail");
        let route_receipt = test_ok(AgentCommsReceipt::from_route(
            &route,
            AgentCommsReceiptStatus::Admitted,
        ))?;
        let receipt = test_ok(AgentCommsReceipt::from_frame(
            &route,
            &frame,
            AgentCommsReceiptStatus::Completed,
        ))?;

        assert!(matches!(
            leak,
            AgentCommsError::UnsafeRoute { .. } | AgentCommsError::ReceiptLeak { .. }
        ));
        assert_eq!(
            route_receipt.frame_kind(),
            AgentCommsFrameKind::RouteMetadata
        );
        assert_eq!(route_receipt.status(), AgentCommsReceiptStatus::Admitted);
        assert_eq!(receipt.transport_family(), AgentCommsTransportFamily::Iroh);
        assert_eq!(receipt.status(), AgentCommsReceiptStatus::Completed);
        assert!(!receipt.redactions().is_empty());
        Ok(())
    }

    #[test]
    fn headless_and_subagent_frames_project_safely() -> TestResult {
        let route = route(WORKSPACE, ADMITTED, Some(SAFE_HASH))?;
        let command = host_command()?;
        let event = host_event()?;
        let subagent = subagent_event()?;
        let command_frame = test_ok(project_headless_command_frame(&route, &command))?;
        let event_frame = test_ok(project_headless_event_frame(&route, &event))?;
        let subagent_frame = test_ok(project_subagent_lifecycle_frame(&route, &subagent))?;

        assert_eq!(command_frame.kind(), AgentCommsFrameKind::HeadlessCommand);
        assert_eq!(event_frame.kind(), AgentCommsFrameKind::HeadlessEvent);
        assert_eq!(
            subagent_frame.kind(),
            AgentCommsFrameKind::SubagentLifecycle
        );
        assert!(
            command_frame
                .redactions()
                .contains(&AgentCommsRedaction::CommandPayloadOmitted)
        );
        assert!(
            event_frame
                .redactions()
                .contains(&AgentCommsRedaction::EventPayloadOmitted)
        );
        assert!(
            subagent_frame
                .redactions()
                .contains(&AgentCommsRedaction::SubagentPayloadOmitted)
        );
        Ok(())
    }
}
