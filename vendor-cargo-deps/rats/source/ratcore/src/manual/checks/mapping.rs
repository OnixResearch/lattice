/// Registry and visual-adapter scope used by parity evidence validation.
pub struct SnapshotPolicy<'a> {
    /// Validated renderer registry.
    pub registry: &'a crate::targeting_core::RendererRegistry,
    /// Ordered visual evidence-adapter scope.
    pub scope: &'a crate::targeting_core::RendererRequirementScope,
}

impl<'a> SnapshotPolicy<'a> {
    /// Creates snapshot evidence policy inputs.
    #[must_use]
    pub const fn new(
        registry: &'a crate::targeting_core::RendererRegistry,
        scope: &'a crate::targeting_core::RendererRequirementScope,
    ) -> Self {
        Self { registry, scope }
    }
}

/// Validates parity evidence against the built-in visual adapter mapping.
#[must_use]
pub fn validate_parity_manifest_gate(
    rows: &[crate::showcase_fixtures::FixtureParityRow],
    entries: &[super::SnapshotManifestEntry],
    pages: &[super::GeneratedComponentPage],
    artifacts: &[super::SnapshotArtifact],
) -> super::SnapshotValidationReport {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::SnapshotVisualAdapters,
    );
    validate_parity_manifest_gate_with_policy(rows, entries, pages, artifacts, SnapshotPolicy::new(&registry, &scope))
}

/// Validates parity evidence through an explicit registry and visual adapter scope.
#[must_use]
pub fn validate_parity_manifest_gate_with_policy(
    rows: &[crate::showcase_fixtures::FixtureParityRow],
    entries: &[super::SnapshotManifestEntry],
    pages: &[super::GeneratedComponentPage],
    artifacts: &[super::SnapshotArtifact],
    policy: SnapshotPolicy<'_>,
) -> super::SnapshotValidationReport {
    let fixture_ids = parity_fixture_ids(rows);
    let mut issues =
        super::validate_snapshot_manifest_with_policy(entries, &fixture_ids, artifacts, policy.registry, policy.scope)
            .issues;
    let indexes = EvidenceIndexes::new(pages, entries);
    for row in rows {
        for cell in row.renderer_cells.iter().filter(|cell| cell.supported) {
            issues.extend(parity_cell_issues(row, cell, &indexes, policy.registry));
        }
    }
    super::SnapshotValidationReport { issues }
}

fn parity_fixture_ids(
    rows: &[crate::showcase_fixtures::FixtureParityRow],
) -> std::collections::BTreeSet<crate::showcase_fixtures::FixtureId> {
    rows.iter().map(|row| row.fixture_id.clone()).collect()
}

struct EvidenceIndexes<'a> {
    pages: std::collections::BTreeSet<(&'a str, &'a str)>,
    entries: std::collections::BTreeSet<(&'a str, &'a crate::targeting_core::EvidenceAdapterId)>,
}

impl<'a> EvidenceIndexes<'a> {
    fn new(pages: &'a [super::GeneratedComponentPage], entries: &'a [super::SnapshotManifestEntry]) -> Self {
        Self {
            pages: pages.iter().map(|page| (page.fixture_id.as_str(), page.path.as_str())).collect(),
            entries: entries.iter().map(|entry| (entry.fixture_id.as_str(), &entry.evidence_adapter)).collect(),
        }
    }
}

fn parity_cell_issues(
    row: &crate::showcase_fixtures::FixtureParityRow,
    cell: &crate::showcase_fixtures::FixtureParityCell,
    indexes: &EvidenceIndexes<'_>,
    registry: &crate::targeting_core::RendererRegistry,
) -> Vec<super::SnapshotIssue> {
    let adapter = super::policy::visual_adapter_for_family(registry, &cell.family);
    [
        missing_page_issue(row, cell, indexes),
        missing_manifest_issue(row, cell, indexes, adapter),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn missing_page_issue(
    row: &crate::showcase_fixtures::FixtureParityRow,
    cell: &crate::showcase_fixtures::FixtureParityCell,
    indexes: &EvidenceIndexes<'_>,
) -> Option<super::SnapshotIssue> {
    (!indexes.pages.contains(&(row.fixture_id.as_str(), row.documentation_target.as_str()))).then(|| {
        super::SnapshotIssue::new(
            super::SnapshotIssueCode::MissingDocPage,
            row.documentation_target.clone(),
            format!("fixture `{}` lacks generated documentation for `{}`", row.fixture_id, cell.family),
        )
    })
}

fn missing_manifest_issue(
    row: &crate::showcase_fixtures::FixtureParityRow,
    cell: &crate::showcase_fixtures::FixtureParityCell,
    indexes: &EvidenceIndexes<'_>,
    adapter: Option<&crate::targeting_core::EvidenceAdapterDescriptor>,
) -> Option<super::SnapshotIssue> {
    let Some(adapter) = adapter else {
        return Some(super::SnapshotIssue::new(
            super::SnapshotIssueCode::InvalidRendererPolicy,
            cell.family.as_str(),
            "supported fixture family has no visual evidence adapter mapping",
        ));
    };
    (!indexes.entries.contains(&(row.fixture_id.as_str(), &adapter.id))).then(|| {
        super::SnapshotIssue::new(
            super::SnapshotIssueCode::MissingManifestEntry,
            format!("{}:{}", row.fixture_id, adapter.id),
            "fixture parity row lacks snapshot manifest evidence",
        )
    })
}
