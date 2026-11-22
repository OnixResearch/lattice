//! The exportable candidate of a retained repair disposition.
//!
//! Only `correct_candidate` and `repaired_candidate` name a candidate, and
//! neither field is trusted alone. The whole run is replayed from its step
//! records: every step must be this attempt's step at its iteration, within
//! the admitted policy's iterations, exited, with the parent and failure
//! classes the loop would have given it; every recorded candidate revision
//! must re-derive from this attempt and fence, its iteration, parent, tree,
//! execute bits and diff, and carry a receipt of this work, attempt, fence,
//! workspace and base; and every step's verdict is re-derived from its check
//! observations and check-copy integrity. Each step before the last must have
//! failed only with admitted classes; the last must pass with the named
//! revision. The shell still binds the final receipt and execute bits to the
//! captured workspace.

// r[impl lattice.implementation_attempt.repair_export.scenario.passing]
// r[impl lattice.implementation_attempt.repair_export.scenario.refused]

#[cfg(test)]
mod tests;

type Disposition = super::state::Disposition;
type Record = super::revision::Record;
type StepRecord = super::retained::StepRecord;
type Verdict = super::checks::Verdict;

/// Why a retained disposition names no exportable candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unexportable {
    /// The disposition is not `correct_candidate` or `repaired_candidate`.
    NotCandidate,
    /// The records contradict the disposition, the admitted policy or this
    /// attempt, or name another attempt or fence.
    Contradicted,
}

/// What this attempt admitted: every retained record is checked against it.
#[derive(Debug, Clone, Copy)]
pub struct Frame<'a> {
    pub binding: &'a super::StepBinding,
    pub receipt: crate::candidate::Bindings<'a>,
    pub base_identity: &'a str,
    pub policy: &'a super::Policy,
    /// The implementing worker's profile revision, never a check executor.
    pub worker_profile: &'a str,
}

/// The retained facts of one disposition.
#[derive(Debug, Clone, Copy)]
pub struct Retained<'a> {
    pub disposition: &'a Disposition,
    pub revisions: &'a [Record],
    /// The records of every started step, in order.
    pub steps: &'a [StepRecord],
}

