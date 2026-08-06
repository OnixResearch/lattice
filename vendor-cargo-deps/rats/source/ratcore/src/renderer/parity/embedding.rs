/// Renderer parity row for Meteorite-hosted Ratzilla surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaSurfaceRow {
    /// Stable feature ID.
    pub feature_id: String,
    /// Feature family.
    pub family: String,
    /// Shared core owner or renderer boundary owner.
    pub shared_core_owner: String,
    /// Subwayrat terminal, Subwayrat Ratzilla, Meteorite embedded-Ratzilla, and Meteorite
    /// native-DOM statuses.
    pub renderers: Vec<super::Support>,
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
    pub fn with_renderer(mut self, renderer: super::Support) -> Self {
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
    pub issues: Vec<super::Issue>,
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

/// Validates embedded Ratzilla rows against their fixed maintained target scope.
#[must_use]
pub fn validate_embedded_ratzilla_surface_rows(rows: &[EmbeddedRatzillaSurfaceRow]) -> EmbeddedRatzillaSurfaceReport {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::EmbeddedRatzillaTargets,
    );
    validate_embedded_ratzilla_surface_rows_with_policy(rows, &registry, &scope)
}

/// Validates embedded Ratzilla rows against an explicit concrete-target scope.
#[must_use]
pub fn validate_embedded_ratzilla_surface_rows_with_policy(
    rows: &[EmbeddedRatzillaSurfaceRow],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> EmbeddedRatzillaSurfaceReport {
    let scope_validation = crate::targeting_core::validate_renderer_scope(registry, scope);
    let mut issues = super::cells::scope_issues(scope, &scope_validation);
    let mut seen_features = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= scope_validation.issues.len());
    for row in rows {
        if !seen_features.insert(row.feature_id.clone()) {
            issues.push(super::Issue::new(
                super::IssueCode::DuplicateFeatureId,
                row.feature_id.clone(),
                "feature_id",
                "duplicate embedded Ratzilla surface feature ID",
            ));
        }
        issues.extend(embedded_surface_row_issues(row, registry, scope));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    EmbeddedRatzillaSurfaceReport {
        issues,
        rows: sorted_rows,
    }
}

fn embedded_surface_row_issues(
    row: &EmbeddedRatzillaSurfaceRow,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::Issue> {
    let issue_bound = row.renderers.len().saturating_add(scope.members.len());
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= scope.members.len());
    if row.feature_id.trim().is_empty() || row.family.trim().is_empty() || row.shared_core_owner.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingRequiredField,
            row.feature_id.clone(),
            "row",
            "feature ID, family, and shared core owner are required",
        ));
    }
    issues.extend(super::cells::specialized_target_issues(&row.feature_id, &row.renderers, registry, scope));
    let mut has_gap = false;
    for support in &row.renderers {
        issues.extend(embedded_support_issues(row, support));
        if super::GAP_STATUSES.contains(&support.status.as_str()) {
            has_gap = true;
        }
    }
    if row.diagnostics_coverage.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingDiagnosticsCoverage,
            row.feature_id.clone(),
            "diagnostics_coverage",
            "embedded Ratzilla rows require diagnostics coverage",
        ));
    }
    if has_gap && row.next_action.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingNextAction,
            row.feature_id.clone(),
            "next_action",
            "embedded Ratzilla support gaps require a next action",
        ));
    }
    issues.extend(native_dom_relationship_issues(row));
    issues
}

fn embedded_support_issues(row: &EmbeddedRatzillaSurfaceRow, support: &super::Support) -> Vec<super::Issue> {
    let mut issues = Vec::with_capacity(super::KNOWN_STATUSES.len());
    let status = support.status.as_str();
    debug_assert!(issues.capacity() == super::KNOWN_STATUSES.len());
    if !super::KNOWN_STATUSES.contains(&status) {
        issues.push(super::Issue::new(
            super::IssueCode::UnknownStatus,
            row.feature_id.clone(),
            support.renderer.clone(),
            format!("unknown embedded Ratzilla surface status `{status}`"),
        ));
        return issues;
    }
    if (status == super::STATUS_COMPLETE || status == super::STATUS_PARTIAL)
        && support.evidence.iter().all(|entry| entry.trim().is_empty())
    {
        issues.push(super::Issue::new(
            super::IssueCode::MissingEvidence,
            row.feature_id.clone(),
            support.renderer.clone(),
            "complete or partial embedded Ratzilla support requires evidence",
        ));
    }
    if (status == super::STATUS_UNSUPPORTED || status == super::STATUS_DIVERGENT)
        && support.reason.as_deref().unwrap_or_default().trim().is_empty()
    {
        issues.push(super::Issue::new(
            super::IssueCode::MissingReason,
            row.feature_id.clone(),
            support.renderer.clone(),
            "unsupported or divergent embedded Ratzilla support requires a reason",
        ));
    }
    issues
}

fn native_dom_relationship_issues(row: &EmbeddedRatzillaSurfaceRow) -> Option<super::Issue> {
    let native_dom = native_dom_support(row)?;
    let has_invalid_evidence = native_dom.status == super::STATUS_COMPLETE
        && native_dom
            .evidence
            .iter()
            .all(|entry| entry.to_ascii_lowercase().contains(super::EMBEDDED_EVIDENCE_MARKER));
    has_invalid_evidence.then(|| {
        super::Issue::new(
            super::IssueCode::InvalidStatusRelationship,
            row.feature_id.clone(),
            super::EMBED_COLUMN_METEORITE_NATIVE_DOM,
            "embedded Ratzilla evidence cannot mark native Meteorite DOM parity complete",
        )
    })
}

fn native_dom_support(row: &EmbeddedRatzillaSurfaceRow) -> Option<&super::Support> {
    row.renderers.iter().find(|support| support.renderer == super::EMBED_COLUMN_METEORITE_NATIVE_DOM)
}
