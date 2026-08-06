//! Optional OpenClaw-inspired gateway control-plane DTOs.
//!
//! This module models only reusable data validation. Hosts own transports,
//! process supervision, credential stores, provider sessions, dashboards, and
//! dedupe-cache storage.

const PROFILE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_profile_id");
const CLIENT_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_client_id");
const SCOPE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_scope");
const FEATURE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_feature");
const CHALLENGE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_challenge_id");
const VERIFIER_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_verifier_id");
const SUMMARY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_summary");
const CORRELATION_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_correlation_id");
const ACTOR_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_actor");
const PAYLOAD_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("gateway_payload_summary");
const STREAM_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_stream_id");
const EVENT_NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_event_name");
const ERROR_CODE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_error_code");
const ERROR_MESSAGE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_error_message");
const ERROR_DETAIL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_error_detail");

/// Active Animus gateway protocol version for the bounded `OpenClaw` profile.
pub const OPENCLAW_PROTOCOL_VERSION: u16 = 1;
/// Lowest accepted gateway protocol version.
pub const MIN_PROTOCOL_VERSION: u16 = OPENCLAW_PROTOCOL_VERSION;
/// Smallest valid sequence position in a gateway stream.
pub const MIN_SEQUENCE_POSITION: u64 = 1;
/// Smallest valid state-version value.
pub const MIN_STATE_VERSION: u64 = 1;

const MIN_REQUIRED_ITEMS: usize = 1;
const LEAK_RAW_PROMPT: &str = "RAW_PROMPT:";
const LEAK_PROVIDER_PAYLOAD: &str = "PROVIDER_PAYLOAD:";
const LEAK_CREDENTIAL: &str = "CREDENTIAL:";
const LEAK_SECRET: &str = "secret=";
const LEAK_AUTHORIZATION: &str = "authorization:";
const LEAK_FULL_PATH: &str = "/home/";
const LEAK_RAW_TOOL_BODY: &str = "RAW_TOOL_BODY:";
const LEAK_UNBOUNDED_OUTPUT: &str = "UNBOUNDED_OUTPUT:";
const UNSAFE_TEXT_MARKERS: &[&str] = &[
    LEAK_RAW_PROMPT,
    LEAK_PROVIDER_PAYLOAD,
    LEAK_CREDENTIAL,
    LEAK_SECRET,
    LEAK_AUTHORIZATION,
    LEAK_FULL_PATH,
    LEAK_RAW_TOOL_BODY,
    LEAK_UNBOUNDED_OUTPUT,
];
const REQUIRED_PROFILE_FEATURES: &[Feature] = &[
    Feature::ChallengeBinding,
    Feature::HelloSnapshots,
    Feature::StateVersions,
    Feature::Events,
    Feature::Idempotency,
];
const REJECTED_OPENCLAW_SURFACES: &[RejectedSurface] = &[
    RejectedSurface::WebSocketServer,
    RejectedSurface::HttpRoutes,
    RejectedSurface::ChannelConnector,
    RejectedSurface::ProviderSession,
    RejectedSurface::Dashboard,
    RejectedSurface::CredentialStore,
    RejectedSurface::DaemonSupervision,
    RejectedSurface::GlobalState,
];

/// Gateway validation error raised before host dispatch.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Generic adapter field validation failed.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 identity failed validation.
    Artifact(crate::ArtifactValidationError),
    /// The profile did not use the bounded `OpenClaw` reference.
    InvalidReference,
    /// The profile attempted to enable ambient transport fallback.
    AmbientTransportFallback,
    /// A required profile feature was absent.
    MissingProfileFeature {
        /// Missing feature.
        feature: Feature,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Field that contained no items.
        field: &'static str,
    },
    /// The protocol range was malformed.
    InvalidProtocolRange,
    /// The client and profile did not share a supported protocol version.
    ProtocolMismatch,
    /// A request appeared before a connect handshake.
    HandshakeRequired,
    /// Text contained a raw prompt, provider payload, credential, path, tool body, or unbounded
    /// output marker.
    UnsafePayload {
        /// Field that contained unsafe text.
        field: &'static str,
        /// Marker that caused rejection.
        marker: &'static str,
    },
    /// A stream sequence position was outside the accepted range.
    InvalidSequence,
    /// A state version was outside the accepted range.
    InvalidStateVersion,
    /// A frame state version was older than the current host version.
    StaleStateVersion,
    /// A side-effecting method did not include idempotency metadata.
    MissingIdempotency,
    /// Idempotency metadata declared a different method than the request.
    IdempotencyMethodMismatch,
    /// Idempotency metadata declared a different actor than the request.
    IdempotencyActorMismatch,
    /// A failed or rejected response did not carry a safe error.
    MissingError,
    /// A successful response carried an error payload.
    UnexpectedError,
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<crate::ArtifactValidationError> for Error {
    fn from(error: crate::ArtifactValidationError) -> Self {
        Self::Artifact(error)
    }
}

/// Bounded reference source for the gateway profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reference {
    /// `OpenClaw` is used only as an architectural reference.
    OpenClaw,
}

impl Reference {
    /// Returns the stable reference label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenClaw => "openclaw",
        }
    }
}

