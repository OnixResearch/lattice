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
