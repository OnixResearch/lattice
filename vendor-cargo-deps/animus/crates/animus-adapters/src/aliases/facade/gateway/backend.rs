const BACKEND_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_server_backend_identity");
const SECRET_SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_server_secret_source");
const SCOPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_server_scope");
const CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_server_capability");
const PROTOCOL_VERSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_server_protocol_version");
const FEATURE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_server_feature");
const ZERO_IDENTITY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const MAX_CAPABILITY_LABELS: usize = 32;
const MAX_FEATURE_LABELS: usize = 32;

/// Unsafe raw-material markers rejected from remote server descriptors and receipts.
pub const REMOTE_SERVER_RECEIPT_LEAK_MARKERS: &[&str] = &[
    "http://",
    "https://",
    "authorization:",
    "bearer ",
    "x-api-key",
    "raw_endpoint",
    "shared_secret=",
    "token=",
    "credential:",
    "-----BEGIN CERTIFICATE-----",
    "certificate_body:",
    "provider_payload:",
    "/home/",
    "/Users/",
];

/// Safe endpoint identity representation used by remote backend descriptors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerEndpointIdentityClass {
    /// Endpoint is represented only by a BLAKE3 hash.
    HashOnly,
    /// Endpoint has a separately reviewed redacted label and a hash.
    RedactedLabel,
}

impl RemoteServerEndpointIdentityClass {
    /// Returns the stable endpoint identity class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HashOnly => "hash_only",
            Self::RedactedLabel => "redacted_label",
        }
    }
}

/// Host-owned transport family for the remote server descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerTransportKind {
    /// HTTPS transport with host-owned TLS verification.
    Https,
    /// Plain HTTP transport, normally denied when TLS is required.
    Http,
    /// Host-owned tunnel whose TLS or equivalent security is summarized separately.
    HostTunnel,
    /// Unsupported transport marker.
    Unsupported,
}

impl RemoteServerTransportKind {
    /// Returns the stable transport label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Https => "https",
            Self::Http => "http",
            Self::HostTunnel => "host_tunnel",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Remote protocol role admitted by the descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerProtocolRole {
    /// Headless control protocol endpoint.
    HeadlessControl,
    /// MCP bridge endpoint.
    Mcp,
    /// Agent communication transport endpoint.
    AgentComms,
    /// Unsupported protocol marker.
    Unsupported,
}

impl RemoteServerProtocolRole {
    /// Returns the stable protocol-role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HeadlessControl => "headless_control",
            Self::Mcp => "mcp",
            Self::AgentComms => "agent_comms",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Platform role advertised by a remote backend descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerPlatformRole {
    /// Goose `serve`-style backend.
    GooseServeBackend,
    /// Animus-owned host shell backend.
    AnimusHost,
    /// Other reviewed backend family.
    ReviewedOther,
}

impl RemoteServerPlatformRole {
    /// Returns the stable platform-role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GooseServeBackend => "goose_serve_backend",
            Self::AnimusHost => "animus_host",
            Self::ReviewedOther => "reviewed_other",
        }
    }
}

/// TLS or equivalent secure-channel requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerTlsRequirement {
    /// Secure transport is required before admission.
    Required,
    /// Secure transport is not required by this descriptor.
    NotRequired,
}

impl RemoteServerTlsRequirement {
    /// Returns the stable TLS-requirement label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::NotRequired => "not_required",
        }
    }
}

/// Certificate fingerprint policy declared by the descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerFingerprintPolicy {
    /// A previously reviewed fingerprint pin must match.
    Pinned,
    /// Trust-on-first-use is pending explicit host approval.
    TrustOnFirstUsePending,
    /// Trust-on-first-use has explicit host approval.
    TrustOnFirstUseApproved,
    /// A rotated pin must still be checked against host-supplied state.
    RotatedPin,
}

impl RemoteServerFingerprintPolicy {
    /// Returns the stable fingerprint-policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pinned => "pinned",
            Self::TrustOnFirstUsePending => "tofu_pending",
            Self::TrustOnFirstUseApproved => "tofu_approved",
            Self::RotatedPin => "rotated_pin",
        }
    }
}

/// Secret source class recorded without exposing the secret value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerSecretSourceClass {
    /// Shared-secret source label.
    SharedSecret,
    /// Token or bearer-source label.
    Token,
    /// No secret is required for this reviewed descriptor.
    None,
}

impl RemoteServerSecretSourceClass {
    /// Returns the stable secret-source class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SharedSecret => "shared_secret_source",
            Self::Token => "token_source",
            Self::None => "none",
        }
    }

    const fn requires_secret(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Credential-source availability supplied by the host shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerCredentialSourceAvailability {
    /// Credential source is available to the host shell.
    Available,
    /// Credential source is unavailable.
    Unavailable,
    /// No credential source is required.
    NotRequired,
}

impl RemoteServerCredentialSourceAvailability {
    /// Returns the stable availability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
            Self::NotRequired => "not_required",
        }
    }
}

/// TLS state observed by the host-owned handshake probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerObservedTls {
    /// TLS or equivalent secure channel was observed.
    Enabled,
    /// TLS was not observed.
    Disabled,
    /// No handshake summary was supplied.
    NotObserved,
}

impl RemoteServerObservedTls {
    /// Returns the stable TLS observation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::NotObserved => "not_observed",
        }
    }
}

/// Certificate fingerprint status supplied by the host-owned probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerFingerprintStatus {
    /// Observed fingerprint matches policy.
    Matched,
    /// Observed fingerprint does not match policy.
    Mismatched,
    /// Required fingerprint evidence is missing.
    Missing,
    /// Trust-on-first-use is still pending explicit approval.
    TofuPending,
    /// Trust-on-first-use has explicit approval.
    TofuApproved,
    /// Pin state is stale and must not be used.
    Stale,
    /// No handshake summary was supplied.
    NotObserved,
}

