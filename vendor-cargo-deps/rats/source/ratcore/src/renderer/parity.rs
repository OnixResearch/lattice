//! Renderer parity matrix validation.
//!
//! This module validates release-facing parity rows that compare Subwayrat and
//! Meteorite feature support without forcing both renderers to converge.

const SUBWAYRAT_LABEL: &str = "subwayrat";
const METEORITE_LABEL: &str = "meteorite";
const EMBED_COLUMN_SUBWAYRAT_TERMINAL: &str = "subwayrat-terminal";
const EMBED_COLUMN_SUBWAYRAT_RATZILLA: &str = "subwayrat-ratzilla";
const EMBED_COLUMN_METEORITE_RATZILLA: &str = "meteorite-embedded-ratzilla";
const EMBED_COLUMN_METEORITE_NATIVE_DOM: &str = "meteorite-native-dom";
const EMBEDDED_EVIDENCE_MARKER: &str = "embedded";
const STATUS_COMPLETE: &str = "complete";
const STATUS_PARTIAL: &str = "partial";
const STATUS_MISSING: &str = "missing";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_DIVERGENT: &str = "divergent";
const GAP_STATUSES: &[&str] = &[
    STATUS_PARTIAL,
    STATUS_MISSING,
    STATUS_UNSUPPORTED,
    STATUS_DIVERGENT,
];
const KNOWN_STATUSES: &[&str] = &[
    STATUS_COMPLETE,
    STATUS_PARTIAL,
    STATUS_MISSING,
    STATUS_UNSUPPORTED,
    STATUS_DIVERGENT,
];
const EMBED_REQUIRED_COLUMNS: &[&str] = &[
    EMBED_COLUMN_SUBWAYRAT_TERMINAL,
    EMBED_COLUMN_SUBWAYRAT_RATZILLA,
    EMBED_COLUMN_METEORITE_RATZILLA,
    EMBED_COLUMN_METEORITE_NATIVE_DOM,
];
#[path = "parity/shared.rs"]
mod shared;

pub use shared::{ContractCandidate, ContractCandidateReport, validate_contract_candidates};

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

/// Cross-renderer parity row for one feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Stable feature ID.
    pub feature_id: String,
    /// Feature family, such as `chrome`, `fixtures`, or `workspace`.
    pub family: String,
    /// Shared core owner or explicit renderer owner.
    pub core_owner: String,
    /// Renderer support states.
    pub renderers: Vec<Support>,
    /// Next action for gaps, follow-up work, or explicit divergence decisions.
    pub next_action: String,
}

impl Row {
    /// Creates a parity row.
    #[must_use]
    pub fn new(
        feature_id: impl Into<String>,
        family: impl Into<String>,
        core_owner: impl Into<String>,
        next_action: impl Into<String>,
    ) -> Self {
        Self {
            feature_id: feature_id.into(),
            family: family.into(),
            core_owner: core_owner.into(),
            renderers: Vec::new(),
            next_action: next_action.into(),
        }
    }

    /// Adds one renderer support state.
    #[must_use]
    pub fn with_renderer(mut self, renderer: Support) -> Self {
        self.renderers.push(renderer);
        self
    }
}

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

/// Renderer parity row for Meteorite-hosted Ratzilla surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaSurfaceRow {
    /// Stable feature ID.
    pub feature_id: String,
    /// Feature family.
    pub family: String,
    /// Shared core owner or renderer boundary owner.
    pub shared_core_owner: String,
    /// Subwayrat terminal, Subwayrat Ratzilla, Meteorite embedded-Ratzilla, and Meteorite native-DOM statuses.
    pub renderers: Vec<Support>,
    /// Diagnostics coverage evidence.
    pub diagnostics_coverage: String,
    /// Next action for gaps or divergence.
    pub next_action: String,
}

impl EmbeddedRatzillaSurfaceRow {
    /// Creates a Meteorite-hosted Ratzilla surface parity row.
    #[must_use]
    pub fn new(
        feature_id: impl Into<String>,
        family: impl Into<String>,
        shared_core_owner: impl Into<String>,
        next_action: impl Into<String>,
    ) -> Self {
        Self {
            feature_id: feature_id.into(),
            family: family.into(),
            shared_core_owner: shared_core_owner.into(),
            renderers: Vec::new(),
            diagnostics_coverage: String::new(),
            next_action: next_action.into(),
        }
    }

    /// Adds a renderer-column status.
    #[must_use]
    pub fn with_renderer(mut self, renderer: Support) -> Self {
        self.renderers.push(renderer);
        self
    }

