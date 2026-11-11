//! Versioned coordinator envelope for one admitted implementation attempt.
//!
//! This module owns only bounded admission and canonical identity for the
//! immutable inputs that a future implementation-attempt workflow must bind
//! before effects. It does not select work, execute a worker, approve a
//! candidate, or publish anything.

use thiserror::Error;

pub const IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA: &str = "lattice.implementation-attempt-envelope.v1";
pub const MAX_IDENTIFIER_BYTES: usize = 256;
pub const MAX_TURNS: u32 = 128;
pub const MAX_SECONDS: u64 = 24 * 60 * 60;
pub const MAX_COST_MICROS: u64 = 1_000_000_000;
pub const MAX_REPOSITORY_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    GitHubIssue,
    CairnTask,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptBudget {
    pub max_turns: u32,
    pub max_seconds: u64,
    pub max_cost_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImplementationAttemptEnvelope {
    pub schema: String,
    pub work_id: String,
    pub work_version: u64,
    pub source_kind: SourceKind,
    pub source_id: String,
    pub repository: String,
    pub base: String,
    pub readiness_snapshot: String,
    pub task_spec_digest: String,
    pub verification_profile: String,
    pub attempt_id: String,
    pub fence: u64,
    pub workspace_id: String,
    pub worker_profile: String,
    pub budget: AttemptBudget,
    pub policy_digest: String,
}

/// Admit the envelope and return the canonical identity of its exact inputs.
pub fn admit_implementation_attempt(
    envelope: &ImplementationAttemptEnvelope,
) -> Result<String, ImplementationAttemptError> {
    envelope.validate()?;
    Ok(envelope.identity())
}

impl ImplementationAttemptEnvelope {
    /// Check every binding is present, bounded, and structurally valid.
    pub fn validate(&self) -> Result<(), ImplementationAttemptError> {
        if self.schema != IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA {
            return Err(ImplementationAttemptError::InvalidSchema);
        }
        for value in [&self.work_id, &self.source_id, &self.attempt_id, &self.workspace_id] {
            validate_identifier(value)?;
        }
        if !repository(&self.repository) {
            return Err(ImplementationAttemptError::InvalidRepository);
        }
        if !commit(&self.base) {
            return Err(ImplementationAttemptError::InvalidBase);
        }
        for digest in [
            &self.readiness_snapshot,
            &self.task_spec_digest,
            &self.verification_profile,
            &self.worker_profile,
            &self.policy_digest,
        ] {
            if !is_blake3(digest) {
                return Err(ImplementationAttemptError::InvalidDigest);
            }
        }
        if self.work_version == 0 || self.fence == 0 {
            return Err(ImplementationAttemptError::InvalidLineage);
        }
        if self.budget.max_turns == 0
            || self.budget.max_turns > MAX_TURNS
            || self.budget.max_seconds == 0
            || self.budget.max_seconds > MAX_SECONDS
            || self.budget.max_cost_micros > MAX_COST_MICROS
        {
            return Err(ImplementationAttemptError::InvalidBudget);
        }
        Ok(())
    }

    /// Canonical BLAKE3 identity over the complete envelope.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut bytes = Vec::new();
        push_str(&mut bytes, &self.schema);
        push_str(&mut bytes, &self.work_id);
        bytes.extend_from_slice(&self.work_version.to_be_bytes());
        bytes.push(match self.source_kind {
            SourceKind::GitHubIssue => 1,
            SourceKind::CairnTask => 2,
        });
        for value in [
            &self.source_id,
            &self.repository,
            &self.base,
            &self.readiness_snapshot,
            &self.task_spec_digest,
            &self.verification_profile,
            &self.attempt_id,
            &self.workspace_id,
            &self.worker_profile,
        ] {
            push_str(&mut bytes, value);
        }
        bytes.extend_from_slice(&self.fence.to_be_bytes());
        bytes.extend_from_slice(&self.budget.max_turns.to_be_bytes());
        bytes.extend_from_slice(&self.budget.max_seconds.to_be_bytes());
        bytes.extend_from_slice(&self.budget.max_cost_micros.to_be_bytes());
        push_str(&mut bytes, &self.policy_digest);
        blake3::hash(&bytes).to_hex().to_string()
    }
}

