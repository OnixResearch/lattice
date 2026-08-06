type SimulationIdentityKind = super::nomenclature::SimulationIdentityKind;
type SimulationIdentityRegistry = super::nomenclature::SimulationIdentityRegistry;
type SimulationInvariantId = super::nomenclature::SimulationInvariantId;

const MAX_ORACLES: usize = 64;
const MAX_ORACLE_DETAIL_BYTES: usize = 96;

/// Provider-neutral invariant aggregation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimulationOracleCategory {
    /// Must pass every time it is observed.
    Always,
    /// Must be observed passing at least once.
    Sometimes,
    /// Named point must be reached at least once.
    Reachable,
    /// Named point must never be reached.
    Unreachable,
}

impl SimulationOracleCategory {
    /// Returns the stable category label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Sometimes => "sometimes",
            Self::Reachable => "reachable",
            Self::Unreachable => "unreachable",
        }
    }
}

/// One cataloged pure oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationOracleCatalogEntry {
    /// Registered invariant identity.
    pub invariant_id: SimulationInvariantId,
    /// Aggregation category.
    pub category: SimulationOracleCategory,
    /// Bounded safe semantic detail.
    pub detail: String,
}

/// One pure observed invariant fact.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationOracleObservation {
    /// Registered invariant identity.
    pub invariant_id: SimulationInvariantId,
    /// Aggregation category, which must match the catalog.
    pub category: SimulationOracleCategory,
    /// Observed condition. Reachable uses `true`; unreachable uses `false`.
    pub condition: bool,
    /// Optional finite non-negative distance to violation.
    pub guidance: Option<f64>,
    /// Bounded safe semantic detail.
    pub detail: String,
}

/// Stable oracle diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationOracleDiagnosticCode {
    /// Oracle catalog exceeds its bound.
    CatalogTooLarge,
    /// Oracle observation history exceeds its bound.
    ObservationHistoryTooLarge,
    /// Invariant identity is unregistered or absent from the catalog.
    UnknownInvariant,
    /// Catalog repeats an invariant.
    DuplicateInvariant,
    /// Observation category differs from catalog.
    CategoryMismatch,
    /// Detail is oversized or contains sensitive/unstable data.
    InvalidDetail,
    /// Guidance is negative, NaN, or infinite.
    InvalidGuidance,
}

/// Stable oracle diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationOracleDiagnostic {
    /// Machine-comparable issue code.
    pub code: SimulationOracleDiagnosticCode,
    /// Safe invariant identity when available.
    pub invariant_id: Option<String>,
}

/// Aggregated status for one cataloged oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationOracleStatus {
    /// Every observed `always` fact passed.
    AlwaysPassed,
    /// An `always` fact failed.
    AlwaysFailed,
    /// A `sometimes` fact was observed true.
    SometimesObserved,
    /// A `sometimes` fact was observed only false.
    SometimesUnobservedSuccess,
    /// A reachable point was reached.
    Reached,
    /// A reachable point was not reached.
    Unreached,
    /// An unreachable point remained unobserved.
    UnreachablePreserved,
    /// An unreachable point was reached.
    UnreachableViolated,
    /// Cataloged invariant had no observations.
    CatalogedButUnobserved,
}

/// Summary row for one cataloged oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationOracleSummary {
    /// Stable invariant identity.
    pub invariant_id: SimulationInvariantId,
    /// Aggregation category.
    pub category: SimulationOracleCategory,
    /// Aggregated status.
    pub status: SimulationOracleStatus,
    /// Number of observed facts.
    pub observations: u32,
}
