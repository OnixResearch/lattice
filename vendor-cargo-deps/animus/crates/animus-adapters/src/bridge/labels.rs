use super::InteropError;
use super::OP_DENIAL;
use super::OP_DISCOVERY;
use super::OP_MALFORMED_SCHEMA;
use super::OP_NETWORK_POLICY;
use super::OP_PASS_THROUGH;
use super::OP_SESSION_START;
use super::OP_TIMEOUT;
use super::OP_UNAVAILABLE_BINARY;
use super::OPERATION_FIELD;
use super::PERMISSION_ASK;
use super::PERMISSION_FIELD;
use super::PERMISSION_READ_ONLY;
use super::PERMISSION_UNRESTRICTED;
use super::POLICY_DENY;
use super::POLICY_FIELD;
use super::POLICY_INTERSECTION;
use super::POLICY_READ_ONLY;
use super::SESSION_FIELD;
use super::SESSION_JOIN_BY_LABEL;
use super::SESSION_NEW;
use super::TRANSPORT_FIELD;
use super::TRANSPORT_STDIO;
use super::TRANSPORT_STREAMABLE_HTTP;
use super::parse_value;

/// MCP transport kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransportKind {
    /// Stdio transport, represented by a host-owned command label.
    Stdio,
    /// Streamable HTTP transport, represented by a host-owned endpoint label.
    StreamableHttp,
}

impl McpTransportKind {
    /// Parses a stable MCP transport label.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, InteropError> {
        parse_value(TRANSPORT_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            TRANSPORT_STDIO => Some(Self::Stdio),
            TRANSPORT_STREAMABLE_HTTP => Some(Self::StreamableHttp),
            _ => None,
        }
    }
}

/// ACP permission mode projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpPermissionMode {
    /// Read-only external mode.
    ReadOnly,
    /// Ask-before-action external mode.
    Ask,
    /// Unrestricted external mode, admitted only by explicit host policy.
    Unrestricted,
}

impl AcpPermissionMode {
    /// Parses a stable ACP permission mode label.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, InteropError> {
        parse_value(PERMISSION_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            PERMISSION_READ_ONLY => Some(Self::ReadOnly),
            PERMISSION_ASK => Some(Self::Ask),
            PERMISSION_UNRESTRICTED => Some(Self::Unrestricted),
            _ => None,
        }
    }
}

/// ACP session correlation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpSessionCorrelation {
    /// Always create a new external session.
    NewSession,
    /// Join or correlate by an explicit redacted session label.
    JoinByLabel,
}

impl AcpSessionCorrelation {
    /// Parses a stable ACP session correlation label.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, InteropError> {
        parse_value(SESSION_FIELD, value, Self::from_normalized).map_err(|error| match error {
            InteropError::Contract(crate::AdapterContractError::EmptyField { .. }) => {
                InteropError::MissingSessionCorrelation
            }
            other => other,
        })
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            SESSION_NEW => Some(Self::NewSession),
            SESSION_JOIN_BY_LABEL => Some(Self::JoinByLabel),
            _ => None,
        }
    }
}

/// ACP tool pass-through policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpToolPassThroughPolicy {
    /// Do not pass tools through.
    Deny,
    /// Allow read-only pass-through after intersection.
    ReadOnly,
    /// Resolve pass-through by full capability intersection.
    Intersection,
}

impl AcpToolPassThroughPolicy {
    /// Parses a stable ACP tool pass-through policy label.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, InteropError> {
        parse_value(POLICY_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            POLICY_DENY => Some(Self::Deny),
            POLICY_READ_ONLY => Some(Self::ReadOnly),
            POLICY_INTERSECTION => Some(Self::Intersection),
            _ => None,
        }
    }
}
/// Interop receipt operation kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteropOperationKind {
    /// Discovery receipt.
    Discovery,
    /// Session start receipt.
    SessionStart,
    /// Tool pass-through receipt.
    PassThrough,
    /// Denial receipt.
    Denial,
    /// Timeout receipt.
    Timeout,
    /// Unavailable binary receipt.
    UnavailableBinary,
    /// Malformed schema receipt.
    MalformedSchema,
    /// Network policy receipt.
    NetworkPolicy,
}

impl InteropOperationKind {
    /// Parses a stable receipt operation label.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, InteropError> {
        parse_value(OPERATION_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            OP_DISCOVERY => Some(Self::Discovery),
            OP_SESSION_START => Some(Self::SessionStart),
            OP_PASS_THROUGH => Some(Self::PassThrough),
            OP_DENIAL => Some(Self::Denial),
            OP_TIMEOUT => Some(Self::Timeout),
            OP_UNAVAILABLE_BINARY => Some(Self::UnavailableBinary),
            OP_MALFORMED_SCHEMA => Some(Self::MalformedSchema),
            OP_NETWORK_POLICY => Some(Self::NetworkPolicy),
            _ => None,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Discovery => OP_DISCOVERY,
            Self::SessionStart => OP_SESSION_START,
            Self::PassThrough => OP_PASS_THROUGH,
            Self::Denial => OP_DENIAL,
            Self::Timeout => OP_TIMEOUT,
            Self::UnavailableBinary => OP_UNAVAILABLE_BINARY,
            Self::MalformedSchema => OP_MALFORMED_SCHEMA,
            Self::NetworkPolicy => OP_NETWORK_POLICY,
        }
    }
}
