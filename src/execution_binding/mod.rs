//! Read-only injected compute port for worker execution binding.
//!
//! Lattice obtains provider compute state only through
//! [`ComputeAllocationPort::observe_allocation`]. The trait has no reserve,
//! activate, drain, release, retire or probe method. The standalone CLI installs
//! [`UnavailableComputePort`], so a request that needs a provider allocation
//! fails closed before effects. An embedding host (for example a Kiln #49 host
//! composition or an Aspen client) supplies a real adapter through
//! `implementation_attempt::cli::run_command_with_ports`. Lattice adds no
//! Aspen crate, Preserves codec or sibling-path dependency for this boundary.

// r[impl lattice.execution_binding.port_boundary]
// r[impl lattice.execution_binding.port_boundary.scenario.read_only]

pub mod format;
mod port;
pub mod serving;

/// The pure decision core, re-exported from `lattice-core`.
pub mod core {
    pub use lattice_core::execution_binding::*;
}

/// Most bytes of provider text a port issue may carry.
pub(crate) const MAX_ISSUE_DETAIL_BYTES: usize = 256;

/// Read-only observation of one declared allocation.
pub trait ComputeAllocationPort: Send {
    /// Observe the allocation. Never reserves, activates, releases or probes.
    fn observe_allocation(&mut self, query: &AllocationQuery) -> Result<AllocationReply, ComputePortIssue>;
}

/// The dispatcher-declared allocation and the admitted Lattice attempt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AllocationQuery {
    pub allocation_id: String,
    pub host_id: String,
    pub consumer_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationReply {
    Observed(Box<lattice_core::execution_binding::model::ComputeAllocationObservation>),
    NotFound,
    Unavailable { reason: PortUnavailableReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortUnavailableReason {
    ProviderNotConfigured,
    ProviderReportedUnavailable,
}

/// A transport-level failure. It never carries more than 256 bytes of
/// provider text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputePortIssue {
    pub kind: PortIssueKind,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortIssueKind {
    Transport,
    Timeout,
    MalformedReply,
}

/// The standalone CLI's port: every observation is unavailable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnavailableComputePort;

/// Deterministic fixture port that replays scripted replies in order and
/// records every query. It is synthetic and never live evidence.
#[derive(Debug, Default)]
pub struct FixtureComputePort {
    replies: std::collections::VecDeque<Result<AllocationReply, ComputePortIssue>>,
    calls: Vec<AllocationQuery>,
}

/// Map one port result to the pure core's reading. `NotFound` rejects;
/// `Unavailable` and every transport issue block.
#[must_use]
pub fn reading(
    reply: &Result<AllocationReply, ComputePortIssue>,
) -> lattice_core::execution_binding::admission::ProviderReading<'_> {
    match reply {
        Ok(AllocationReply::Observed(observation)) => {
            lattice_core::execution_binding::admission::ProviderReading::Observed(observation)
        }
        Ok(AllocationReply::NotFound) => lattice_core::execution_binding::admission::ProviderReading::NotFound,
        Ok(AllocationReply::Unavailable { .. }) => {
            lattice_core::execution_binding::admission::ProviderReading::Unavailable
        }
        Err(_) => lattice_core::execution_binding::admission::ProviderReading::PortIssue,
    }
}