fn validate_identifier(value: &str) -> Result<(), ImplementationAttemptError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.chars().any(char::is_control) {
        return Err(ImplementationAttemptError::InvalidIdentifier);
    }
    Ok(())
}

fn repository(value: &str) -> bool {
    let Some((owner, name)) = value.split_once('/') else {
        return false;
    };
    !owner.is_empty()
        && !name.is_empty()
        && owner != "."
        && owner != ".."
        && name != "."
        && name != ".."
        && !name.contains('/')
        && value.len() <= MAX_REPOSITORY_BYTES
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/'))
}

fn commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_blake3(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn push_str(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImplementationAttemptError {
    #[error("implementation attempt envelope schema is not supported")]
    InvalidSchema,
    #[error("implementation attempt identifier is empty, unbounded, or contains control characters")]
    InvalidIdentifier,
    #[error("implementation attempt repository is malformed")]
    InvalidRepository,
    #[error("implementation attempt base is not a 40- or 64-hex commit")]
    InvalidBase,
    #[error("implementation attempt digest is malformed")]
    InvalidDigest,
    #[error("implementation attempt lineage is missing")]
    InvalidLineage,
    #[error("implementation attempt budget is invalid or over bound")]
    InvalidBudget,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> ImplementationAttemptEnvelope {
        ImplementationAttemptEnvelope {
            schema: IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA.to_string(),
            work_id: "onix/kiln#45".to_string(),
            work_version: 3,
            source_kind: SourceKind::CairnTask,
            source_id: "change/task".to_string(),
            repository: "onix/kiln".to_string(),
            base: "a".repeat(40),
            readiness_snapshot: "b".repeat(64),
            task_spec_digest: "c".repeat(64),
            verification_profile: "d".repeat(64),
            attempt_id: "attempt-1".to_string(),
            fence: 7,
            workspace_id: "workspace-1".to_string(),
            worker_profile: "e".repeat(64),
            budget: AttemptBudget {
                max_turns: 8,
                max_seconds: 600,
                max_cost_micros: 10_000,
            },
            policy_digest: "f".repeat(64),
        }
    }

    #[test]
    // r[verify lattice.implementation_attempt.envelope]
    fn admits_a_complete_envelope_with_stable_identity() {
        let envelope = envelope();
        let first = admit_implementation_attempt(&envelope).expect("envelope admits");
        let second = admit_implementation_attempt(&envelope).expect("envelope admits");
        assert_eq!(first, second);
        assert!(is_blake3(&first));
    }

    #[test]
    fn changed_inputs_or_invalid_bounds_are_rejected() {
        let mut changed = envelope();
        changed.budget.max_turns += 1;
        assert_ne!(
            admit_implementation_attempt(&envelope()).expect("base admits"),
            admit_implementation_attempt(&changed).expect("changed admits")
        );
        changed.schema = "lattice.implementation-attempt-envelope.v2".to_string();
        assert_eq!(admit_implementation_attempt(&changed), Err(ImplementationAttemptError::InvalidSchema));
        let mut invalid = envelope();
        invalid.base = "abc".to_string();
        assert_eq!(admit_implementation_attempt(&invalid), Err(ImplementationAttemptError::InvalidBase));
        invalid = envelope();
        invalid.budget.max_seconds = MAX_SECONDS + 1;
        assert_eq!(admit_implementation_attempt(&invalid), Err(ImplementationAttemptError::InvalidBudget));
        invalid = envelope();
        invalid.repository = "../kiln".to_string();
        assert_eq!(admit_implementation_attempt(&invalid), Err(ImplementationAttemptError::InvalidRepository));
    }
}
