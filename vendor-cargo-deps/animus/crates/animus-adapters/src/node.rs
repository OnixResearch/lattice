//! Optional OpenClaw-inspired node capability-host DTOs.
//!
//! Nodes are modeled as host-owned peripherals. This module validates claims,
//! invocation plans, approval evidence, and safe outcomes only; hosts own device
//! apps, platform APIs, transports, pairing stores, approval persistence, and
//! command execution.

#![cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]

const NODE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_id");
const GATEWAY_PROFILE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("gateway_profile_id");
const PLATFORM_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_platform_family");
const SCOPE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_approved_scope");
const TOKEN_CLASS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_token_identity_class");
const CWD_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_cwd_label");
const RAW_COMMAND_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("node_raw_command_label");
const SESSION_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_session_label");
const AGENT_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_agent_label");
const APPROVAL_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_approval_id");
const APPROVAL_SUMMARY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_approval_summary");
const RESULT_SUMMARY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_result_summary");
const RECEIPT_SUMMARY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("node_receipt_summary");

/// Minimum accepted host timeout in milliseconds.
pub const MIN_TIMEOUT_MILLIS: u64 = 1;

const MIN_REQUIRED_ITEMS: usize = 1;
const LEAK_RAW_PROMPT: &str = "RAW_PROMPT:";
const LEAK_PROVIDER_PAYLOAD: &str = "PROVIDER_PAYLOAD:";
const LEAK_CREDENTIAL: &str = "CREDENTIAL:";
const LEAK_ENV_VALUE: &str = "ENV=";
const LEAK_AUTHORIZATION: &str = "authorization:";
const LEAK_FULL_PATH: &str = "/home/";
const LEAK_RAW_COMMAND: &str = "RAW_COMMAND:";
const LEAK_UNBOUNDED_OUTPUT: &str = "UNBOUNDED_OUTPUT:";
const UNSAFE_TEXT_MARKERS: &[&str] = &[
    LEAK_RAW_PROMPT,
    LEAK_PROVIDER_PAYLOAD,
    LEAK_CREDENTIAL,
    LEAK_ENV_VALUE,
    LEAK_AUTHORIZATION,
    LEAK_FULL_PATH,
    LEAK_RAW_COMMAND,
    LEAK_UNBOUNDED_OUTPUT,
];

/// Node capability-host validation error raised before reusable code dispatches effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Generic adapter field validation failed.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 identity failed validation.
    Artifact(crate::ArtifactValidationError),
    /// A node descriptor attempted to claim gateway-owned authority.
    GatewayOwnedClaim {
        /// Rejected gateway-owned claim.
        claim: GatewayOwnedClaim,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Field that contained no items.
        field: &'static str,
    },
    /// A descriptor attempted to enable ambient device, process, socket, or platform fallback.
    AmbientExecutionFallback,
    /// The node was not paired with the current host policy.
    UnpairedNode,
    /// Pinned node metadata changed and requires repair or re-approval.
    MetadataDrift,
    /// The requested command was not in the node allowlist.
    UnsupportedCommand,
    /// The invocation requested a permission the node did not advertise.
    PermissionMismatch,
    /// A required host node service was not injected.
    MissingService,
    /// Host policy denied the selected node service.
    ServiceDenied,
    /// The selected node service was unavailable.
    ServiceUnavailable,
    /// The selected node service timed out.
    Timeout,
    /// The invocation was cancelled.
    Cancelled,
    /// Host output was malformed or a completed result lacked required identity.
    MalformedResult,
    /// A high-impact node command lacked canonical exec metadata.
    MissingExecPlan,
    /// A high-impact node command lacked per-node approval evidence.
    MissingApproval,
    /// Approval evidence explicitly denied dispatch.
    ApprovalDenied,
    /// Approval evidence was for another node.
    ApprovalNodeMismatch,
    /// Approval evidence was for another command.
    ApprovalCommandMismatch,
    /// Idempotency metadata was for another node.
    IdempotencyNodeMismatch,
    /// Idempotency metadata was for another command.
    IdempotencyCommandMismatch,
    /// Idempotency metadata was for another parameter identity.
    IdempotencyParamsMismatch,
    /// Text contained a raw prompt, provider payload, credential, path, command, or unbounded
    /// output marker.
    UnsafePayload {
        /// Field that contained unsafe text.
        field: &'static str,
        /// Marker that caused rejection.
        marker: &'static str,
    },
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

/// Gateway-owned authority that node descriptors cannot claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayOwnedClaim {
    /// Gateway control-plane ownership.
    GatewayControl,
    /// Model routing ownership.
    ModelRouting,
    /// Channel connector ownership.
    ChannelConnector,
    /// Provider session ownership.
    ProviderSession,
    /// Durable session-store ownership.
    DurableSessionStore,
}

impl GatewayOwnedClaim {
    /// Returns the stable claim label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GatewayControl => "gateway_control",
            Self::ModelRouting => "model_routing",
            Self::ChannelConnector => "channel_connector",
            Self::ProviderSession => "provider_session",
            Self::DurableSessionStore => "durable_session_store",
        }
    }
}

/// Host-owned node capability claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Camera access claim.
    Camera,
    /// Screen capture or screen-recording claim.
    Screen,
    /// Location access claim.
    Location,
    /// SMS or telephony claim.
    Sms,
    /// Shell-like host command claim.
    Shell,
    /// Safe metadata inspection claim.
    Metadata,
    /// User notification claim.
    Notification,
}

impl Capability {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::Screen => "screen",
            Self::Location => "location",
            Self::Sms => "sms",
            Self::Shell => "shell",
            Self::Metadata => "metadata",
            Self::Notification => "notification",
        }
    }
}

