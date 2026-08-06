//! Renderer parity matrix validation.
//!
//! This module validates release-facing parity rows that compare Subwayrat and
//! Meteorite feature support without forcing both renderers to converge.

const EMBED_COLUMN_METEORITE_NATIVE_DOM: &str = "meteorite-native-dom";
const EMBEDDED_EVIDENCE_MARKER: &str = "embedded";
const STATUS_COMPLETE: &str = "complete";
const STATUS_PARTIAL: &str = "partial";
const STATUS_MISSING: &str = "missing";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_DIVERGENT: &str = "divergent";
const GAP_STATUSES: &[&str] = &[STATUS_PARTIAL, STATUS_MISSING, STATUS_UNSUPPORTED, STATUS_DIVERGENT];
const KNOWN_STATUSES: &[&str] = &[
    STATUS_COMPLETE,
    STATUS_PARTIAL,
    STATUS_MISSING,
    STATUS_UNSUPPORTED,
    STATUS_DIVERGENT,
];
#[path = "parity/cells.rs"]
mod cells;
#[path = "parity/shared.rs"]
mod shared;

pub use shared::ContractCandidate;
pub use shared::ContractCandidateReport;
pub use shared::validate_contract_candidates;

/// One renderer's support state for a feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Support {
    /// Stable renderer label, normally `subwayrat` or `meteorite`.
    pub renderer: String,
    /// Support status: `complete`, `partial`, `missing`, `unsupported`, or `divergent`.
    pub status: String,
    /// Evidence links or checked commands proving complete or partial support.
    pub evidence: Vec<String>,
    /// Reason for unsupported or intentionally divergent support.
    pub reason: Option<String>,
}

impl Support {
    /// Creates renderer support metadata.
    #[must_use]
    pub fn new(renderer: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            renderer: renderer.into(),
            status: status.into(),
            evidence: Vec::new(),
            reason: None,
        }
    }

    /// Adds evidence for this renderer status.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Adds an unsupported or divergent reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[path = "parity/general.rs"]
mod general;
pub use general::FamilySupport;
pub use general::Report;
pub use general::Row;
pub use general::validate;
pub use general::validate_with_renderer_policy;
#[path = "parity/embedding.rs"]
mod embedding;
pub use embedding::EmbeddedRatzillaSurfaceReport;
pub use embedding::EmbeddedRatzillaSurfaceRow;
pub use embedding::validate_embedded_ratzilla_surface_rows;
pub use embedding::validate_embedded_ratzilla_surface_rows_with_policy;

/// Stable renderer parity diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueCode {
    /// Feature ID is duplicated.
    DuplicateFeatureId,
    /// Feature ID, family, or owner is empty.
    MissingRequiredField,
    /// A Subwayrat or Meteorite renderer entry is missing.
    MissingRendererLabel,
    /// A renderer status is not in the supported vocabulary.
    UnknownStatus,
    /// A complete renderer state lacks evidence.
    MissingEvidence,
    /// Unsupported or divergent support lacks a reason.
    MissingReason,
    /// A gap row lacks a next action.
    MissingNextAction,
    /// An extracted shared contract candidate lacks positive or negative test evidence.
    MissingTests,
    /// Diagnostics coverage is missing from a specialized parity row.
    MissingDiagnosticsCoverage,
    /// Renderer status relationship is invalid for the row type.
    InvalidStatusRelationship,
    /// Renderer registry or scope policy is invalid.
    InvalidRendererScope,
    /// A renderer family cell is duplicated.
    DuplicateRenderer,
    /// A renderer family cell is unknown.
    UnknownRenderer,
    /// A renderer family cell is outside the named scope.
    OutOfScopeRenderer,
}

/// Deterministic renderer parity validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Stable diagnostic code.
    pub code: IssueCode,
    /// Affected feature ID.
    pub feature_id: String,
    /// Affected renderer label or row field.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl Issue {
    pub(crate) fn new(
        code: IssueCode,
        feature_id: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            feature_id: feature_id.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
