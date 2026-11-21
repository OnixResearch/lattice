//! Lifecycle transitions. Each returns a new lifecycle with its outcome, or
//! a refusal that leaves the caller's lifecycle untouched.

pub mod prepare;
pub mod retention;
pub mod selection;
pub mod sessions;

/// The lifecycle after one applied command, and what happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied<T> {
    pub lifecycle: super::state::Lifecycle,
    pub outcome: T,
}

/// An outcome that changes nothing keeps the caller's lifecycle.
pub(crate) fn unchanged<T>(lifecycle: &super::state::Lifecycle, outcome: T) -> Applied<T> {
    Applied {
        lifecycle: lifecycle.clone(),
        outcome,
    }
}

pub(crate) fn next(generation: u64) -> Result<u64, super::Refusal> {
    generation.checked_add(1).ok_or(super::Refusal::GenerationExhausted)
}

pub(crate) fn candidate<'a>(
    lifecycle: &'a super::state::Lifecycle,
    revision: &str,
) -> Result<&'a super::state::Candidate, super::Refusal> {
    lifecycle.candidates.get(revision).ok_or(super::Refusal::UnknownRevision)
}

pub(crate) fn candidate_mut<'a>(
    lifecycle: &'a mut super::state::Lifecycle,
    revision: &str,
) -> Result<&'a mut super::state::Candidate, super::Refusal> {
    lifecycle.candidates.get_mut(revision).ok_or(super::Refusal::UnknownRevision)
}

/// Refuse a revoked revision.
pub(crate) fn usable(lifecycle: &super::state::Lifecycle, digest: &str) -> Result<(), super::Refusal> {
    if lifecycle.revoked.revisions.contains(digest) {
        return Err(super::Refusal::RevisionRevoked);
    }
    Ok(())
}

/// Refuse a revoked qualification attachment.
pub(crate) fn attested(lifecycle: &super::state::Lifecycle, qualification_ref: &str) -> Result<(), super::Refusal> {
    if lifecycle.revoked.qualifications.contains(qualification_ref) {
        return Err(super::Refusal::QualificationRevoked);
    }
    Ok(())
}

/// Refuse adding one more item to a collection of `count` past `bound`.
pub(crate) fn room(count: usize, bound: u32) -> Result<(), super::Refusal> {
    if count.checked_add(1).is_some_and(|after| super::is_within(after, bound)) {
        Ok(())
    } else {
        Err(super::Refusal::CapacityExhausted)
    }
}

/// Whether nothing references `revision`: its preparation is settled without
/// a usable result, it occupies no artifact, nothing holds it, and it is not
/// desired, selected, last usable or pinned.
fn is_forgettable(lifecycle: &super::state::Lifecycle, revision: &str, candidate: &super::state::Candidate) -> bool {
    let is_settled = matches!(
        candidate.preparation,
        super::state::Preparation::Unprepared {}
            | super::state::Preparation::Failed { .. }
            | super::state::Preparation::Superseded { .. }
    );
    let is_named = |value: Option<&String>| value.is_some_and(|value| value == revision);
    let checks = [
        is_settled,
        candidate.artifacts.is_empty(),
        candidate.holds.is_empty(),
        !is_named(lifecycle.desired.as_ref().map(|desired| &desired.revision)),
        !is_named(lifecycle.selected.as_ref().map(|selected| &selected.revision)),
        !is_named(lifecycle.last_usable.as_ref()),
        !lifecycle.sessions.values().any(|pin| pin.revision == revision),
    ];
    checks.iter().all(|check| *check)
}

/// Forget every revision nothing references, with its qualifications and
/// selection history. Revocations are never forgotten, so a forgotten
/// revoked revision stays refused.
pub(crate) fn forget_unreferenced(lifecycle: &mut super::state::Lifecycle) {
    let forgotten = lifecycle
        .candidates
        .iter()
        .filter(|(revision, candidate)| is_forgettable(lifecycle, revision, candidate))
        .map(|(revision, _)| revision.clone())
        .collect::<std::collections::BTreeSet<_>>();
    lifecycle.candidates.retain(|revision, _| !forgotten.contains(revision));
    lifecycle.qualifications.retain(|admitted| !forgotten.contains(&admitted.revision));
    lifecycle.previously_selected.retain(|revision| !forgotten.contains(revision));
    debug_assert!(forgotten.iter().all(|revision| !lifecycle.candidates.contains_key(revision)));
}
