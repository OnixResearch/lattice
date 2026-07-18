//! Shared contract extraction inventory validation.

const EXTRACTED_ISSUE_BOUND: usize = 2;
const SINGLE_ISSUE_BOUND: usize = 1;

const STATUS_EXTRACT_NOW: &str = "extract-now";
const STATUS_EXTRACTED: &str = "extracted";
const STATUS_RENDERER_OWNED: &str = "renderer-owned";
const STATUS_BLOCKED: &str = "blocked";
const STATUS_INTENTIONALLY_DIVERGENT: &str = "intentionally-divergent";
const STATUS_NEEDS_EVIDENCE: &str = "needs-evidence";
const KNOWN_STATUSES: &[&str] = &[
    STATUS_EXTRACT_NOW,
    STATUS_EXTRACTED,
    STATUS_RENDERER_OWNED,
    STATUS_BLOCKED,
    STATUS_INTENTIONALLY_DIVERGENT,
    STATUS_NEEDS_EVIDENCE,
];
const INCOMPLETE_STATUSES: &[&str] = &[STATUS_EXTRACT_NOW, STATUS_BLOCKED, STATUS_NEEDS_EVIDENCE];
const REASON_STATUSES: &[&str] = &[STATUS_RENDERER_OWNED, STATUS_INTENTIONALLY_DIVERGENT];

/// Shared contract extraction candidate inventory row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractCandidate {
    /// Stable feature ID.
    pub feature_id: String,
    /// Feature family, such as `chrome`, `workspace`, or `pointer`.
    pub family: String,
    /// Current Subwayrat owner or status summary.
    pub subwayrat_status: String,
    /// Current Meteorite owner or status summary.
    pub meteorite_status: String,
    /// Proposed Ratcore owner, or an explicit renderer-owned decision.
    pub proposed_owner: String,
    /// Extraction status: `extract-now`, `extracted`, `renderer-owned`, `blocked`, `intentionally-divergent`, or `needs-evidence`.
    pub extraction_status: String,
    /// Evidence links, commands, or source modules supporting the decision.
    pub evidence: Vec<String>,
    /// Positive/negative test evidence for extracted shared contracts.
    pub tests: Vec<String>,
    /// Reason for renderer-owned or intentionally divergent behavior.
    pub reason: Option<String>,
    /// Next action for incomplete, blocked, or upkeep decisions.
    pub next_action: String,
}

impl ContractCandidate {
    /// Creates a shared contract candidate row.
    #[must_use]
    pub fn new(
        feature_id: impl Into<String>,
        family: impl Into<String>,
        proposed_owner: impl Into<String>,
        extraction_status: impl Into<String>,
    ) -> Self {
        Self {
            feature_id: feature_id.into(),
            family: family.into(),
            subwayrat_status: String::new(),
            meteorite_status: String::new(),
            proposed_owner: proposed_owner.into(),
            extraction_status: extraction_status.into(),
            evidence: Vec::new(),
            tests: Vec::new(),
            reason: None,
            next_action: String::new(),
        }
    }

    /// Adds the current Subwayrat owner or status summary.
    #[must_use]
    pub fn with_subwayrat_status(mut self, status: impl Into<String>) -> Self {
        self.subwayrat_status = status.into();
        self
    }

    /// Adds the current Meteorite owner or status summary.
    #[must_use]
    pub fn with_meteorite_status(mut self, status: impl Into<String>) -> Self {
        self.meteorite_status = status.into();
        self
    }

    /// Adds evidence for this candidate decision.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Adds test evidence for an extracted shared contract.
    #[must_use]
    pub fn with_test(mut self, test: impl Into<String>) -> Self {
        self.tests.push(test.into());
        self
    }

    /// Adds a reason for renderer-owned or intentionally divergent behavior.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Adds the next action for this candidate decision.
    #[must_use]
    pub fn with_next_action(mut self, next_action: impl Into<String>) -> Self {
        self.next_action = next_action.into();
        self
    }
}

/// Validation report for shared contract extraction candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractCandidateReport {
    /// Deterministic validation issues.
    pub issues: Vec<super::Issue>,
    /// Candidate rows sorted by stable feature ID.
    pub rows: Vec<ContractCandidate>,
}

