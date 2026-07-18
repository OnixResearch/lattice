//! Full mouse support matrix validation.

#[path = "matrix/checks.rs"]
mod checks;

const RENDERER_SUBWAYRAT_TERMINAL: &str = "subwayrat-terminal";
const RENDERER_SUBWAYRAT_RATZILLA: &str = "subwayrat-ratzilla";
const RENDERER_METEORITE_DOM: &str = "meteorite-dom";
const STATUS_COMPLETE: &str = "complete";
const STATUS_PARTIAL: &str = "partial";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_BLOCKED: &str = "blocked";
const STATUS_DIVERGENT: &str = "divergent";
const REQUIRED_RENDERERS: &[&str] = &[
    RENDERER_SUBWAYRAT_TERMINAL,
    RENDERER_SUBWAYRAT_RATZILLA,
    RENDERER_METEORITE_DOM,
];
const KNOWN_STATUSES: &[&str] = &[
    STATUS_COMPLETE,
    STATUS_PARTIAL,
    STATUS_UNSUPPORTED,
    STATUS_BLOCKED,
    STATUS_DIVERGENT,
];
const GAP_STATUSES: &[&str] = &[
    STATUS_PARTIAL,
    STATUS_UNSUPPORTED,
    STATUS_BLOCKED,
    STATUS_DIVERGENT,
];

/// One renderer status in the full mouse support matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullMouseRendererStatus {
    /// Renderer label.
    pub renderer: String,
    /// Support status: `complete`, `partial`, `unsupported`, `blocked`, or `divergent`.
    pub status: String,
    /// Evidence proving complete or partial support.
    pub evidence: Vec<String>,
    /// Reason for unsupported, blocked, or divergent support.
    pub reason: Option<String>,
}

impl FullMouseRendererStatus {
    /// Creates a renderer status row cell.
    #[must_use]
    pub fn new(renderer: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            renderer: renderer.into(),
            status: status.into(),
            evidence: Vec::new(),
            reason: None,
        }
    }

    /// Adds evidence to the renderer status.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Adds a blocked, unsupported, or divergent reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Full mouse support matrix row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullMouseSupportRow {
    /// Stable capability ID.
    pub capability_id: String,
    /// Pointer event class.
    pub event_class: String,
    /// Surface family.
    pub surface_family: String,
    /// Shared core owner.
    pub shared_core_owner: String,
    /// Renderer status cells.
    pub renderers: Vec<FullMouseRendererStatus>,
    /// Diagnostic coverage evidence.
    pub diagnostics_coverage: String,
    /// Next action for gaps or upkeep.
    pub next_action: String,
}

impl FullMouseSupportRow {
    /// Creates a full mouse matrix row with a stable capability ID.
    #[must_use]
    pub fn new(capability_id: impl Into<String>) -> Self {
        Self {
            capability_id: capability_id.into(),
            event_class: String::new(),
            surface_family: String::new(),
            shared_core_owner: String::new(),
            renderers: Vec::new(),
            diagnostics_coverage: String::new(),
            next_action: String::new(),
        }
    }

    /// Adds the pointer event class.
    #[must_use]
    pub fn with_event_class(mut self, event_class: impl Into<String>) -> Self {
        self.event_class = event_class.into();
        self
    }

    /// Adds the surface family.
    #[must_use]
    pub fn with_surface_family(mut self, surface_family: impl Into<String>) -> Self {
        self.surface_family = surface_family.into();
        self
    }

    /// Adds the shared core owner.
    #[must_use]
    pub fn with_shared_core_owner(mut self, shared_core_owner: impl Into<String>) -> Self {
        self.shared_core_owner = shared_core_owner.into();
        self
    }

    /// Adds diagnostic coverage evidence.
    #[must_use]
    pub fn with_diagnostics_coverage(mut self, diagnostics_coverage: impl Into<String>) -> Self {
        self.diagnostics_coverage = diagnostics_coverage.into();
        self
    }

    /// Adds the next action.
    #[must_use]
    pub fn with_next_action(mut self, next_action: impl Into<String>) -> Self {
        self.next_action = next_action.into();
        self
    }

    /// Adds a renderer status cell.
    #[must_use]
    pub fn with_renderer(mut self, renderer: FullMouseRendererStatus) -> Self {
        self.renderers.push(renderer);
        self
    }
}

/// Stable full mouse matrix diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FullMouseSupportIssueCode {
    /// Capability ID is duplicated.
    DuplicateCapabilityId,
    /// Required row field is empty.
    MissingRequiredField,
    /// A required renderer status is missing.
    MissingRenderer,
    /// Renderer status is not recognized.
    UnknownStatus,
    /// Complete or partial support lacks evidence.
    MissingEvidence,
    /// Unsupported, blocked, or divergent support lacks a reason.
    MissingReason,
    /// Diagnostic coverage is empty.
    MissingDiagnosticsCoverage,
    /// A gap row lacks a next action.
    MissingNextAction,
}

/// Deterministic full mouse matrix validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullMouseSupportIssue {
    /// Stable issue code.
    pub code: FullMouseSupportIssueCode,
    /// Affected capability ID.
    pub capability_id: String,
    /// Affected field or renderer.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl FullMouseSupportIssue {
    fn new(
        code: FullMouseSupportIssueCode,
        capability_id: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            capability_id: capability_id.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Validation report for full mouse matrix rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullMouseSupportReport {
    /// Validation issues.
    pub issues: Vec<FullMouseSupportIssue>,
    /// Rows sorted by capability ID.
    pub rows: Vec<FullMouseSupportRow>,
}

impl FullMouseSupportReport {
    /// Returns true when the matrix is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validates full mouse support matrix rows.
#[must_use]
pub fn validate_full_mouse_support_matrix(rows: &[FullMouseSupportRow]) -> FullMouseSupportReport {
    let mut issues = Vec::with_capacity(rows.len());
    let mut seen = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= rows.len());
    for row in rows {
        if !seen.insert(row.capability_id.clone()) {
            issues.push(FullMouseSupportIssue::new(
                FullMouseSupportIssueCode::DuplicateCapabilityId,
                row.capability_id.clone(),
                "capability_id",
                "duplicate full mouse capability ID",
            ));
        }
        issues.extend(checks::row_issues(row));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    FullMouseSupportReport {
        issues,
        rows: sorted_rows,
    }
}

#[cfg(test)]
#[path = "cap/tests.rs"]
mod tests;