impl RemoteServerFingerprintStatus {
    /// Returns the stable fingerprint-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::Mismatched => "mismatched",
            Self::Missing => "missing",
            Self::TofuPending => "tofu_pending",
            Self::TofuApproved => "tofu_approved",
            Self::Stale => "stale",
            Self::NotObserved => "not_observed",
        }
    }
}

/// Authentication result supplied by the host-owned handshake probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerAuthOutcome {
    /// Remote auth succeeded.
    Authorized,
    /// Remote auth failed.
    Unauthorized,
    /// Secret source was unavailable.
    SecretUnavailable,
    /// Required auth evidence was missing.
    Missing,
    /// No authentication was required.
    NotRequired,
    /// No handshake summary was supplied.
    NotObserved,
}

impl RemoteServerAuthOutcome {
    /// Returns the stable auth-outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authorized => "authorized",
            Self::Unauthorized => "unauthorized",
            Self::SecretUnavailable => "secret_unavailable",
            Self::Missing => "missing",
            Self::NotRequired => "not_required",
            Self::NotObserved => "not_observed",
        }
    }
}

/// Protocol version support state supplied by the host-owned probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerProtocolVersionStatus {
    /// Protocol version is supported.
    Supported,
    /// Protocol version is unsupported.
    Unsupported,
    /// No handshake summary was supplied.
    NotObserved,
}

impl RemoteServerProtocolVersionStatus {
    /// Returns the stable protocol-version status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::NotObserved => "not_observed",
        }
    }
}

/// Safe failure class supplied by the host-owned probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerFailureClass {
    /// No failure was observed.
    None,
    /// Remote network was unavailable to the host.
    NetworkUnavailable,
    /// TLS or certificate verification failed in the host probe.
    TlsFailure,
    /// Authentication failed in the host probe.
    AuthFailure,
    /// Protocol negotiation failed in the host probe.
    ProtocolFailure,
}

impl RemoteServerFailureClass {
    /// Returns the stable failure-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::NetworkUnavailable => "network_unavailable",
            Self::TlsFailure => "tls_failure",
            Self::AuthFailure => "auth_failure",
            Self::ProtocolFailure => "protocol_failure",
        }
    }
}

/// Remote server admission outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerAdmissionOutcome {
    /// Backend may be used by the host shell for the admitted scope.
    Admitted,
    /// Backend must not be used.
    Denied,
}

impl RemoteServerAdmissionOutcome {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

/// Fail-closed remote server admission reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteServerAdmissionReason {
    /// Host did not supply a handshake summary.
    MissingHandshake,
    /// TLS was required but not observed.
    MissingTls,
    /// Fingerprint was missing or mismatched.
    FingerprintMismatch,
    /// Trust-on-first-use was pending without explicit approval.
    TofuPending,
    /// Secret source was unavailable or missing.
    SecretUnavailable,
    /// Remote authentication failed.
    Unauthorized,
    /// Protocol role or version is unsupported.
    UnsupportedProtocol,
    /// Pin state is stale.
    StalePin,
    /// Requested scope is outside the descriptor boundary.
    ScopeDenied,
    /// Remote audience did not match the descriptor expectation.
    AudienceMismatch,
    /// Requested capabilities exceed the descriptor ceiling.
    CapabilityWidening,
    /// Fallback to another backend was requested without separate admission.
    FallbackDenied,
}

impl RemoteServerAdmissionReason {
    /// Returns the stable denial-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingHandshake => "missing_handshake",
            Self::MissingTls => "missing_tls",
            Self::FingerprintMismatch => "fingerprint_mismatch",
            Self::TofuPending => "tofu_pending",
            Self::SecretUnavailable => "secret_unavailable",
            Self::Unauthorized => "unauthorized",
            Self::UnsupportedProtocol => "unsupported_protocol",
            Self::StalePin => "stale_pin",
            Self::ScopeDenied => "scope_denied",
            Self::AudienceMismatch => "audience_mismatch",
            Self::CapabilityWidening => "capability_widening",
            Self::FallbackDenied => "fallback_denied",
        }
    }
}

/// Remote server admission validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteServerAdmissionError {
    /// Boundary field validation failed.
    Contract(crate::AdapterContractError),
    /// Safe hash or receipt identity validation failed.
    Artifact(crate::ArtifactValidationError),
    /// Descriptor or receipt text contained unsafe raw material.
    UnsafeMaterial {
        /// Matched unsafe marker.
        marker: &'static str,
    },
}

impl core::fmt::Display for RemoteServerAdmissionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "remote server contract error: {error}"),
            Self::Artifact(error) => write!(formatter, "remote server artifact error: {error}"),
            Self::UnsafeMaterial { marker } => {
                write!(formatter, "remote server evidence leaked marker {marker}")
            }
        }
    }
}

impl From<crate::AdapterContractError> for RemoteServerAdmissionError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<crate::ArtifactValidationError> for RemoteServerAdmissionError {
    fn from(error: crate::ArtifactValidationError) -> Self {
        Self::Artifact(error)
    }
}

