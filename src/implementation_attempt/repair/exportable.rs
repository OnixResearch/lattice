//! The exportable candidate of a retained repair disposition.
//!
//! Only `correct_candidate` and `repaired_candidate` name a candidate. The
//! named revision must be the last recorded one, recorded by the step the
//! disposition implies, and every recorded revision must re-derive from the
//! attempt's binding, its iteration, its parent and its receipt, each parent
//! being the revision recorded before it. A record of another attempt or
//! fence therefore never re-derives. The shell still binds the receipt to the
//! request and the workspace to the receipt.

// r[impl lattice.implementation_attempt.repair_export.scenario.passing]
// r[impl lattice.implementation_attempt.repair_export.scenario.refused]

#[cfg(test)]
mod tests;

type Disposition = super::state::Disposition;
type Record = super::revision::Record;

/// Why a retained disposition names no exportable candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unexportable {
    /// The disposition is not `correct_candidate` or `repaired_candidate`.
    NotCandidate,
    /// The records contradict the disposition or name another attempt or
    /// fence.
    Contradicted,
}

/// The retained facts of one disposition.
#[derive(Debug, Clone, Copy)]
pub struct Retained<'a> {
    pub disposition: &'a Disposition,
    /// Worker steps started, as the disposition records them.
    pub steps: u32,
    pub revisions: &'a [Record],
}

/// The exact final candidate revision of a passing disposition.
pub fn candidate<'a>(binding: &super::StepBinding, retained: Retained<'a>) -> Result<&'a Record, Unexportable> {
    let (named, iteration) = match retained.disposition {
        Disposition::CorrectCandidate { revision } => (revision, 1),
        Disposition::RepairedCandidate { revision, repairs } if *repairs > 0 => {
            (revision, repairs.checked_add(1).ok_or(Unexportable::Contradicted)?)
        }
        Disposition::RepairedCandidate { .. } => return Err(Unexportable::Contradicted),
        Disposition::BudgetExhausted { .. }
        | Disposition::Cancelled
        | Disposition::Uncertain { .. }
        | Disposition::NotRepairable { .. }
        | Disposition::ChecksRejected { .. }
        | Disposition::LineageBlocked { .. }
        | Disposition::KilnOwned { .. }
        | Disposition::FenceSuperseded
        | Disposition::AuthorityWithdrawn { .. } => return Err(Unexportable::NotCandidate),
    };
    debug_assert!(iteration > 0, "iterations count from one");
    let last = retained.revisions.last().ok_or(Unexportable::Contradicted)?;
    let is_named = last.revision == *named && last.iteration == iteration && retained.steps == iteration;
    if !is_named || !is_chain(binding, retained.revisions) {
        return Err(Unexportable::Contradicted);
    }
    debug_assert_eq!(last.iteration, retained.steps, "the last step recorded the named revision");
    Ok(last)
}

/// Whether every revision re-derives from `binding` in strictly increasing
/// iterations, each parent the revision before it.
fn is_chain(binding: &super::StepBinding, revisions: &[Record]) -> bool {
    let is_bounded =
        u32::try_from(revisions.len()).is_ok_and(|count| count <= super::MAX_REPAIR_ITERATIONS.saturating_add(1));
    if !is_bounded {
        return false;
    }
    let mut parent: Option<&str> = None;
    let mut iteration = 0;
    for record in revisions {
        if record.iteration <= iteration || record.parent.as_deref() != parent {
            return false;
        }
        let derived = super::revision::identity(super::revision::Input {
            binding,
            iteration: record.iteration,
            parent,
            candidate_identity: record.receipt.candidate_identity.as_deref().unwrap_or_default(),
            diff_identity: record.receipt.diff_identity.as_deref().unwrap_or_default(),
        });
        if derived != record.revision {
            return false;
        }
        debug_assert!(record.iteration > 0 && record.iteration > iteration, "iterations strictly increase");
        parent = Some(&record.revision);
        iteration = record.iteration;
    }
    debug_assert!(revisions.last().is_none_or(|last| last.iteration == iteration));
    true
}
