//! Versioned candidate receipt for one admitted implementation attempt.
//!
//! This module owns only the host-side result contract. Workspace acquisition,
//! worker execution, diff capture, and independent verification remain shell
//! responsibilities.

use thiserror::Error;

mod admission;
mod snapshots;

pub const CANDIDATE_RECEIPT_SCHEMA: &str = "lattice.candidate-receipt.v1";
pub const MAX_IDENTIFIER_BYTES: usize = 256;
pub const MAX_CHANGED_PATHS: usize = 4_096;
pub const MAX_SNAPSHOT_FILES: usize = 50_000;
pub const MAX_SNAPSHOT_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateDisposition {
    /// A host-observed candidate and diff exist.
    Produced,
    /// A prerequisite or admitted boundary blocked the attempt.
    Blocked,
    /// The worker or capture failed.
    Failed,
    /// The attempt was observed as cancelled.
    Cancelled,
    /// The outcome could not be observed.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CandidateReceipt {
    pub schema: String,
    pub work_id: String,
    pub attempt_id: String,
    pub fence: String,
    pub workspace_id: String,
    pub base_identity: String,
    pub candidate_identity: Option<String>,
    pub diff_identity: Option<String>,
    pub changed_paths: Vec<String>,
    pub disposition: CandidateDisposition,
    pub blockers: Vec<String>,
}

impl CandidateReceipt {
    /// Admit a receipt only when its bindings and disposition agree.
    pub fn validate(&self) -> Result<(), CandidateReceiptError> {
        if self.schema != CANDIDATE_RECEIPT_SCHEMA {
            return Err(CandidateReceiptError::InvalidSchema);
        }
        for identifier in [&self.work_id, &self.attempt_id, &self.fence, &self.workspace_id] {
            admission::identifier(identifier)?;
        }
        if !snapshots::is_lowercase_blake3(&self.base_identity) {
            return Err(CandidateReceiptError::InvalidIdentity);
        }
        for identity in [&self.candidate_identity, &self.diff_identity].into_iter().flatten() {
            if !snapshots::is_lowercase_blake3(identity) {
                return Err(CandidateReceiptError::InvalidIdentity);
            }
        }
        admission::changed_paths(&self.changed_paths)?;
        if self.disposition == CandidateDisposition::Produced {
            if self.candidate_identity.is_none() || self.diff_identity.is_none() {
                return Err(CandidateReceiptError::MissingCandidate);
            }
            if self.changed_paths.is_empty() {
                return Err(CandidateReceiptError::MissingChangedPaths);
            }
            if !self.blockers.is_empty() {
                return Err(CandidateReceiptError::ProducedWithBlockers);
            }
        }
        if matches!(self.disposition, CandidateDisposition::Blocked | CandidateDisposition::Failed)
            && self.blockers.is_empty()
        {
            return Err(CandidateReceiptError::MissingBlocker);
        }
        Ok(())
    }

    /// True only for a produced candidate with complete host-derived bindings.
    pub fn is_usable_candidate(&self) -> bool {
        self.validate().is_ok() && self.disposition == CandidateDisposition::Produced
    }

    /// Serialize only after the receipt passes admission.
    pub fn to_json(&self) -> Result<String, CandidateReceiptError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| CandidateReceiptError::InvalidJson)
    }

    /// Parse and validate a persisted receipt before use.
    pub fn from_json(text: &str) -> Result<Self, CandidateReceiptError> {
        let receipt: Self = serde_json::from_str(text).map_err(|_| CandidateReceiptError::InvalidJson)?;
        receipt.validate()?;
        Ok(receipt)
    }
}

/// Explicit host observation for independently verified already-satisfied work.
/// This is never a generic produced candidate or accepted completion.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnchangedObservation {
    pub schema: String,
    pub work_id: String,
    pub attempt_id: String,
    pub fence: String,
    pub workspace_id: String,
    pub base_identity: String,
    pub candidate_identity: String,
    pub diff_identity: String,
    pub changed_paths: Vec<String>,
    pub disposition: String,
    pub blockers: Vec<String>,
}

/// Named host-request identities borrowed during receipt construction or verification.
#[derive(Debug)]
pub struct Bindings<'a> {
    pub work_id: &'a str,
    pub attempt_id: &'a str,
    pub fence: &'a str,
    pub workspace_id: &'a str,
}