// r[impl agent_harness_runtime.goose_remote_server_admission.descriptors]
/// Borrowed remote backend descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteServerDescriptorDraft<'a> {
    /// Safe backend identity label.
    pub backend_identity: &'a str,
    /// BLAKE3 hash of the endpoint identity; raw URLs are not accepted.
    pub endpoint_identity_hash: &'a str,
    /// Endpoint identity representation class.
    pub endpoint_identity_class: RemoteServerEndpointIdentityClass,
    /// Host-owned transport kind.
    pub transport: RemoteServerTransportKind,
    /// Remote protocol role.
    pub protocol_role: RemoteServerProtocolRole,
    /// Remote platform role.
    pub platform_role: RemoteServerPlatformRole,
    /// TLS or equivalent secure-channel requirement.
    pub tls_requirement: RemoteServerTlsRequirement,
    /// Certificate fingerprint policy.
    pub fingerprint_policy: RemoteServerFingerprintPolicy,
    /// Safe label for the host-owned secret source, never the secret value.
    pub secret_source_label: &'a str,
    /// Secret source class.
    pub secret_source_class: RemoteServerSecretSourceClass,
    /// Workspace or session scope boundary.
    pub scope: &'a str,
    /// Capability ceiling labels admitted for this backend.
    pub capability_ceilings: &'a [&'a str],
    /// Expected remote audience hash.
    pub expected_audience_hash: &'a str,
    /// Provenance descriptor hash.
    pub provenance_hash: &'a str,
}

/// Validated remote backend descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteServerDescriptor {
    backend_identity: alloc::string::String,
    backend_identity_hash: crate::ArtifactHash,
    endpoint_identity_hash: crate::ArtifactHash,
    endpoint_identity_class: RemoteServerEndpointIdentityClass,
    transport: RemoteServerTransportKind,
    protocol_role: RemoteServerProtocolRole,
    platform_role: RemoteServerPlatformRole,
    tls_requirement: RemoteServerTlsRequirement,
    fingerprint_policy: RemoteServerFingerprintPolicy,
    secret_source_label: alloc::string::String,
    secret_source_class: RemoteServerSecretSourceClass,
    scope: alloc::string::String,
    capability_ceilings: alloc::vec::Vec<alloc::string::String>,
    expected_audience_hash: crate::ArtifactHash,
    provenance_hash: crate::ArtifactHash,
}

impl RemoteServerDescriptor {
    /// Builds a validated remote backend descriptor without opening a transport.
    ///
    /// # Errors
    ///
    /// Returns [`RemoteServerAdmissionError`] when labels, hashes, or capability
    /// collections are invalid or contain unsafe raw material.
    pub fn new(
        draft: &RemoteServerDescriptorDraft<'_>,
    ) -> Result<Self, RemoteServerAdmissionError> {
        let backend_identity = safe_label(BACKEND_IDENTITY_FIELD, draft.backend_identity)?;
        let backend_identity_hash = identity_for_label(IdentityLabel::new(
            "backend_identity",
            backend_identity.as_str(),
        ))?;
        Ok(Self {
            backend_identity,
            backend_identity_hash,
            endpoint_identity_hash: crate::ArtifactHash::new(draft.endpoint_identity_hash)?,
            endpoint_identity_class: draft.endpoint_identity_class,
            transport: draft.transport,
            protocol_role: draft.protocol_role,
            platform_role: draft.platform_role,
            tls_requirement: draft.tls_requirement,
            fingerprint_policy: draft.fingerprint_policy,
            secret_source_label: safe_label(SECRET_SOURCE_FIELD, draft.secret_source_label)?,
            secret_source_class: draft.secret_source_class,
            scope: safe_label(SCOPE_FIELD, draft.scope)?,
            capability_ceilings: safe_labels(
                CAPABILITY_FIELD,
                draft.capability_ceilings,
                MAX_CAPABILITY_LABELS,
            )?,
            expected_audience_hash: crate::ArtifactHash::new(draft.expected_audience_hash)?,
            provenance_hash: crate::ArtifactHash::new(draft.provenance_hash)?,
        })
    }

    /// Returns the safe backend identity label.
    #[must_use]
    pub fn backend_identity(&self) -> &str {
        self.backend_identity.as_str()
    }

    /// Returns the deterministic backend identity hash.
    #[must_use]
    pub const fn backend_identity_hash(&self) -> &crate::ArtifactHash {
        &self.backend_identity_hash
    }

    /// Returns endpoint identity hash.
    #[must_use]
    pub const fn endpoint_identity_hash(&self) -> &crate::ArtifactHash {
        &self.endpoint_identity_hash
    }

    /// Returns endpoint identity class.
    #[must_use]
    pub const fn endpoint_identity_class(&self) -> RemoteServerEndpointIdentityClass {
        self.endpoint_identity_class
    }

    /// Returns transport kind.
    #[must_use]
    pub const fn transport(&self) -> RemoteServerTransportKind {
        self.transport
    }

    /// Returns protocol role.
    #[must_use]
    pub const fn protocol_role(&self) -> RemoteServerProtocolRole {
        self.protocol_role
    }

    /// Returns platform role.
    #[must_use]
    pub const fn platform_role(&self) -> RemoteServerPlatformRole {
        self.platform_role
    }

    /// Returns TLS requirement.
    #[must_use]
    pub const fn tls_requirement(&self) -> RemoteServerTlsRequirement {
        self.tls_requirement
    }

    /// Returns fingerprint policy.
    #[must_use]
    pub const fn fingerprint_policy(&self) -> RemoteServerFingerprintPolicy {
        self.fingerprint_policy
    }

    /// Returns safe secret-source label.
    #[must_use]
    pub fn secret_source_label(&self) -> &str {
        self.secret_source_label.as_str()
    }

    /// Returns secret-source class.
    #[must_use]
    pub const fn secret_source_class(&self) -> RemoteServerSecretSourceClass {
        self.secret_source_class
    }

