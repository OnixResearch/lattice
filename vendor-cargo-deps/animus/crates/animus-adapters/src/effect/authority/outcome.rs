/// Authorization status for a guarded action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationStatus {
    /// Request is authorized and local constraints passed.
    Allowed,
    /// Request failed closed before guarded dispatch.
    Denied,
}

impl AuthorizationStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
        }
    }
}

/// Typed authorization issue code safe for receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationIssue {
    /// Authorization completed successfully.
    None,
    /// Nickel-reviewed contract did not permit the request.
    ContractMismatch,
    /// No verified UCAN grant authorized the request.
    MissingUcanGrant,
    /// A local caveat denied the request.
    LocalPolicyDenied,
    /// Required sandbox descriptor was malformed.
    SandboxMalformed,
    /// Required sandbox could not be enforced by the selected adapter.
    SandboxUnavailable,
}

impl AuthorizationIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ContractMismatch => "contract_mismatch",
            Self::MissingUcanGrant => "missing_ucan_grant",
            Self::LocalPolicyDenied => "local_policy_denied",
            Self::SandboxMalformed => "sandbox_malformed",
            Self::SandboxUnavailable => "sandbox_unavailable",
        }
    }
}

/// Unsafe receipt evidence class rejected by authorization evidence validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationLeak {
    /// Raw prompt text appeared in receipt evidence.
    RawPrompt,
    /// Credential-like material appeared in receipt evidence.
    Credential,
    /// Private absolute path material appeared in receipt evidence.
    PrivatePath,
    /// Raw provider payload appeared in receipt evidence.
    ProviderPayload,
    /// Raw tool body appeared in receipt evidence.
    RawToolBody,
    /// Raw model output appeared in receipt evidence.
    RawModelOutput,
}
