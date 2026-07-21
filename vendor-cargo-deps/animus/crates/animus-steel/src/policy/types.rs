const ACTION_MODEL: &str = "model";
const ACTION_TOOL: &str = "tool";
const ACTION_RESPOND: &str = "respond";
const ACTION_STOP: &str = "stop";
const FALLBACK_BLOCK: &str = "block";
const FALLBACK_STOP: &str = "stop";
const REDACTION_HASH_ONLY: &str = "hash-only";
const REDACTION_METADATA_ONLY: &str = "metadata-only";

/// Runtime host action that a Steel profile may expose.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HostAction {
    /// Permit Steel to request a model adapter call.
    Model,
    /// Permit Steel to request a tool adapter call.
    Tool,
    /// Permit Steel to return a final response.
    Respond,
    /// Permit Steel to stop without a final response.
    Stop,
}

impl HostAction {
    /// Parses a policy host-action label.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the label is not a supported action.
    pub fn parse(label: &str) -> Result<Self, Error> {
        match label {
            ACTION_MODEL => Ok(Self::Model),
            ACTION_TOOL => Ok(Self::Tool),
            ACTION_RESPOND => Ok(Self::Respond),
            ACTION_STOP => Ok(Self::Stop),
            _unknown => Err(Error::new(
                DenyReason::UnknownHostAction,
                "allowed_host_actions",
            )),
        }
    }

    /// Returns the canonical policy label for this action.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => ACTION_MODEL,
            Self::Tool => ACTION_TOOL,
            Self::Respond => ACTION_RESPOND,
            Self::Stop => ACTION_STOP,
        }
    }
}

/// Explicit fallback mode declared by a Steel profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackMode {
    /// Deny activation and do not attempt a fallback decision.
    Block,
    /// Use a bounded Rust-native stop decision when a caller implements fallback.
    Stop,
}

impl FallbackMode {
    /// Parses a fallback-mode label.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the label is unsupported.
    pub fn parse(label: &str) -> Result<Self, Error> {
        match label {
            FALLBACK_BLOCK => Ok(Self::Block),
            FALLBACK_STOP => Ok(Self::Stop),
            _unknown => Err(Error::new(DenyReason::UnsupportedFallback, "fallback_mode")),
        }
    }

    /// Returns the canonical policy label for this fallback mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Block => FALLBACK_BLOCK,
            Self::Stop => FALLBACK_STOP,
        }
    }
}

/// Receipt redaction mode declared by a Steel profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptRedaction {
    /// Receipt values are safe labels plus BLAKE3 hashes only.
    HashOnly,
    /// Receipt values are bounded metadata plus BLAKE3 hashes only.
    MetadataOnly,
}

impl ReceiptRedaction {
    /// Parses a receipt-redaction label.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the label is unsupported.
    pub fn parse(label: &str) -> Result<Self, Error> {
        match label {
            REDACTION_HASH_ONLY => Ok(Self::HashOnly),
            REDACTION_METADATA_ONLY => Ok(Self::MetadataOnly),
            _unknown => Err(Error::new(
                DenyReason::UnsupportedReceiptRedaction,
                "receipt_redaction",
            )),
        }
    }

    /// Returns the canonical policy label for this redaction mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HashOnly => REDACTION_HASH_ONLY,
            Self::MetadataOnly => REDACTION_METADATA_ONLY,
        }
    }
}

/// Typed reason class for Steel configuration errors or denials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DenyReason {
    /// A profile name was empty or outside the safe label grammar.
    BadProfileName,
    /// A script binding or source was missing.
    MissingScript,
    /// A script binding escaped the script namespace.
    PathEscape,
    /// A declared or observed script hash was invalid or mismatched.
    HashMismatch,
    /// A runtime turn budget was zero or exceeded the profile budget.
    InvalidBudget,
    /// A host action label was not known to the runtime.
    UnknownHostAction,
    /// Activation requested an action absent from the profile allowlist.
    HostActionDenied,
    /// The fallback mode was not supported.
    UnsupportedFallback,
    /// The receipt redaction mode was not supported.
    UnsupportedReceiptRedaction,
    /// The requested profile did not match the checked profile.
    ProfileMismatch,
    /// The profile is disabled.
    DisabledProfile,
}

impl DenyReason {
    /// Returns a stable machine-readable reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadProfileName => "bad_profile_name",
            Self::MissingScript => "missing_script",
            Self::PathEscape => "path_escape",
            Self::HashMismatch => "hash_mismatch",
            Self::InvalidBudget => "invalid_budget",
            Self::UnknownHostAction => "unknown_host_action",
            Self::HostActionDenied => "host_action_denied",
            Self::UnsupportedFallback => "unsupported_fallback",
            Self::UnsupportedReceiptRedaction => "unsupported_receipt_redaction",
            Self::ProfileMismatch => "profile_mismatch",
            Self::DisabledProfile => "disabled_profile",
        }
    }
}

/// Steel configuration error raised before a profile can activate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    reason: DenyReason,
    field: &'static str,
}

impl Error {
    pub(crate) const fn new(reason: DenyReason, field: &'static str) -> Self {
        Self { reason, field }
    }

    /// Returns the typed reason class.
    #[must_use]
    pub const fn reason(&self) -> DenyReason {
        self.reason
    }

    /// Returns the rejected field name.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "steel policy field {} failed: {}",
            self.field,
            self.reason.as_str()
        )
    }
}

impl std::error::Error for Error {}
