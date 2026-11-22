//! Wire records of `lattice.vm-execution-profile.v1` (C1) and
//! `lattice.vm-attempt-binding.v1` (C2). Every record denies unknown fields;
//! values a profile may carry but this revision refuses (another guest
//! network, an unknown device, `live_precopy`) stay representable so that
//! admission names them with a typed refusal instead of a parse error.

pub const PROFILE_SCHEMA: &str = "lattice.vm-execution-profile.v1";
pub const BINDING_SCHEMA: &str = "lattice.vm-attempt-binding.v1";
/// Kiln's guest image manifest schema the profile is admitted for.
pub const GUEST_MANIFEST_SCHEMA: &str = "kiln.lattice-worker-guest.v1";
/// The guest workflow exchange transport (S2's C5).
pub const EXCHANGE_TRANSPORT_SCHEMA: &str = "lattice.workflow-vsock-transport-profile.v1";
/// The only admitted guest network: no NIC.
pub const GUEST_NETWORK_NONE: &str = "none";
/// The only admitted deadline authority (D4).
pub const DEADLINE_AUTHORITY_HOST: &str = "host";
/// `VMADDR_CID_HOST`: the guest reaches its host gateways only here.
pub const HOST_PEER_CID: u32 = 2;
/// The admitted device set, in canonical order.
pub const ADMITTED_DEVICES: [&str; 2] = ["virtio-blk-raw", "vsock"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Architecture {
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "aarch64")]
    Aarch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VmmKind {
    #[serde(rename = "cloud-hypervisor")]
    CloudHypervisor,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vmm {
    pub kind: VmmKind,
    pub version: String,
}

/// One AF_VSOCK port, serialized as its bare number.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(transparent)]
pub struct VsockPort(pub u32);

impl VsockPort {
    /// `VMADDR_PORT_ANY`; never a concrete gateway port.
    pub const ANY: Self = Self(u32::MAX);

    /// A port a listener can bind and a guest can name: neither 0 nor `ANY`.
    #[must_use]
    pub const fn is_concrete(self) -> bool {
        self.0 != 0 && self.0 != Self::ANY.0
    }
}

/// Host gateway endpoints the guest reaches over vsock.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gateway {
    pub exchange_vsock_port: VsockPort,
    pub model_vsock_port: VsockPort,
    pub host_peer_cid: u32,
    pub exchange_transport_schema: String,
}

/// One mobility mode. `LivePrecopy` is representable and refused at this
/// revision, mirroring Kiln's `require_live_capability`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum Mobility {
    Pinned,
    StoppedTransfer,
    LivePrecopy,
    ApplicationCheckpoint,
}

/// The application checkpoint profile a guest workload offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Checkpoint {
    #[serde(rename = "none")]
    NotOffered,
    #[serde(rename = "workflow-node-boundary-v1")]
    WorkflowNodeBoundaryV1,
}

impl Checkpoint {
    /// The wire token, also the inventory `checkpoint` fact when offered.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::NotOffered => "none",
            Self::WorkflowNodeBoundaryV1 => "workflow-node-boundary-v1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub vcpus: u16,
    pub memory_mib: u32,
    pub workspace_mib: u32,
}

/// C1: the immutable VM execution profile. Authored in Nickel and evaluated
/// to JSON before identity.
#[allow(unknown_lints)]
#[allow(
    path_segment_repetition,
    reason = "the relocation contract names C1 the VM execution profile; callers spell `vm_profile::ExecutionProfile`"
)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionProfile {
    pub schema: String,
    pub architecture: Architecture,
    pub vmm: Vmm,
    pub guest_manifest_schema: String,
    pub guest_network: String,
    pub gateway: Gateway,
    pub mobility: Vec<Mobility>,
    pub checkpoint: Checkpoint,
    pub deadline_authority: String,
    pub resources: Resources,
    pub devices: Vec<String>,
    pub non_claims: Vec<String>,
}

/// Kiln's workspace volume binding (`WorkspaceBinding`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceBinding {
    pub identity: String,
    pub generation: u64,
}