/// `OpenClaw` product surfaces that reusable Animus crates reject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedSurface {
    /// `WebSocket` server implementation.
    WebSocketServer,
    /// `HTTP` route implementation.
    HttpRoutes,
    /// Channel connector implementation.
    ChannelConnector,
    /// Provider session implementation.
    ProviderSession,
    /// Dashboard implementation.
    Dashboard,
    /// Credential store implementation.
    CredentialStore,
    /// Daemon supervision implementation.
    DaemonSupervision,
    /// Ambient global mutable state.
    GlobalState,
}

impl RejectedSurface {
    /// Returns the stable rejected-surface label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebSocketServer => "websocket_server",
            Self::HttpRoutes => "http_routes",
            Self::ChannelConnector => "channel_connector",
            Self::ProviderSession => "provider_session",
            Self::Dashboard => "dashboard",
            Self::CredentialStore => "credential_store",
            Self::DaemonSupervision => "daemon_supervision",
            Self::GlobalState => "global_state",
        }
    }
}

/// Gateway protocol feature advertised by profiles and clients.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Feature {
    /// Challenge evidence is bound into the connect handshake.
    ChallengeBinding,
    /// Safe hello snapshots are exchanged during connect.
    HelloSnapshots,
    /// Frames may carry state-version metadata.
    StateVersions,
    /// Event frames carry ordered stream sequence metadata.
    Events,
    /// Side-effecting methods require idempotency metadata.
    Idempotency,
}

impl Feature {
    /// Returns the stable feature label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChallengeBinding => "challenge_binding",
            Self::HelloSnapshots => "hello_snapshots",
            Self::StateVersions => "state_versions",
            Self::Events => "events",
            Self::Idempotency => "idempotency",
        }
    }
}

/// Borrowed optional gateway profile before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileDraft<'a> {
    /// Stable profile label.
    pub profile_id: &'a str,
    /// Bounded reference source.
    pub reference: Reference,
    /// Supported protocol features.
    pub features: &'a [Feature],
    /// Whether host code may fall back to ambient transport behavior.
    pub ambient_transport_fallback: bool,
}

/// Validated optional gateway protocol profile.
// r[impl agent_harness_runtime.openclaw_gateway.scope]
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    profile_id: alloc::string::String,
    reference: Reference,
    protocol_version: u16,
    features: alloc::vec::Vec<Feature>,
}

impl Profile {
    /// Builds a validated optional gateway profile.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the profile lacks the `OpenClaw` reference,
    /// required features, safe identity, or rejects ambient transport fallback.
    pub fn new(draft: &ProfileDraft<'_>) -> Result<Self, Error> {
        if draft.reference != Reference::OpenClaw {
            return Err(Error::InvalidReference);
        }
        if draft.ambient_transport_fallback {
            return Err(Error::AmbientTransportFallback);
        }
        require_non_empty(FEATURE_FIELD, draft.features.len())?;
        for feature in REQUIRED_PROFILE_FEATURES {
            if !draft.features.contains(feature) {
                return Err(Error::MissingProfileFeature { feature: *feature });
            }
        }
        Ok(Self {
            profile_id: required_text(PROFILE_ID_FIELD, draft.profile_id)?,
            reference: draft.reference,
            protocol_version: OPENCLAW_PROTOCOL_VERSION,
            features: draft.features.to_vec(),
        })
    }

    /// Returns the stable profile label.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the bounded reference source.
    #[must_use]
    pub const fn reference(&self) -> Reference {
        self.reference
    }

    /// Returns the active protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    /// Returns supported feature descriptors.
    #[must_use]
    pub fn features(&self) -> &[Feature] {
        self.features.as_slice()
    }

    /// Returns `OpenClaw` product surfaces rejected by reusable Animus crates.
    #[allow(clippy::unused_self)]
    #[must_use]
    pub const fn rejected_surfaces(&self) -> &'static [RejectedSurface] {
        REJECTED_OPENCLAW_SURFACES
    }
}

/// Borrowed protocol range advertised by a client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolRangeDraft {
    /// Lowest protocol version accepted by the client.
    pub minimum: u16,
    /// Highest protocol version accepted by the client.
    pub maximum: u16,
}

/// Validated gateway protocol version range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolRange {
    minimum: u16,
    maximum: u16,
}

impl ProtocolRange {
    /// Builds a validated protocol range.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidProtocolRange`] when bounds are empty or inverted.
    pub const fn new(draft: ProtocolRangeDraft) -> Result<Self, Error> {
        if draft.minimum < MIN_PROTOCOL_VERSION || draft.maximum < draft.minimum {
            return Err(Error::InvalidProtocolRange);
        }
        Ok(Self {
            minimum: draft.minimum,
            maximum: draft.maximum,
        })
    }

    /// Returns the lowest accepted version.
    #[must_use]
    pub const fn minimum(self) -> u16 {
        self.minimum
    }

    /// Returns the highest accepted version.
    #[must_use]
    pub const fn maximum(self) -> u16 {
        self.maximum
    }

    const fn contains(self, version: u16) -> bool {
        self.minimum <= version && version <= self.maximum
    }
}

/// Negotiates the profile protocol version against a client range.
///
/// # Errors
///
/// Returns [`Error::ProtocolMismatch`] when the profile version is not in range.
pub fn negotiate_protocol(profile: &Profile, range: ProtocolRange) -> Result<u16, Error> {
    let version = profile.protocol_version();
    if range.contains(version) {
        Ok(version)
    } else {
        Err(Error::ProtocolMismatch)
    }
}

