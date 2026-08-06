/// Classification that keeps semantic evidence separate from runtime proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticEvidenceKind {
    /// Pure semantic projection and routing evidence.
    SemanticProjection,
    /// Browser accessibility runtime receipt.
    BrowserRuntime,
    /// Native platform accessibility runtime receipt.
    NativeRuntime,
    /// Structured terminal summary evidence.
    TerminalSummary,
}

/// Deterministic semantic fixture evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEvidence {
    /// Stable fixture identity.
    pub fixture_id: String,
    /// Snapshot generation.
    pub generation: super::super::SemanticGeneration,
    /// BLAKE3 hash of canonical debug encoding.
    pub snapshot_hash: String,
    /// Truthful evidence classification.
    pub kind: SemanticEvidenceKind,
    /// Optional external runtime receipt.
    pub runtime_receipt: Option<String>,
    /// Renderer support rows used by the fixture.
    pub support: Vec<super::super::SemanticSupport>,
}

/// Returns a deterministic BLAKE3 snapshot hash.
#[must_use]
pub fn semantic_snapshot_hash(snapshot: &super::super::SemanticSnapshot) -> String {
    blake3::hash(format!("{snapshot:?}").as_bytes()).to_hex().to_string()
}

/// Creates semantic-only fixture evidence with all renderer support rows.
#[must_use]
pub fn semantic_fixture_evidence(
    fixture_id: impl Into<String>,
    snapshot: &super::super::SemanticSnapshot,
) -> SemanticEvidence {
    let renderer_rows = [
        super::super::SemanticRendererTarget::MeteoriteDom,
        super::super::SemanticRendererTarget::SubwayratTerminal,
        super::super::SemanticRendererTarget::NativeAccessibility,
    ]
    .into_iter()
    .flat_map(|target| super::super::project_semantic_summary(snapshot, target).nodes)
    .flat_map(|node| node.support)
    .collect();
    SemanticEvidence {
        fixture_id: fixture_id.into(),
        generation: snapshot.generation,
        snapshot_hash: semantic_snapshot_hash(snapshot),
        kind: SemanticEvidenceKind::SemanticProjection,
        runtime_receipt: None,
        support: renderer_rows,
    }
}

/// Validates fixture freshness, classification, and support reasons.
#[must_use]
pub fn validate_semantic_evidence(
    evidence: &SemanticEvidence,
    snapshot: &super::super::SemanticSnapshot,
) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(evidence.support.len().saturating_add(2));
    issues.extend(stale_issue(evidence, snapshot));
    issues.extend(classification_issue(evidence));
    issues.extend(evidence.support.iter().filter_map(|row| support_issue(evidence, row)));
    issues.sort();
    issues
}

fn stale_issue(
    evidence: &SemanticEvidence,
    snapshot: &super::super::SemanticSnapshot,
) -> Option<super::super::SemanticIssue> {
    let is_stale =
        evidence.generation != snapshot.generation || evidence.snapshot_hash != semantic_snapshot_hash(snapshot);
    is_stale.then(|| {
        super::super::SemanticIssue::new(
            super::super::SemanticIssueCode::StaleEvidence,
            &evidence.fixture_id,
            "semantic fixture generation or hash is stale",
        )
    })
}

fn classification_issue(evidence: &SemanticEvidence) -> Option<super::super::SemanticIssue> {
    let is_runtime_claim =
        matches!(evidence.kind, SemanticEvidenceKind::BrowserRuntime | SemanticEvidenceKind::NativeRuntime);
    let is_receipt_missing = evidence.runtime_receipt.as_ref().is_none_or(|receipt| receipt.trim().is_empty());
    (is_runtime_claim && is_receipt_missing).then(|| {
        super::super::SemanticIssue::new(
            super::super::SemanticIssueCode::MisclassifiedEvidence,
            &evidence.fixture_id,
            "platform runtime evidence requires an external receipt",
        )
    })
}

fn support_issue(
    evidence: &SemanticEvidence,
    support: &super::super::SemanticSupport,
) -> Option<super::super::SemanticIssue> {
    let needs_reason = support.level != super::super::SemanticSupportLevel::Direct;
    let is_reason_missing = support.reason.as_ref().is_none_or(|reason| reason.trim().is_empty());
    (needs_reason && is_reason_missing).then(|| {
        super::super::SemanticIssue::new(
            super::super::SemanticIssueCode::MissingSupportReason,
            &evidence.fixture_id,
            "non-direct renderer support requires a reason",
        )
    })
}