/// The exact final candidate revision of a passing disposition.
pub fn candidate<'a>(frame: &Frame<'_>, retained: Retained<'a>) -> Result<&'a Record, Unexportable> {
    let (named, repairs) = match retained.disposition {
        Disposition::CorrectCandidate { revision } => (revision, 0),
        Disposition::RepairedCandidate { revision, repairs } if *repairs > 0 => (revision, *repairs),
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
    let max_repairs = frame.policy.max_iterations.min(super::MAX_REPAIR_ITERATIONS);
    let is_counted = repairs <= max_repairs
        && u32::try_from(retained.steps.len()).is_ok_and(|steps| steps == repairs.saturating_add(1));
    if !is_counted || !is_chain(frame, retained.revisions) || !is_replayed(frame, retained) {
        return Err(Unexportable::Contradicted);
    }
    let last = retained.revisions.last().ok_or(Unexportable::Contradicted)?;
    if last.revision != *named || last.iteration != repairs.saturating_add(1) {
        return Err(Unexportable::Contradicted);
    }
    debug_assert!(last.iteration <= max_repairs.saturating_add(1), "bounded by the admitted policy");
    Ok(last)
}

/// Whether every revision is this attempt's, within the admitted
/// iterations, in strictly increasing iterations, each parent the revision
/// before it, and re-derives from its own facts.
fn is_chain(frame: &Frame<'_>, revisions: &[Record]) -> bool {
    let last_iteration = frame.policy.max_iterations.min(super::MAX_REPAIR_ITERATIONS).saturating_add(1);
    let mut parent: Option<&str> = None;
    let mut iteration = 0;
    for record in revisions {
        let is_ordered = record.iteration > iteration && record.iteration <= last_iteration;
        if !is_ordered || record.parent.as_deref() != parent || !is_bound_receipt(frame, &record.receipt) {
            return false;
        }
        let derived = super::revision::identity(super::revision::Input {
            binding: frame.binding,
            iteration: record.iteration,
            parent,
            candidate_identity: record.receipt.candidate_identity.as_deref().unwrap_or_default(),
            executables: &record.executables,
            diff_identity: record.receipt.diff_identity.as_deref().unwrap_or_default(),
        });
        if derived != record.revision || !record.base_files.keys().eq(record.receipt.changed_paths.iter()) {
            return false;
        }
        debug_assert!(record.iteration > iteration, "iterations strictly increase");
        parent = Some(&record.revision);
        iteration = record.iteration;
    }
    debug_assert!(iteration <= last_iteration);
    true
}

/// A produced receipt of this work, attempt, fence, workspace and base.
fn is_bound_receipt(frame: &Frame<'_>, receipt: &crate::candidate::CandidateReceipt) -> bool {
    receipt.is_usable_candidate()
        && receipt.work_id == frame.receipt.work_id
        && receipt.attempt_id == frame.receipt.attempt_id
        && receipt.fence == frame.receipt.fence
        && receipt.workspace_id == frame.receipt.workspace_id
        && receipt.base_identity == frame.base_identity
}

/// Whether a disposition's kind agrees with what its step records
/// re-derive, so no recorded kind or verdict is trusted alone: every recorded
/// verdict must be the one its observations and copy integrity give, a
/// passing kind must replay to its exact revision, and a kind decided by the
/// last revision's checks must be that verdict. Kinds decided by the clock,
/// cancellation, the fence, authority, the lineage or Kiln are not derivable
/// from step records.
#[must_use]
pub fn is_consistent(frame: &Frame<'_>, retained: Retained<'_>) -> bool {
    let rederived =
        |step: &StepRecord| step.candidate.as_ref().and_then(|candidate| rederived_verdict(frame, candidate, step));
    let is_faithful = retained.steps.iter().all(|step| step.verdict.is_none() || step.verdict == rederived(step));
    let last = retained.steps.last();
    let last_verdict = last.and_then(|step| step.verdict.clone());
    let is_derived = match retained.disposition {
        Disposition::CorrectCandidate { .. } | Disposition::RepairedCandidate { .. } => {
            candidate(frame, retained).is_ok()
        }
        Disposition::NotRepairable { classes } => {
            let is_no_candidate = last.is_some_and(|step| step.candidate.is_none())
                && classes.len() == 1
                && classes.first().is_some_and(|class| class == super::NO_CANDIDATE);
            let is_failed = last_verdict
                == Some(Verdict::Failed {
                    classes: classes.clone(),
                });
            !frame.policy.admits(classes) && (is_no_candidate || is_failed)
        }
        Disposition::ChecksRejected { defect } => last_verdict == Some(Verdict::Rejected { defect: defect.clone() }),
        Disposition::Uncertain {
            cause: super::state::Uncertainty::CheckOutcome,
        } => last_verdict == Some(Verdict::Unknown),
        Disposition::BudgetExhausted { .. }
        | Disposition::Cancelled
        | Disposition::Uncertain { .. }
        | Disposition::LineageBlocked { .. }
        | Disposition::KilnOwned { .. }
        | Disposition::FenceSuperseded
        | Disposition::AuthorityWithdrawn { .. } => true,
    };
    debug_assert!(
        !is_faithful || retained.steps.iter().all(|step| step.verdict.is_none() || step.candidate.is_some()),
        "a faithful verdict judges a candidate"
    );
    debug_assert_eq!(last.is_some(), !retained.steps.is_empty());
    is_faithful && is_derived
}

/// Whether the step records replay a run that repaired only admitted
/// failures and ended with a pass of its last revision.
fn is_replayed(frame: &Frame<'_>, retained: Retained<'_>) -> bool {
    let mut revisions = retained.revisions.iter();
    let mut parent: Option<&str> = None;
    let mut failures: Vec<String> = Vec::new();
    for (index, step) in retained.steps.iter().enumerate() {
        let is_last = index.saturating_add(1) == retained.steps.len();
        if !is_requested(frame, step, index, parent, &failures) {
            return false;
        }
        let verdict = match &step.candidate {
            Some(candidate) => {
                if revisions.next() != Some(candidate) || candidate.iteration != step.step.iteration {
                    return false;
                }
                parent = Some(&candidate.revision);
                rederived_verdict(frame, candidate, step)
            }
            None if step.checks.is_empty() && step.copies.is_empty() => None,
            None => return false,
        };
        if verdict != step.verdict {
            return false;
        }
        failures = match (is_last, verdict) {
            (true, Some(Verdict::Passed)) => Vec::new(),
            (false, Some(Verdict::Failed { classes })) if frame.policy.admits(&classes) => classes,
            (false, None) if step.candidate.is_none() => vec![super::NO_CANDIDATE.to_owned()],
            _ => return false,
        };
        if !is_last && !frame.policy.admits(&failures) {
            return false;
        }
    }
    debug_assert!(failures.is_empty() || retained.steps.is_empty());
    revisions.next().is_none() && !retained.steps.is_empty()
}

/// Whether `step` is this attempt's exited step at `index`, asked for with
/// the parent and failure classes the loop would have given it.
fn is_requested(frame: &Frame<'_>, step: &StepRecord, index: usize, parent: Option<&str>, failures: &[String]) -> bool {
    let phase = if index == 0 {
        super::step::Phase::Edit
    } else {
        super::step::Phase::Repair
    };
    let is_at = u32::try_from(index).is_ok_and(|index| step.step.iteration == index.saturating_add(1));
    step.schema == super::retained::STEP_SCHEMA
        && is_at
        && step.step.binding == *frame.binding
        && step.step.phase == phase
        && step.step.parent.as_deref() == parent
        && step.step.failures == failures
        && matches!(step.report.outcome, super::step::WorkerOutcome::Exited { .. })
        && step.report.proposal.is_none()
}

/// The verdict of `candidate`'s checks re-derived from the observations and
/// the integrity of each declared check's copy, in invocation order; `None`
/// when they do not line up.
fn rederived_verdict(frame: &Frame<'_>, candidate: &Record, step: &StepRecord) -> Option<Verdict> {
    let is_aligned = step.copies.len() == step.checks.len()
        && step.copies.len() <= frame.policy.checks.len()
        && step.copies.iter().zip(&frame.policy.checks).all(|(copy, declared)| copy.check == declared.name);
    let drifted = step.copies.iter().position(|copy| !copy.is_intact);
    // The loop stops at the first drifted copy, so only the last can drift.
    if !is_aligned || drifted.is_some_and(|index| index.saturating_add(1) != step.copies.len()) {
        return None;
    }
    let subject = super::checks::Subject {
        policy: frame.policy,
        revision: &candidate.revision,
        worker_profile: frame.worker_profile,
    };
    let drifted = drifted.and_then(|index| step.copies.get(index)).map(|copy| copy.check.as_str());
    Some(super::checks::classify(subject, &step.checks, drifted))
}
