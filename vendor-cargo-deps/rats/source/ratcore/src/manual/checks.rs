use super::*;

#[path = "checks/mapping.rs"]
mod mapping;
#[path = "checks/policy.rs"]
mod policy;
pub use mapping::SnapshotPolicy;
pub use mapping::validate_parity_manifest_gate;
pub use mapping::validate_parity_manifest_gate_with_policy;

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
    /// Renderer registry or evidence scope policy is invalid.
    InvalidRendererPolicy,
    /// Evidence adapter identity is not registered or in scope.
    UnknownEvidenceAdapter,
    /// Legacy renderer text and nominal adapter identity disagree.
    MismatchedEvidenceAdapter,
    /// Evidence adapter does not produce visual snapshot evidence.
    WrongEvidenceRole,
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
    pub(super) fn new(code: SnapshotIssueCode, subject: impl Into<String>, message: impl Into<String>) -> Self {
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

/// Validates a snapshot manifest against the built-in visual adapter scope.
#[must_use]
pub fn validate_snapshot_manifest(
    entries: &[SnapshotManifestEntry],
    fixture_ids: &std::collections::BTreeSet<crate::showcase_fixtures::FixtureId>,
    artifacts: &[SnapshotArtifact],
) -> SnapshotValidationReport {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::SnapshotVisualAdapters,
    );
    validate_snapshot_manifest_with_policy(entries, fixture_ids, artifacts, &registry, &scope)
}

/// Validates a snapshot manifest against an explicit visual evidence-adapter scope.
#[must_use]
pub fn validate_snapshot_manifest_with_policy(
    entries: &[SnapshotManifestEntry],
    fixture_ids: &std::collections::BTreeSet<crate::showcase_fixtures::FixtureId>,
    artifacts: &[SnapshotArtifact],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> SnapshotValidationReport {
    let artifact_map = artifacts
        .iter()
        .map(|artifact| (artifact.path.clone(), artifact.bytes.as_slice()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut issues = policy::registry_scope_issues(registry, scope);
    issues.reserve(entries.len());
    debug_assert!(artifact_map.len() <= artifacts.len());
    debug_assert!(issues.capacity() >= entries.len());
    let mut ids = std::collections::BTreeSet::new();
    for entry in entries {
        let identity = format!("{}:{}", entry.fixture_id, entry.evidence_adapter);
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
                entry.fixture_id.to_string(),
                "snapshot entry references a missing fixture",
            ));
        }
        issues.extend(policy::manifest_adapter_issues(entry, registry, scope));
        match artifact_map.get(&entry.artifact_path) {
            Some(bytes) if crate::interaction_replay::blake3_hex(bytes) != entry.blake3 => {
                issues.push(SnapshotIssue::new(
                    SnapshotIssueCode::StaleHash,
                    entry.artifact_path.clone(),
                    "snapshot artifact hash is stale",
                ))
            }
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
