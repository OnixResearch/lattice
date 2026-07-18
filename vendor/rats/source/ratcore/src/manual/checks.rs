use super::*;

/// Stable snapshot validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapshotIssueCode {
    /// Manifest entry references a missing fixture.
    MissingFixture,
    /// Artifact bytes are missing.
    MissingArtifact,
    /// Fixture/renderer entry is duplicated.
    DuplicateId,
    /// Renderer label is unsupported.
    UnsupportedRenderer,
    /// Artifact hash does not match the manifest.
    StaleHash,
    /// Documentation source is incomplete.
    MissingSnippet,
    /// Parity baseline support lacks a generated component page.
    MissingDocPage,
    /// Parity baseline support lacks a snapshot manifest entry.
    MissingManifestEntry,
    /// Checked-in generated documentation no longer matches generated output.
    StaleGeneratedDoc,
}

/// Deterministic snapshot validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotIssue {
    /// Stable issue code.
    pub code: SnapshotIssueCode,
    /// Manifest entry or field associated with the issue.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl SnapshotIssue {
    pub(super) fn new(
        code: SnapshotIssueCode,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Snapshot validation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotValidationReport {
    /// Deterministic validation issues.
    pub issues: Vec<SnapshotIssue>,
}

impl SnapshotValidationReport {
    /// Returns true when no issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validates a snapshot manifest against known fixtures and artifacts.
#[must_use]
pub fn validate_snapshot_manifest(
    entries: &[SnapshotManifestEntry],
    fixture_ids: &std::collections::BTreeSet<String>,
    artifacts: &[SnapshotArtifact],
) -> SnapshotValidationReport {
    let artifact_map = artifacts
        .iter()
        .map(|artifact| (artifact.path.clone(), artifact.bytes.as_slice()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut issues = Vec::with_capacity(entries.len());
    debug_assert!(artifact_map.len() <= artifacts.len());
    debug_assert!(issues.capacity() >= entries.len());
    let mut ids = std::collections::BTreeSet::new();
    for entry in entries {
        let identity = format!("{}:{}", entry.fixture_id, entry.renderer);
        if !ids.insert(identity.clone()) {
            issues.push(SnapshotIssue::new(
                SnapshotIssueCode::DuplicateId,
                identity,
                "duplicate fixture renderer snapshot entry",
            ));
        }
        if !fixture_ids.contains(&entry.fixture_id) {
            issues.push(SnapshotIssue::new(
                SnapshotIssueCode::MissingFixture,
                entry.fixture_id.clone(),
                "snapshot entry references a missing fixture",
            ));
        }
        if !super::fixtures::SUPPORTED_RENDERERS.contains(&entry.renderer.as_str()) {
            issues.push(SnapshotIssue::new(
                SnapshotIssueCode::UnsupportedRenderer,
                entry.renderer.clone(),
                "snapshot renderer is unsupported",
            ));
        }
        match artifact_map.get(&entry.artifact_path) {
            Some(bytes) if crate::interaction_replay::blake3_hex(bytes) != entry.blake3 => issues
                .push(SnapshotIssue::new(
                    SnapshotIssueCode::StaleHash,
                    entry.artifact_path.clone(),
                    "snapshot artifact hash is stale",
                )),
            Some(_) => {}
            None => issues.push(SnapshotIssue::new(
                SnapshotIssueCode::MissingArtifact,
                entry.artifact_path.clone(),
                "snapshot artifact is missing",
            )),
        }
    }
    SnapshotValidationReport { issues }
}

/// Named input for generated-document freshness validation.
pub struct GeneratedDocumentCheck<'a> {
    /// Stable document path or subject.
    pub subject: String,
    /// Freshly generated document contents.
    pub expected: &'a str,
    /// Checked-in document contents.
    pub checked_in: &'a str,
}

/// Validates checked-in generated documentation against freshly generated content.
#[must_use]
pub fn validate_generated_document(check: GeneratedDocumentCheck<'_>) -> SnapshotValidationReport {
    if check.expected == check.checked_in {
        SnapshotValidationReport { issues: Vec::new() }
    } else {
        SnapshotValidationReport {
            issues: vec![SnapshotIssue::new(
                SnapshotIssueCode::StaleGeneratedDoc,
                check.subject,
                "checked-in generated documentation is stale",
            )],
        }
    }
}

/// Validates snapshot documentation evidence required by fixture parity rows.
#[must_use]
pub fn validate_parity_manifest_gate(
    rows: &[crate::showcase_fixtures::FixtureParityRow],
    entries: &[SnapshotManifestEntry],
    pages: &[GeneratedComponentPage],
    artifacts: &[SnapshotArtifact],
) -> SnapshotValidationReport {
    let fixture_ids = rows
        .iter()
        .map(|row| row.fixture_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut issues = validate_snapshot_manifest(entries, &fixture_ids, artifacts).issues;
    let page_index = pages
        .iter()
        .map(|page| ((page.fixture_id.as_str(), page.path.as_str()), page))
        .collect::<std::collections::BTreeMap<_, _>>();
    let entry_index = entries
        .iter()
        .map(|entry| ((entry.fixture_id.as_str(), entry.renderer.as_str()), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    debug_assert!(page_index.len() <= pages.len());
    debug_assert!(entry_index.len() <= entries.len());
    for row in rows {
        for renderer in row.renderers.iter().filter(|renderer| renderer.supported) {
            let snapshot_renderer = snapshot_renderer_label(renderer.renderer);
            if !page_index
                .contains_key(&(row.fixture_id.as_str(), row.documentation_target.as_str()))
            {
                issues.push(SnapshotIssue::new(
                    SnapshotIssueCode::MissingDocPage,
                    row.documentation_target.clone(),
                    format!(
                        "fixture `{}` lacks generated documentation for `{}`",
                        row.fixture_id,
                        renderer.renderer.label()
                    ),
                ));
            }
            if !entry_index.contains_key(&(row.fixture_id.as_str(), snapshot_renderer)) {
                issues.push(SnapshotIssue::new(
                    SnapshotIssueCode::MissingManifestEntry,
                    format!("{}:{snapshot_renderer}", row.fixture_id),
                    "fixture parity row lacks snapshot manifest evidence",
                ));
            }
        }
    }
    SnapshotValidationReport { issues }
}

fn snapshot_renderer_label(renderer: crate::showcase_fixtures::ShowcaseRenderer) -> &'static str {
    match renderer {
        crate::showcase_fixtures::ShowcaseRenderer::Subwayrat => {
            super::fixtures::RENDERER_SUBWAYRAT_TYPST
        }
        crate::showcase_fixtures::ShowcaseRenderer::Meteorite => {
            super::fixtures::RENDERER_METEORITE_DOM
        }
    }
}
