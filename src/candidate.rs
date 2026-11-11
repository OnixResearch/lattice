//! Versioned candidate receipt for one admitted implementation attempt.
//!
//! This module owns only the host-side result contract. Workspace acquisition,
//! worker execution, diff capture, and independent verification remain shell
//! responsibilities.

use std::collections::BTreeMap;

use thiserror::Error;

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
            validate_identifier(identifier)?;
        }
        if !is_lowercase_blake3(&self.base_identity) {
            return Err(CandidateReceiptError::InvalidIdentity);
        }
        for identity in [&self.candidate_identity, &self.diff_identity].into_iter().flatten() {
            if !is_lowercase_blake3(identity) {
                return Err(CandidateReceiptError::InvalidIdentity);
            }
        }
        validate_changed_paths(&self.changed_paths)?;
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

/// Derive a produced receipt from host-captured base and candidate file maps.
///
/// The maps are the shell's actual workspace snapshot; worker-reported hashes
/// are not accepted here. Paths must already be relative and canonical.
pub fn produced_receipt_from_snapshots(
    work_id: impl Into<String>,
    attempt_id: impl Into<String>,
    fence: impl Into<String>,
    workspace_id: impl Into<String>,
    base: &BTreeMap<String, Vec<u8>>,
    candidate: &BTreeMap<String, Vec<u8>>,
) -> Result<CandidateReceipt, CandidateReceiptError> {
    validate_snapshot(base)?;
    validate_snapshot(candidate)?;
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
        work_id: work_id.into(),
        attempt_id: attempt_id.into(),
        fence: fence.into(),
        workspace_id: workspace_id.into(),
        base_identity: tree_identity(base),
        candidate_identity: Some(tree_identity(candidate)),
        diff_identity: Some(diff_identity(base, candidate, &changed_paths)),
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
    expected_work_id: &str,
    expected_attempt_id: &str,
    expected_fence: &str,
    expected_workspace_id: &str,
    base: &BTreeMap<String, Vec<u8>>,
    candidate: &BTreeMap<String, Vec<u8>>,
) -> Result<(), CandidateReceiptError> {
    receipt.validate()?;
    if receipt.work_id != expected_work_id
        || receipt.attempt_id != expected_attempt_id
        || receipt.fence != expected_fence
        || receipt.workspace_id != expected_workspace_id
    {
        return Err(CandidateReceiptError::BindingMismatch);
    }
    let expected = produced_receipt_from_snapshots(
        expected_work_id.to_string(),
        expected_attempt_id.to_string(),
        expected_fence.to_string(),
        expected_workspace_id.to_string(),
        base,
        candidate,
    )?;
    if *receipt != expected {
        return Err(CandidateReceiptError::ReceiptMismatch);
    }
    Ok(())
}

fn validate_snapshot(snapshot: &BTreeMap<String, Vec<u8>>) -> Result<(), CandidateReceiptError> {
    if snapshot.len() > MAX_SNAPSHOT_FILES {
        return Err(CandidateReceiptError::SnapshotTooLarge);
    }
    let mut total = 0usize;
    for (path, bytes) in snapshot {
        validate_changed_paths(std::slice::from_ref(path))?;
        total = total.checked_add(bytes.len()).ok_or(CandidateReceiptError::SnapshotTooLarge)?;
    }
    if total > MAX_SNAPSHOT_BYTES {
        return Err(CandidateReceiptError::SnapshotTooLarge);
    }
    Ok(())
}

