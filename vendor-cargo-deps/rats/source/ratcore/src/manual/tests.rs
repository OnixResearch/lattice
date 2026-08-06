use super::*;

mod doc {
    pub(super) mod assertions;
}

const FIXTURE: &str = "dialog.basic";
const ARTIFACT_PATH: &str = "snapshots/dialog.typ";
const METEORITE_ARTIFACT_PATH: &str = "snapshots/dialog.html";
const DOC_PATH: &str = "docs/generated/components/dialog-basic.md";
const ARTIFACT: &[u8] = b"typst evidence";
const METEORITE_ARTIFACT: &[u8] = b"dom evidence";
const TUI_SNAPSHOT_WIDTH: u16 = 80;
const TUI_SNAPSHOT_HEIGHT: u16 = 24;
const WEB_SNAPSHOT_WIDTH: u16 = 640;
const WEB_SNAPSHOT_HEIGHT: u16 = 480;
const EXPECTED_BASELINE_ENTRY_COUNT: usize = 8;
const SNAPSHOT_MANIFEST_DOC: &str = "docs/generated/snapshot-manifest.md";
const DIALOG_DOC: &str = "docs/generated/components/dialog-basic.md";
const CHROME_DOC: &str = "docs/generated/components/chrome-shell.md";
const KNOWLEDGE_DOC: &str = "docs/generated/components/knowledge-workflow.md";
const THEME_DOC: &str = "docs/generated/components/theme-tokens.md";
const CHECKED_IN_MANIFEST: &str = include_str!("../../../docs/generated/snapshot-manifest.md");
const CHECKED_IN_DIALOG_DOC: &str = include_str!("../../../docs/generated/components/dialog-basic.md");
const CHECKED_IN_CHROME_DOC: &str = include_str!("../../../docs/generated/components/chrome-shell.md");
const CHECKED_IN_KNOWLEDGE_DOC: &str = include_str!("../../../docs/generated/components/knowledge-workflow.md");
const CHECKED_IN_THEME_DOC: &str = include_str!("../../../docs/generated/components/theme-tokens.md");

#[test]
fn manifest_validates_with_matching_artifact_hash() {
    let entry = sample_entry();
    let fixtures = std::collections::BTreeSet::from([crate::showcase_fixtures::FixtureId::new(FIXTURE)]);
    let artifacts = vec![SnapshotArtifact::new(ARTIFACT_PATH, ARTIFACT)];

    let outcome = validate_snapshot_manifest(&[entry], &fixtures, &artifacts);

    assert!(outcome.is_valid());
}

#[test]
fn manifest_rejects_missing_stale_duplicate_and_unsupported() {
    let mut stale = sample_entry();
    stale.blake3 = "stale".to_string();
    let duplicate = stale.clone();
    let invalid_renderer_entry = SnapshotManifestEntry {
        fixture_id: crate::showcase_fixtures::FixtureId::new("missing"),
        renderer: "browser".to_string(),
        artifact_path: "missing.png".to_string(),
        ..sample_entry()
    };
    let fixtures = std::collections::BTreeSet::from([crate::showcase_fixtures::FixtureId::new(FIXTURE)]);
    let artifacts = vec![SnapshotArtifact::new(ARTIFACT_PATH, ARTIFACT)];

    let outcome = validate_snapshot_manifest(&[stale, duplicate, invalid_renderer_entry], &fixtures, &artifacts);

    assert!(!outcome.is_valid());
    assert!(!outcome.issues.is_empty());
    assert_issue(&outcome, SnapshotIssueCode::DuplicateId);
    assert_issue(&outcome, SnapshotIssueCode::MissingFixture);
    assert_issue(&outcome, SnapshotIssueCode::MissingArtifact);
    assert_issue(&outcome, SnapshotIssueCode::UnsupportedRenderer);
    assert_issue(&outcome, SnapshotIssueCode::StaleHash);
}