    /// Returns admitted scope boundary.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns admitted capability ceilings.
    #[must_use]
    pub fn capability_ceilings(&self) -> &[alloc::string::String] {
        self.capability_ceilings.as_slice()
    }

    /// Returns expected remote audience hash.
    #[must_use]
    pub const fn expected_audience_hash(&self) -> &crate::ArtifactHash {
        &self.expected_audience_hash
    }

    /// Returns provenance descriptor hash.
    #[must_use]
    pub const fn provenance_hash(&self) -> &crate::ArtifactHash {
        &self.provenance_hash
    }
}

// r[impl agent_harness_runtime.goose_remote_server_admission.handshake]
/// Borrowed host-supplied handshake summary before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteServerHandshakeSummaryDraft<'a> {
    /// TLS state observed by the host probe.
    pub observed_tls: RemoteServerObservedTls,
    /// Certificate fingerprint status observed by the host probe.
    pub fingerprint_status: RemoteServerFingerprintStatus,
    /// Authentication outcome observed by the host probe.
    pub auth_outcome: RemoteServerAuthOutcome,
    /// Safe protocol version label or class.
    pub protocol_version: &'a str,
    /// Protocol version support state.
    pub protocol_version_status: RemoteServerProtocolVersionStatus,
    /// Accepted feature labels.
    pub accepted_features: &'a [&'a str],
    /// Remote audience hash observed by the host probe.
    pub audience_hash: &'a str,
    /// Safe host-owned failure class.
    pub safe_failure_class: RemoteServerFailureClass,
}

/// Validated host-supplied handshake summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteServerHandshakeSummary {
    observed_tls: RemoteServerObservedTls,
    fingerprint_status: RemoteServerFingerprintStatus,
    auth_outcome: RemoteServerAuthOutcome,
    protocol_version: alloc::string::String,
    protocol_version_status: RemoteServerProtocolVersionStatus,
    accepted_features: alloc::vec::Vec<alloc::string::String>,
    audience_hash: crate::ArtifactHash,
    safe_failure_class: RemoteServerFailureClass,
}

impl RemoteServerHandshakeSummary {
    /// Builds a validated host-supplied handshake summary.
    ///
    /// # Errors
    ///
    /// Returns [`RemoteServerAdmissionError`] when labels or hashes are invalid
    /// or contain unsafe raw material.
    pub fn new(
        draft: &RemoteServerHandshakeSummaryDraft<'_>,
    ) -> Result<Self, RemoteServerAdmissionError> {
        Ok(Self {
            observed_tls: draft.observed_tls,
            fingerprint_status: draft.fingerprint_status,
            auth_outcome: draft.auth_outcome,
            protocol_version: safe_label(PROTOCOL_VERSION_FIELD, draft.protocol_version)?,
            protocol_version_status: draft.protocol_version_status,
            accepted_features: safe_labels(
                FEATURE_FIELD,
                draft.accepted_features,
                MAX_FEATURE_LABELS,
            )?,
            audience_hash: crate::ArtifactHash::new(draft.audience_hash)?,
            safe_failure_class: draft.safe_failure_class,
        })
    }

    /// Returns observed TLS state.
    #[must_use]
    pub const fn observed_tls(&self) -> RemoteServerObservedTls {
        self.observed_tls
    }

    /// Returns observed fingerprint status.
    #[must_use]
    pub const fn fingerprint_status(&self) -> RemoteServerFingerprintStatus {
        self.fingerprint_status
    }

    /// Returns observed authentication outcome.
    #[must_use]
    pub const fn auth_outcome(&self) -> RemoteServerAuthOutcome {
        self.auth_outcome
    }

    /// Returns safe protocol version label.
    #[must_use]
    pub fn protocol_version(&self) -> &str {
        self.protocol_version.as_str()
    }

    /// Returns protocol version support state.
    #[must_use]
    pub const fn protocol_version_status(&self) -> RemoteServerProtocolVersionStatus {
        self.protocol_version_status
    }

    /// Returns accepted feature labels.
    #[must_use]
    pub fn accepted_features(&self) -> &[alloc::string::String] {
        self.accepted_features.as_slice()
    }

    /// Returns observed remote audience hash.
    #[must_use]
    pub const fn audience_hash(&self) -> &crate::ArtifactHash {
        &self.audience_hash
    }

    /// Returns safe failure class.
    #[must_use]
    pub const fn safe_failure_class(&self) -> RemoteServerFailureClass {
        self.safe_failure_class
    }
}

/// Borrowed remote admission policy before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteServerAdmissionPolicyDraft<'a> {
    /// Policy content hash.
    pub policy_hash: &'a str,
    /// Scope requested for this admission.
    pub requested_scope: &'a str,
    /// Capability labels requested for this admission.
    pub requested_capabilities: &'a [&'a str],
    /// Host-reported credential-source availability.
    pub credential_source_availability: RemoteServerCredentialSourceAvailability,
    /// Whether pending TOFU was explicitly approved by policy.
    pub tofu_approved: bool,
    /// Whether fallback to another backend was requested.
    pub fallback_requested: bool,
}

/// Validated remote admission policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteServerAdmissionPolicy {
    policy_hash: crate::ArtifactHash,
    requested_scope: alloc::string::String,
    requested_capabilities: alloc::vec::Vec<alloc::string::String>,
    credential_source_availability: RemoteServerCredentialSourceAvailability,
    tofu_approved: bool,
    fallback_requested: bool,
}