/// Client role declared during gateway connect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    /// Human or supervisory operator.
    Operator,
    /// User interface shell.
    Ui,
    /// Capability node.
    Node,
    /// Read-only observer.
    Observer,
}

impl Role {
    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Ui => "ui",
            Self::Node => "node",
            Self::Observer => "observer",
        }
    }
}

/// Borrowed challenge evidence before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChallengeEvidenceDraft<'a> {
    /// Challenge label issued by the host-owned verifier.
    pub challenge_id: &'a str,
    /// Safe verifier label.
    pub verifier_id: &'a str,
    /// BLAKE3 hash of host-held challenge evidence.
    pub evidence_hash: &'a str,
}

/// Validated challenge evidence metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeEvidence {
    challenge_id: alloc::string::String,
    verifier_id: alloc::string::String,
    evidence_hash: crate::ArtifactHash,
}

impl ChallengeEvidence {
    /// Builds validated challenge evidence.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when labels or hash text are invalid.
    pub fn new(draft: &ChallengeEvidenceDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            challenge_id: required_text(CHALLENGE_ID_FIELD, draft.challenge_id)?,
            verifier_id: required_text(VERIFIER_ID_FIELD, draft.verifier_id)?,
            evidence_hash: crate::ArtifactHash::new(draft.evidence_hash)?,
        })
    }

    /// Returns the challenge label.
    #[must_use]
    pub fn challenge_id(&self) -> &str {
        self.challenge_id.as_str()
    }

    /// Returns the verifier label.
    #[must_use]
    pub fn verifier_id(&self) -> &str {
        self.verifier_id.as_str()
    }

    /// Returns the challenge evidence hash.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::ArtifactHash {
        &self.evidence_hash
    }
}

/// Validated gateway state version.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateVersion {
    value: u64,
}

impl StateVersion {
    /// Builds a validated state version.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidStateVersion`] when the value is below the accepted minimum.
    pub const fn new(value: u64) -> Result<Self, Error> {
        if value < MIN_STATE_VERSION {
            return Err(Error::InvalidStateVersion);
        }
        Ok(Self { value })
    }

    /// Returns the numeric state version.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.value
    }
}

/// Borrowed safe gateway snapshot before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotDraft<'a> {
    /// State version represented by the snapshot.
    pub state_version: StateVersion,
    /// BLAKE3 hash of host-held snapshot content.
    pub snapshot_hash: &'a str,
    /// Safe bounded snapshot summary.
    pub summary: &'a str,
}

/// Safe gateway snapshot metadata.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    state_version: StateVersion,
    snapshot_hash: crate::ArtifactHash,
    summary: alloc::string::String,
}

impl Snapshot {
    /// Builds a safe snapshot descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the hash or summary is unsafe.
    pub fn new(draft: &SnapshotDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            state_version: draft.state_version,
            snapshot_hash: crate::ArtifactHash::new(draft.snapshot_hash)?,
            summary: safe_text(SUMMARY_FIELD, draft.summary)?,
        })
    }

    /// Returns the snapshot state version.
    #[must_use]
    pub const fn state_version(&self) -> StateVersion {
        self.state_version
    }

    /// Returns the snapshot hash.
    #[must_use]
    pub const fn snapshot_hash(&self) -> &crate::ArtifactHash {
        &self.snapshot_hash
    }

    /// Returns the safe snapshot summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Borrowed connect handshake before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectHandshakeDraft<'a> {
    /// Protocol range advertised by the client.
    pub protocol_range: ProtocolRange,
    /// Stable client identity label.
    pub client_id: &'a str,
    /// Client role.
    pub role: Role,
    /// Requested scope labels.
    pub scopes: &'a [&'a str],
    /// Advertised client features.
    pub features: &'a [Feature],
    /// Challenge evidence supplied by host-owned verifier flow.
    pub challenge: ChallengeEvidenceDraft<'a>,
    /// Safe hello snapshot metadata.
    pub hello: SnapshotDraft<'a>,
}

/// Validated first-frame connect handshake.
// r[impl agent_harness_runtime.openclaw_gateway.handshake]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectHandshake {
    protocol_version: u16,
    client_id: alloc::string::String,
    role: Role,
    scopes: alloc::vec::Vec<alloc::string::String>,
    features: alloc::vec::Vec<Feature>,
    challenge: ChallengeEvidence,
    hello: Snapshot,
}

impl ConnectHandshake {
    /// Builds and negotiates a connect handshake.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when protocol negotiation fails or required
    /// handshake fields are absent, malformed, or unsafe.
    pub fn new(profile: &Profile, draft: &ConnectHandshakeDraft<'_>) -> Result<Self, Error> {
        let protocol_version = negotiate_protocol(profile, draft.protocol_range)?;
        require_non_empty(SCOPE_FIELD, draft.scopes.len())?;
        require_non_empty(FEATURE_FIELD, draft.features.len())?;
        Ok(Self {
            protocol_version,
            client_id: required_text(CLIENT_ID_FIELD, draft.client_id)?,
            role: draft.role,
            scopes: collect_texts(SCOPE_FIELD, draft.scopes)?,
            features: draft.features.to_vec(),
            challenge: ChallengeEvidence::new(&draft.challenge)?,
            hello: Snapshot::new(&draft.hello)?,
        })
    }

