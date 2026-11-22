//! The typed retained records of one repair run, and what they prove about
//! its workers.
//!
//! The shell writes `repair/step-N.json` and `repair/disposition.json` from
//! these types and reads them back into them: an unknown field, a missing
//! field or a nullable field left out is refused. Nothing here is trusted
//! alone; export and inventory re-derive what they report from the whole
//! record set.

// r[impl lattice.implementation_attempt.repair_export.scenario.inventory]

#[cfg(test)]
mod tests;

/// The disposition record. Version 2 adds check-copy integrity, execute bits
/// and changed-path base digests.
pub const OBSERVATION_SCHEMA: &str = "lattice.implementation-attempt-repair-observation.v2";
/// The first published disposition record. Recovery still prints it; export
/// and inventory never act on it, because it lacks that evidence.
pub const LEGACY_OBSERVATION_SCHEMA: &str = "lattice.implementation-attempt-repair-observation.v1";
pub const STEP_SCHEMA: &str = "lattice.implementation-attempt-repair-step.v2";

type Bound = super::state::Bound;
type Disposition = super::state::Disposition;
type Uncertainty = super::state::Uncertainty;
type WorkerOutcome = super::step::WorkerOutcome;

/// A conditional field the record leaves out; the reader checks the
/// condition.
const fn absent<T>() -> Option<T> {
    None
}

/// A nullable field that must still be present.
fn required<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer)
}

/// Whether one check's read-only copy still captured to the candidate, with
/// every sealed file type and mode, after the check ran.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyIntegrity {
    pub check: String,
    pub is_intact: bool,
}

/// The record of one step, written once after its checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepRecord {
    pub schema: String,
    pub step: super::step::Request,
    pub report: super::step::Report,
    #[serde(deserialize_with = "required")]
    pub candidate: Option<super::revision::Record>,
    /// The check port's observations, in invocation order.
    pub checks: Vec<super::checks::Observation>,
    /// The integrity of each observation's copy, in the same order.
    pub copies: Vec<CopyIntegrity>,
    #[serde(deserialize_with = "required")]
    pub verdict: Option<super::checks::Verdict>,
}

/// The authority a later step found withdrawn before its intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Withdrawal {
    /// The step that never spawned.
    pub iteration: u32,
    pub cause: super::state::Authority,
    /// The binding's revalidation record, the revision seam's refusal, or
    /// the error that kept the seam from answering.
    pub evidence: serde_json::Value,
}

/// The disposition record of one run, printed and written once.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema: String,
    pub envelope: super::super::ImplementationAttemptEnvelope,
    pub request_identity: String,
    pub worker_request: String,
    pub execution_binding: String,
    pub repair_policy: String,
    #[serde(deserialize_with = "required")]
    pub budget_lineage: Option<String>,
    pub disposition: Disposition,
    pub steps: u32,
    pub revisions: Vec<super::revision::Record>,
    pub usage: Vec<super::step::Usage>,
    pub drawn: super::super::lineage::Draws,
    #[serde(deserialize_with = "required")]
    pub unaccounted_dimensions: Option<Vec<String>>,
    pub accepted: bool,
    pub integrated: bool,
    /// Present exactly when the request carries a dispatch plan.
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub dispatch_plan: Option<String>,
    /// Present exactly when the disposition is `authority_withdrawn`.
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub authority_withdrawn: Option<Withdrawal>,
}

/// Whether the records prove that every worker step the run started has
/// stopped. `steps` are the records of every started step, in order;
/// `has_unrecorded_intent` is a step intent after them without a record.
///
/// A step's worker stopped when its record reports it exited or cancelled;
/// an unknown outcome, including an unknown teardown, proves nothing, so
/// `uncertain` (`worker_outcome`) is never settled. An intent without a
/// record is a step whose worker was never called only when the elapsed
/// bound or the clock stopped the loop after the intent; with any other
/// disposition that worker may have run.
#[must_use]
pub fn is_stopped(disposition: &Disposition, steps: &[StepRecord], has_unrecorded_intent: bool) -> bool {
    let is_observed = steps
        .iter()
        .all(|step| matches!(step.report.outcome, WorkerOutcome::Exited { .. } | WorkerOutcome::Cancelled));
    let is_worker_uncertain = matches!(disposition, Disposition::Uncertain {
        cause: Uncertainty::WorkerOutcome
    });
    let is_never_called = matches!(
        disposition,
        Disposition::BudgetExhausted { bound: Bound::Elapsed }
            | Disposition::Uncertain {
                cause: Uncertainty::Clock
            }
    );
    debug_assert!(!is_worker_uncertain || !is_never_called, "one disposition");
    is_observed && !is_worker_uncertain && (!has_unrecorded_intent || is_never_called)
}
