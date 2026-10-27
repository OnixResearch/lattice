//! Pure validation, projection, and evidence for the optional Pijul CLI adapter.
//!
//! This module operates on loaded values. It does not read files or the
//! environment, discover repositories or executables, open `.pijul` databases,
//! spawn processes, access networks, inspect keys, use clocks, or persist facts.

mod profile;
mod projection;
mod validate;

use std::fmt;

use animus_adapters::pijul::{PijulConfirmation, PijulOperation, PijulSignatureStatus};
use animus_adapters::{CheckpointRecord, SnapshotRecord, SpeculativeWorkspaceDescriptor};

pub use profile::{PijulBounds, PijulProfile, parse_pijul_profile_json};
pub use projection::{project_pijul_log, project_pijul_receipt};
pub use validate::{validate_pijul_request, validate_pijul_response};

use crate::provenance::ProvenanceFact;

pub(super) const PROFILE_SCHEMA: &str = "animus.pijul-vcs-adapter.v1";
pub(super) const PROJECTION_DOMAIN: &str = "animus.pijul-projection.v1";
pub(super) const RECEIPT_DOMAIN: &str = "animus.pijul-receipt.v1";
pub(super) const CHECKPOINT_BEFORE_DOMAIN: &str = "animus.pijul-checkpoint-before.v1";
pub(super) const WORKSPACE_DOMAIN: &str = "animus.pijul-workspace.v1";
pub(super) const PROVENANCE_SCHEMA: &str = "animus.pijul-provenance-observation.v1";
pub(super) const CHANGE_HASH_CHARS: usize = 53;
pub(super) const STATE_ID_CHARS: usize = 53;
pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const REQUIRED_NON_CLAIMS: &[&str] = &[
    "not-repository-integrity",
    "not-change-semantic-correctness",
    "not-content-correctness",
    "not-merge-safety",
    "not-authorship-authenticity",
    "not-signature-verification",
    "not-key-custody",
    "not-network-or-remote-correctness",
    "not-conflict-resolution-quality",
    "not-persistence-or-durability",
    "not-production-readiness",
    "not-release-eligibility",
];
pub(super) const PROJECTION_TARGETS: &[&str] =
    &["checkpoint", "snapshot_ledger", "provenance", "workspace"];
pub(super) const ALLOWED_DIFF_OPERATIONS: &[&str] = &[
    "file move",
    "file del",
    "file undel",
    "solve name conflict",
    "unsolve name conflict",
    "file add",
    "edit",
    "replacement",
    "solve order conflict",
    "unsolve order conflict",
    "resurrect zombies",
    "root",
    "unroot",
];

/// Pure Pijul validation issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PijulValidationIssue {
    /// JSON or profile structure was malformed.
    MalformedProfile,
    /// Profile drifted from the exact reviewed cohort.
    CohortMismatch,
    /// A finite bound was zero or inconsistent.
    InvalidBound,
    /// An operation was unknown or denied.
    UnsupportedOperation,
    /// A protected operation is not implemented by this adapter.
    ProtectedMutationDenied,
    /// Repository identity, path, confirmation, or selector was invalid.
    InvalidRequest,
    /// Executor status or exit code was incoherent.
    InvalidExitStatus,
    /// Output exceeded a configured bound or was truncated.
    BoundExceeded,
    /// Loaded protocol variant did not match the request.
    ProtocolMismatch,
    /// A hash or state identifier was malformed.
    InvalidIdentity,
    /// A hash prefix was ambiguous or too short.
    AmbiguousPrefix,
    /// A channel was malformed or outside the allowlist.
    InvalidChannel,
    /// A status class or ordering fact was invalid.
    InvalidStatus,
    /// Loaded output contained prompt-like interactive text.
    InteractiveOutput,
    /// A receipt or projection would contain excluded data.
    UnsafeEvidence,
    /// An existing owner rejected the projection.
    ProjectionRejected,
}

/// Safe validation error that excludes raw output and private paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulValidationError {
    issue: PijulValidationIssue,
    field: &'static str,
}

impl PijulValidationError {
    pub(super) const fn new(issue: PijulValidationIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(&self) -> PijulValidationIssue {
        self.issue
    }

    /// Returns the field class that failed.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl fmt::Display for PijulValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Pijul field `{}` failed ({:?})",
            self.field, self.issue
        )
    }
}