/// Command a node claims it can accept after policy admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    /// Capture an image through a host-owned platform API.
    CameraCapture,
    /// Record or capture a screen through a host-owned platform API.
    ScreenRecord,
    /// Read location through a host-owned platform API.
    LocationRead,
    /// Send SMS through a host-owned platform API.
    SmsSend,
    /// Run a canonical command through a host-owned executor.
    SystemRun,
    /// Resolve a binary or command through a host-owned executor.
    SystemWhich,
    /// Emit a user notification through a host-owned platform API.
    Notify,
    /// Read safe node metadata.
    MetadataRead,
}

impl Command {
    /// Returns the stable command label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CameraCapture => "camera.capture",
            Self::ScreenRecord => "screen.record",
            Self::LocationRead => "location.read",
            Self::SmsSend => "sms.send",
            Self::SystemRun => "system.run",
            Self::SystemWhich => "system.which",
            Self::Notify => "notify",
            Self::MetadataRead => "metadata.read",
        }
    }

    /// Returns whether this command requires a canonical exec plan and approval evidence.
    #[must_use]
    pub const fn is_high_impact(self) -> bool {
        matches!(self, Self::SystemRun | Self::SystemWhich)
    }
}

/// Permission claim attached to a node command surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionClaim {
    /// Foreground app or active-user presence is required.
    Foreground,
    /// Camera permission is required.
    Camera,
    /// Screen-capture permission is required.
    Screen,
    /// Location permission is required.
    Location,
    /// SMS permission is required.
    Sms,
    /// Filesystem permission is required.
    Filesystem,
    /// Shell execution permission is required.
    Shell,
    /// Network permission is required.
    Network,
}

impl PermissionClaim {
    /// Returns the stable permission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Foreground => "foreground",
            Self::Camera => "camera",
            Self::Screen => "screen",
            Self::Location => "location",
            Self::Sms => "sms",
            Self::Filesystem => "filesystem",
            Self::Shell => "shell",
            Self::Network => "network",
        }
    }
}

/// Role a paired node plays for a host shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeRole {
    /// Mobile or desktop device peripheral.
    Device,
    /// Remote machine capability host.
    Machine,
    /// Platform-local tool host.
    Platform,
    /// Read-only metadata source.
    Observer,
}

impl NodeRole {
    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Device => "device",
            Self::Machine => "machine",
            Self::Platform => "platform",
            Self::Observer => "observer",
        }
    }
}

/// Safe class of host-held pairing token identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenIdentityClass {
    /// Local pairing secret identity.
    LocalPairingSecret,
    /// Hardware or platform-bound token identity.
    PlatformAttested,
    /// Operator-approved session token identity.
    OperatorApproved,
}

impl TokenIdentityClass {
    /// Returns the stable token class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalPairingSecret => "local_pairing_secret",
            Self::PlatformAttested => "platform_attested",
            Self::OperatorApproved => "operator_approved",
        }
    }
}

/// Borrowed node metadata pin before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataPinDraft<'a> {
    /// Safe platform family label pinned for this node.
    pub platform_family: &'a str,
    /// BLAKE3 identity of the command ceiling metadata.
    pub command_ceiling_hash: &'a str,
    /// BLAKE3 identity of the host-held descriptor metadata.
    pub descriptor_hash: &'a str,
}

/// Validated node metadata pin used for drift checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataPin {
    platform_family: alloc::string::String,
    command_ceiling_hash: crate::ArtifactHash,
    descriptor_hash: crate::ArtifactHash,
}

impl MetadataPin {
    /// Builds a validated metadata pin.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when labels or hashes are invalid or unsafe.
    pub fn new(draft: &MetadataPinDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            platform_family: safe_text(PLATFORM_FIELD, draft.platform_family)?,
            command_ceiling_hash: crate::ArtifactHash::new(draft.command_ceiling_hash)?,
            descriptor_hash: crate::ArtifactHash::new(draft.descriptor_hash)?,
        })
    }

    /// Returns the safe platform family label.
    #[must_use]
    pub fn platform_family(&self) -> &str {
        self.platform_family.as_str()
    }

    /// Returns the command ceiling identity.
    #[must_use]
    pub const fn command_ceiling_hash(&self) -> &crate::ArtifactHash {
        &self.command_ceiling_hash
    }

    /// Returns the descriptor identity.
    #[must_use]
    pub const fn descriptor_hash(&self) -> &crate::ArtifactHash {
        &self.descriptor_hash
    }
}

/// Borrowed node advertisement before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeAdvertisementDraft<'a> {
    /// Stable node identity label.
    pub node_id: &'a str,
    /// Gateway profile associated with this node.
    pub gateway_profile_id: &'a str,
    /// Safe platform family label.
    pub platform_family: &'a str,
    /// Advertised capability claims.
    pub capabilities: &'a [Capability],
    /// Advertised command allowlist.
    pub commands: &'a [Command],
    /// Advertised permission claims.
    pub permissions: &'a [PermissionClaim],
    /// Optional metadata-pinning evidence.
    pub metadata_pin: Option<MetadataPinDraft<'a>>,
    /// Rejected gateway-owned authority claims.
    pub gateway_owned_claims: &'a [GatewayOwnedClaim],
    /// Whether reusable code may fall back to ambient execution.
    pub ambient_execution_fallback: bool,
}

/// Validated node advertisement claims.
// r[impl agent_harness_runtime.openclaw_nodes.scope]
// r[impl agent_harness_runtime.openclaw_nodes.advertisement]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeAdvertisement {
    node_id: alloc::string::String,
    gateway_profile_id: alloc::string::String,
    platform_family: alloc::string::String,
    capabilities: alloc::vec::Vec<Capability>,
    commands: alloc::vec::Vec<Command>,
    permissions: alloc::vec::Vec<PermissionClaim>,
    metadata_pin: Option<MetadataPin>,
}