    /// Adds diagnostics coverage evidence.
    #[must_use]
    pub fn with_diagnostics_coverage(mut self, diagnostics_coverage: impl Into<String>) -> Self {
        self.diagnostics_coverage = diagnostics_coverage.into();
        self
    }
}

/// Validation report for embedded Ratzilla surface parity rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaSurfaceReport {
    /// Deterministic validation issues.
    pub issues: Vec<Issue>,
    /// Rows sorted by stable feature ID.
    pub rows: Vec<EmbeddedRatzillaSurfaceRow>,
}

impl EmbeddedRatzillaSurfaceReport {
    /// Returns true when no embedded Ratzilla surface issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validation report for renderer parity rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Deterministic validation issues.
    pub issues: Vec<Issue>,
    /// Rows sorted by stable feature ID.
    pub rows: Vec<Row>,
}

impl Report {
    /// Returns true when no parity validation issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validates embedded Ratzilla surface parity rows and returns rows sorted by feature ID.
#[must_use]
pub fn validate_embedded_ratzilla_surface_rows(
    rows: &[EmbeddedRatzillaSurfaceRow],
) -> EmbeddedRatzillaSurfaceReport {
    let mut issues = Vec::with_capacity(rows.len());
    let mut seen_features = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= rows.len());
    for row in rows {
        if !seen_features.insert(row.feature_id.clone()) {
            issues.push(Issue::new(
                IssueCode::DuplicateFeatureId,
                row.feature_id.clone(),
                "feature_id",
                "duplicate embedded Ratzilla surface feature ID",
            ));
        }
        issues.extend(embedded_surface_row_issues(row));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    EmbeddedRatzillaSurfaceReport {
        issues,
        rows: sorted_rows,
    }
}

/// Validates renderer parity rows and returns rows sorted by feature ID.
#[must_use]
pub fn validate(rows: &[Row]) -> Report {
    let mut issues = Vec::with_capacity(rows.len());
    let mut seen_features = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= rows.len());
    debug_assert!(seen_features.len() <= rows.len());
    for row in rows {
        if !seen_features.insert(row.feature_id.clone()) {
            issues.push(Issue::new(
                IssueCode::DuplicateFeatureId,
                row.feature_id.clone(),
                "feature_id",
                "duplicate renderer parity feature ID",
            ));
        }
        issues.extend(row_issues(row));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    Report {
        issues,
        rows: sorted_rows,
    }
}

fn embedded_surface_row_issues(row: &EmbeddedRatzillaSurfaceRow) -> Vec<Issue> {
    let issue_bound = row
        .renderers
        .len()
        .saturating_add(EMBED_REQUIRED_COLUMNS.len());
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= EMBED_REQUIRED_COLUMNS.len());
    if row.feature_id.trim().is_empty()
        || row.family.trim().is_empty()
        || row.shared_core_owner.trim().is_empty()
    {
        issues.push(Issue::new(
            IssueCode::MissingRequiredField,
            row.feature_id.clone(),
            "row",
            "feature ID, family, and shared core owner are required",
        ));
    }
    let renderer_labels = row
        .renderers
        .iter()
        .map(|entry| entry.renderer.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in EMBED_REQUIRED_COLUMNS {
        if !renderer_labels.contains(required) {
            issues.push(Issue::new(
                IssueCode::MissingRendererLabel,
                row.feature_id.clone(),
                *required,
                "embedded Ratzilla row is missing required renderer metadata",
            ));
        }
    }
    let mut has_gap = false;
    for support in &row.renderers {
        issues.extend(embedded_support_issues(row, support));
        if GAP_STATUSES.contains(&support.status.as_str()) {
            has_gap = true;
        }
    }
    if row.diagnostics_coverage.trim().is_empty() {
        issues.push(Issue::new(
            IssueCode::MissingDiagnosticsCoverage,
            row.feature_id.clone(),
            "diagnostics_coverage",
            "embedded Ratzilla rows require diagnostics coverage",
        ));
    }
    if has_gap && row.next_action.trim().is_empty() {
        issues.push(Issue::new(
            IssueCode::MissingNextAction,
            row.feature_id.clone(),
            "next_action",
            "embedded Ratzilla support gaps require a next action",
        ));
    }
    issues.extend(native_dom_relationship_issues(row));
    issues
}

fn embedded_support_issues(row: &EmbeddedRatzillaSurfaceRow, support: &Support) -> Vec<Issue> {
    let mut issues = Vec::with_capacity(KNOWN_STATUSES.len());
    let status = support.status.as_str();
    debug_assert!(issues.capacity() == KNOWN_STATUSES.len());
    if !KNOWN_STATUSES.contains(&status) {
        issues.push(Issue::new(
            IssueCode::UnknownStatus,
            row.feature_id.clone(),
            support.renderer.clone(),
            format!("unknown embedded Ratzilla surface status `{status}`"),
        ));
        return issues;
    }
    if (status == STATUS_COMPLETE || status == STATUS_PARTIAL)
        && support.evidence.iter().all(|entry| entry.trim().is_empty())
    {
        issues.push(Issue::new(
            IssueCode::MissingEvidence,
            row.feature_id.clone(),
            support.renderer.clone(),
            "complete or partial embedded Ratzilla support requires evidence",
        ));
    }
    if (status == STATUS_UNSUPPORTED || status == STATUS_DIVERGENT)
        && support
            .reason
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        issues.push(Issue::new(
            IssueCode::MissingReason,
            row.feature_id.clone(),
            support.renderer.clone(),
            "unsupported or divergent embedded Ratzilla support requires a reason",
        ));
    }
    issues
}

