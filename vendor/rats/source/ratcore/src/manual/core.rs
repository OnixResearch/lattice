//! Snapshot documentation manifest and generator core.
//!
//! Snapshot docs are generated from fixture metadata plus renderer artifacts.
//! This pure core validates manifests, hashes artifacts with BLAKE3, and emits
//! deterministic Markdown pages. Renderers and filesystem writers live outside
//! the core.

mod artifact {
    pub(super) mod text;
}
mod checks;
mod fixtures;
mod render;

pub use checks::{
    GeneratedDocumentCheck, SnapshotIssue, SnapshotIssueCode, SnapshotValidationReport,
    validate_generated_document, validate_parity_manifest_gate, validate_snapshot_manifest,
};
pub use fixtures::{
    built_in_component_doc_sources, built_in_component_pages,
    built_in_generated_component_documents, built_in_snapshot_artifacts,
    built_in_snapshot_fixture_ids, built_in_snapshot_manifest_entries,
    built_in_snapshot_parity_rows,
};
pub use render::{generate_component_page, generate_snapshot_manifest_page, publication_checklist};
/// Logical snapshot size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotSize {
    /// Width in logical cells, pixels, or CSS units.
    pub width: u16,
    /// Height in logical cells, pixels, or CSS units.
    pub height: u16,
}

impl SnapshotSize {
    /// Creates a snapshot size.
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// Rendered snapshot artifact manifest entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotManifestEntry {
    /// Fixture ID.
    pub fixture_id: String,
    /// Renderer label.
    pub renderer: String,
    /// Artifact path relative to the docs root.
    pub artifact_path: String,
    /// BLAKE3 hash of the artifact bytes.
    pub blake3: String,
    /// Logical artifact size.
    pub size: SnapshotSize,
    /// Generator version metadata.
    pub generator: String,
    /// Source fixture reference.
    pub source_fixture: String,
}

impl SnapshotManifestEntry {
    /// Creates a manifest entry.
    #[must_use]
    pub fn new(
        fixture_id: impl Into<String>,
        renderer: impl Into<String>,
        artifact_path: impl Into<String>,
        artifact_bytes: &[u8],
        size: SnapshotSize,
    ) -> Self {
        let fixture_id = fixture_id.into();
        Self {
            source_fixture: fixture_id.clone(),
            fixture_id,
            renderer: renderer.into(),
            artifact_path: artifact_path.into(),
            blake3: crate::interaction_replay::blake3_hex(artifact_bytes),
            size,
            generator: fixtures::GENERATOR_VERSION.to_string(),
        }
    }
}

/// Artifact bytes supplied by a renderer adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotArtifact {
    /// Artifact path.
    pub path: String,
    /// Artifact bytes.
    pub bytes: Vec<u8>,
}

impl SnapshotArtifact {
    /// Creates a snapshot artifact.
    #[must_use]
    pub fn new(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

/// Generated component documentation page metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedComponentPage {
    /// Fixture ID documented by the page.
    pub fixture_id: String,
    /// Generated page path or stable target.
    pub path: String,
}

impl GeneratedComponentPage {
    /// Creates generated component page metadata.
    #[must_use]
    pub fn new(fixture_id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            path: path.into(),
        }
    }
}

/// Generated component documentation page content and validation diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedComponentDocument {
    /// Page metadata.
    pub page: GeneratedComponentPage,
    /// Generated Markdown content.
    pub content: String,
    /// Issues found while generating the page.
    pub issues: Vec<SnapshotIssue>,
}

/// Component documentation source derived from fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDocSource {
    /// Component ID.
    pub component_id: String,
    /// Component title.
    pub title: String,
    /// State labels.
    pub states: Vec<String>,
    /// Control descriptions.
    pub controls: Vec<String>,
    /// Edge case descriptions.
    pub edge_cases: Vec<String>,
    /// Code snippets.
    pub snippets: Vec<String>,
}

impl ComponentDocSource {
    /// Creates a component documentation source.
    #[must_use]
    pub fn new(component_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            component_id: component_id.into(),
            title: title.into(),
            states: Vec::new(),
            controls: Vec::new(),
            edge_cases: Vec::new(),
            snippets: Vec::new(),
        }
    }

    /// Adds a state label.
    #[must_use]
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.states.push(state.into());
        self
    }

    /// Adds a control description.
    #[must_use]
    pub fn with_control(mut self, control: impl Into<String>) -> Self {
        self.controls.push(control.into());
        self
    }

    /// Adds an edge-case description.
    #[must_use]
    pub fn with_edge_case(mut self, edge_case: impl Into<String>) -> Self {
        self.edge_cases.push(edge_case.into());
        self
    }

    /// Adds a code snippet.
    #[must_use]
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippets.push(snippet.into());
        self
    }
}
/// Builds generated component documents from documentation sources and page targets.
#[must_use]
pub fn build_generated_component_documents(
    sources: &[ComponentDocSource],
    pages: &[GeneratedComponentPage],
    entries: &[SnapshotManifestEntry],
) -> Vec<GeneratedComponentDocument> {
    let source_map = sources
        .iter()
        .map(|source| (source.component_id.as_str(), source))
        .collect::<std::collections::BTreeMap<_, _>>();
    debug_assert!(source_map.len() <= sources.len());
    debug_assert!(pages.len() <= sources.len().saturating_add(pages.len()));
    pages
        .iter()
        .map(|page| match source_map.get(page.fixture_id.as_str()) {
            Some(source) => {
                let (content, issues) = generate_component_page(source, entries);
                GeneratedComponentDocument {
                    page: page.clone(),
                    content,
                    issues,
                }
            }
            None => GeneratedComponentDocument {
                page: page.clone(),
                content: String::new(),
                issues: vec![SnapshotIssue::new(
                    SnapshotIssueCode::MissingFixture,
                    page.fixture_id.clone(),
                    "component documentation source is missing",
                )],
            },
        })
        .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