impl NodeAdvertisement {
    /// Builds a validated node advertisement.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the advertisement is empty, unsafe, claims gateway-owned
    /// authority, or requests ambient execution fallback.
    pub fn new(draft: &NodeAdvertisementDraft<'_>) -> Result<Self, Error> {
        if draft.ambient_execution_fallback {
            return Err(Error::AmbientExecutionFallback);
        }
        if let Some(claim) = draft.gateway_owned_claims.first() {
            return Err(Error::GatewayOwnedClaim { claim: *claim });
        }
        require_non_empty(NODE_ID_FIELD, draft.capabilities.len())?;
        require_non_empty(NODE_ID_FIELD, draft.commands.len())?;
        require_non_empty(NODE_ID_FIELD, draft.permissions.len())?;
        Ok(Self {
            node_id: required_text(NODE_ID_FIELD, draft.node_id)?,
            gateway_profile_id: required_text(GATEWAY_PROFILE_FIELD, draft.gateway_profile_id)?,
            platform_family: safe_text(PLATFORM_FIELD, draft.platform_family)?,
            capabilities: draft.capabilities.to_vec(),
            commands: draft.commands.to_vec(),
            permissions: draft.permissions.to_vec(),
            metadata_pin: draft.metadata_pin.map(|metadata| MetadataPin::new(&metadata)).transpose()?,
        })
    }

    /// Returns stable node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns associated gateway profile identity.
    #[must_use]
    pub fn gateway_profile_id(&self) -> &str {
        self.gateway_profile_id.as_str()
    }

    /// Returns platform family label.
    #[must_use]
    pub fn platform_family(&self) -> &str {
        self.platform_family.as_str()
    }

    /// Returns advertised capability claims.
    #[must_use]
    pub fn capabilities(&self) -> &[Capability] {
        self.capabilities.as_slice()
    }

    /// Returns advertised commands.
    #[must_use]
    pub fn commands(&self) -> &[Command] {
        self.commands.as_slice()
    }

    /// Returns advertised permissions.
    #[must_use]
    pub fn permissions(&self) -> &[PermissionClaim] {
        self.permissions.as_slice()
    }

    /// Returns optional metadata-pinning evidence.
    #[must_use]
    pub const fn metadata_pin(&self) -> Option<&MetadataPin> {
        self.metadata_pin.as_ref()
    }
}

/// Borrowed node pairing descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PairingDescriptorDraft<'a> {
    /// Paired node identity.
    pub node_id: &'a str,
    /// Role admitted for this node.
    pub role: NodeRole,
    /// Approved scopes or command ceilings.
    pub approved_scopes: &'a [&'a str],
    /// Admitted command ceiling.
    pub command_ceiling: &'a [Command],
    /// Safe class of host-held token identity.
    pub token_identity_class: TokenIdentityClass,
    /// Pinned metadata for drift checks.
    pub pinned_metadata: MetadataPinDraft<'a>,
}

/// Validated pairing descriptor.
// r[impl agent_harness_runtime.openclaw_nodes.pairing]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingDescriptor {
    node_id: alloc::string::String,
    role: NodeRole,
    approved_scopes: alloc::vec::Vec<alloc::string::String>,
    command_ceiling: alloc::vec::Vec<Command>,
    token_identity_class: TokenIdentityClass,
    pinned_metadata: MetadataPin,
}

impl PairingDescriptor {
    /// Builds a validated pairing descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when required pairing fields are absent or unsafe.
    pub fn new(draft: &PairingDescriptorDraft<'_>) -> Result<Self, Error> {
        require_non_empty(SCOPE_FIELD, draft.approved_scopes.len())?;
        require_non_empty(TOKEN_CLASS_FIELD, draft.command_ceiling.len())?;
        Ok(Self {
            node_id: required_text(NODE_ID_FIELD, draft.node_id)?,
            role: draft.role,
            approved_scopes: collect_texts(SCOPE_FIELD, draft.approved_scopes)?,
            command_ceiling: draft.command_ceiling.to_vec(),
            token_identity_class: draft.token_identity_class,
            pinned_metadata: MetadataPin::new(&draft.pinned_metadata)?,
        })
    }

    /// Returns paired node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns node role.
    #[must_use]
    pub const fn role(&self) -> NodeRole {
        self.role
    }

    /// Returns approved scopes.
    #[must_use]
    pub fn approved_scopes(&self) -> &[alloc::string::String] {
        self.approved_scopes.as_slice()
    }

    /// Returns admitted command ceiling.
    #[must_use]
    pub fn command_ceiling(&self) -> &[Command] {
        self.command_ceiling.as_slice()
    }

    /// Returns token identity class.
    #[must_use]
    pub const fn token_identity_class(&self) -> TokenIdentityClass {
        self.token_identity_class
    }

    /// Returns pinned metadata.
    #[must_use]
    pub const fn pinned_metadata(&self) -> &MetadataPin {
        &self.pinned_metadata
    }
}

/// Validates that a node remains paired and pinned metadata has not drifted.
///
/// # Errors
///
/// Returns [`Error::UnpairedNode`] for absent or mismatched pairings and
/// [`Error::MetadataDrift`] for missing, changed, or widened metadata.
///
/// # Panics
///
/// Panics if node/pairing identity or metadata-pin post-conditions are violated
/// after validation.
pub fn validate_node_pairing(node: &NodeAdvertisement, pairing: Option<&PairingDescriptor>) -> Result<(), Error> {
    let Some(pairing) = pairing else {
        return Err(Error::UnpairedNode);
    };
    if node.node_id() != pairing.node_id() {
        return Err(Error::UnpairedNode);
    }
    assert_eq!(node.node_id(), pairing.node_id());
    let Some(metadata_pin) = node.metadata_pin() else {
        return Err(Error::MetadataDrift);
    };
    if metadata_pin != pairing.pinned_metadata() {
        return Err(Error::MetadataDrift);
    }
    assert_eq!(metadata_pin, pairing.pinned_metadata());
    for command in node.commands() {
        if !pairing.command_ceiling().contains(command) {
            return Err(Error::MetadataDrift);
        }
    }
    Ok(())
}

