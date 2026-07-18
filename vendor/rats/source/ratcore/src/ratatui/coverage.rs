//! `Ratatui` API-family coverage inventory validation.
//!
//! The core in this module is renderer-neutral: callers provide in-memory rows,
//! and validation returns deterministic diagnostics without reading files or
//! renderer state.

#[path = "coverage/baseline.rs"]
mod baseline;

const STATUS_PASS_THROUGH: &str = "pass-through";
const STATUS_WRAPPED: &str = "wrapped";
const STATUS_SHOWCASE_ONLY: &str = "showcase-only";
const STATUS_RENDERER_OWNED: &str = "renderer-owned";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_NEEDS_EVIDENCE: &str = "needs-evidence";
const COMPLETE_STATUSES: &[&str] = &[
    STATUS_PASS_THROUGH,
    STATUS_WRAPPED,
    STATUS_SHOWCASE_ONLY,
    STATUS_RENDERER_OWNED,
];
const KNOWN_STATUSES: &[&str] = &[
    STATUS_PASS_THROUGH,
    STATUS_WRAPPED,
    STATUS_SHOWCASE_ONLY,
    STATUS_RENDERER_OWNED,
    STATUS_UNSUPPORTED,
    STATUS_NEEDS_EVIDENCE,
];
const BASELINE_ROW_COUNT: usize = 17;

/// One checked-in `Ratatui` API-family coverage decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageRow {
    /// Stable feature ID for the API family.
    pub feature_id: String,
    /// Human-readable `Ratatui` API family.
    pub api_family: String,
    /// Rats workspace owner for the decision.
    pub owner: String,
    /// Coverage status vocabulary value.
    pub status: String,
    /// Optional relationship to `Ratcore`, Meteorite, or a renderer boundary.
    pub related_surface: Option<String>,
    /// Checked docs, code, tests, or commands that support complete decisions.
    pub evidence: Vec<String>,
    /// Required reason for unsupported decisions.
    pub reason: Option<String>,
    /// Required follow-up for incomplete decisions.
    pub next_action: String,
}

impl CoverageRow {
    /// Creates a coverage row with required fields.
    #[must_use]
    pub fn new(
        feature_id: impl Into<String>,
        api_family: impl Into<String>,
        owner: impl Into<String>,
        status: impl Into<String>,
        next_action: impl Into<String>,
    ) -> Self {
        Self {
            feature_id: feature_id.into(),
            api_family: api_family.into(),
            owner: owner.into(),
            status: status.into(),
            related_surface: None,
            evidence: Vec::new(),
            reason: None,
            next_action: next_action.into(),
        }
    }

    /// Adds an implementation or documentation evidence pointer.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Adds a related Rats surface for this `Ratatui` family.
    #[must_use]
    pub fn with_related_surface(mut self, related_surface: impl Into<String>) -> Self {
        self.related_surface = Some(related_surface.into());
        self
    }

    /// Adds an unsupported or divergent reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Stable `Ratatui` coverage diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoverageIssueCode {
    /// Feature ID is duplicated.
    DuplicateFeatureId,
    /// Feature ID, API family, owner, or status is empty.
    MissingRequiredField,
    /// Status is not in the supported vocabulary.
    UnknownStatus,
    /// Complete coverage decision lacks evidence.
    MissingEvidence,
    /// Unsupported coverage decision lacks a reason.
    MissingReason,
    /// Incomplete coverage decision lacks a next action.
    MissingNextAction,
}

/// Deterministic `Ratatui` coverage validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageIssue {
    /// Stable diagnostic code.
    pub code: CoverageIssueCode,
    /// Affected feature ID.
    pub feature_id: String,
    /// Affected field name.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl CoverageIssue {
    fn new(
        code: CoverageIssueCode,
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

/// Validation report for `Ratatui` coverage rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageReport {
    /// Deterministic validation issues.
    pub issues: Vec<CoverageIssue>,
    /// Rows sorted by stable feature ID.
    pub rows: Vec<CoverageRow>,
}

impl CoverageReport {
    /// Returns true when no coverage validation issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Returns the checked-in baseline `Ratatui` API-family coverage rows.
#[must_use]
pub fn baseline_ratatui_coverage_rows() -> Vec<CoverageRow> {
    baseline::rows()
}

/// Validates `Ratatui` coverage rows and returns rows sorted by feature ID.
#[must_use]
pub fn validate_ratatui_coverage(rows: &[CoverageRow]) -> CoverageReport {
    let mut issues = Vec::with_capacity(rows.len());
    let mut seen_features = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= rows.len());
    debug_assert!(seen_features.len() <= rows.len());
    for row in rows {
        if !seen_features.insert(row.feature_id.clone()) {
            issues.push(CoverageIssue::new(
                CoverageIssueCode::DuplicateFeatureId,
                row.feature_id.clone(),
                "feature_id",
                "duplicate Ratatui coverage feature ID",
            ));
        }
        issues.extend(row_issues(row));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    CoverageReport {
        issues,
        rows: sorted_rows,
    }
}

fn row_issues(row: &CoverageRow) -> Vec<CoverageIssue> {
    let mut issues = Vec::with_capacity(KNOWN_STATUSES.len());
    debug_assert!(issues.capacity() == KNOWN_STATUSES.len());
    if has_missing_required_field(row) {
        issues.push(CoverageIssue::new(
            CoverageIssueCode::MissingRequiredField,
            row.feature_id.clone(),
            "row",
            "feature ID, API family, owner, and status are required",
        ));
    }
    let status = row.status.as_str();
    if !KNOWN_STATUSES.contains(&status) {
        issues.push(CoverageIssue::new(
            CoverageIssueCode::UnknownStatus,
            row.feature_id.clone(),
            "status",
            format!("unknown Ratatui coverage status `{status}`"),
        ));
        return issues;
    }
    if COMPLETE_STATUSES.contains(&status)
        && row.evidence.iter().all(|entry| entry.trim().is_empty())
    {
        issues.push(CoverageIssue::new(
            CoverageIssueCode::MissingEvidence,
            row.feature_id.clone(),
            "evidence",
            "complete Ratatui coverage decisions require evidence",
        ));
    }
    if status == STATUS_UNSUPPORTED && row.reason.as_deref().unwrap_or_default().trim().is_empty() {
        issues.push(CoverageIssue::new(
            CoverageIssueCode::MissingReason,
            row.feature_id.clone(),
            "reason",
            "unsupported Ratatui coverage decisions require a reason",
        ));
    }
    if status == STATUS_NEEDS_EVIDENCE && row.next_action.trim().is_empty() {
        issues.push(CoverageIssue::new(
            CoverageIssueCode::MissingNextAction,
            row.feature_id.clone(),
            "next_action",
            "incomplete Ratatui coverage decisions require a next action",
        ));
    }
    issues
}

fn has_missing_required_field(row: &CoverageRow) -> bool {
    let is_feature_empty = row.feature_id.trim().is_empty();
    let is_family_empty = row.api_family.trim().is_empty();
    let is_owner_empty = row.owner.trim().is_empty();
    let is_status_empty = row.status.trim().is_empty();
    is_feature_empty || is_family_empty || is_owner_empty || is_status_empty
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