impl ContractCandidateReport {
    /// Returns true when no shared contract candidate issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validates shared contract extraction candidate rows and sorts them by feature ID.
#[must_use]
pub fn validate_contract_candidates(rows: &[ContractCandidate]) -> ContractCandidateReport {
    let mut issues = Vec::with_capacity(rows.len());
    let mut seen_features = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= rows.len());
    for row in rows {
        if !seen_features.insert(row.feature_id.clone()) {
            issues.push(super::Issue::new(
                super::IssueCode::DuplicateFeatureId,
                row.feature_id.clone(),
                "feature_id",
                "duplicate shared contract candidate feature ID",
            ));
        }
        issues.extend(candidate_issues(row));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    ContractCandidateReport {
        issues,
        rows: sorted_rows,
    }
}

fn candidate_issues(row: &ContractCandidate) -> Vec<super::Issue> {
    let issue_bound = KNOWN_STATUSES.len().saturating_add(SINGLE_ISSUE_BOUND);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    if has_missing_required_field(row) {
        issues.push(super::Issue::new(
            super::IssueCode::MissingRequiredField,
            row.feature_id.clone(),
            "row",
            "feature ID, family, renderer statuses, proposed owner, and extraction status are required",
        ));
    }
    let status = row.extraction_status.as_str();
    if !KNOWN_STATUSES.contains(&status) {
        issues.push(super::Issue::new(
            super::IssueCode::UnknownStatus,
            row.feature_id.clone(),
            "extraction_status",
            format!("unknown shared contract extraction status `{status}`"),
        ));
        return issues;
    }
    issues.extend(extracted_issues(row, status));
    issues.extend(reason_issues(row, status));
    issues.extend(next_action_issues(row, status));
    issues
}

fn has_missing_required_field(row: &ContractCandidate) -> bool {
    let required_fields = [
        row.feature_id.as_str(),
        row.family.as_str(),
        row.subwayrat_status.as_str(),
        row.meteorite_status.as_str(),
        row.proposed_owner.as_str(),
        row.extraction_status.as_str(),
    ];
    required_fields.iter().any(|field| field.trim().is_empty())
}

fn extracted_issues(row: &ContractCandidate, status: &str) -> Vec<super::Issue> {
    let mut issues = Vec::with_capacity(EXTRACTED_ISSUE_BOUND);
    debug_assert!(issues.capacity() >= EXTRACTED_ISSUE_BOUND);
    if status != STATUS_EXTRACTED {
        return issues;
    }
    debug_assert_eq!(status, STATUS_EXTRACTED);
    if row.evidence.iter().all(|entry| entry.trim().is_empty()) {
        issues.push(super::Issue::new(
            super::IssueCode::MissingEvidence,
            row.feature_id.clone(),
            "evidence",
            "extracted shared contract candidates require evidence",
        ));
    }
    if row.tests.iter().all(|entry| entry.trim().is_empty()) {
        issues.push(super::Issue::new(
            super::IssueCode::MissingTests,
            row.feature_id.clone(),
            "tests",
            "extracted shared contract candidates require positive and negative test evidence",
        ));
    }
    issues
}

fn reason_issues(row: &ContractCandidate, status: &str) -> Vec<super::Issue> {
    let mut issues = Vec::with_capacity(SINGLE_ISSUE_BOUND);
    if REASON_STATUSES.contains(&status)
        && row.reason.as_deref().unwrap_or_default().trim().is_empty()
    {
        issues.push(super::Issue::new(
            super::IssueCode::MissingReason,
            row.feature_id.clone(),
            "reason",
            "renderer-owned or intentionally divergent candidates require a reason",
        ));
    }
    issues
}

fn next_action_issues(row: &ContractCandidate, status: &str) -> Vec<super::Issue> {
    let mut issues = Vec::with_capacity(SINGLE_ISSUE_BOUND);
    if INCOMPLETE_STATUSES.contains(&status) && row.next_action.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingNextAction,
            row.feature_id.clone(),
            "next_action",
            "incomplete shared contract candidates require a next action",
        ));
    }
    issues
}