    /// Returns negotiated protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    /// Returns client identity.
    #[must_use]
    pub fn client_id(&self) -> &str {
        self.client_id.as_str()
    }

    /// Returns client role.
    #[must_use]
    pub const fn role(&self) -> Role {
        self.role
    }

    /// Returns requested scopes.
    #[must_use]
    pub fn scopes(&self) -> &[alloc::string::String] {
        self.scopes.as_slice()
    }

    /// Returns advertised features.
    #[must_use]
    pub fn features(&self) -> &[Feature] {
        self.features.as_slice()
    }

    /// Returns challenge evidence.
    #[must_use]
    pub const fn challenge(&self) -> &ChallengeEvidence {
        &self.challenge
    }

    /// Returns safe hello snapshot.
    #[must_use]
    pub const fn hello(&self) -> &Snapshot {
        &self.hello
    }
}

/// First inbound frame classification used before dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstFrame<'a> {
    /// First frame is a validated connect handshake.
    Connect(&'a ConnectHandshake),
    /// First frame is a request, response, or event before connect.
    NonConnect,
}

/// Validates that the first inbound frame is a connect handshake.
///
/// # Errors
///
/// Returns [`Error::HandshakeRequired`] when a non-connect frame arrives first.
pub fn validate_first_frame(frame: FirstFrame<'_>) -> Result<(), Error> {
    match frame {
        FirstFrame::Connect(_handshake) => Ok(()),
        FirstFrame::NonConnect => Err(Error::HandshakeRequired),
    }
}

/// Gateway method admitted by the protocol layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// Read safe state metadata.
    GetState,
    /// Attach a host-owned shell or observer.
    Attach,
    /// Send bounded input to a host-owned target.
    Send,
    /// Dispatch a tool request through host-owned services.
    ToolDispatch,
    /// Mutate session metadata through host-owned services.
    SessionMutate,
}

impl Method {
    /// Returns the stable method label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GetState => "get_state",
            Self::Attach => "attach",
            Self::Send => "send",
            Self::ToolDispatch => "tool_dispatch",
            Self::SessionMutate => "session_mutate",
        }
    }

    /// Returns whether the method needs idempotency metadata.
    #[must_use]
    pub const fn is_side_effecting(self) -> bool {
        matches!(self, Self::Send | Self::ToolDispatch | Self::SessionMutate)
    }
}

/// Bounded payload class carried by gateway frames.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadClass {
    /// Safe bounded summary text.
    SafeSummary,
    /// Safe selector metadata.
    Selector,
    /// BLAKE3 hash reference to host-held content.
    HashReference,
    /// Bounded metadata labels.
    BoundedMetadata,
    /// Safe snapshot summary.
    SnapshotSummary,
}

impl PayloadClass {
    /// Returns the stable payload-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SafeSummary => "safe_summary",
            Self::Selector => "selector",
            Self::HashReference => "hash_reference",
            Self::BoundedMetadata => "bounded_metadata",
            Self::SnapshotSummary => "snapshot_summary",
        }
    }
}

/// Borrowed stream sequence metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceDraft<'a> {
    /// Stream identity.
    pub stream_id: &'a str,
    /// Monotonic sequence position.
    pub position: u64,
}

/// Validated stream sequence metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sequence {
    stream_id: alloc::string::String,
    position: u64,
}

impl Sequence {
    /// Builds validated stream sequence metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the stream ID is invalid or the position is below the minimum.
    pub fn new(draft: &SequenceDraft<'_>) -> Result<Self, Error> {
        if draft.position < MIN_SEQUENCE_POSITION {
            return Err(Error::InvalidSequence);
        }
        Ok(Self {
            stream_id: required_text(STREAM_ID_FIELD, draft.stream_id)?,
            position: draft.position,
        })
    }

    /// Returns the stream identity.
    #[must_use]
    pub fn stream_id(&self) -> &str {
        self.stream_id.as_str()
    }

    /// Returns the stream position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }
}

/// Borrowed idempotency metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdempotencyDraft<'a> {
    /// BLAKE3 idempotency key identity.
    pub key_hash: &'a str,
    /// Method this key is bound to.
    pub method: Method,
    /// Actor this key is bound to.
    pub actor: &'a str,
    /// BLAKE3 identity of safe target metadata.
    pub target_hash: &'a str,
}

/// Validated idempotency metadata for a side-effecting method.
// r[impl agent_harness_runtime.openclaw_gateway.idempotency]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Idempotency {
    key_hash: crate::ArtifactHash,
    method: Method,
    actor: alloc::string::String,
    target_hash: crate::ArtifactHash,
}

impl Idempotency {
    /// Builds idempotency metadata and checks request binding.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when hashes or request-binding metadata are invalid.
    pub fn new(draft: &IdempotencyDraft<'_>, request_method: Method, request_actor: &str) -> Result<Self, Error> {
        if draft.method != request_method {
            return Err(Error::IdempotencyMethodMismatch);
        }
        let actor = required_text(ACTOR_FIELD, draft.actor)?;
        if actor != request_actor {
            return Err(Error::IdempotencyActorMismatch);
        }
        Ok(Self {
            key_hash: crate::ArtifactHash::new(draft.key_hash)?,
            method: draft.method,
            actor,
            target_hash: crate::ArtifactHash::new(draft.target_hash)?,
        })
    }

