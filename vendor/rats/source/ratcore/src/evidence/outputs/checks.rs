/// Stable renderer snapshot artifact validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkspaceShellRendererSnapshotIssueCode {
    /// Artifact references a fixture that is not present in replay evidence.
    MissingFixture,
    /// A fixture/renderer/path identity appears more than once.
    DuplicateArtifact,
    /// Renderer label is not part of the workspace shell artifact contract.
    UnsupportedRenderer,
    /// Generator identity is not accepted.
    UnsupportedGenerator,
    /// Artifact bytes are empty.
    EmptyArtifact,
    /// Artifact BLAKE3 hash does not match its bytes.
    StaleHash,
    /// A fixture is missing a required renderer artifact.
    MissingRenderer,
}

/// Renderer snapshot artifact validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceShellRendererSnapshotIssue {
    /// Stable issue code.
    pub code: WorkspaceShellRendererSnapshotIssueCode,
    /// Fixture, renderer, or artifact subject.
    pub subject: String,
    /// Human-readable issue details.
    pub message: String,
}

impl WorkspaceShellRendererSnapshotIssue {
    fn new(
        code: WorkspaceShellRendererSnapshotIssueCode,
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

/// Validation report for workspace shell renderer snapshot artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceShellRendererSnapshotReport {
    /// Deterministic validation issues.
    pub issues: Vec<WorkspaceShellRendererSnapshotIssue>,
}

impl WorkspaceShellRendererSnapshotReport {
    /// Returns true when all required artifact evidence is present and fresh.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validates renderer artifact metadata against replay evidence rows.
#[must_use]
pub fn validate_workspace_shell_renderer_snapshots(
    rows: &[super::super::widgets::WorkspaceShellReplayEvidenceRow],
    artifacts: &[super::WorkspaceShellRendererSnapshot],
) -> WorkspaceShellRendererSnapshotReport {
    debug_assert!(super::SUPPORTED_RENDERERS.len() > 1);
    debug_assert!(artifacts.len() <= artifacts.len().saturating_add(rows.len()));
    let fixture_ids = fixture_id_set(rows);
    let mut identities = std::collections::BTreeSet::new();
    let mut coverage = std::collections::BTreeSet::new();
    let mut issues = Vec::with_capacity(artifacts.len().saturating_add(rows.len()));
    for artifact in artifacts {
        let identity = artifact_identity(artifact);
        if !identities.insert(identity.clone()) {
            issues.push(issue(
                WorkspaceShellRendererSnapshotIssueCode::DuplicateArtifact,
                identity,
                "duplicate workspace shell renderer artifact",
            ));
        }
        let (has_known_renderer, artifact_issues) =
            validate_artifact_metadata(artifact, &fixture_ids);
        if has_known_renderer {
            coverage.insert((artifact.fixture_id.as_str(), artifact.renderer.as_str()));
        }
        issues.extend(artifact_issues);
    }
    issues.extend(missing_renderer_issues(rows, &coverage));
    WorkspaceShellRendererSnapshotReport { issues }
}

fn validate_artifact_metadata(
    artifact: &super::WorkspaceShellRendererSnapshot,
    fixture_ids: &std::collections::BTreeSet<&str>,
) -> (bool, Vec<WorkspaceShellRendererSnapshotIssue>) {
    debug_assert!(!artifact.artifact_path.is_empty());
    debug_assert!(!artifact.renderer.is_empty());
    let mut issues = Vec::new();
    if !fixture_ids.contains(artifact.fixture_id.as_str()) {
        issues.push(issue(
            WorkspaceShellRendererSnapshotIssueCode::MissingFixture,
            artifact.fixture_id.clone(),
            "workspace shell artifact references a missing replay fixture",
        ));
    }
    let has_known_renderer = super::SUPPORTED_RENDERERS.contains(&artifact.renderer.as_str());
    if !has_known_renderer {
        issues.push(issue(
            WorkspaceShellRendererSnapshotIssueCode::UnsupportedRenderer,
            artifact.renderer.clone(),
            "workspace shell artifact renderer is unsupported",
        ));
    }
    issues.extend(validate_artifact_content(artifact));
    (has_known_renderer, issues)
}

fn validate_artifact_content(
    artifact: &super::WorkspaceShellRendererSnapshot,
) -> Vec<WorkspaceShellRendererSnapshotIssue> {
    debug_assert!(!artifact.artifact_path.is_empty());
    debug_assert!(!artifact.fixture_id.is_empty());
    let mut issues = Vec::new();
    if artifact.generator != super::GENERATOR_VERSION {
        issues.push(issue(
            WorkspaceShellRendererSnapshotIssueCode::UnsupportedGenerator,
            artifact.generator.clone(),
            "workspace shell artifact generator is unsupported",
        ));
    }
    if artifact.bytes.is_empty() {
        issues.push(issue(
            WorkspaceShellRendererSnapshotIssueCode::EmptyArtifact,
            artifact.artifact_path.clone(),
            "workspace shell artifact bytes are empty",
        ));
    }
    if super::super::blake3_hex(&artifact.bytes) != artifact.blake3 {
        issues.push(issue(
            WorkspaceShellRendererSnapshotIssueCode::StaleHash,
            artifact.artifact_path.clone(),
            "workspace shell artifact hash is stale",
        ));
    }
    issues
}

fn missing_renderer_issues(
    rows: &[super::super::widgets::WorkspaceShellReplayEvidenceRow],
    coverage: &std::collections::BTreeSet<(&str, &str)>,
) -> Vec<WorkspaceShellRendererSnapshotIssue> {
    let issue_bound = rows.len().saturating_mul(super::SUPPORTED_RENDERERS.len());
    let mut issues = Vec::with_capacity(issue_bound);
    for row in rows {
        for renderer in super::SUPPORTED_RENDERERS {
            if !coverage.contains(&(row.fixture_id.as_str(), *renderer)) {
                issues.push(issue(
                    WorkspaceShellRendererSnapshotIssueCode::MissingRenderer,
                    format!("{}:{renderer}", row.fixture_id),
                    "workspace shell fixture is missing required renderer artifact",
                ));
            }
        }
    }
    issues
}

fn fixture_id_set(
    rows: &[super::super::widgets::WorkspaceShellReplayEvidenceRow],
) -> std::collections::BTreeSet<&str> {
    rows.iter().map(|row| row.fixture_id.as_str()).collect()
}

fn artifact_identity(artifact: &super::WorkspaceShellRendererSnapshot) -> String {
    format!(
        "{}:{}:{}",
        artifact.fixture_id, artifact.renderer, artifact.artifact_path
    )
}

fn issue(
    code: WorkspaceShellRendererSnapshotIssueCode,
    subject: impl Into<String>,
    message: impl Into<String>,
) -> WorkspaceShellRendererSnapshotIssue {
    WorkspaceShellRendererSnapshotIssue::new(code, subject, message)
}