/// Timeout posture for a node invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeoutPolicy {
    millis: u64,
}

impl TimeoutPolicy {
    /// Builds a timeout policy.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Timeout`] when the timeout is below the accepted minimum.
    pub const fn new(millis: u64) -> Result<Self, Error> {
        if millis < MIN_TIMEOUT_MILLIS {
            return Err(Error::Timeout);
        }
        Ok(Self { millis })
    }

    /// Returns timeout in milliseconds.
    #[must_use]
    pub const fn millis(self) -> u64 {
        self.millis
    }
}

/// Host node service availability before dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeServiceState {
    /// A matching host node service is injected.
    Injected,
    /// No matching service is present.
    Missing,
    /// Host policy denied the service.
    Denied,
    /// Host service is unavailable.
    Unavailable,
}

/// Cancellation support advertised by the host service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancellationPosture {
    /// A cancellation handle is available.
    Available,
    /// Cancellation is unavailable for this invocation.
    Unavailable,
}

/// Borrowed node idempotency metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeIdempotencyDraft<'a> {
    /// BLAKE3 idempotency key identity.
    pub key_hash: &'a str,
    /// Node identity bound to the key.
    pub node_id: &'a str,
    /// Command bound to the key.
    pub command: Command,
    /// BLAKE3 identity of safe parameter metadata.
    pub params_hash: &'a str,
}

/// Validated node idempotency metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeIdempotency {
    key_hash: crate::ArtifactHash,
    node_id: alloc::string::String,
    command: Command,
    params_hash: crate::ArtifactHash,
}

impl NodeIdempotency {
    /// Builds idempotency metadata and checks request binding.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the key or request-binding metadata is invalid.
    pub fn new(
        draft: &NodeIdempotencyDraft<'_>,
        request_node_id: &str,
        request_command: Command,
        request_params_hash: &crate::ArtifactHash,
    ) -> Result<Self, Error> {
        let node_id = required_text(NODE_ID_FIELD, draft.node_id)?;
        if node_id != request_node_id {
            return Err(Error::IdempotencyNodeMismatch);
        }
        if draft.command != request_command {
            return Err(Error::IdempotencyCommandMismatch);
        }
        let params_hash = crate::ArtifactHash::new(draft.params_hash)?;
        if &params_hash != request_params_hash {
            return Err(Error::IdempotencyParamsMismatch);
        }
        Ok(Self {
            key_hash: crate::ArtifactHash::new(draft.key_hash)?,
            node_id,
            command: draft.command,
            params_hash,
        })
    }

    /// Returns idempotency key identity.
    #[must_use]
    pub const fn key_hash(&self) -> &crate::ArtifactHash {
        &self.key_hash
    }

    /// Returns bound node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns bound command.
    #[must_use]
    pub const fn command(&self) -> Command {
        self.command
    }

    /// Returns bound params identity.
    #[must_use]
    pub const fn params_hash(&self) -> &crate::ArtifactHash {
        &self.params_hash
    }
}

/// Borrowed canonical node exec plan before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeExecPlanDraft<'a> {
    /// BLAKE3 identity of canonical argv metadata.
    pub argv_hash: &'a str,
    /// Safe current-directory label or hash.
    pub cwd_label: &'a str,
    /// Safe raw-command label or hash.
    pub raw_command_label: &'a str,
    /// Safe agent/session metadata label.
    pub session_label: &'a str,
    /// Safe agent identity label.
    pub agent_label: &'a str,
}

/// Validated canonical node-host exec metadata.
// r[impl agent_harness_runtime.openclaw_nodes.approvals]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeExecPlan {
    argv_hash: crate::ArtifactHash,
    cwd_label: alloc::string::String,
    raw_command_label: alloc::string::String,
    session_label: alloc::string::String,
    agent_label: alloc::string::String,
}

impl NodeExecPlan {
    /// Builds a canonical node exec plan.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when canonical metadata is missing, malformed, or unsafe.
    pub fn new(draft: &NodeExecPlanDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            argv_hash: crate::ArtifactHash::new(draft.argv_hash)?,
            cwd_label: safe_text(CWD_LABEL_FIELD, draft.cwd_label)?,
            raw_command_label: safe_text(RAW_COMMAND_LABEL_FIELD, draft.raw_command_label)?,
            session_label: safe_text(SESSION_LABEL_FIELD, draft.session_label)?,
            agent_label: safe_text(AGENT_LABEL_FIELD, draft.agent_label)?,
        })
    }

    /// Returns argv metadata identity.
    #[must_use]
    pub const fn argv_hash(&self) -> &crate::ArtifactHash {
        &self.argv_hash
    }

    /// Returns safe cwd label.
    #[must_use]
    pub fn cwd_label(&self) -> &str {
        self.cwd_label.as_str()
    }

    /// Returns safe raw-command label.
    #[must_use]
    pub fn raw_command_label(&self) -> &str {
        self.raw_command_label.as_str()
    }

    /// Returns safe session label.
    #[must_use]
    pub fn session_label(&self) -> &str {
        self.session_label.as_str()
    }

    /// Returns safe agent label.
    #[must_use]
    pub fn agent_label(&self) -> &str {
        self.agent_label.as_str()
    }
}

/// Approval status attached to a high-impact node command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalPosture {
    /// Per-node approval is present and current.
    Approved,
    /// Per-node approval was denied or is absent.
    Denied,
}