    /// Returns the idempotency key hash.
    #[must_use]
    pub const fn key_hash(&self) -> &crate::ArtifactHash {
        &self.key_hash
    }

    /// Returns the method bound to this key.
    #[must_use]
    pub const fn method(&self) -> Method {
        self.method
    }

    /// Returns the actor bound to this key.
    #[must_use]
    pub fn actor(&self) -> &str {
        self.actor.as_str()
    }

    /// Returns the safe target metadata hash.
    #[must_use]
    pub const fn target_hash(&self) -> &crate::ArtifactHash {
        &self.target_hash
    }
}

/// Borrowed request frame before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestFrameDraft<'a> {
    /// Correlation identity.
    pub correlation_id: &'a str,
    /// Request actor.
    pub actor: &'a str,
    /// Gateway method.
    pub method: Method,
    /// Bounded payload class.
    pub payload_class: PayloadClass,
    /// Safe payload summary or reference label.
    pub payload_summary: &'a str,
    /// Optional client-observed state version.
    pub state_version: Option<StateVersion>,
    /// Optional idempotency metadata.
    pub idempotency: Option<IdempotencyDraft<'a>>,
}

/// Validated request frame.
// r[impl agent_harness_runtime.openclaw_gateway.frames]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestFrame {
    correlation_id: alloc::string::String,
    actor: alloc::string::String,
    method: Method,
    payload_class: PayloadClass,
    payload_summary: alloc::string::String,
    state_version: Option<StateVersion>,
    idempotency: Option<Idempotency>,
}

impl RequestFrame {
    /// Builds a validated request frame without dispatching host work.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when frame metadata is malformed, unsafe, or lacks required idempotency.
    pub fn new(draft: &RequestFrameDraft<'_>) -> Result<Self, Error> {
        let actor = required_text(ACTOR_FIELD, draft.actor)?;
        let idempotency = match draft.idempotency {
            Some(idempotency) => Some(Idempotency::new(&idempotency, draft.method, actor.as_str())?),
            None if draft.method.is_side_effecting() => {
                return Err(Error::MissingIdempotency);
            }
            None => None,
        };
        Ok(Self {
            correlation_id: required_text(CORRELATION_ID_FIELD, draft.correlation_id)?,
            actor,
            method: draft.method,
            payload_class: draft.payload_class,
            payload_summary: safe_text(PAYLOAD_SUMMARY_FIELD, draft.payload_summary)?,
            state_version: draft.state_version,
            idempotency,
        })
    }

    /// Returns the correlation identity.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    /// Returns the actor label.
    #[must_use]
    pub fn actor(&self) -> &str {
        self.actor.as_str()
    }

    /// Returns the gateway method.
    #[must_use]
    pub const fn method(&self) -> Method {
        self.method
    }

    /// Returns the payload class.
    #[must_use]
    pub const fn payload_class(&self) -> PayloadClass {
        self.payload_class
    }

    /// Returns the safe payload summary.
    #[must_use]
    pub fn payload_summary(&self) -> &str {
        self.payload_summary.as_str()
    }

    /// Returns optional state-version metadata.
    #[must_use]
    pub const fn state_version(&self) -> Option<StateVersion> {
        self.state_version
    }

    /// Returns optional idempotency metadata.
    #[must_use]
    pub const fn idempotency(&self) -> Option<&Idempotency> {
        self.idempotency.as_ref()
    }
}

/// Gateway response status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseStatus {
    /// Request was accepted and completed at the protocol boundary.
    Accepted,
    /// Request was rejected before host dispatch.
    Rejected,
    /// Request failed after host-owned handling.
    Failed,
}

impl ResponseStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

/// Borrowed safe error before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SafeErrorDraft<'a> {
    /// Stable safe error code.
    pub code: &'a str,
    /// Safe bounded error message.
    pub message: &'a str,
    /// Optional safe bounded details.
    pub detail: Option<&'a str>,
}

/// Validated safe gateway error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeError {
    code: alloc::string::String,
    message: alloc::string::String,
    detail: Option<alloc::string::String>,
}

impl SafeError {
    /// Builds a safe error descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when error text is malformed or unsafe.
    pub fn new(draft: &SafeErrorDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            code: required_text(ERROR_CODE_FIELD, draft.code)?,
            message: safe_text(ERROR_MESSAGE_FIELD, draft.message)?,
            detail: draft.detail.map(|detail| safe_text(ERROR_DETAIL_FIELD, detail)).transpose()?,
        })
    }

    /// Returns the safe error code.
    #[must_use]
    pub fn code(&self) -> &str {
        self.code.as_str()
    }

    /// Returns the safe error message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    /// Returns optional safe detail text.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// Borrowed response frame before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponseFrameDraft<'a> {
    /// Correlation identity.
    pub correlation_id: &'a str,
    /// Response status.
    pub status: ResponseStatus,
    /// Optional stream sequence metadata.
    pub sequence: Option<SequenceDraft<'a>>,
    /// Optional response state version.
    pub state_version: Option<StateVersion>,
    /// Optional safe snapshot metadata.
    pub snapshot: Option<SnapshotDraft<'a>>,
    /// Optional safe error.
    pub error: Option<SafeErrorDraft<'a>>,
}

