use super::*;

/// Generates a deterministic Markdown snapshot manifest page.
#[must_use]
pub fn generate_snapshot_manifest_page(entries: &[SnapshotManifestEntry]) -> String {
    let mut sorted_entries = entries.iter().collect::<Vec<_>>();
    sorted_entries.sort_by(|left, right| {
        left.fixture_id
            .cmp(&right.fixture_id)
            .then_with(|| left.renderer.cmp(&right.renderer))
            .then_with(|| left.artifact_path.cmp(&right.artifact_path))
    });
    debug_assert_eq!(sorted_entries.len(), entries.len());
    debug_assert!(sorted_entries.len() <= entries.len());
    let mut page = String::new();
    page.push_str(super::fixtures::GENERATED_COMMENT);
    page.push_str("\n# Snapshot manifest\n\n");
    page.push_str("| Fixture | Renderer | Artifact | Size | BLAKE3 | Generator | Source fixture |\n");
    page.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for entry in sorted_entries {
        page.push_str("| `");
        page.push_str(entry.fixture_id.as_str());
        page.push_str("` | `");
        page.push_str(&entry.renderer);
        page.push_str("` | `");
        page.push_str(&entry.artifact_path);
        page.push_str("` | ");
        page.push_str(&entry.size.width.to_string());
        page.push('x');
        page.push_str(&entry.size.height.to_string());
        page.push_str(" | `");
        page.push_str(&entry.blake3);
        page.push_str("` | `");
        page.push_str(&entry.generator);
        page.push_str("` | `");
        page.push_str(entry.source_fixture.as_str());
        page.push_str("` |\n");
    }
    page
}

/// Generates a deterministic Markdown component page from fixtures and snapshots.
#[must_use]
pub fn generate_component_page(
    source: &ComponentDocSource,
    entries: &[SnapshotManifestEntry],
) -> (String, Vec<SnapshotIssue>) {
    let mut issues = Vec::new();
    debug_assert!(source.title.trim().len() <= source.title.len());
    debug_assert!(source.component_id.as_str().trim().len() <= source.component_id.as_str().len());
    if source.snippets.is_empty() {
        issues.push(SnapshotIssue::new(
            SnapshotIssueCode::MissingSnippet,
            source.component_id.to_string(),
            "component docs require at least one code snippet",
        ));
    }
    let mut page = String::new();
    page.push_str(super::fixtures::GENERATED_COMMENT);
    page.push('\n');
    page.push_str("# ");
    page.push_str(&source.title);
    page.push_str("\n\n## Snapshots\n\n");
    for entry in entries.iter().filter(|entry| entry.fixture_id == source.component_id) {
        page.push_str("- `");
        page.push_str(&entry.renderer);
        page.push_str("` `");
        page.push_str(&entry.artifact_path);
        page.push_str("` ");
        page.push_str(&entry.size.width.to_string());
        page.push('x');
        page.push_str(&entry.size.height.to_string());
        page.push_str(" `");
        page.push_str(&entry.blake3);
        page.push_str("`\n");
    }
    page.push_str("\n## States\n\n");
    for row in &source.states {
        page.push_str("- ");
        page.push_str(row);
        page.push('\n');
    }
    page.push_str("\n## Controls\n\n");
    for row in &source.controls {
        page.push_str("- ");
        page.push_str(row);
        page.push('\n');
    }
    page.push_str("\n## Edge cases\n\n");
    for row in &source.edge_cases {
        page.push_str("- ");
        page.push_str(row);
        page.push('\n');
    }
    page.push_str("\n## Snippets\n\n");
    for snippet in &source.snippets {
        page.push_str("```rust\n");
        page.push_str(snippet);
        page.push_str("\n```\n");
    }
    (page, issues)
}

/// Publication checklist for maintainers reviewing snapshot docs.
#[must_use]
pub fn publication_checklist() -> Vec<&'static str> {
    vec![
        "regenerate renderer artifacts",
        "review BLAKE3 hashes in manifest entries",
        "review generated component pages",
        "run snapshot manifest validation",
        "commit fixture, artifact, manifest, and docs changes together",
    ]
}