fn native_dom_relationship_issues(row: &EmbeddedRatzillaSurfaceRow) -> Vec<Issue> {
    let Some(native_dom) = row
        .renderers
        .iter()
        .find(|support| support.renderer == EMBED_COLUMN_METEORITE_NATIVE_DOM)
    else {
        return Vec::new();
    };
    if native_dom.status != STATUS_COMPLETE {
        return Vec::new();
    }
    if native_dom.evidence.iter().all(|entry| {
        entry
            .to_ascii_lowercase()
            .contains(EMBEDDED_EVIDENCE_MARKER)
    }) {
        return vec![Issue::new(
            IssueCode::InvalidStatusRelationship,
            row.feature_id.clone(),
            EMBED_COLUMN_METEORITE_NATIVE_DOM,
            "embedded Ratzilla evidence cannot mark native Meteorite DOM parity complete",
        )];
    }
    Vec::new()
}

fn row_issues(row: &Row) -> Vec<Issue> {
    let issue_bound = row.renderers.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    if row.feature_id.trim().is_empty()
        || row.family.trim().is_empty()
        || row.core_owner.trim().is_empty()
    {
        issues.push(Issue::new(
            IssueCode::MissingRequiredField,
            row.feature_id.clone(),
            "row",
            "feature ID, family, and core owner are required",
        ));
    }
    let renderer_labels = row
        .renderers
        .iter()
        .map(|entry| entry.renderer.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [SUBWAYRAT_LABEL, METEORITE_LABEL] {
        if !renderer_labels.contains(required) {
            issues.push(Issue::new(
                IssueCode::MissingRendererLabel,
                row.feature_id.clone(),
                required,
                "renderer parity row is missing required renderer metadata",
            ));
        }
    }
    let mut has_gap = false;
    for support in &row.renderers {
        issues.extend(support_issues(row, support));
        if GAP_STATUSES.contains(&support.status.as_str()) {
            has_gap = true;
        }
    }
    if has_gap && row.next_action.trim().is_empty() {
        issues.push(Issue::new(
            IssueCode::MissingNextAction,
            row.feature_id.clone(),
            "next_action",
            "renderer parity gaps require a next action",
        ));
    }
    issues
}

fn support_issues(row: &Row, support: &Support) -> Vec<Issue> {
    let mut issues = Vec::with_capacity(KNOWN_STATUSES.len());
    let status = support.status.as_str();
    debug_assert!(issues.capacity() == KNOWN_STATUSES.len());
    debug_assert!(!support.renderer.is_empty() || !status.is_empty());
    if !KNOWN_STATUSES.contains(&status) {
        issues.push(Issue::new(
            IssueCode::UnknownStatus,
            row.feature_id.clone(),
            support.renderer.clone(),
            format!("unknown renderer parity status `{status}`"),
        ));
        return issues;
    }
    if status == STATUS_COMPLETE && support.evidence.iter().all(|entry| entry.trim().is_empty()) {
        issues.push(Issue::new(
            IssueCode::MissingEvidence,
            row.feature_id.clone(),
            support.renderer.clone(),
            "complete renderer support requires evidence",
        ));
    }
    if (status == STATUS_UNSUPPORTED || status == STATUS_DIVERGENT)
        && support
            .reason
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        issues.push(Issue::new(
            IssueCode::MissingReason,
            row.feature_id.clone(),
            support.renderer.clone(),
            "unsupported or divergent renderer support requires a reason",
        ));
    }
    issues
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
