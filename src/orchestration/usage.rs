//! Versioned command-worker usage observations.
//!
//! Reservations, ceilings, estimates, provider reports, host measurements, and
//! unknown observations remain distinct. This module does not price models or
//! schedule work.

use thiserror::Error;

mod validation;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod host_tests;

pub const USAGE_OBSERVATION_SCHEMA: &str = "lattice.command-worker-usage.v1";
pub const MAX_IDENTIFIER_BYTES: usize = 256;
pub const MAX_USAGE_OBSERVATION_BYTES: usize = 1_048_576;
pub(crate) const MAX_PROVIDER_REPORTS: usize = 4096;
const MAX_ACCOUNTING_DIAGNOSTIC_BYTES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageProvenance {
    HostMeasured,
    ProviderReported,
    WorkerReported,
    Estimated,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Run,
    Node,
    /// Host-owned run receipt; does not assert a qualified worker profile.
    HostRun,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandWorkerObservation {
    pub schema: String,
    pub scope: Scope,
    pub workflow_revision: String,
    pub instance_id: String,
    pub dispatch_id: String,
    pub run_id: u64,
    pub node_id: u64,
    pub invocation_id: String,
    pub worker_profile: String,
    pub attempt_id: String,
    pub agent_invocations: Option<u64>,
    pub model_requests: Option<u64>,
    pub subagent_calls: Option<u64>,
    pub tokens: Option<u64>,
    pub cost_microunits: Option<u64>,
    pub duration_millis: Option<u64>,
    pub provenance: UsageProvenance,
    pub complete: bool,
    #[serde(default = "absent_accounting", skip_serializing_if = "Option::is_none")]
    pub accounting: Option<RunAccounting>,
}

/// Frozen usage settlement with monotonically refinable owned outcome evidence.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunAccounting {
    pub schema: String,
    pub reserved: super::core::BudgetCharge,
    pub charged: super::core::BudgetCharge,
    pub host_interval: Option<HostInterval>,
    pub provider_reports: Vec<ProviderReport>,
    pub unknown_tokens: bool,
    pub unknown_cost: bool,
    pub unknown_model_requests: bool,
    pub unknown_subagent_calls: bool,
    pub diagnostic: String,
    #[serde(default = "absent_outcome", skip_serializing_if = "Option::is_none")]
    pub outcome: Option<RunOutcomeObservation>,
}

fn absent_accounting() -> Option<RunAccounting> {
    None
}

fn absent_outcome() -> Option<RunOutcomeObservation> {
    None
}

impl RunAccounting {
    /// Add cancellation evidence without changing an already observed outcome.
    pub(crate) fn refine_outcome(&mut self, incoming: RunOutcomeObservation) -> Result<(), ObservationError> {
        if let Some(current) = self.outcome {
            if current.succeeded != incoming.succeeded
                || (current.cancelled.is_some()
                    && incoming.cancelled.is_some()
                    && current.cancelled != incoming.cancelled)
            {
                return Err(ObservationError::ConflictingOutcome);
            }
            if current.cancelled.is_some() {
                return Ok(());
            }
        }
        self.outcome = Some(incoming);
        Ok(())
    }
}

/// None is absent cancellation telemetry, not observed non-cancellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunOutcomeObservation {
    pub succeeded: bool,
    pub cancelled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostInterval {
    pub started_epoch_ms: i64,
    pub finished_epoch_ms: i64,
}

impl HostInterval {
    pub fn duration_millis(&self) -> Option<u64> {
        u64::try_from(self.finished_epoch_ms.checked_sub(self.started_epoch_ms)?).ok()
    }
}

/// Provider values are reports, not measurements or authoritative invoices.
/// Missing service/request/weight identities are not invented from a model name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderReport {
    pub node_id: u64,
    pub provider: String,
    pub model: Option<String>,
    pub tokens: Option<u64>,
    /// Currency/pricing identity is absent from the current adapter protocol.
    /// Retained for diagnostics only; cannot lower a monetary reservation.
    pub cost_microunits: Option<u64>,
}

pub const RUN_USAGE_ACCOUNTING_SCHEMA: &str = "lattice.host-run-accounting.v1";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ObservationError {
    #[error("usage observation schema is not supported")]
    InvalidSchema,
    #[error("usage observation identifier is empty, unbounded, or contains control characters")]
    InvalidIdentifier,
    #[error("unknown usage cannot carry values or claim completeness")]
    UnknownCarriesValues,
    #[error("complete usage observation must carry at least one value")]
    EmptyCompleteObservation,
    #[error("host accounting receipt has invalid scope, interval, bounds, or fallback charge")]
    InvalidHostReceipt,
    #[error("owned run outcome contradicts previously persisted evidence")]
    ConflictingOutcome,
    #[error("usage observation JSON is malformed: {0}")]
    InvalidJson(String),
}
