/// Typed renderer-family support cell used by generic parity validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilySupport {
    /// Nominal renderer family identity.
    pub family: crate::targeting_core::RendererFamilyId,
    /// super::Support status vocabulary retained from the compatibility cell.
    pub status: String,
    /// Evidence links or checked commands.
    pub evidence: Vec<String>,
    /// Unsupported or divergent reason.
    pub reason: Option<String>,
}

impl FamilySupport {
    /// Creates typed renderer-family support metadata.
    #[must_use]
    pub fn new(family: impl Into<crate::targeting_core::RendererFamilyId>, status: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            status: status.into(),
            evidence: Vec::new(),
            reason: None,
        }
    }

    /// Adds evidence.
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

impl From<&super::Support> for FamilySupport {
    fn from(renderer: &super::Support) -> Self {
        Self {
            family: crate::targeting_core::RendererFamilyId::new(&renderer.renderer),
            status: renderer.status.clone(),
            evidence: renderer.evidence.clone(),
            reason: renderer.reason.clone(),
        }
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
    /// Compatibility renderer support states.
    pub renderers: Vec<super::Support>,
    /// Typed ordered family cells used by scoped validation.
    pub renderer_cells: Vec<FamilySupport>,
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
            renderer_cells: Vec::new(),
            next_action: next_action.into(),
        }
    }

    /// Adds one renderer support state.
    #[must_use]
    pub fn with_renderer(mut self, renderer: super::Support) -> Self {
        self.renderer_cells.push(FamilySupport::from(&renderer));
        self.renderers.push(renderer);
        self
    }

    /// Adds a typed renderer-family cell outside the compatibility view.
    #[must_use]
    pub fn with_renderer_cell(mut self, cell: FamilySupport) -> Self {
        self.renderer_cells.push(cell);
        self
    }
}

/// Validation report for renderer parity rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Deterministic validation issues.
    pub issues: Vec<super::Issue>,
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

/// Validates renderer parity rows against the maintained default family scope.
#[must_use]
pub fn validate(rows: &[Row]) -> Report {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::GeneralParityFamilies,
    );
    validate_with_renderer_policy(rows, &registry, &scope)
}

/// Validates renderer parity rows against an explicit ordered family scope.
#[must_use]
pub fn validate_with_renderer_policy(
    rows: &[Row],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Report {
    let scope_validation = crate::targeting_core::validate_renderer_scope(registry, scope);
    let mut issues = super::cells::scope_issues(scope, &scope_validation);
    let mut seen_features = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= scope_validation.issues.len());
    debug_assert!(seen_features.len() <= rows.len());
    for row in rows {
        if !seen_features.insert(row.feature_id.clone()) {
            issues.push(super::Issue::new(
                super::IssueCode::DuplicateFeatureId,
                row.feature_id.clone(),
                "feature_id",
                "duplicate renderer parity feature ID",
            ));
        }
        issues.extend(row_issues(row, registry, scope, scope_validation.is_valid()));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    Report {
        issues,
        rows: sorted_rows,
    }
}

fn row_issues(
    row: &Row,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
    is_scope_valid: bool,
) -> Vec<super::Issue> {
    let issue_bound = row.renderer_cells.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    if row.feature_id.trim().is_empty() || row.family.trim().is_empty() || row.core_owner.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingRequiredField,
            row.feature_id.clone(),
            "row",
            "feature ID, family, and core owner are required",
        ));
    }
    if is_scope_valid {
        issues.extend(super::cells::row_cell_issues(row, registry, scope));
    }
    let mut has_gap = false;
    for support in &row.renderer_cells {
        issues.extend(family_support_issues(row, support));
        if super::GAP_STATUSES.contains(&support.status.as_str()) {
            has_gap = true;
        }
    }
    if has_gap && row.next_action.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingNextAction,
            row.feature_id.clone(),
            "next_action",
            "renderer parity gaps require a next action",
        ));
    }
    issues
}

fn family_support_issues(row: &Row, support: &FamilySupport) -> Vec<super::Issue> {
    let mut issues = Vec::with_capacity(super::KNOWN_STATUSES.len());
    let status = support.status.as_str();
    debug_assert!(issues.capacity() == super::KNOWN_STATUSES.len());
    debug_assert!(!support.family.as_str().is_empty() || !status.is_empty());
    if !super::KNOWN_STATUSES.contains(&status) {
        issues.push(super::Issue::new(
            super::IssueCode::UnknownStatus,
            row.feature_id.clone(),
            support.family.as_str(),
            format!("unknown renderer parity status `{status}`"),
        ));
        return issues;
    }
    if status == super::STATUS_COMPLETE && support.evidence.iter().all(|entry| entry.trim().is_empty()) {
        issues.push(super::Issue::new(
            super::IssueCode::MissingEvidence,
            row.feature_id.clone(),
            support.family.as_str(),
            "complete renderer support requires evidence",
        ));
    }
    if (status == super::STATUS_UNSUPPORTED || status == super::STATUS_DIVERGENT)
        && support.reason.as_deref().unwrap_or_default().trim().is_empty()
    {
        issues.push(super::Issue::new(
            super::IssueCode::MissingReason,
            row.feature_id.clone(),
            support.family.as_str(),
            "unsupported or divergent renderer support requires a reason",
        ));
    }
    issues
}