fn tree_identity(snapshot: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice-candidate-tree-v1");
    update_len(&mut hasher, snapshot.len());
    for (path, bytes) in snapshot {
        update_len(&mut hasher, path.len());
        hasher.update(path.as_bytes());
        update_len(&mut hasher, bytes.len());
        hasher.update(blake3::hash(bytes).as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn diff_identity(base: &BTreeMap<String, Vec<u8>>, candidate: &BTreeMap<String, Vec<u8>>, paths: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice-candidate-diff-v1");
    update_len(&mut hasher, paths.len());
    for path in paths {
        update_len(&mut hasher, path.len());
        hasher.update(path.as_bytes());
        update_optional_bytes(&mut hasher, base.get(path).map(Vec::as_slice));
        update_optional_bytes(&mut hasher, candidate.get(path).map(Vec::as_slice));
    }
    hasher.finalize().to_hex().to_string()
}

fn update_optional_bytes(hasher: &mut blake3::Hasher, bytes: Option<&[u8]>) {
    match bytes {
        Some(bytes) => {
            hasher.update(b"present");
            update_len(hasher, bytes.len());
            hasher.update(blake3::hash(bytes).as_bytes());
        }
        None => {
            hasher.update(b"absent");
        }
    }
}

fn update_len(hasher: &mut blake3::Hasher, len: usize) {
    hasher.update(&(len as u64).to_le_bytes());
}

fn validate_identifier(value: &str) -> Result<(), CandidateReceiptError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.chars().any(char::is_control) {
        return Err(CandidateReceiptError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_changed_paths(paths: &[String]) -> Result<(), CandidateReceiptError> {
    if paths.len() > MAX_CHANGED_PATHS {
        return Err(CandidateReceiptError::TooManyChangedPaths);
    }
    let mut previous: Option<&str> = None;
    for path in paths {
        if path.is_empty()
            || path.len() > MAX_IDENTIFIER_BYTES
            || path.starts_with('/')
            || path.starts_with('\\')
            || path.split('/').any(|component| component == ".." || component.is_empty())
        {
            return Err(CandidateReceiptError::InvalidChangedPath);
        }
        if previous.is_some_and(|previous| previous >= path.as_str()) {
            return Err(CandidateReceiptError::ChangedPathsNotCanonical);
        }
        previous = Some(path);
    }
    Ok(())
}

fn is_lowercase_blake3(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt() -> CandidateReceipt {
        CandidateReceipt {
            schema: CANDIDATE_RECEIPT_SCHEMA.to_string(),
            work_id: "work-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            fence: "7".to_string(),
            workspace_id: "workspace-1".to_string(),
            base_identity: "a".repeat(64),
            candidate_identity: Some("b".repeat(64)),
            diff_identity: Some("c".repeat(64)),
            changed_paths: vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
            disposition: CandidateDisposition::Produced,
            blockers: Vec::new(),
        }
    }

    #[test]
    fn produced_candidate_requires_complete_bindings() {
        let receipt = receipt();
        assert_eq!(receipt.validate(), Ok(()));
        assert!(receipt.is_usable_candidate());
    }

    #[test]
    fn exit_zero_without_candidate_is_not_usable() {
        let mut receipt = receipt();
        receipt.candidate_identity = None;
        assert_eq!(receipt.validate(), Err(CandidateReceiptError::MissingCandidate));
        assert!(!receipt.is_usable_candidate());
    }

    #[test]
    fn malformed_identities_and_paths_are_rejected() {
        let mut identity = receipt();
        identity.base_identity = "A".repeat(64);
        assert_eq!(identity.validate(), Err(CandidateReceiptError::InvalidIdentity));

        let mut absolute = receipt();
        absolute.changed_paths = vec!["/etc/passwd".to_string()];
        assert_eq!(absolute.validate(), Err(CandidateReceiptError::InvalidChangedPath));

        let mut escaping = receipt();
        escaping.changed_paths = vec!["src/../secret".to_string()];
        assert_eq!(escaping.validate(), Err(CandidateReceiptError::InvalidChangedPath));

        let mut unordered = receipt();
        unordered.changed_paths = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
        assert_eq!(unordered.validate(), Err(CandidateReceiptError::ChangedPathsNotCanonical));

        let mut duplicate = receipt();
        duplicate.changed_paths = vec!["src/lib.rs".to_string(), "src/lib.rs".to_string()];
        assert_eq!(duplicate.validate(), Err(CandidateReceiptError::ChangedPathsNotCanonical));
    }

    #[test]
    fn non_success_dispositions_remain_precise() {
        let mut blocked = receipt();
        blocked.disposition = CandidateDisposition::Blocked;
        blocked.candidate_identity = None;
        blocked.diff_identity = None;
        blocked.changed_paths.clear();
        assert_eq!(blocked.validate(), Err(CandidateReceiptError::MissingBlocker));
        blocked.blockers = vec!["base is stale".to_string()];
        assert_eq!(blocked.validate(), Ok(()));
        assert!(!blocked.is_usable_candidate());

        let mut unknown = receipt();
        unknown.disposition = CandidateDisposition::Unknown;
        unknown.candidate_identity = None;
        unknown.diff_identity = None;
        unknown.changed_paths.clear();
        assert_eq!(unknown.validate(), Ok(()));
        assert!(!unknown.is_usable_candidate());
    }

    #[test]
    fn receipt_json_round_trips_only_after_admission() {
        let receipt = receipt();
        let json = receipt.to_json().expect("valid receipt serializes");
        assert_eq!(CandidateReceipt::from_json(&json), Ok(receipt.clone()));

        let mut invalid = receipt;
        invalid.changed_paths = vec!["../escape".to_string()];
        assert_eq!(invalid.to_json(), Err(CandidateReceiptError::InvalidChangedPath));
        assert_eq!(CandidateReceipt::from_json("{not-json"), Err(CandidateReceiptError::InvalidJson));
    }

    #[test]
    fn snapshot_receipt_captures_changed_added_and_deleted_paths() {
        let base = BTreeMap::from([
            ("src/lib.rs".to_string(), b"old".to_vec()),
            ("src/removed.rs".to_string(), b"gone".to_vec()),
        ]);
        let candidate = BTreeMap::from([
            ("src/lib.rs".to_string(), b"new".to_vec()),
            ("src/added.rs".to_string(), b"added".to_vec()),
        ]);
        let receipt = produced_receipt_from_snapshots("work-1", "attempt-1", "7", "workspace-1", &base, &candidate)
            .expect("snapshot receipt derives");

        assert_eq!(receipt.changed_paths, vec!["src/added.rs", "src/lib.rs", "src/removed.rs"]);
        assert_eq!(receipt.disposition, CandidateDisposition::Produced);
        assert!(receipt.validate().is_ok());
        assert!(receipt.is_usable_candidate());

        let repeated = produced_receipt_from_snapshots("work-1", "attempt-1", "7", "workspace-1", &base, &candidate)
            .expect("snapshot receipt repeats");
        assert_eq!(receipt, repeated);
    }

    #[test]
    fn snapshot_receipt_rejects_no_change_and_invalid_paths() {
        let base = BTreeMap::from([("src/lib.rs".to_string(), b"same".to_vec())]);
        assert_eq!(
            produced_receipt_from_snapshots("work-1", "attempt-1", "7", "workspace-1", &base, &base),
            Err(CandidateReceiptError::NoCandidateChanges)
        );

        let invalid = BTreeMap::from([("../escape.rs".to_string(), b"x".to_vec())]);
        assert_eq!(
            produced_receipt_from_snapshots("work-1", "attempt-1", "7", "workspace-1", &base, &invalid),
            Err(CandidateReceiptError::InvalidChangedPath)
        );
    }

    #[test]
    fn independent_verification_rejects_crossed_or_stale_receipts() {
        let base = BTreeMap::from([("src/lib.rs".to_string(), b"old".to_vec())]);
        let candidate = BTreeMap::from([("src/lib.rs".to_string(), b"new".to_vec())]);
        let receipt = produced_receipt_from_snapshots("work-1", "attempt-1", "7", "workspace-1", &base, &candidate)
            .expect("receipt derives");

        assert_eq!(
            verify_receipt_from_snapshots(&receipt, "work-1", "attempt-1", "7", "workspace-1", &base, &candidate),
            Ok(())
        );
        assert_eq!(
            verify_receipt_from_snapshots(&receipt, "work-1", "attempt-2", "7", "workspace-1", &base, &candidate),
            Err(CandidateReceiptError::BindingMismatch)
        );
        let changed = BTreeMap::from([("src/lib.rs".to_string(), b"different".to_vec())]);
        assert_eq!(
            verify_receipt_from_snapshots(&receipt, "work-1", "attempt-1", "7", "workspace-1", &base, &changed),
            Err(CandidateReceiptError::ReceiptMismatch)
        );
    }
}
