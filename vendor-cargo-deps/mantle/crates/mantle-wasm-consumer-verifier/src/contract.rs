//! Public consumer-facing DTOs for materialization-bundle verification.
//!
//! Every type is owned, serializable, and free of Mantle scheduler,
//! derivation, store, builder, cache, release, or CLI types. Member labels
//! carry roles and identities only; logical store paths never enter a report.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

/// Fixed verification layers, in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsumerLayer {
    /// In-memory schema, identity, stage-linkage, and bound validation.
    Structural,
    /// Capability-root byte remeasurement of every required member.
    Bytes,
}

/// Outcome of one required member at the byte layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsumerMemberStatus {
    /// Declared length and BLAKE3 identity matched the remeasured bytes.
    Matched,
    /// Required bytes could not be resolved under the capability root.
    Missing,
    /// Bytes were readable but the declared length or identity drifted.
    Mismatched,
}

/// Files and source directories have distinct identity algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsumerMemberKind {
    File,
    SourceTree,
}

/// One regular file in a verified source tree. Paths are source-relative,
/// never absolute consumer paths; payload bytes are not included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerSourceFile {
    pub relative_path: String,
    pub digest_blake3: String,
    pub size_bytes: u64,
}

/// One bounded member observation in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerMemberObservation {
    /// Stable role label, for example `final-portable` or `stage-receipt`.
    pub role: String,
    /// Declared BLAKE3 identity of the member object.
    pub digest_blake3: String,
    /// Declared byte length of the member object.
    pub size_bytes: u64,
    /// Observed identity algorithm. Absent when the member is unavailable.
    #[serde(default)]
    pub kind: Option<ConsumerMemberKind>,
    /// Exact regular-file inventory for a source tree; empty for files.
    #[serde(default)]
    pub source_files: Vec<ConsumerSourceFile>,
    /// Byte-layer outcome. Absent for structural-only reports.
    pub status: Option<ConsumerMemberStatus>,
}

/// Overall verification outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsumerVerificationStatus {
    /// Structural pass and, when the byte layer ran, every member matched.
    Verified,
    /// Structural pass, but required bytes were unavailable.
    Blocked,
    /// Structural or byte-layer validation produced blockers.
    Rejected,
}

/// Bounded safe verification report.
///
/// The report binds schemas, identities, member observations, completed
/// layers, blockers, and non-claims. It omits private absolute paths, source
/// payloads, credentials, environment values, and raw tool diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerVerificationReport {
    /// Consumer report schema identifier.
    pub schema: String,
    /// Verified bundle schema identifier.
    pub bundle_schema: String,
    /// Canonical bundle identity when the structural layer accepted it.
    pub bundle_identity_blake3: Option<String>,
    /// Runtime profile the bundle binds, when present.
    pub expected_runtime_profile_blake3: Option<String>,
    /// Member observations in declared role order.
    pub members: Vec<ConsumerMemberObservation>,
    /// Completed verification layers.
    pub layers_completed: Vec<ConsumerLayer>,
    /// Structural and byte-layer blocker codes in order.
    pub blockers: Vec<String>,
    /// Overall outcome.
    pub status: ConsumerVerificationStatus,
    /// Exact verifier non-claims. Never caller-extensible.
    pub non_claims: Vec<String>,
}