impl RemoteServerAdmissionPolicy {
    /// Builds validated policy inputs for pure remote server admission.
    ///
    /// # Errors
    ///
    /// Returns [`RemoteServerAdmissionError`] when policy hashes, scope, or
    /// capability labels are invalid or contain unsafe raw material.
    pub fn new(
        draft: &RemoteServerAdmissionPolicyDraft<'_>,
    ) -> Result<Self, RemoteServerAdmissionError> {
        Ok(Self {
            policy_hash: crate::ArtifactHash::new(draft.policy_hash)?,
            requested_scope: safe_label(SCOPE_FIELD, draft.requested_scope)?,
            requested_capabilities: safe_labels(
                CAPABILITY_FIELD,
                draft.requested_capabilities,
                MAX_CAPABILITY_LABELS,
            )?,
            credential_source_availability: draft.credential_source_availability,
            tofu_approved: draft.tofu_approved,
            fallback_requested: draft.fallback_requested,
        })
    }

    /// Returns policy content hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::ArtifactHash {
        &self.policy_hash
    }

    /// Returns requested scope.
    #[must_use]
    pub fn requested_scope(&self) -> &str {
        self.requested_scope.as_str()
    }

    /// Returns requested capability labels.
    #[must_use]
    pub fn requested_capabilities(&self) -> &[alloc::string::String] {
        self.requested_capabilities.as_slice()
    }

    /// Returns credential-source availability.
    #[must_use]
    pub const fn credential_source_availability(&self) -> RemoteServerCredentialSourceAvailability {
        self.credential_source_availability
    }

    /// Returns whether pending TOFU was approved.
    #[must_use]
    pub const fn tofu_approved(&self) -> bool {
        self.tofu_approved
    }

    /// Returns whether fallback was requested.
    #[must_use]
    pub const fn fallback_requested(&self) -> bool {
        self.fallback_requested
    }
}

/// Pure remote server admission request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteServerAdmissionRequest<'a> {
    /// Validated remote server descriptor.
    pub descriptor: &'a RemoteServerDescriptor,
    /// Optional host-supplied handshake summary.
    pub handshake: Option<&'a RemoteServerHandshakeSummary>,
    /// Validated remote admission policy inputs.
    pub policy: &'a RemoteServerAdmissionPolicy,
}

// r[impl agent_harness_runtime.goose_remote_server_admission.receipts]
/// Safe remote server admission receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteServerAdmissionReceipt {
    outcome: RemoteServerAdmissionOutcome,
    reason: Option<RemoteServerAdmissionReason>,
    backend_identity_hash: crate::ArtifactHash,
    endpoint_identity_class: RemoteServerEndpointIdentityClass,
    endpoint_identity_hash: crate::ArtifactHash,
    tls_status: RemoteServerObservedTls,
    fingerprint_status: RemoteServerFingerprintStatus,
    auth_status: RemoteServerAuthOutcome,
    protocol_version_status: RemoteServerProtocolVersionStatus,
    audience_hash: crate::ArtifactHash,
    secret_source_class: RemoteServerSecretSourceClass,
    scope: alloc::string::String,
    capability_ceilings: alloc::vec::Vec<alloc::string::String>,
    policy_hash: crate::ArtifactHash,
    failure_class: RemoteServerFailureClass,
    receipt_hash: crate::ArtifactHash,
}

impl RemoteServerAdmissionReceipt {
    /// Returns admission outcome.
    #[must_use]
    pub const fn outcome(&self) -> RemoteServerAdmissionOutcome {
        self.outcome
    }

    /// Returns fail-closed reason when denied.
    #[must_use]
    pub const fn reason(&self) -> Option<RemoteServerAdmissionReason> {
        self.reason
    }

    /// Returns backend identity hash.
    #[must_use]
    pub const fn backend_identity_hash(&self) -> &crate::ArtifactHash {
        &self.backend_identity_hash
    }

    /// Returns endpoint identity class.
    #[must_use]
    pub const fn endpoint_identity_class(&self) -> RemoteServerEndpointIdentityClass {
        self.endpoint_identity_class
    }

    /// Returns endpoint identity hash.
    #[must_use]
    pub const fn endpoint_identity_hash(&self) -> &crate::ArtifactHash {
        &self.endpoint_identity_hash
    }

    /// Returns TLS status.
    #[must_use]
    pub const fn tls_status(&self) -> RemoteServerObservedTls {
        self.tls_status
    }

    /// Returns fingerprint status.
    #[must_use]
    pub const fn fingerprint_status(&self) -> RemoteServerFingerprintStatus {
        self.fingerprint_status
    }

    /// Returns auth status.
    #[must_use]
    pub const fn auth_status(&self) -> RemoteServerAuthOutcome {
        self.auth_status
    }

    /// Returns protocol-version status.
    #[must_use]
    pub const fn protocol_version_status(&self) -> RemoteServerProtocolVersionStatus {
        self.protocol_version_status
    }

    /// Returns remote audience hash.
    #[must_use]
    pub const fn audience_hash(&self) -> &crate::ArtifactHash {
        &self.audience_hash
    }

    /// Returns secret-source class.
    #[must_use]
    pub const fn secret_source_class(&self) -> RemoteServerSecretSourceClass {
        self.secret_source_class
    }

    /// Returns admitted scope label.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns capability ceiling labels.
    #[must_use]
    pub fn capability_ceilings(&self) -> &[alloc::string::String] {
        self.capability_ceilings.as_slice()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::ArtifactHash {
        &self.policy_hash
    }

    /// Returns safe failure class.
    #[must_use]
    pub const fn failure_class(&self) -> RemoteServerFailureClass {
        self.failure_class
    }

    /// Returns deterministic BLAKE3 receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::ArtifactHash {
        &self.receipt_hash
    }