impl std::error::Error for PijulValidationError {}

/// Validated, redaction-first change observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPijulChange {
    /// Full base32 BLAKE3 change hash.
    pub hash: String,
    /// Optional full channel state identity.
    pub state: Option<String>,
    /// Count of descriptive author values, without author details.
    pub author_count: u32,
    /// Whether a descriptive timestamp was present.
    pub timestamp_present: bool,
    /// Signature status without authenticity claims.
    pub signature_status: PijulSignatureStatus,
}

/// Validated, path-redacted diff observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPijulDiff {
    /// Number of bounded diff status records.
    pub record_count: u32,
    /// Number of records with conflict status classes.
    pub conflict_count: u32,
    /// Domain-separated identity over validated status classes and positions.
    pub observation_identity: String,
}

/// Validated channel observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPijulChannel {
    /// Validated channel name.
    pub name: String,
    /// Whether the CLI marked this channel as current.
    pub is_current: bool,
}

/// Validated exact-cohort output ready for safe projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidatedPijulOutput {
    /// Ordered validated log records.
    Log(Vec<ValidatedPijulChange>),
    /// Path-redacted diff status summary.
    Diff(ValidatedPijulDiff),
    /// Ordered validated channel records.
    Channels(Vec<ValidatedPijulChannel>),
}

impl ValidatedPijulOutput {
    /// Returns the bounded record count.
    #[must_use]
    pub fn record_count(&self) -> u32 {
        match self {
            Self::Log(records) => u32::try_from(records.len()).unwrap_or(u32::MAX),
            Self::Diff(summary) => summary.record_count,
            Self::Channels(records) => u32::try_from(records.len()).unwrap_or(u32::MAX),
        }
    }

    /// Returns the bounded conflict count.
    #[must_use]
    pub const fn conflict_count(&self) -> u32 {
        match self {
            Self::Diff(summary) => summary.conflict_count,
            Self::Log(_) | Self::Channels(_) => 0,
        }
    }
}

/// Safe projection bundle using existing VCS-neutral owners.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulProjectionBundle {
    /// Existing checkpoint owner record.
    pub checkpoint: CheckpointRecord,
    /// Existing snapshot-ledger owner record.
    pub snapshot: SnapshotRecord,
    /// Existing provenance owner fact.
    pub provenance: ProvenanceFact,
    /// Existing workspace owner descriptor.
    pub workspace: SpeculativeWorkspaceDescriptor,
    /// Domain-separated projection identity.
    pub projection_identity: String,
}

/// Borrowed checked executor identity needed to build one receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PijulReceiptContext<'a> {
    /// Injected executor identity.
    pub executor_identity: &'a str,
}

/// Metadata-only receipt for one validated Pijul observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulReceipt {
    /// Exact cohort identity.
    pub cohort_identity: String,
    /// Caller-supplied checked policy identity.
    pub policy_identity: String,
    /// Injected executor identity.
    pub executor_identity: String,
    /// Safe repository class without a path or locator.
    pub repository_class: String,
    /// Operation class.
    pub operation: PijulOperation,
    /// Safe status label.
    pub status: String,
    /// Bounded record count.
    pub record_count: u32,
    /// Bounded conflict count.
    pub conflict_count: u32,
    /// Bounded file count. The initial redacted query profile reports zero.
    pub file_count: u32,
    /// Confirmation state.
    pub confirmation: PijulConfirmation,
    /// Redaction class.
    pub redaction_class: String,
    /// Content omission class.
    pub omission_class: String,
    /// Output truncation class.
    pub truncation_class: String,
    /// Existing owner projection labels.
    pub projection_targets: Vec<String>,
    /// Required bounded non-claims.
    pub non_claims: Vec<String>,
    /// Domain-separated BLAKE3 receipt identity.
    pub receipt_identity: String,
}

pub(super) fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub(super) fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn validate_blake3_hex(
    value: &str,
    field: &'static str,
) -> Result<(), PijulValidationError> {
    if value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::InvalidIdentity,
            field,
        ))
    }
}

#[cfg(test)]
mod tests;