/// C2: one attempt bound to one admitted profile, guest image, environment,
/// envelope, execution and workspace. It names no placement and no host
/// endpoint, so a relocation keeps its identity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptBinding {
    pub schema: String,
    /// `b3:` identity of the admitted C1 profile.
    pub profile: String,
    /// Kiln guest manifest identity (64 lowercase hex).
    pub guest_image: String,
    /// Kiln guest environment identity (64 lowercase hex).
    pub environment: String,
    /// `ImplementationAttemptEnvelope::identity()` (64 lowercase hex).
    pub envelope: String,
    pub attempt_id: String,
    pub fence: u64,
    /// Kiln `AttemptRequest.execution` (64 lowercase hex).
    pub execution: String,
    pub workspace: WorkspaceBinding,
    pub requested_mobility: Mobility,
    /// The original admitted absolute wall deadline; never extended.
    pub deadline_unix_ms: u64,
}

/// Kiln's drain disposition a requested mobility mode maps to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrainDisposition {
    Pinned,
    Migrated,
    Checkpointed,
}

/// Why a profile, a binding or a requested mode is refused before any effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refusal {
    UnsupportedSchema,
    UnsupportedVmm,
    UnsupportedGuestManifest,
    GuestNetworkNotNone,
    UnsupportedExchangeTransport,
    HostPeerCid,
    InvalidGatewayPorts,
    InvalidMobility,
    LivePrecopyUnqualified,
    CheckpointRequired,
    DeadlineAuthorityNotHost,
    InvalidResources,
    UnsupportedDevice,
    InvalidNonClaims,
    MalformedBinding,
    CrossedProfile,
    CrossedGuestImage,
    CrossedEnvironment,
    CrossedEnvelope,
    CrossedAttempt,
    CrossedFence,
    CrossedExecution,
    CrossedWorkspace,
    CrossedDeadline,
    MobilityNotOffered,
    Identity,
}

impl Refusal {
    /// The stable snake-case code, as serialized.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedSchema => "unsupported_schema",
            Self::UnsupportedVmm => "unsupported_vmm",
            Self::UnsupportedGuestManifest => "unsupported_guest_manifest",
            Self::GuestNetworkNotNone => "guest_network_not_none",
            Self::UnsupportedExchangeTransport => "unsupported_exchange_transport",
            Self::HostPeerCid => "host_peer_cid",
            Self::InvalidGatewayPorts => "invalid_gateway_ports",
            Self::InvalidMobility => "invalid_mobility",
            Self::LivePrecopyUnqualified => "live_precopy_unqualified",
            Self::CheckpointRequired => "checkpoint_required",
            Self::DeadlineAuthorityNotHost => "deadline_authority_not_host",
            Self::InvalidResources => "invalid_resources",
            Self::UnsupportedDevice => "unsupported_device",
            Self::InvalidNonClaims => "invalid_non_claims",
            Self::MalformedBinding => "malformed_binding",
            Self::CrossedProfile => "crossed_profile",
            Self::CrossedGuestImage => "crossed_guest_image",
            Self::CrossedEnvironment => "crossed_environment",
            Self::CrossedEnvelope => "crossed_envelope",
            Self::CrossedAttempt => "crossed_attempt",
            Self::CrossedFence => "crossed_fence",
            Self::CrossedExecution => "crossed_execution",
            Self::CrossedWorkspace => "crossed_workspace",
            Self::CrossedDeadline => "crossed_deadline",
            Self::MobilityNotOffered => "mobility_not_offered",
            Self::Identity => "identity",
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "VM execution profile or attempt binding refused: {}", self.code())
    }
}

impl std::error::Error for Refusal {}

/// The values a verifier knows independently of the binding. The guest
/// shell knows the profile and envelope; Kiln and the host gateway also know
/// image, environment, execution, workspace and deadline. An absent
/// expectation is not checked by that verifier.
#[derive(Debug, Clone, Copy)]
pub struct Expected<'a> {
    pub profile: &'a ExecutionProfile,
    pub envelope: Option<&'a str>,
    pub attempt_id: Option<&'a str>,
    pub fence: Option<u64>,
    pub guest_image: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub execution: Option<&'a str>,
    pub workspace: Option<&'a WorkspaceBinding>,
    pub deadline_unix_ms: Option<u64>,
}

/// An admitted binding and the identities every later check compares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBinding {
    pub profile_identity: String,
    pub binding_identity: String,
    pub disposition: DrainDisposition,
}