/// Borrowed per-node approval evidence before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeApprovalDraft<'a> {
    /// Stable approval identity label.
    pub approval_id: &'a str,
    /// Node identity bound to the approval.
    pub node_id: &'a str,
    /// Command bound to the approval.
    pub command: Command,
    /// BLAKE3 identity of active policy posture.
    pub policy_hash: &'a str,
    /// Approval posture.
    pub posture: ApprovalPosture,
    /// Safe approval summary.
    pub summary: &'a str,
}

/// Validated per-node approval evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeApproval {
    approval_id: alloc::string::String,
    node_id: alloc::string::String,
    command: Command,
    policy_hash: crate::ArtifactHash,
    summary: alloc::string::String,
}

impl NodeApproval {
    /// Builds and binds per-node approval evidence.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when approval is denied, unsafe, or bound to another request.
    pub fn new(draft: &NodeApprovalDraft<'_>, request_node_id: &str, request_command: Command) -> Result<Self, Error> {
        if draft.posture != ApprovalPosture::Approved {
            return Err(Error::ApprovalDenied);
        }
        let node_id = required_text(NODE_ID_FIELD, draft.node_id)?;
        if node_id != request_node_id {
            return Err(Error::ApprovalNodeMismatch);
        }
        if draft.command != request_command {
            return Err(Error::ApprovalCommandMismatch);
        }
        Ok(Self {
            approval_id: required_text(APPROVAL_ID_FIELD, draft.approval_id)?,
            node_id,
            command: draft.command,
            policy_hash: crate::ArtifactHash::new(draft.policy_hash)?,
            summary: safe_text(APPROVAL_SUMMARY_FIELD, draft.summary)?,
        })
    }

    /// Returns approval identity.
    #[must_use]
    pub fn approval_id(&self) -> &str {
        self.approval_id.as_str()
    }

    /// Returns bound node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns bound command.
    #[must_use]
    pub const fn command(&self) -> Command {
        self.command
    }

    /// Returns policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::ArtifactHash {
        &self.policy_hash
    }

    /// Returns safe approval summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Borrowed invocation plan inputs before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeInvocationPlanDraft<'a> {
    /// Node advertisement to invoke.
    pub node: &'a NodeAdvertisement,
    /// Optional host pairing descriptor.
    pub pairing: Option<&'a PairingDescriptor>,
    /// Requested node command.
    pub command: Command,
    /// Permission required by the selected command, if any.
    pub required_permission: Option<PermissionClaim>,
    /// BLAKE3 identity of safe parameter metadata.
    pub params_hash: &'a str,
    /// Timeout posture.
    pub timeout: TimeoutPolicy,
    /// Idempotency metadata.
    pub idempotency: NodeIdempotencyDraft<'a>,
    /// Host node service state.
    pub service_state: NodeServiceState,
    /// Cancellation posture.
    pub cancellation: CancellationPosture,
    /// Optional canonical exec metadata.
    pub exec_plan: Option<NodeExecPlanDraft<'a>>,
    /// Optional per-node approval evidence.
    pub approval: Option<NodeApprovalDraft<'a>>,
}

/// Validated node invocation request over injected host services.
// r[impl agent_harness_runtime.openclaw_nodes.invoke]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeInvocationPlan {
    node_id: alloc::string::String,
    command: Command,
    params_hash: crate::ArtifactHash,
    timeout: TimeoutPolicy,
    idempotency: NodeIdempotency,
    cancellation: CancellationPosture,
    exec_plan: Option<NodeExecPlan>,
    approval: Option<NodeApproval>,
}

impl NodeInvocationPlan {
    /// Returns selected node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns selected command.
    #[must_use]
    pub const fn command(&self) -> Command {
        self.command
    }

    /// Returns safe parameter metadata identity.
    #[must_use]
    pub const fn params_hash(&self) -> &crate::ArtifactHash {
        &self.params_hash
    }

    /// Returns timeout posture.
    #[must_use]
    pub const fn timeout(&self) -> TimeoutPolicy {
        self.timeout
    }

    /// Returns idempotency metadata.
    #[must_use]
    pub const fn idempotency(&self) -> &NodeIdempotency {
        &self.idempotency
    }

    /// Returns cancellation posture.
    #[must_use]
    pub const fn cancellation(&self) -> CancellationPosture {
        self.cancellation
    }

    /// Returns optional canonical exec metadata.
    #[must_use]
    pub const fn exec_plan(&self) -> Option<&NodeExecPlan> {
        self.exec_plan.as_ref()
    }

    /// Returns optional per-node approval evidence.
    #[must_use]
    pub const fn approval(&self) -> Option<&NodeApproval> {
        self.approval.as_ref()
    }
}

/// Builds a deterministic node invocation plan before host dispatch.
///
/// # Errors
///
/// Returns [`Error`] when pairing, service availability, command allowlists,
/// permissions, idempotency, canonical exec metadata, or approval evidence fail.
///
/// # Panics
///
/// Panics if post-validation invariants for injected service state or command
/// allowlisting are violated.
pub fn plan_node_invocation(draft: &NodeInvocationPlanDraft<'_>) -> Result<NodeInvocationPlan, Error> {
    validate_node_pairing(draft.node, draft.pairing)?;
    validate_service_state(draft.service_state)?;
    assert_eq!(draft.service_state, NodeServiceState::Injected);
    if !draft.node.commands().contains(&draft.command) {
        return Err(Error::UnsupportedCommand);
    }
    assert!(draft.node.commands().contains(&draft.command));
    if let Some(permission) = draft.required_permission {
        if !draft.node.permissions().contains(&permission) {
            return Err(Error::PermissionMismatch);
        }
    }
    let params_hash = crate::ArtifactHash::new(draft.params_hash)?;
    let idempotency = NodeIdempotency::new(&draft.idempotency, draft.node.node_id(), draft.command, &params_hash)?;
    let exec_plan = match draft.exec_plan {
        Some(exec_plan) => Some(NodeExecPlan::new(&exec_plan)?),
        None if draft.command.is_high_impact() => return Err(Error::MissingExecPlan),
        None => None,
    };
    let approval = match draft.approval {
        Some(approval) => Some(NodeApproval::new(&approval, draft.node.node_id(), draft.command)?),
        None if draft.command.is_high_impact() => return Err(Error::MissingApproval),
        None => None,
    };
    Ok(NodeInvocationPlan {
        node_id: alloc::string::String::from(draft.node.node_id()),
        command: draft.command,
        params_hash,
        timeout: draft.timeout,
        idempotency,
        cancellation: draft.cancellation,
        exec_plan,
        approval,
    })
}

