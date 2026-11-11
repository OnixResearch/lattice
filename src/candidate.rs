//! Versioned candidate receipt for one admitted implementation attempt.
//!
//! This module owns only the host-side result contract. Workspace acquisition,
//! worker execution, diff capture, and independent verification remain shell
//! responsibilities.

use thiserror::Error;

pub const CANDIDATE_RECEIPT_SCHEMA: &str = "lattice.candidate-receipt.v1";
pub const MAX_IDENTIFIER_BYTES: usize = 256;
pub const MAX_CHANGED_PATHS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
}
