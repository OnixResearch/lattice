const ARTIFACT_ROOT: &str = "docs/generated/replay-snapshots";
const GENERATOR_VERSION: &str = "full-mouse-replay-routes-v1";
const RENDERER_SUBWAYRAT_TUI: &str = "subwayrat-tui";
const RENDERER_METEORITE_DOM: &str = "meteorite-dom";
const ADAPTER_SUBWAYRAT: &str = "normalize_subwayrat_mouse_input";
const ADAPTER_METEORITE: &str = "normalize_dom_pointer_input";
const RENDERER_ARTIFACTS_PER_ROW: usize = 2;

/// Deterministic renderer route artifact for a full mouse replay fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerMouseRendererArtifact {
    /// Source fixture ID.
    pub fixture_id: String,
    /// Renderer projection label.
    pub renderer: String,
    /// Renderer adapter identity.
    pub adapter: String,
    /// Artifact path relative to the repository root.
    pub artifact_path: String,
    /// Generator identity used to build the artifact bytes.
    pub generator: String,
    /// BLAKE3 hash of `bytes`.
    pub blake3: String,
    /// Deterministic route artifact bytes.
    pub bytes: Vec<u8>,
}

impl PointerMouseRendererArtifact {
    fn new(
        fixture_id: impl Into<String>,
        renderer: impl Into<String>,
        adapter: impl Into<String>,
        artifact_path: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        let bytes = bytes.into();
        Self {
            fixture_id: fixture_id.into(),
            renderer: renderer.into(),
            adapter: adapter.into(),
            artifact_path: artifact_path.into(),
            generator: GENERATOR_VERSION.to_string(),
            blake3: super::super::super::blake3_hex(&bytes),
            bytes,
        }
    }
}

/// Builds deterministic TUI and DOM route artifacts for full mouse replay rows.
#[must_use]
pub fn pointer_mouse_renderer_artifacts() -> Vec<PointerMouseRendererArtifact> {
    let evidence = super::fixtures::pointer_mouse_replay_proof();
    artifacts_for_rows(&evidence.rows)
}

fn artifacts_for_rows(
    rows: &[super::fixtures::PointerMouseReplayProofRow],
) -> Vec<PointerMouseRendererArtifact> {
    let mut artifacts = Vec::with_capacity(rows.len().saturating_mul(RENDERER_ARTIFACTS_PER_ROW));
    for row in rows {
        artifacts.push(render_tui_artifact(row));
        artifacts.push(render_dom_artifact(row));
    }
    artifacts
}

fn render_tui_artifact(
    row: &super::fixtures::PointerMouseReplayProofRow,
) -> PointerMouseRendererArtifact {
    debug_assert!(!row.fixture_id.is_empty());
    debug_assert!(!row.route_summary_hash.is_empty());
    let path = artifact_path(ArtifactPathInput {
        fixture_id: &row.fixture_id,
        suffix: "subwayrat.txt",
    });
    let bytes = format!(
        "fixture={}\nrenderer={}\nadapter={}\ngenerator={}\nscenario={}\nstate-blake3={}\nroute-summary-blake3={}\ndiagnostics={}\n",
        row.fixture_id,
        RENDERER_SUBWAYRAT_TUI,
        ADAPTER_SUBWAYRAT,
        GENERATOR_VERSION,
        row.scenario,
        row.state_hash,
        row.route_summary_hash,
        row.diagnostics,
    )
    .into_bytes();
    PointerMouseRendererArtifact::new(
        row.fixture_id.clone(),
        RENDERER_SUBWAYRAT_TUI,
        ADAPTER_SUBWAYRAT,
        path,
        bytes,
    )
}

fn render_dom_artifact(
    row: &super::fixtures::PointerMouseReplayProofRow,
) -> PointerMouseRendererArtifact {
    debug_assert!(!row.fixture_id.is_empty());
    debug_assert!(!row.state_hash.is_empty());
    let path = artifact_path(ArtifactPathInput {
        fixture_id: &row.fixture_id,
        suffix: "meteorite.html",
    });
    let bytes = format!(
        "<!doctype html>\n<html lang=\"en\">\n<meta charset=\"utf-8\">\n<title>Full mouse replay route summary</title>\n<section data-fixture=\"{}\" data-renderer=\"{}\" data-adapter=\"{}\" data-generator=\"{}\">\n  <h1>{}</h1>\n  <dl>\n    <dt>state</dt><dd>{}</dd>\n    <dt>route summary</dt><dd>{}</dd>\n    <dt>diagnostics</dt><dd>{}</dd>\n  </dl>\n</section>\n</html>\n",
        html_escape(&row.fixture_id),
        RENDERER_METEORITE_DOM,
        ADAPTER_METEORITE,
        GENERATOR_VERSION,
        html_escape(&row.scenario),
        row.state_hash,
        row.route_summary_hash,
        html_escape(&row.diagnostics),
    )
    .into_bytes();
    PointerMouseRendererArtifact::new(
        row.fixture_id.clone(),
        RENDERER_METEORITE_DOM,
        ADAPTER_METEORITE,
        path,
        bytes,
    )
}

struct ArtifactPathInput<'a> {
    fixture_id: &'a str,
    suffix: &'a str,
}

fn artifact_path(input: ArtifactPathInput<'_>) -> String {
    format!(
        "{ARTIFACT_ROOT}/{}.{}",
        input.fixture_id.replace('.', "-"),
        input.suffix,
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