#[test]
fn generator_emits_fixture_backed_page() {
    let source = ComponentDocSource::new(FIXTURE, "Dialog")
        .with_state("default")
        .with_control("Enter accepts")
        .with_edge_case("tiny viewport clips safely")
        .with_snippet("render_dialog();");

    let (page, issues) = generate_component_page(&source, &[sample_entry()]);

    assert!(issues.is_empty());
    assert!(page.contains(super::fixtures::GENERATED_COMMENT));
    assert!(page.contains(ARTIFACT_PATH));
    assert!(page.contains("tiny viewport"));
}

#[test]
fn generator_requires_snippets_and_publication_checklist_names_review_steps() {
    let source = ComponentDocSource::new(FIXTURE, "Dialog");

    let (_page, issues) = generate_component_page(&source, &[sample_entry()]);

    assert_eq!(issues[0].code, SnapshotIssueCode::MissingSnippet);
    assert!(publication_checklist().iter().any(|step| step.contains("BLAKE3")));
}

#[test]
fn parity_manifest_gate_accepts_complete_renderer_evidence() {
    let rows = vec![parity_row()];
    let entries = vec![sample_entry(), meteorite_entry()];
    let pages = vec![GeneratedComponentPage::new(FIXTURE, DOC_PATH)];
    let artifacts = vec![
        SnapshotArtifact::new(ARTIFACT_PATH, ARTIFACT),
        SnapshotArtifact::new(METEORITE_ARTIFACT_PATH, METEORITE_ARTIFACT),
    ];

    let outcome = validate_parity_manifest_gate(&rows, &entries, &pages, &artifacts);

    assert!(outcome.is_valid());
}

#[test]
fn parity_manifest_gate_rejects_missing_page_missing_entry_and_stale_hash() {
    let rows = vec![parity_row()];
    let mut stale_entry = sample_entry();
    stale_entry.blake3 = "stale".to_string();
    let artifacts = vec![SnapshotArtifact::new(ARTIFACT_PATH, ARTIFACT)];

    let outcome = validate_parity_manifest_gate(&rows, &[stale_entry], &[], &artifacts);

    assert!(!outcome.is_valid());
    assert_issue(&outcome, SnapshotIssueCode::MissingDocPage);
    assert_issue(&outcome, SnapshotIssueCode::MissingManifestEntry);
    assert_issue(&outcome, SnapshotIssueCode::StaleHash);
}

#[test]
fn built_in_snapshot_manifest_validates_real_artifacts() {
    let entries = built_in_snapshot_manifest_entries();
    let fixtures = built_in_snapshot_fixture_ids();
    let artifacts = built_in_snapshot_artifacts();

    let outcome = validate_snapshot_manifest(&entries, &fixtures, &artifacts);

    assert!(outcome.is_valid());
    assert_eq!(entries.len(), EXPECTED_BASELINE_ENTRY_COUNT);
    assert!(
        entries
            .iter()
            .any(|entry| { entry.fixture_id == "chrome.shell" && entry.renderer == "subwayrat-typst" })
    );
    assert!(
        entries
            .iter()
            .any(|entry| { entry.fixture_id == "knowledge.workflow" && entry.renderer == "meteorite-dom" })
    );
    assert!(
        entries
            .iter()
            .any(|entry| { entry.fixture_id == "theme.tokens" && entry.renderer == "meteorite-dom" })
    );
}

#[test]
fn built_in_shared_snapshot_parity_gate_accepts_real_baselines() {
    let rows = built_in_snapshot_parity_rows();
    let entries = built_in_snapshot_manifest_entries();
    let pages = built_in_component_pages();
    let artifacts = built_in_snapshot_artifacts();

    let outcome = validate_parity_manifest_gate(&rows, &entries, &pages, &artifacts);

    assert!(outcome.is_valid());
}

#[test]
fn built_in_snapshot_manifest_rejects_stale_artifact_bytes() {
    let entries = built_in_snapshot_manifest_entries();
    let fixtures = built_in_snapshot_fixture_ids();
    let mut artifacts = built_in_snapshot_artifacts();
    if let Some(first) = artifacts.first_mut() {
        first.bytes.push(b'!');
    }

    let outcome = validate_snapshot_manifest(&entries, &fixtures, &artifacts);

    assert!(!outcome.is_valid());
    assert_issue(&outcome, SnapshotIssueCode::StaleHash);
}