/// Safe terminal node invocation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeResultStatus {
    /// Host reported a completed node command.
    Completed,
    /// Host requires foreground presence.
    ForegroundRequired,
    /// Host permission is missing.
    PermissionMissing,
    /// Host policy denied execution.
    Denied,
    /// Host service or platform is unavailable.
    Unavailable,
    /// Host service timed out.
    TimedOut,
    /// Invocation was cancelled.
    Cancelled,
    /// Host returned malformed output.
    Malformed,
}

impl NodeResultStatus {
    /// Returns the stable result status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::ForegroundRequired => "foreground_required",
            Self::PermissionMissing => "permission_missing",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::Malformed => "malformed",
        }
    }
}

/// Borrowed node result before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeResultDraft<'a> {
    /// Node identity reported by the host service.
    pub node_id: &'a str,
    /// Command reported by the host service.
    pub command: Command,
    /// Safe result status.
    pub status: NodeResultStatus,
    /// Optional BLAKE3 identity of host-held result output.
    pub result_hash: Option<&'a str>,
    /// Safe bounded result summary.
    pub summary: &'a str,
}

/// Safe node result envelope.
// r[impl agent_harness_runtime.openclaw_nodes.invoke]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeResultEnvelope {
    node_id: alloc::string::String,
    command: Command,
    status: NodeResultStatus,
    result_hash: Option<crate::ArtifactHash>,
    summary: alloc::string::String,
}

impl NodeResultEnvelope {
    /// Builds a safe result envelope for a validated plan.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when result identity, command binding, or safe summary validation fails.
    pub fn new(plan: &NodeInvocationPlan, draft: &NodeResultDraft<'_>) -> Result<Self, Error> {
        let node_id = required_text(NODE_ID_FIELD, draft.node_id)?;
        if node_id != plan.node_id() {
            return Err(Error::UnpairedNode);
        }
        if draft.command != plan.command() {
            return Err(Error::UnsupportedCommand);
        }
        if draft.status == NodeResultStatus::Completed && draft.result_hash.is_none() {
            return Err(Error::MalformedResult);
        }
        Ok(Self {
            node_id,
            command: draft.command,
            status: draft.status,
            result_hash: draft.result_hash.map(crate::ArtifactHash::new).transpose()?,
            summary: safe_text(RESULT_SUMMARY_FIELD, draft.summary)?,
        })
    }

    /// Returns node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns command.
    #[must_use]
    pub const fn command(&self) -> Command {
        self.command
    }

    /// Returns result status.
    #[must_use]
    pub const fn status(&self) -> NodeResultStatus {
        self.status
    }

    /// Returns optional result hash.
    #[must_use]
    pub const fn result_hash(&self) -> Option<&crate::ArtifactHash> {
        self.result_hash.as_ref()
    }

    /// Returns safe summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Borrowed safe node invocation receipt before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeInvocationReceiptDraft<'a> {
    /// Node identity.
    pub node_id: &'a str,
    /// Command identity.
    pub command: Command,
    /// Result status.
    pub status: NodeResultStatus,
    /// BLAKE3 identity of host-held evidence.
    pub evidence_hash: &'a str,
    /// Safe receipt summary.
    pub summary: &'a str,
}

/// Safe node invocation receipt metadata.
// r[impl agent_harness_runtime.openclaw_nodes.approvals]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeInvocationReceipt {
    node_id: alloc::string::String,
    command: Command,
    status: NodeResultStatus,
    evidence_hash: crate::ArtifactHash,
    summary: alloc::string::String,
}

impl NodeInvocationReceipt {
    /// Builds safe node invocation receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when identity, evidence, or safe summary validation fails.
    pub fn new(draft: &NodeInvocationReceiptDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            node_id: required_text(NODE_ID_FIELD, draft.node_id)?,
            command: draft.command,
            status: draft.status,
            evidence_hash: crate::ArtifactHash::new(draft.evidence_hash)?,
            summary: safe_text(RECEIPT_SUMMARY_FIELD, draft.summary)?,
        })
    }

    /// Returns node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        self.node_id.as_str()
    }

    /// Returns command.
    #[must_use]
    pub const fn command(&self) -> Command {
        self.command
    }

    /// Returns result status.
    #[must_use]
    pub const fn status(&self) -> NodeResultStatus {
        self.status
    }

    /// Returns evidence identity.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::ArtifactHash {
        &self.evidence_hash
    }

    /// Returns safe receipt summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