/// Validated response frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseFrame {
    correlation_id: alloc::string::String,
    status: ResponseStatus,
    sequence: Option<Sequence>,
    state_version: Option<StateVersion>,
    snapshot: Option<Snapshot>,
    error: Option<SafeError>,
}

impl ResponseFrame {
    /// Builds a validated response frame.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when frame metadata or safe errors are invalid.
    pub fn new(draft: &ResponseFrameDraft<'_>) -> Result<Self, Error> {
        let error = draft.error.map(|error| SafeError::new(&error)).transpose()?;
        match (draft.status, error.as_ref()) {
            (ResponseStatus::Accepted, Some(_error)) => {
                return Err(Error::UnexpectedError);
            }
            (ResponseStatus::Rejected | ResponseStatus::Failed, None) => {
                return Err(Error::MissingError);
            }
            _valid => {}
        }
        Ok(Self {
            correlation_id: required_text(CORRELATION_ID_FIELD, draft.correlation_id)?,
            status: draft.status,
            sequence: draft.sequence.map(|sequence| Sequence::new(&sequence)).transpose()?,
            state_version: draft.state_version,
            snapshot: draft.snapshot.map(|snapshot| Snapshot::new(&snapshot)).transpose()?,
            error,
        })
    }

    /// Returns the correlation identity.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    /// Returns the response status.
    #[must_use]
    pub const fn status(&self) -> ResponseStatus {
        self.status
    }

    /// Returns optional sequence metadata.
    #[must_use]
    pub const fn sequence(&self) -> Option<&Sequence> {
        self.sequence.as_ref()
    }

    /// Returns optional state-version metadata.
    #[must_use]
    pub const fn state_version(&self) -> Option<StateVersion> {
        self.state_version
    }

    /// Returns optional snapshot metadata.
    #[must_use]
    pub const fn snapshot(&self) -> Option<&Snapshot> {
        self.snapshot.as_ref()
    }

    /// Returns optional safe error metadata.
    #[must_use]
    pub const fn error(&self) -> Option<&SafeError> {
        self.error.as_ref()
    }
}

/// Borrowed event frame before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventFrameDraft<'a> {
    /// Correlation identity.
    pub correlation_id: &'a str,
    /// Event name.
    pub event_name: &'a str,
    /// Event payload class.
    pub payload_class: PayloadClass,
    /// Safe payload summary or reference label.
    pub payload_summary: &'a str,
    /// Required stream sequence metadata.
    pub sequence: SequenceDraft<'a>,
    /// Optional state-version metadata.
    pub state_version: Option<StateVersion>,
    /// Optional snapshot metadata.
    pub snapshot: Option<SnapshotDraft<'a>>,
}

/// Validated event frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventFrame {
    correlation_id: alloc::string::String,
    event_name: alloc::string::String,
    payload_class: PayloadClass,
    payload_summary: alloc::string::String,
    sequence: Sequence,
    state_version: Option<StateVersion>,
    snapshot: Option<Snapshot>,
}

impl EventFrame {
    /// Builds a validated event frame.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when event metadata is malformed or unsafe.
    pub fn new(draft: &EventFrameDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            correlation_id: required_text(CORRELATION_ID_FIELD, draft.correlation_id)?,
            event_name: required_text(EVENT_NAME_FIELD, draft.event_name)?,
            payload_class: draft.payload_class,
            payload_summary: safe_text(PAYLOAD_SUMMARY_FIELD, draft.payload_summary)?,
            sequence: Sequence::new(&draft.sequence)?,
            state_version: draft.state_version,
            snapshot: draft.snapshot.map(|snapshot| Snapshot::new(&snapshot)).transpose()?,
        })
    }

    /// Returns the correlation identity.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    /// Returns the event name.
    #[must_use]
    pub fn event_name(&self) -> &str {
        self.event_name.as_str()
    }

    /// Returns the event payload class.
    #[must_use]
    pub const fn payload_class(&self) -> PayloadClass {
        self.payload_class
    }

    /// Returns the safe payload summary.
    #[must_use]
    pub fn payload_summary(&self) -> &str {
        self.payload_summary.as_str()
    }

    /// Returns stream sequence metadata.
    #[must_use]
    pub const fn sequence(&self) -> &Sequence {
        &self.sequence
    }

    /// Returns optional state-version metadata.
    #[must_use]
    pub const fn state_version(&self) -> Option<StateVersion> {
        self.state_version
    }

    /// Returns optional snapshot metadata.
    #[must_use]
    pub const fn snapshot(&self) -> Option<&Snapshot> {
        self.snapshot.as_ref()
    }
}

/// Ensures a frame-observed state version is not stale.
///
/// # Errors
///
/// Returns [`Error::StaleStateVersion`] when `observed` is older than `current`.
pub const fn ensure_state_fresh(observed: StateVersion, current: StateVersion) -> Result<(), Error> {
    if observed.get() < current.get() {
        Err(Error::StaleStateVersion)
    } else {
        Ok(())
    }
}

fn require_non_empty(field_name: crate::validation::FieldName, len: usize) -> Result<(), Error> {
    if len < MIN_REQUIRED_ITEMS {
        return Err(Error::EmptyCollection {
            field: field_name.as_str(),
        });
    }
    Ok(())
}

fn required_text(field_name: crate::validation::FieldName, value: &str) -> Result<alloc::string::String, Error> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(Error::from)
}