    /// Renders safe receipt metadata without raw endpoints or credentials.
    #[must_use]
    pub fn render_safe_text(&self) -> alloc::string::String {
        render_receipt_text(self)
    }
}

/// Remote server admission decision and receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteServerAdmissionDecision {
    outcome: RemoteServerAdmissionOutcome,
    reason: Option<RemoteServerAdmissionReason>,
    receipt: RemoteServerAdmissionReceipt,
}

impl RemoteServerAdmissionDecision {
    /// Returns admission outcome.
    #[must_use]
    pub const fn outcome(&self) -> RemoteServerAdmissionOutcome {
        self.outcome
    }

    /// Returns fail-closed reason when denied.
    #[must_use]
    pub const fn reason(&self) -> Option<RemoteServerAdmissionReason> {
        self.reason
    }

    /// Returns safe admission receipt.
    #[must_use]
    pub const fn receipt(&self) -> &RemoteServerAdmissionReceipt {
        &self.receipt
    }

    /// Returns true only when the backend may be used by the host shell.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self.outcome, RemoteServerAdmissionOutcome::Admitted)
    }
}

// r[impl agent_harness_runtime.goose_remote_server_admission.admission]
// r[impl agent_harness_runtime.goose_remote_server_admission.fail_closed]
/// Evaluates remote server admission using only descriptor, handshake, and policy data.
///
/// # Errors
///
/// Returns [`RemoteServerAdmissionError`] if safe receipt identity construction fails.
pub fn evaluate_remote_server_admission(
    request: RemoteServerAdmissionRequest<'_>,
) -> Result<RemoteServerAdmissionDecision, RemoteServerAdmissionError> {
    let reason = admission_reason(request);
    let outcome = if reason.is_some() {
        RemoteServerAdmissionOutcome::Denied
    } else {
        RemoteServerAdmissionOutcome::Admitted
    };
    let receipt = build_receipt(request, outcome, reason)?;
    Ok(RemoteServerAdmissionDecision {
        outcome,
        reason,
        receipt,
    })
}

/// Rejects unsafe rendered remote server admission receipt text.
///
/// # Errors
///
/// Returns [`RemoteServerAdmissionError::UnsafeMaterial`] when text contains a
/// known endpoint, secret, header, token, credential, certificate, provider,
/// prompt, or path leak marker.
pub fn validate_remote_server_admission_receipt_text(
    text: &str,
) -> Result<(), RemoteServerAdmissionError> {
    ensure_no_remote_server_leak(text)
}

fn admission_reason(
    request: RemoteServerAdmissionRequest<'_>,
) -> Option<RemoteServerAdmissionReason> {
    fallback_denial(request.policy)
        .or_else(|| handshake_denial(request))
        .or_else(|| tls_denial(request))
        .or_else(|| fingerprint_denial(request))
        .or_else(|| secret_denial(request))
        .or_else(|| protocol_denial(request))
        .or_else(|| audience_denial(request))
        .or_else(|| scope_denial(request))
        .or_else(|| capability_denial(request))
}

fn fallback_denial(policy: &RemoteServerAdmissionPolicy) -> Option<RemoteServerAdmissionReason> {
    policy
        .fallback_requested()
        .then_some(RemoteServerAdmissionReason::FallbackDenied)
}

fn handshake_denial(
    request: RemoteServerAdmissionRequest<'_>,
) -> Option<RemoteServerAdmissionReason> {
    request
        .handshake
        .is_none()
        .then_some(RemoteServerAdmissionReason::MissingHandshake)
}

fn tls_denial(request: RemoteServerAdmissionRequest<'_>) -> Option<RemoteServerAdmissionReason> {
    if request.descriptor.tls_requirement() != RemoteServerTlsRequirement::Required {
        return None;
    }
    if request.descriptor.transport() == RemoteServerTransportKind::Http {
        return Some(RemoteServerAdmissionReason::MissingTls);
    }
    let handshake = request.handshake?;
    (handshake.observed_tls() != RemoteServerObservedTls::Enabled)
        .then_some(RemoteServerAdmissionReason::MissingTls)
}

fn fingerprint_denial(
    request: RemoteServerAdmissionRequest<'_>,
) -> Option<RemoteServerAdmissionReason> {
    let handshake = request.handshake?;
    match handshake.fingerprint_status() {
        RemoteServerFingerprintStatus::Stale => Some(RemoteServerAdmissionReason::StalePin),
        RemoteServerFingerprintStatus::Mismatched
        | RemoteServerFingerprintStatus::Missing
        | RemoteServerFingerprintStatus::NotObserved => {
            Some(RemoteServerAdmissionReason::FingerprintMismatch)
        }
        RemoteServerFingerprintStatus::TofuPending => tofu_denial(request.policy),
        RemoteServerFingerprintStatus::Matched | RemoteServerFingerprintStatus::TofuApproved => {
            pinned_denial(request.descriptor, handshake)
        }
    }
}

fn tofu_denial(policy: &RemoteServerAdmissionPolicy) -> Option<RemoteServerAdmissionReason> {
    (!policy.tofu_approved()).then_some(RemoteServerAdmissionReason::TofuPending)
}

fn pinned_denial(
    descriptor: &RemoteServerDescriptor,
    handshake: &RemoteServerHandshakeSummary,
) -> Option<RemoteServerAdmissionReason> {
    if descriptor.fingerprint_policy() != RemoteServerFingerprintPolicy::Pinned {
        return None;
    }
    (handshake.fingerprint_status() != RemoteServerFingerprintStatus::Matched)
        .then_some(RemoteServerAdmissionReason::FingerprintMismatch)
}