#[test]
fn checked_in_snapshot_artifacts_match_built_in_generation() {
    let artifacts = built_in_snapshot_artifacts();

    assert_eq!(artifacts.len(), EXPECTED_BASELINE_ENTRY_COUNT);
    assert!(artifacts.iter().all(|artifact| !artifact.bytes.is_empty()));
    for expected in doc::assertions::EXPECTED_ARTIFACTS {
        doc::assertions::assert_artifact_bytes(&artifacts, expected);
    }
}

#[test]
fn generated_manifest_and_component_pages_match_checked_in_docs() {
    let entries = built_in_snapshot_manifest_entries();
    let manifest = generate_snapshot_manifest_page(&entries);
    let manifest_status = validate_generated_document(GeneratedDocumentCheck {
        subject: SNAPSHOT_MANIFEST_DOC.to_string(),
        expected: &manifest,
        checked_in: CHECKED_IN_MANIFEST,
    });

    assert!(manifest_status.is_valid());
    assert!(manifest.contains("# Snapshot manifest"));
    let expected_docs = [
        (DIALOG_DOC, CHECKED_IN_DIALOG_DOC),
        (CHROME_DOC, CHECKED_IN_CHROME_DOC),
        (KNOWLEDGE_DOC, CHECKED_IN_KNOWLEDGE_DOC),
        (THEME_DOC, CHECKED_IN_THEME_DOC),
    ];
    for (path, checked_in) in expected_docs {
        doc::assertions::assert_generated_document(doc::assertions::ExpectedGeneratedDocument { path, checked_in });
    }
}

#[test]
fn generated_document_validation_rejects_stale_bytes() {
    let entries = built_in_snapshot_manifest_entries();
    let manifest = generate_snapshot_manifest_page(&entries);
    let stale_manifest = manifest.replace("snapshot-docs-v1", "snapshot-docs-v0");

    let outcome = validate_generated_document(GeneratedDocumentCheck {
        subject: SNAPSHOT_MANIFEST_DOC.to_string(),
        expected: &manifest,
        checked_in: &stale_manifest,
    });

    assert!(!outcome.is_valid());
    assert_issue(&outcome, SnapshotIssueCode::StaleGeneratedDoc);
}

fn sample_entry() -> SnapshotManifestEntry {
    SnapshotManifestEntry::new(
        FIXTURE,
        "subwayrat-typst",
        ARTIFACT_PATH,
        ARTIFACT,
        SnapshotSize::new(TUI_SNAPSHOT_WIDTH, TUI_SNAPSHOT_HEIGHT),
    )
}

fn meteorite_entry() -> SnapshotManifestEntry {
    SnapshotManifestEntry::new(
        FIXTURE,
        "meteorite-dom",
        METEORITE_ARTIFACT_PATH,
        METEORITE_ARTIFACT,
        SnapshotSize::new(WEB_SNAPSHOT_WIDTH, WEB_SNAPSHOT_HEIGHT),
    )
}

fn parity_row() -> crate::showcase_fixtures::FixtureParityRow {
    crate::showcase_fixtures::FixtureParityRow::new(FIXTURE, "dialog", DOC_PATH)
        .with_renderer(crate::showcase_fixtures::FixtureParityRenderer::supported(
            crate::showcase_fixtures::ShowcaseRenderer::Subwayrat,
            crate::showcase_fixtures::FixtureParityEvidenceStatus::Complete,
        ))
        .with_renderer(crate::showcase_fixtures::FixtureParityRenderer::supported(
            crate::showcase_fixtures::ShowcaseRenderer::Meteorite,
            crate::showcase_fixtures::FixtureParityEvidenceStatus::Complete,
        ))
}

fn assert_issue(report: &SnapshotValidationReport, code: SnapshotIssueCode) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}