pub fn observe_unchanged_from_snapshots(
    bindings: Bindings<'_>,
    base: &std::collections::BTreeMap<String, Vec<u8>>,
    candidate: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<UnchangedObservation, CandidateReceiptError> {
    snapshots::validate_snapshot(base)?;
    snapshots::validate_snapshot(candidate)?;
    for identifier in [
        bindings.work_id,
        bindings.attempt_id,
        bindings.fence,
        bindings.workspace_id,
    ] {
        admission::identifier(identifier)?;
    }
    if base != candidate {
        return Err(CandidateReceiptError::NoCandidateChanges);
    }
    Ok(unchanged_observation(bindings, base))
}

fn unchanged_observation(
    bindings: Bindings<'_>,
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> UnchangedObservation {
    let identity = snapshots::tree_identity(snapshot);
    UnchangedObservation {
        schema: "lattice.unchanged-candidate-observation.v1".into(),
        work_id: bindings.work_id.to_string(),
        attempt_id: bindings.attempt_id.to_string(),
        fence: bindings.fence.to_string(),
        workspace_id: bindings.workspace_id.to_string(),
        base_identity: identity.clone(),
        candidate_identity: identity,
        diff_identity: snapshots::diff_identity(snapshot, snapshot, &[]),
        changed_paths: Vec::new(),
        disposition: "unchanged".into(),
        blockers: Vec::new(),
    }
}

/// Derive a produced receipt from host-captured base and candidate file maps.
///
/// The maps are the shell's actual workspace snapshot; worker-reported hashes
/// are not accepted here. Paths must already be relative and canonical.
pub fn produced_receipt_from_snapshots(
    bindings: Bindings<'_>,
    base: &std::collections::BTreeMap<String, Vec<u8>>,
    candidate: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<CandidateReceipt, CandidateReceiptError> {
    snapshots::validate_snapshot(base)?;
    snapshots::validate_snapshot(candidate)?;
    let changed_paths = base
        .keys()
        .chain(candidate.keys())
        .filter(|path| base.get(*path) != candidate.get(*path))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if changed_paths.is_empty() {
        return Err(CandidateReceiptError::NoCandidateChanges);
    }
    let receipt = CandidateReceipt {
        schema: CANDIDATE_RECEIPT_SCHEMA.to_string(),
        work_id: bindings.work_id.to_string(),
        attempt_id: bindings.attempt_id.to_string(),
        fence: bindings.fence.to_string(),
        workspace_id: bindings.workspace_id.to_string(),
        base_identity: snapshots::tree_identity(base),
        candidate_identity: Some(snapshots::tree_identity(candidate)),
        diff_identity: Some(snapshots::diff_identity(base, candidate, &changed_paths)),
        changed_paths,
        disposition: CandidateDisposition::Produced,
        blockers: Vec::new(),
    };
    receipt.validate()?;
    Ok(receipt)
}

/// Verify a receipt against expected attempt bindings and actual snapshots.
pub fn verify_receipt_from_snapshots(
    receipt: &CandidateReceipt,
    bindings: Bindings<'_>,
    base: &std::collections::BTreeMap<String, Vec<u8>>,
    candidate: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(), CandidateReceiptError> {
    receipt.validate()?;
    if receipt.work_id != bindings.work_id
        || receipt.attempt_id != bindings.attempt_id
        || receipt.fence != bindings.fence
        || receipt.workspace_id != bindings.workspace_id
    {
        return Err(CandidateReceiptError::BindingMismatch);
    }
    let expected = produced_receipt_from_snapshots(bindings, base, candidate)?;
    if *receipt != expected {
        return Err(CandidateReceiptError::ReceiptMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

/// Validate and identify a host-captured base before admitting a worker.
pub fn snapshot_identity(
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<String, CandidateReceiptError> {
    snapshots::validate_snapshot(snapshot)?;
    Ok(snapshots::tree_identity(snapshot))
}

/// Validate one relative candidate path before storing artifact bytes.
pub fn validate_candidate_path(path: &str) -> Result<(), CandidateReceiptError> {
    admission::path(path)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CandidateReceiptError {
    #[error("candidate receipt schema is not supported")]
    InvalidSchema,
    #[error("candidate receipt identifier is empty, unbounded, or contains control characters")]
    InvalidIdentifier,
    #[error("candidate receipt identity must be one lowercase BLAKE3 hex digest")]
    InvalidIdentity,
    #[error("produced candidate receipt is missing candidate or diff identity")]
    MissingCandidate,
    #[error("produced candidate receipt is missing changed paths")]
    MissingChangedPaths,
    #[error("produced candidate receipt cannot carry blockers")]
    ProducedWithBlockers,
    #[error("blocked or failed candidate receipt must name at least one blocker")]
    MissingBlocker,
    #[error("candidate changed path is empty, absolute, escaping, or unbounded")]
    InvalidChangedPath,
    #[error("candidate changed paths must be sorted and unique")]
    ChangedPathsNotCanonical,
    #[error("candidate changed paths exceed the admitted count")]
    TooManyChangedPaths,
    #[error("candidate snapshot exceeds the admitted file or byte count")]
    SnapshotTooLarge,
    #[error("candidate snapshot contains no changed path")]
    NoCandidateChanges,
    #[error("candidate receipt JSON is malformed or does not match the schema")]
    InvalidJson,
    #[error("candidate receipt does not match the expected attempt bindings")]
    BindingMismatch,
    #[error("candidate receipt does not match the actual workspace snapshots")]
    ReceiptMismatch,
}