fn validate_service_state(state: NodeServiceState) -> Result<(), Error> {
    match state {
        NodeServiceState::Injected => Ok(()),
        NodeServiceState::Missing => Err(Error::MissingService),
        NodeServiceState::Denied => Err(Error::ServiceDenied),
        NodeServiceState::Unavailable => Err(Error::ServiceUnavailable),
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

    const NODE_ID: &str = "node-main";
    const GATEWAY_PROFILE_ID: &str = "openclaw-gateway";
    const PLATFORM_FAMILY: &str = "linux-host";
    const SCOPE_SHELL: &str = "shell-safe";
    const APPROVAL_ID: &str = "approval-main";
    const SAFE_CWD_LABEL: &str = "workspace-hash";
    const SAFE_RAW_COMMAND_LABEL: &str = "command-hash";
    const SAFE_SESSION_LABEL: &str = "session-main";
    const SAFE_AGENT_LABEL: &str = "agent-main";
    const SAFE_SUMMARY: &str = "safe bounded node outcome";
    const UNSAFE_SUMMARY: &str = "CREDENTIAL:raw-token";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const TIMEOUT_MILLIS: u64 = 30_000;
    const COMMANDS: &[Command] = &[Command::SystemRun, Command::MetadataRead];
    const CAPABILITIES: &[Capability] = &[Capability::Shell, Capability::Metadata];
    const PERMISSIONS: &[PermissionClaim] = &[PermissionClaim::Shell, PermissionClaim::Foreground];

    #[test]
    fn advertisement_pairing_invocation_approval_result_and_receipt_accept_safe_dtos() {
        // r[verify agent_harness_runtime.openclaw_nodes.scope]
        // r[verify agent_harness_runtime.openclaw_nodes.advertisement]
        // r[verify agent_harness_runtime.openclaw_nodes.invoke]
        // r[verify agent_harness_runtime.openclaw_nodes.approvals]
        // r[verify agent_harness_runtime.openclaw_nodes.pairing]
        // r[verify agent_harness_runtime.openclaw_nodes.verification]
        let node = advertisement();
        let pairing = pairing();
        let deadline_policy = TimeoutPolicy::new(TIMEOUT_MILLIS).expect("timeout should validate");
        let plan = plan_node_invocation(&NodeInvocationPlanDraft {
            node: &node,
            pairing: Some(&pairing),
            command: Command::SystemRun,
            required_permission: Some(PermissionClaim::Shell),
            params_hash: HASH_C,
            timeout: deadline_policy,
            idempotency: idempotency(Command::SystemRun, HASH_C),
            service_state: NodeServiceState::Injected,
            cancellation: CancellationPosture::Available,
            exec_plan: Some(exec_plan()),
            approval: Some(approval(Command::SystemRun)),
        })
        .expect("node invocation plan should validate");
        let result = NodeResultEnvelope::new(&plan, &NodeResultDraft {
            node_id: NODE_ID,
            command: Command::SystemRun,
            status: NodeResultStatus::Completed,
            result_hash: Some(HASH_D),
            summary: SAFE_SUMMARY,
        })
        .expect("node result should validate");
        let receipt = NodeInvocationReceipt::new(&NodeInvocationReceiptDraft {
            node_id: NODE_ID,
            command: Command::SystemRun,
            status: result.status(),
            evidence_hash: HASH_D,
            summary: SAFE_SUMMARY,
        })
        .expect("node receipt should validate");

        assert_eq!(node.gateway_profile_id(), GATEWAY_PROFILE_ID);
        assert_eq!(pairing.role(), NodeRole::Machine);
        assert_eq!(plan.command(), Command::SystemRun);
        assert!(plan.exec_plan().is_some());
        assert!(plan.approval().is_some());
        assert_eq!(result.status(), NodeResultStatus::Completed);
        assert_eq!(receipt.status(), NodeResultStatus::Completed);
    }

    #[test]
    fn unpaired_drift_unsupported_command_and_permission_mismatch_reject_before_dispatch() {
        // r[verify agent_harness_runtime.openclaw_nodes.advertisement]
        // r[verify agent_harness_runtime.openclaw_nodes.invoke]
        // r[verify agent_harness_runtime.openclaw_nodes.pairing]
        // r[verify agent_harness_runtime.openclaw_nodes.verification]
        let node = advertisement();
        let pairing = pairing();
        let drifted_pairing = PairingDescriptor::new(&PairingDescriptorDraft {
            node_id: NODE_ID,
            role: NodeRole::Machine,
            approved_scopes: &[SCOPE_SHELL],
            command_ceiling: COMMANDS,
            token_identity_class: TokenIdentityClass::OperatorApproved,
            pinned_metadata: MetadataPinDraft {
                platform_family: "different-platform",
                command_ceiling_hash: HASH_A,
                descriptor_hash: HASH_B,
            },
        })
        .expect("pairing with drifted pin should parse");

        assert_eq!(validate_node_pairing(&node, None), Err(Error::UnpairedNode));
        assert_eq!(validate_node_pairing(&node, Some(&drifted_pairing)), Err(Error::MetadataDrift));
        assert_eq!(
            plan_node_invocation(&base_plan(
                &node,
                &pairing,
                Command::ScreenRecord,
                BasePlanOptions::new(Some(PermissionClaim::Screen), NodeServiceState::Injected),
            )),
            Err(Error::UnsupportedCommand)
        );
        assert_eq!(
            plan_node_invocation(&base_plan(
                &node,
                &pairing,
                Command::MetadataRead,
                BasePlanOptions::new(Some(PermissionClaim::Network), NodeServiceState::Injected),
            )),
            Err(Error::PermissionMismatch)
        );
    }

    #[test]
    fn service_approval_and_ambient_fallback_fail_closed() {
        // r[verify agent_harness_runtime.openclaw_nodes.scope]
        // r[verify agent_harness_runtime.openclaw_nodes.invoke]
        // r[verify agent_harness_runtime.openclaw_nodes.approvals]
        // r[verify agent_harness_runtime.openclaw_nodes.verification]
        let node = advertisement();
        let pairing = pairing();
        let missing_service = plan_node_invocation(&base_plan(
            &node,
            &pairing,
            Command::MetadataRead,
            BasePlanOptions::new(None, NodeServiceState::Missing),
        ))
        .expect_err("missing host service should fail closed");
        let missing_exec = plan_node_invocation(&NodeInvocationPlanDraft {
            exec_plan: None,
            approval: Some(approval(Command::SystemRun)),
            ..base_plan(
                &node,
                &pairing,
                Command::SystemRun,
                BasePlanOptions::new(Some(PermissionClaim::Shell), NodeServiceState::Injected),
            )
        })
        .expect_err("system.run without exec plan should fail");
        let missing_approval = plan_node_invocation(&NodeInvocationPlanDraft {
            approval: None,
            ..base_plan(
                &node,
                &pairing,
                Command::SystemRun,
                BasePlanOptions::new(Some(PermissionClaim::Shell), NodeServiceState::Injected),
            )
        })
        .expect_err("system.run without approval should fail");
        let ambient_descriptor = NodeAdvertisement::new(&NodeAdvertisementDraft {
            ambient_execution_fallback: true,
            ..advertisement_draft()
        })
        .expect_err("ambient execution fallback should fail");

        assert_eq!(missing_service, Error::MissingService);
        assert_eq!(missing_exec, Error::MissingExecPlan);
        assert_eq!(missing_approval, Error::MissingApproval);
        assert_eq!(ambient_descriptor, Error::AmbientExecutionFallback);
    }

    #[test]
    fn malformed_result_and_receipt_leak_fail_closed() {
        // r[verify agent_harness_runtime.openclaw_nodes.invoke]
        // r[verify agent_harness_runtime.openclaw_nodes.verification]
        let node = advertisement();
        let pairing = pairing();
        let safe_plan = plan_node_invocation(&base_plan(
            &node,
            &pairing,
            Command::SystemRun,
            BasePlanOptions::new(Some(PermissionClaim::Shell), NodeServiceState::Injected),
        ))
        .expect("system.run should plan");
        let malformed_result = NodeResultEnvelope::new(&safe_plan, &NodeResultDraft {
            node_id: NODE_ID,
            command: Command::SystemRun,
            status: NodeResultStatus::Completed,
            result_hash: None,
            summary: SAFE_SUMMARY,
        })
        .expect_err("completed result without hash should be malformed");
        let unsafe_receipt = NodeInvocationReceipt::new(&NodeInvocationReceiptDraft {
            node_id: NODE_ID,
            command: Command::SystemRun,
            status: NodeResultStatus::TimedOut,
            evidence_hash: HASH_D,
            summary: UNSAFE_SUMMARY,
        })
        .expect_err("receipt leak should fail");

        assert_eq!(malformed_result, Error::MalformedResult);
        assert!(matches!(unsafe_receipt, Error::UnsafePayload { .. }));
    }

    fn advertisement() -> NodeAdvertisement {
        NodeAdvertisement::new(&advertisement_draft()).expect("advertisement should validate")
    }

    const fn advertisement_draft() -> NodeAdvertisementDraft<'static> {
        NodeAdvertisementDraft {
            node_id: NODE_ID,
            gateway_profile_id: GATEWAY_PROFILE_ID,
            platform_family: PLATFORM_FAMILY,
            capabilities: CAPABILITIES,
            commands: COMMANDS,
            permissions: PERMISSIONS,
            metadata_pin: Some(metadata_pin()),
            gateway_owned_claims: &[],
            ambient_execution_fallback: false,
        }
    }

    fn pairing() -> PairingDescriptor {
        PairingDescriptor::new(&PairingDescriptorDraft {
            node_id: NODE_ID,
            role: NodeRole::Machine,
            approved_scopes: &[SCOPE_SHELL],
            command_ceiling: COMMANDS,
            token_identity_class: TokenIdentityClass::OperatorApproved,
            pinned_metadata: metadata_pin(),
        })
        .expect("pairing should validate")
    }

    const fn metadata_pin() -> MetadataPinDraft<'static> {
        MetadataPinDraft {
            platform_family: PLATFORM_FAMILY,
            command_ceiling_hash: HASH_A,
            descriptor_hash: HASH_B,
        }
    }

    const fn exec_plan() -> NodeExecPlanDraft<'static> {
        NodeExecPlanDraft {
            argv_hash: HASH_A,
            cwd_label: SAFE_CWD_LABEL,
            raw_command_label: SAFE_RAW_COMMAND_LABEL,
            session_label: SAFE_SESSION_LABEL,
            agent_label: SAFE_AGENT_LABEL,
        }
    }

    const fn approval(command: Command) -> NodeApprovalDraft<'static> {
        NodeApprovalDraft {
            approval_id: APPROVAL_ID,
            node_id: NODE_ID,
            command,
            policy_hash: HASH_B,
            posture: ApprovalPosture::Approved,
            summary: SAFE_SUMMARY,
        }
    }

    const fn idempotency(command: Command, params_hash: &'static str) -> NodeIdempotencyDraft<'static> {
        NodeIdempotencyDraft {
            key_hash: HASH_A,
            node_id: NODE_ID,
            command,
            params_hash,
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct BasePlanOptions {
        required_permission: Option<PermissionClaim>,
        service_state: NodeServiceState,
    }

    impl BasePlanOptions {
        const fn new(required_permission: Option<PermissionClaim>, service_state: NodeServiceState) -> Self {
            Self {
                required_permission,
                service_state,
            }
        }
    }

    fn base_plan<'a>(
        node: &'a NodeAdvertisement,
        pairing: &'a PairingDescriptor,
        command: Command,
        options: BasePlanOptions,
    ) -> NodeInvocationPlanDraft<'a> {
        NodeInvocationPlanDraft {
            node,
            pairing: Some(pairing),
            command,
            required_permission: options.required_permission,
            params_hash: HASH_C,
            timeout: TimeoutPolicy::new(TIMEOUT_MILLIS).expect("timeout should validate"),
            idempotency: idempotency(command, HASH_C),
            service_state: options.service_state,
            cancellation: CancellationPosture::Available,
            exec_plan: Some(exec_plan()),
            approval: Some(approval(command)),
        }
    }
}
