//! Identity of one host-derived candidate revision.

const DOMAIN: &str = "lattice.implementation-attempt-candidate-revision.v1 identity";

/// The attempt, the iteration, the parent revision and the host candidate
/// tree and diff a revision binds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input<'a> {
    pub binding: &'a super::StepBinding,
    pub iteration: u32,
    pub parent: Option<&'a str>,
    pub candidate_identity: &'a str,
    pub diff_identity: &'a str,
}

/// Domain-separated BLAKE3 identity of `input`.
#[must_use]
pub fn identity(input: Input<'_>) -> String {
    debug_assert!(input.iteration > 0, "iterations count from one");
    debug_assert!(input.parent.is_none_or(|parent| !parent.is_empty()), "a parent is a revision");
    let mut hasher = blake3::Hasher::new_derive_key(DOMAIN);
    for value in [&input.binding.envelope, &input.binding.attempt_id] {
        super::super::lineage::frame(&mut hasher, value.as_bytes());
    }
    hasher.update(&input.binding.fence.to_be_bytes());
    hasher.update(&input.iteration.to_be_bytes());
    hasher.update(&[u8::from(input.parent.is_some())]);
    super::super::lineage::frame(&mut hasher, input.parent.unwrap_or_default().as_bytes());
    super::super::lineage::frame(&mut hasher, input.candidate_identity.as_bytes());
    super::super::lineage::frame(&mut hasher, input.diff_identity.as_bytes());
    hasher.finalize().to_hex().to_string()
}