fn secret_denial(request: RemoteServerAdmissionRequest<'_>) -> Option<RemoteServerAdmissionReason> {
    debug_assert!(!request.descriptor.secret_source_label().is_empty());
    debug_assert!(!request.descriptor.scope().is_empty());
    if !request.descriptor.secret_source_class().requires_secret() {
        return None;
    }
    if request.policy.credential_source_availability()
        != RemoteServerCredentialSourceAvailability::Available
    {
        return Some(RemoteServerAdmissionReason::SecretUnavailable);
    }
    let handshake = request.handshake?;
    match handshake.auth_outcome() {
        RemoteServerAuthOutcome::Authorized | RemoteServerAuthOutcome::NotRequired => None,
        RemoteServerAuthOutcome::Unauthorized => Some(RemoteServerAdmissionReason::Unauthorized),
        RemoteServerAuthOutcome::SecretUnavailable
        | RemoteServerAuthOutcome::Missing
        | RemoteServerAuthOutcome::NotObserved => {
            Some(RemoteServerAdmissionReason::SecretUnavailable)
        }
    }
}

fn protocol_denial(
    request: RemoteServerAdmissionRequest<'_>,
) -> Option<RemoteServerAdmissionReason> {
    if request.descriptor.protocol_role() == RemoteServerProtocolRole::Unsupported {
        return Some(RemoteServerAdmissionReason::UnsupportedProtocol);
    }
    let handshake = request.handshake?;
    (handshake.protocol_version_status() != RemoteServerProtocolVersionStatus::Supported)
        .then_some(RemoteServerAdmissionReason::UnsupportedProtocol)
}

fn audience_denial(
    request: RemoteServerAdmissionRequest<'_>,
) -> Option<RemoteServerAdmissionReason> {
    let handshake = request.handshake?;
    (handshake.audience_hash() != request.descriptor.expected_audience_hash())
        .then_some(RemoteServerAdmissionReason::AudienceMismatch)
}

fn scope_denial(request: RemoteServerAdmissionRequest<'_>) -> Option<RemoteServerAdmissionReason> {
    (request.policy.requested_scope() != request.descriptor.scope())
        .then_some(RemoteServerAdmissionReason::ScopeDenied)
}

fn capability_denial(
    request: RemoteServerAdmissionRequest<'_>,
) -> Option<RemoteServerAdmissionReason> {
    request
        .policy
        .requested_capabilities()
        .iter()
        .any(|requested| !capability_allowed(request.descriptor, requested))
        .then_some(RemoteServerAdmissionReason::CapabilityWidening)
}

fn capability_allowed(descriptor: &RemoteServerDescriptor, requested: &str) -> bool {
    descriptor
        .capability_ceilings()
        .iter()
        .any(|ceiling| ceiling.as_str() == requested)
}

fn build_receipt(
    request: RemoteServerAdmissionRequest<'_>,
    outcome: RemoteServerAdmissionOutcome,
    reason: Option<RemoteServerAdmissionReason>,
) -> Result<RemoteServerAdmissionReceipt, RemoteServerAdmissionError> {
    debug_assert_eq!(
        matches!(outcome, RemoteServerAdmissionOutcome::Denied),
        reason.is_some()
    );
    debug_assert!(!request.descriptor.capability_ceilings().is_empty());
    let mut receipt = RemoteServerAdmissionReceipt {
        outcome,
        reason,
        backend_identity_hash: request.descriptor.backend_identity_hash().clone(),
        endpoint_identity_class: request.descriptor.endpoint_identity_class(),
        endpoint_identity_hash: request.descriptor.endpoint_identity_hash().clone(),
        tls_status: handshake_tls(request.handshake),
        fingerprint_status: handshake_fingerprint(request.handshake),
        auth_status: handshake_auth(request.handshake),
        protocol_version_status: handshake_protocol_status(request.handshake),
        audience_hash: handshake_audience_hash(request)?,
        secret_source_class: request.descriptor.secret_source_class(),
        scope: alloc::string::String::from(request.descriptor.scope()),
        capability_ceilings: request.descriptor.capability_ceilings().to_vec(),
        policy_hash: request.policy.policy_hash().clone(),
        failure_class: handshake_failure_class(request.handshake),
        receipt_hash: zero_identity()?,
    };
    receipt.receipt_hash = identity_for_payload(render_receipt_text(&receipt).as_str())?;
    Ok(receipt)
}

fn handshake_tls(handshake: Option<&RemoteServerHandshakeSummary>) -> RemoteServerObservedTls {
    handshake.map_or(
        RemoteServerObservedTls::NotObserved,
        RemoteServerHandshakeSummary::observed_tls,
    )
}

fn handshake_fingerprint(
    handshake: Option<&RemoteServerHandshakeSummary>,
) -> RemoteServerFingerprintStatus {
    handshake.map_or(
        RemoteServerFingerprintStatus::NotObserved,
        RemoteServerHandshakeSummary::fingerprint_status,
    )
}

fn handshake_auth(handshake: Option<&RemoteServerHandshakeSummary>) -> RemoteServerAuthOutcome {
    handshake.map_or(
        RemoteServerAuthOutcome::NotObserved,
        RemoteServerHandshakeSummary::auth_outcome,
    )
}

fn handshake_protocol_status(
    handshake: Option<&RemoteServerHandshakeSummary>,
) -> RemoteServerProtocolVersionStatus {
    handshake.map_or(
        RemoteServerProtocolVersionStatus::NotObserved,
        RemoteServerHandshakeSummary::protocol_version_status,
    )
}

fn handshake_failure_class(
    handshake: Option<&RemoteServerHandshakeSummary>,
) -> RemoteServerFailureClass {
    handshake.map_or(
        RemoteServerFailureClass::NetworkUnavailable,
        RemoteServerHandshakeSummary::safe_failure_class,
    )
}

