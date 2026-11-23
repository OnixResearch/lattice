//! Read-only injected serving-owner port and the per-request serving gate.
//!
//! Lattice runs Animus's pure `animus_adapters::serving` screen and dispatch
//! gate itself, inside the attempt-local model broker, for every request on a
//! route the execution binding declares as served. Serving facts arrive only
//! through [`OwnerPort`]: an availability snapshot, a request-scoped
//! serving-owner admission and a current recheck. Lattice never reserves,
//! loads, drains, releases or probes serving capacity. The standalone CLI
//! installs [`UnavailableOwner`], so a served route blocks before any upstream
//! effect. An embedding host (for example a Kiln #49 composition) supplies a
//! live adapter through `implementation_attempt::cli::run_command_with_ports`.

// r[impl lattice.serving_binding.port_boundary]
// r[impl lattice.serving_binding.producer]

mod draft;
pub mod gate;
mod port;
pub mod readiness;
mod receipt;

type Intent = lattice_core::execution_binding::serving::record::DispatchIntent;
type Record = lattice_core::execution_binding::serving::record::Observation;

/// Read-only serving-owner observations and request-scoped admission.
pub trait OwnerPort: Send {
    /// The newest availability snapshot covering the declared service.
    fn observe_snapshot(&mut self, query: &SnapshotQuery) -> SnapshotReply;
    /// Ask the serving owner to admit exactly this request. A grant is the
    /// owner's decision; Lattice holds no capacity.
    fn admit_request(
        &mut self,
        query: &animus_adapters::serving::AdmissionQuery,
    ) -> animus_adapters::serving::OwnerReply;
    /// Current serving state for the admitted request, read immediately
    /// before dispatch.
    fn recheck(&mut self, query: &animus_adapters::serving::AdmissionQuery) -> Current;
    /// Where this port's facts come from; only an embedder-supplied port can
    /// back a `live_endpoint` record, and only for a dispatched request.
    fn provenance(&self) -> PortProvenance {
        PortProvenance::Embedder
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortProvenance {
    Fixture,
    Unavailable,
    Embedder,
}

/// The declared route and service one snapshot must cover.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SnapshotQuery {
    pub attempt_id: String,
    pub route_name: String,
    pub service_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotReply {
    /// A producer-validated snapshot and the port-supplied logical time.
    Observed {
        snapshot: Box<animus_adapters::serving::AvailabilitySnapshot>,
        now_tick: Option<u64>,
    },
    Unavailable,
    PortIssue,
}

/// Owned form of the producer's borrowed `CurrentState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentFacts {
    pub service_id: String,
    pub endpoint_identity: String,
    pub deployment_id: String,
    pub deployment_generation: u64,
    pub model_id: String,
    pub configuration_ref: String,
    pub readiness: animus_adapters::serving::Readiness,
    pub admission_epoch: u64,
    pub now_tick: Option<u64>,
    pub policy_generation: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Current {
    Observed(Box<CurrentFacts>),
    Unavailable,
    PortIssue,
}

/// Durable, write-once request evidence of one attempt.
pub trait Sink: Send {
    /// Create and persist the dispatch intent before upstream contact. An
    /// intent that already exists for its request, even with identical
    /// bytes, MUST be refused: each request reaches upstream at most once.
    fn create_intent(&mut self, intent: &Intent) -> Result<(), String>;
    /// Persist the request observation after refusal or the upstream call;
    /// an identical replay is accepted, a different one refused.
    fn write_record(&mut self, record: &Record) -> Result<(), String>;
    /// The highest ordinal any retained intent or record holds on
    /// `route_name`, or `0` without one. A session continues after it.
    fn highest_ordinal(&self, route_name: &str) -> Result<u64, String>;
}

/// The standalone CLI's port: every serving fact is unavailable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnavailableOwner;

/// One recorded port call of [`FixtureOwner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Snapshot(SnapshotQuery),
    Admit { request_id: String },
    Recheck { request_id: String },
}

/// One scripted serving-owner answer of [`FixtureOwner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerScript {
    /// Return this reply verbatim, for example a grant for another request.
    Reply(animus_adapters::serving::OwnerReply),
    /// Grant exactly the queried request, deployment, model and replica.
    GrantQueried {
        grant_ref: String,
        admission_epoch: u64,
        issued_tick: u64,
        valid_until_tick: u64,
    },
}

/// Deterministic port that replays scripted replies in order and records
/// every call. An exhausted script answers unavailable. Fixture evidence only.
#[derive(Debug, Default)]
pub struct FixtureOwner {
    snapshots: std::collections::VecDeque<SnapshotReply>,
    grants: std::collections::VecDeque<OwnerScript>,
    rechecks: std::collections::VecDeque<Current>,
    calls: Vec<Call>,
}

/// In-memory sink for deterministic fixtures.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemorySink {
    pub intents: Vec<Intent>,
    pub records: Vec<Record>,
}
