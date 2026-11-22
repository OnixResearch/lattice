//! Identity of one host-derived candidate revision.

const DOMAIN: &str = "lattice.implementation-attempt-candidate-revision.v2 identity";

/// Captured paths with an execute bit, mapped to their execute bits
/// (`mode & 0o111`). Capture admits only regular files, so this is the whole
/// type and mode state a check's read-only copy is sealed with besides the
/// read bits.
pub type Executables = std::collections::BTreeMap<String, u32>;

/// The attempt, the iteration, the parent revision and the host candidate
/// tree, execute bits and diff a revision binds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input<'a> {
    pub binding: &'a super::StepBinding,
    pub iteration: u32,
    pub parent: Option<&'a str>,
    pub candidate_identity: &'a str,
    pub executables: &'a Executables,
    pub diff_identity: &'a str,
}

/// One recorded host-derived candidate revision, as a step record and the
/// retained disposition carry it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub iteration: u32,
    pub revision: String,
    pub parent: Option<String>,
    pub receipt: crate::candidate::CandidateReceipt,
    /// The execute bits the candidate was captured, and checked, with.
    pub executables: Executables,
    /// The base digest of every changed path, `None` where the base had no
    /// file: with the captured candidate it re-derives the whole receipt.
    pub base_files: std::collections::BTreeMap<String, Option<crate::candidate::digests::FileDigest>>,
}

/// The execute bits of captured `modes`, keeping only paths that have one.
#[must_use]
pub fn executables(modes: &std::collections::BTreeMap<String, u32>) -> Executables {
    modes
        .iter()
        .filter(|(_, mode)| *mode & 0o111 != 0)
        .map(|(path, mode)| (path.clone(), mode & 0o111))
        .collect()
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
    hasher.update(&(input.executables.len() as u64).to_be_bytes());
    for (path, bits) in input.executables {
        super::super::lineage::frame(&mut hasher, path.as_bytes());
        hasher.update(&bits.to_be_bytes());
    }
    super::super::lineage::frame(&mut hasher, input.diff_identity.as_bytes());
    hasher.finalize().to_hex().to_string()
}