fn handshake_audience_hash(
    request: RemoteServerAdmissionRequest<'_>,
) -> Result<crate::ArtifactHash, RemoteServerAdmissionError> {
    match request.handshake {
        Some(handshake) => Ok(handshake.audience_hash().clone()),
        None => zero_identity(),
    }
}

fn render_receipt_text(receipt: &RemoteServerAdmissionReceipt) -> alloc::string::String {
    debug_assert!(!receipt.scope().is_empty());
    debug_assert!(!receipt.capability_ceilings().is_empty());
    let mut renderer = ReceiptRenderer::new();
    renderer.push(ReceiptLine::new("outcome", receipt.outcome().as_str()));
    if let Some(reason) = receipt.reason() {
        renderer.push(ReceiptLine::new("denial_class", reason.as_str()));
    }
    renderer.push(ReceiptLine::new(
        "backend_identity_hash",
        receipt.backend_identity_hash().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "endpoint_identity_class",
        receipt.endpoint_identity_class().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "endpoint_identity_hash",
        receipt.endpoint_identity_hash().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "tls_status",
        receipt.tls_status().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "fingerprint_status",
        receipt.fingerprint_status().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "auth_status",
        receipt.auth_status().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "protocol_version_status",
        receipt.protocol_version_status().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "audience_hash",
        receipt.audience_hash().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "secret_source_class",
        receipt.secret_source_class().as_str(),
    ));
    renderer.push(ReceiptLine::new("scope", receipt.scope()));
    for capability in receipt.capability_ceilings() {
        renderer.push(ReceiptLine::new("capability_ceiling", capability.as_str()));
    }
    renderer.push(ReceiptLine::new(
        "policy_hash",
        receipt.policy_hash().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "failure_class",
        receipt.failure_class().as_str(),
    ));
    renderer.push(ReceiptLine::new(
        "receipt_hash",
        receipt.receipt_hash().as_str(),
    ));
    renderer.finish()
}

#[derive(Clone, Copy)]
struct ReceiptLine<'a> {
    key: &'a str,
    value: &'a str,
}

impl<'a> ReceiptLine<'a> {
    const fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}

struct ReceiptRenderer {
    output: alloc::string::String,
}

impl ReceiptRenderer {
    fn new() -> Self {
        Self {
            output: alloc::string::String::new(),
        }
    }

    fn push(&mut self, line: ReceiptLine<'_>) {
        debug_assert!(!line.key.is_empty());
        debug_assert!(!line.value.is_empty());
        self.output.push_str(line.key);
        self.output.push('=');
        self.output.push_str(line.value);
        self.output.push('\n');
    }

    fn finish(self) -> alloc::string::String {
        self.output
    }
}

fn safe_label(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, RemoteServerAdmissionError> {
    let normalized = crate::validation::normalized_adapter_field(field, value)?;
    ensure_no_remote_server_leak(normalized.as_str())?;
    Ok(normalized)
}

fn safe_labels(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<alloc::vec::Vec<alloc::string::String>, RemoteServerAdmissionError> {
    debug_assert!(!field.as_str().is_empty());
    debug_assert!(max_items > 0);
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field.as_str(),
        }
        .into());
    }
    if values.len() > max_items {
        return Err(crate::AdapterContractError::FieldTooLong {
            field: field.as_str(),
            max_chars: max_items,
            actual_chars: values.len(),
        }
        .into());
    }
    values
        .iter()
        .map(|value| safe_label(field, value))
        .collect()
}

fn ensure_no_remote_server_leak(text: &str) -> Result<(), RemoteServerAdmissionError> {
    for marker in super::LEAK_MARKERS {
        if text.contains(marker) {
            return Err(RemoteServerAdmissionError::UnsafeMaterial { marker });
        }
    }
    for marker in REMOTE_SERVER_RECEIPT_LEAK_MARKERS {
        if text.contains(marker) {
            return Err(RemoteServerAdmissionError::UnsafeMaterial { marker });
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct IdentityLabel<'a> {
    label: &'a str,
    value: &'a str,
}

impl<'a> IdentityLabel<'a> {
    const fn new(label: &'a str, value: &'a str) -> Self {
        Self { label, value }
    }
}

fn identity_for_label(
    identity: IdentityLabel<'_>,
) -> Result<crate::ArtifactHash, RemoteServerAdmissionError> {
    debug_assert!(!identity.label.is_empty());
    debug_assert!(!identity.value.is_empty());
    let mut renderer = ReceiptRenderer::new();
    renderer.push(ReceiptLine::new("label", identity.label));
    renderer.push(ReceiptLine::new("value", identity.value));
    let payload = renderer.finish();
    identity_for_payload(payload.as_str())
}

fn identity_for_payload(payload: &str) -> Result<crate::ArtifactHash, RemoteServerAdmissionError> {
    debug_assert!(!payload.is_empty());
    let envelope = crate::ArtifactEnvelope::new(crate::ArtifactEnvelopeDraft::new(
        crate::ArtifactKind::RuntimeReceipt,
        crate::ArtifactRedactionClass::MetadataOnly,
        payload,
    ))?;
    Ok(envelope.identity().clone())
}

fn zero_identity() -> Result<crate::ArtifactHash, RemoteServerAdmissionError> {
    crate::ArtifactHash::new(ZERO_IDENTITY_HEX).map_err(RemoteServerAdmissionError::from)
}

const _: () = assert!(MAX_CAPABILITY_LABELS > 0);
const _: () = assert!(MAX_FEATURE_LABELS > 0);