fn safe_text(field_name: crate::validation::FieldName, value: &str) -> Result<alloc::string::String, Error> {
    let normalized = required_text(field_name, value)?;
    reject_unsafe_text(field_name, normalized.as_str())?;
    Ok(normalized)
}

fn collect_texts(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, Error> {
    values.iter().map(|value| required_text(field_name, value)).collect()
}

fn reject_unsafe_text(field_name: crate::validation::FieldName, value: &str) -> Result<(), Error> {
    let lower = value.to_ascii_lowercase();
    for marker in UNSAFE_TEXT_MARKERS {
        if lower.contains(&marker.to_ascii_lowercase()) {
            return Err(Error::UnsafePayload {
                field: field_name.as_str(),
                marker,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE_ID: &str = "openclaw-gateway";
    const CLIENT_ID: &str = "client-main";
    const CHALLENGE_ID: &str = "challenge-main";
    const VERIFIER_ID: &str = "verifier-main";
    const SCOPE_SESSION: &str = "session-main";
    const CORRELATION_ID: &str = "corr-main";
    const ACTOR_ID: &str = "actor-main";
    const STREAM_ID: &str = "events-main";
    const EVENT_STARTED: &str = "session_started";
    const SAFE_SUMMARY: &str = "safe bounded summary";
    const ERROR_CODE: &str = "denied";
    const ERROR_MESSAGE: &str = "request denied safely";
    const RAW_PROVIDER_DETAIL: &str = "PROVIDER_PAYLOAD:{hidden}";
    const HASH_A: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const HASH_B: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const HASH_C: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const BAD_HASH: &str = "not-a-blake3-hash";
    const CURRENT_STATE_VALUE: u64 = 7;
    const STALE_STATE_VALUE: u64 = 6;
    const EVENT_POSITION: u64 = 3;
    const PROTOCOL_VERSION_INCREMENT: u16 = 1;
    const NEXT_PROTOCOL_VERSION: u16 = OPENCLAW_PROTOCOL_VERSION.saturating_add(PROTOCOL_VERSION_INCREMENT);
    const FEATURES: &[Feature] = &[
        Feature::ChallengeBinding,
        Feature::HelloSnapshots,
        Feature::StateVersions,
        Feature::Events,
        Feature::Idempotency,
    ];

    #[test]
    fn profile_handshake_frames_and_idempotency_accept_safe_dtos() {
        // r[verify agent_harness_runtime.openclaw_gateway.scope]
        // r[verify agent_harness_runtime.openclaw_gateway.handshake]
        // r[verify agent_harness_runtime.openclaw_gateway.frames]
        // r[verify agent_harness_runtime.openclaw_gateway.idempotency]
        // r[verify agent_harness_runtime.openclaw_gateway.verification]
        let profile = profile();
        let state_version = StateVersion::new(CURRENT_STATE_VALUE).expect("state version should parse");
        let handshake = handshake(&profile, state_version);
        let idempotency = IdempotencyDraft {
            key_hash: HASH_B,
            method: Method::Send,
            actor: ACTOR_ID,
            target_hash: HASH_C,
        };
        let request = RequestFrame::new(&RequestFrameDraft {
            correlation_id: CORRELATION_ID,
            actor: ACTOR_ID,
            method: Method::Send,
            payload_class: PayloadClass::SafeSummary,
            payload_summary: SAFE_SUMMARY,
            state_version: Some(state_version),
            idempotency: Some(idempotency),
        })
        .expect("request should validate");
        let response = ResponseFrame::new(&ResponseFrameDraft {
            correlation_id: CORRELATION_ID,
            status: ResponseStatus::Accepted,
            sequence: Some(sequence()),
            state_version: Some(state_version),
            snapshot: Some(snapshot(state_version)),
            error: None,
        })
        .expect("response should validate");
        let event = EventFrame::new(&EventFrameDraft {
            correlation_id: CORRELATION_ID,
            event_name: EVENT_STARTED,
            payload_class: PayloadClass::SnapshotSummary,
            payload_summary: SAFE_SUMMARY,
            sequence: sequence(),
            state_version: Some(state_version),
            snapshot: Some(snapshot(state_version)),
        })
        .expect("event should validate");

        validate_first_frame(FirstFrame::Connect(&handshake)).expect("connect first frame should pass");
        ensure_state_fresh(state_version, state_version).expect("matching state version should pass");

        assert_eq!(profile.reference(), Reference::OpenClaw);
        assert_eq!(profile.rejected_surfaces(), REJECTED_OPENCLAW_SURFACES);
        assert_eq!(handshake.protocol_version(), OPENCLAW_PROTOCOL_VERSION);
        assert_eq!(handshake.scopes(), &[alloc::string::String::from(SCOPE_SESSION)]);
        assert_eq!(request.method(), Method::Send);
        assert!(request.idempotency().is_some());
        assert_eq!(response.sequence().expect("sequence").position(), EVENT_POSITION);
        assert_eq!(event.event_name(), EVENT_STARTED);
    }

    #[test]
    fn first_frame_protocol_state_and_ambient_fallback_reject() {
        // r[verify agent_harness_runtime.openclaw_gateway.handshake]
        // r[verify agent_harness_runtime.openclaw_gateway.frames]
        // r[verify agent_harness_runtime.openclaw_gateway.verification]
        let profile = profile();
        let current = StateVersion::new(CURRENT_STATE_VALUE).expect("state version");
        let stale = StateVersion::new(STALE_STATE_VALUE).expect("stale state version");
        let mismatched_range = ProtocolRange::new(ProtocolRangeDraft {
            minimum: NEXT_PROTOCOL_VERSION,
            maximum: NEXT_PROTOCOL_VERSION,
        })
        .expect("range should parse before negotiation");
        let ambient_profile = Profile::new(&ProfileDraft {
            profile_id: PROFILE_ID,
            reference: Reference::OpenClaw,
            features: FEATURES,
            ambient_transport_fallback: true,
        })
        .expect_err("ambient fallback should fail");

        assert_eq!(validate_first_frame(FirstFrame::NonConnect), Err(Error::HandshakeRequired));
        assert_eq!(negotiate_protocol(&profile, mismatched_range), Err(Error::ProtocolMismatch));
        assert_eq!(ensure_state_fresh(stale, current), Err(Error::StaleStateVersion));
        assert_eq!(ambient_profile, Error::AmbientTransportFallback);
    }

    #[test]
    fn idempotency_failures_reject_before_dispatch() {
        // r[verify agent_harness_runtime.openclaw_gateway.idempotency]
        // r[verify agent_harness_runtime.openclaw_gateway.verification]
        let current = StateVersion::new(CURRENT_STATE_VALUE).expect("state version");
        let missing_idempotency = RequestFrame::new(&request_draft(current, None))
            .expect_err("side-effecting request should require idempotency");
        let mismatched_idempotency = RequestFrame::new(&request_draft(
            current,
            Some(IdempotencyDraft {
                key_hash: HASH_B,
                method: Method::ToolDispatch,
                actor: ACTOR_ID,
                target_hash: HASH_C,
            }),
        ))
        .expect_err("mismatched method should fail");
        let malformed_idempotency = RequestFrame::new(&request_draft(
            current,
            Some(IdempotencyDraft {
                key_hash: BAD_HASH,
                method: Method::Send,
                actor: ACTOR_ID,
                target_hash: HASH_C,
            }),
        ))
        .expect_err("bad idempotency hash should fail");

        assert_eq!(missing_idempotency, Error::MissingIdempotency);
        assert_eq!(mismatched_idempotency, Error::IdempotencyMethodMismatch);
        assert!(matches!(malformed_idempotency, Error::Artifact(_)));
    }

    #[test]
    fn unsafe_error_details_reject_durable_response() {
        // r[verify agent_harness_runtime.openclaw_gateway.frames]
        // r[verify agent_harness_runtime.openclaw_gateway.verification]
        let current = StateVersion::new(CURRENT_STATE_VALUE).expect("state version");
        let unsafe_error = ResponseFrame::new(&ResponseFrameDraft {
            correlation_id: CORRELATION_ID,
            status: ResponseStatus::Failed,
            sequence: Some(sequence()),
            state_version: Some(current),
            snapshot: None,
            error: Some(SafeErrorDraft {
                code: ERROR_CODE,
                message: ERROR_MESSAGE,
                detail: Some(RAW_PROVIDER_DETAIL),
            }),
        })
        .expect_err("unsafe provider detail should fail");

        assert!(matches!(unsafe_error, Error::UnsafePayload { .. }));
    }

    const fn request_draft(
        state_version: StateVersion,
        idempotency: Option<IdempotencyDraft<'static>>,
    ) -> RequestFrameDraft<'static> {
        RequestFrameDraft {
            correlation_id: CORRELATION_ID,
            actor: ACTOR_ID,
            method: Method::Send,
            payload_class: PayloadClass::SafeSummary,
            payload_summary: SAFE_SUMMARY,
            state_version: Some(state_version),
            idempotency,
        }
    }

    fn profile() -> Profile {
        Profile::new(&ProfileDraft {
            profile_id: PROFILE_ID,
            reference: Reference::OpenClaw,
            features: FEATURES,
            ambient_transport_fallback: false,
        })
        .expect("profile should validate")
    }

    fn handshake(profile: &Profile, state_version: StateVersion) -> ConnectHandshake {
        let range = ProtocolRange::new(ProtocolRangeDraft {
            minimum: OPENCLAW_PROTOCOL_VERSION,
            maximum: OPENCLAW_PROTOCOL_VERSION,
        })
        .expect("range should parse");
        let handshake = ConnectHandshake::new(profile, &ConnectHandshakeDraft {
            protocol_range: range,
            client_id: CLIENT_ID,
            role: Role::Operator,
            scopes: &[SCOPE_SESSION],
            features: FEATURES,
            challenge: ChallengeEvidenceDraft {
                challenge_id: CHALLENGE_ID,
                verifier_id: VERIFIER_ID,
                evidence_hash: HASH_A,
            },
            hello: snapshot(state_version),
        })
        .expect("handshake should validate");

        assert_eq!(handshake.client_id(), CLIENT_ID);
        assert_eq!(handshake.hello().state_version(), state_version);
        handshake
    }

    const fn sequence() -> SequenceDraft<'static> {
        SequenceDraft {
            stream_id: STREAM_ID,
            position: EVENT_POSITION,
        }
    }

    const fn snapshot(state_version: StateVersion) -> SnapshotDraft<'static> {
        SnapshotDraft {
            state_version,
            snapshot_hash: HASH_A,
            summary: SAFE_SUMMARY,
        }
    }
}
