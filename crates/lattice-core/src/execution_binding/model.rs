//! `lattice.compute-allocation-observation.v1`: the Lattice-owned consumer DTO.
//!
//! Field names are snake_case and follow the producer field order of Aspen
//! `aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4`. Enum values keep the producer's
//! kebab-case spellings exactly. Nothing is defaulted or inferred.

// r[impl lattice.execution_binding.allocation_observation]
// r[impl lattice.execution_binding.allocation_observation.scenario.admitted]

pub const OBSERVATION_SCHEMA: &str = "lattice.compute-allocation-observation.v1";
// r[impl lattice.execution_binding.deferred.scenario.live]
/// Aspen revision whose published records this DTO mirrors.
pub const PRODUCER_REVISION: &str = "aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4";
pub const ALLOCATION_LABEL: &str = "compute-capacity-allocation-v2";
pub const READBACK_LABEL: &str = "compute-capacity-readback-v1";
pub const OFFER_LABEL: &str = "compute-capacity-offer-v2";
pub const RESERVATION_SCHEMA: &str = "molten.compute.reservation.v1";
pub const OFFER_SCHEMA: &str = "molten.compute.offer.v1";
pub const TIME_DOMAIN: &str = "logical";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputeAllocationObservation {
    pub schema: String,
    pub producer: ProducerCohort,
    pub port: PortProvenance,
    pub allocation: AllocationRecord,
    pub readback: ReadbackProjection,
    pub offer: Option<OfferPoolFacts>,
}

/// Exact producer labels; any other value is rejected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerCohort {
    pub allocation_label: String,
    pub readback_label: String,
    pub offer_label: String,
    pub reservation_schema: String,
    pub offer_schema: String,
    pub time_domain: String,
}

/// Embedding-host adapter identity. Authenticity of the projection is
/// attributed to this port, not recomputed by Lattice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortProvenance {
    pub port_id: String,
    pub port_profile_ref: String,
    pub observation_sequence: u64,
    /// Port-supplied producer tick. `None` keeps expiry and freshness unknown.
    pub provider_logical_now: Option<u64>,
}

/// `compute-capacity-allocation-v2`: all 24 producer fields in producer order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationRecord {
    pub record_ref: String,
    pub schema: String,
    pub id: String,
    pub host_id: String,
    pub host_incarnation: String,
    pub device_incarnation: String,
    pub offer_revision: u64,
    pub topology_generation: u64,
    pub charge_cursor: u64,
    pub charge_at: u64,
    pub valid_until: u64,
    pub authority_ids: Vec<String>,
    pub view_id: String,
    pub granularity: Granularity,
    pub consumer_id: String,
    pub operation_id: String,
    pub attempt_id: String,
    pub owner_generation: u64,
    /// A reference only; never a grant.
    pub grant_ref: String,
    pub claims: Vec<Claim>,
    pub serving_allocation_id: Option<String>,
    pub kind: AllocationKind,
    pub status: ReservationStatus,
    pub enforcement: Enforcement,
    /// Persisted with the admission observation only; never projected.
    pub activation_backend_ref: Option<String>,
}

/// `compute-claim-v1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub pool_id: String,
    pub quantity: u64,
}

/// `compute-capacity-readback-v1` host fields and the one matching row.
/// Other consumers' rows are counted but not retained.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadbackProjection {
    pub record_ref: String,
    pub host_id: String,
    pub revision: u64,
    pub is_fresh: bool,
    pub health: Health,
    /// Qualification scope only, not live qualification.
    pub qualification_refs: Vec<String>,
    pub row_count: u64,
    pub row: ReadbackRow,
}

/// `compute-allocation-readback-v1` fields 0-6.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadbackRow {
    pub id: String,
    pub consumer_id: String,
    pub owner_generation: u64,
    pub granularity: Granularity,
    pub status: ReservationStatus,
    pub is_logically_fenced: bool,
    pub enforcement: Enforcement,
}

/// Claimed-pool facts of `compute-capacity-offer-v2` at the allocation's
/// offer revision. Optional: the producer may not retain that offer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferPoolFacts {
    pub record_ref: String,
    pub host_id: String,
    pub host_incarnation: String,
    pub device_incarnation: String,
    pub revision: u64,
    pub topology_generation: u64,
    pub health: Health,
    pub label_authority: LabelAuthority,
    pub compatibility: OfferCompatibility,
    pub claimed_pools: Vec<ClaimedPool>,
}

/// `compute-compatibility-v1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferCompatibility {
    pub backend: Option<String>,
    pub runtime: Option<String>,
    pub architecture: Option<String>,
    pub topology: Option<String>,
    pub qualification_refs: Vec<String>,
}

/// `compute-pool-v1` fields 0-2 for one claimed pool.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimedPool {
    pub pool_id: String,
    pub kind: PoolKind,
    pub authority_id: String,
}

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
#[serde(rename_all = "kebab-case")]
pub enum Granularity {
    Chip,
    MmioBundle,
    Tray,
    Mesh,
    Partition,
    Alias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AllocationKind {
    Execution,
    ServingDeployment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReservationStatus {
    Intent,
    Active,
    Quarantined,
    Released,
}

/// Producer order: `logical-fence-only` < `backend-acknowledged` < `backend-fenced`.
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
#[serde(rename_all = "kebab-case")]
pub enum Enforcement {
    LogicalFenceOnly,
    BackendAcknowledged,
    BackendFenced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Health {
    Healthy,
    Degraded,
    Unavailable,
    Unknown,
}

/// Producer order: `advisory` < `measured` < `operator-declared` < `authoritative`.
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
#[serde(rename_all = "kebab-case")]
pub enum LabelAuthority {
    Advisory,
    Measured,
    OperatorDeclared,
    Authoritative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PoolKind {
    DedicatedMemoryBytes,
    UnifiedMemoryBytes,
    Chips,
}

impl Granularity {
    /// Producer spelling used by identity framing.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Chip => "chip",
            Self::MmioBundle => "mmio-bundle",
            Self::Tray => "tray",
            Self::Mesh => "mesh",
            Self::Partition => "partition",
            Self::Alias => "alias",
        }
    }
}

impl AllocationKind {
    /// Producer spelling used by identity framing.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Execution => "execution",
            Self::ServingDeployment => "serving-deployment",
        }
    }
}
