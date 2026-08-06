mod checks;
pub use checks::WorkspaceShellRendererSnapshotIssue;
pub use checks::WorkspaceShellRendererSnapshotIssueCode;
pub use checks::WorkspaceShellRendererSnapshotReport;
pub use checks::validate_workspace_shell_renderer_snapshots;

const RENDERER_SUBWAYRAT_TUI: &str = "subwayrat-tui";
const RENDERER_METEORITE_DOM: &str = "meteorite-dom";
const GENERATOR_VERSION: &str = "workspace-shell-replay-snapshots-v1";
const ARTIFACT_ROOT: &str = "docs/generated/replay-snapshots";
const TUI_SNAPSHOT_WIDTH: u16 = 80;
const TUI_SNAPSHOT_HEIGHT: u16 = 24;
const DOM_SNAPSHOT_WIDTH: u16 = 640;
const DOM_SNAPSHOT_HEIGHT: u16 = 480;
const SUPPORTED_RENDERERS: &[&str] = &[RENDERER_SUBWAYRAT_TUI, RENDERER_METEORITE_DOM];

/// Deterministic renderer snapshot artifact for a workspace shell replay fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceShellRendererSnapshot {
    /// Source replay fixture ID.
    pub fixture_id: String,
    /// Renderer projection label.
    pub renderer: String,
    /// Artifact path relative to the repository root.
    pub artifact_path: String,
    /// Generator identity used to build the artifact bytes.
    pub generator: String,
    /// BLAKE3 hash of `bytes`.
    pub blake3: String,
    /// Deterministic artifact bytes.
    pub bytes: Vec<u8>,
}

impl WorkspaceShellRendererSnapshot {
    /// Creates an artifact and hashes its content bytes.
    #[must_use]
    pub fn new(
        fixture_id: impl Into<String>,
        renderer: impl Into<String>,
        artifact_path: impl Into<String>,
        generator: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        let bytes = bytes.into();
        Self {
            fixture_id: fixture_id.into(),
            renderer: renderer.into(),
            artifact_path: artifact_path.into(),
            generator: generator.into(),
            blake3: super::blake3_hex(&bytes),
            bytes,
        }
    }
}

struct ArtifactPathInput<'a> {
    fixture_id: &'a str,
    renderer: &'a str,
}

/// Builds deterministic DOM/TUI artifacts for the workspace shell replay fixtures.
#[must_use]
pub fn workspace_shell_renderer_snapshots() -> Vec<WorkspaceShellRendererSnapshot> {
    let evidence = super::widgets::workspace_shell_replay_evidence();
    renderer_artifacts_for_rows(&evidence.rows)
}

fn renderer_artifacts_for_rows(
    rows: &[super::widgets::WorkspaceShellReplayEvidenceRow],
) -> Vec<WorkspaceShellRendererSnapshot> {
    let mut artifacts = Vec::with_capacity(rows.len().saturating_mul(SUPPORTED_RENDERERS.len()));
    for row in rows {
        artifacts.push(render_artifact(row, RENDERER_SUBWAYRAT_TUI));
        artifacts.push(render_artifact(row, RENDERER_METEORITE_DOM));
    }
    artifacts
}

fn render_artifact(
    row: &super::widgets::WorkspaceShellReplayEvidenceRow,
    renderer: &str,
) -> WorkspaceShellRendererSnapshot {
    let artifact_path = artifact_path(ArtifactPathInput {
        fixture_id: &row.fixture_id,
        renderer,
    });
    let text = match renderer {
        RENDERER_METEORITE_DOM => render_dom_artifact(row),
        _ => render_tui_artifact(row),
    };
    WorkspaceShellRendererSnapshot::new(
        row.fixture_id.clone(),
        renderer,
        artifact_path,
        GENERATOR_VERSION,
        text.into_bytes(),
    )
}

fn render_tui_artifact(row: &super::widgets::WorkspaceShellReplayEvidenceRow) -> String {
    format!(
        "fixture={}\nrenderer={}\ngenerator={}\nlogical-size={}x{}\nscenario={}\nstate-blake3={}\nworkspace-snapshot-blake3={}\ndiagnostics={}\n",
        row.fixture_id,
        RENDERER_SUBWAYRAT_TUI,
        GENERATOR_VERSION,
        TUI_SNAPSHOT_WIDTH,
        TUI_SNAPSHOT_HEIGHT,
        row.scenario,
        row.state_hash,
        row.snapshot_hash,
        row.diagnostics,
    )
}

fn render_dom_artifact(row: &super::widgets::WorkspaceShellReplayEvidenceRow) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<meta charset=\"utf-8\">\n<title>Workspace shell replay snapshot</title>\n<section data-fixture=\"{}\" data-renderer=\"{}\" data-generator=\"{}\" data-size=\"{}x{}\">\n  <h1>{}</h1>\n  <dl>\n    <dt>state</dt><dd>{}</dd>\n    <dt>workspace snapshot</dt><dd>{}</dd>\n    <dt>diagnostics</dt><dd>{}</dd>\n  </dl>\n</section>\n</html>\n",
        html_escape(&row.fixture_id),
        RENDERER_METEORITE_DOM,
        GENERATOR_VERSION,
        DOM_SNAPSHOT_WIDTH,
        DOM_SNAPSHOT_HEIGHT,
        html_escape(&row.scenario),
        row.state_hash,
        row.snapshot_hash,
        html_escape(&row.diagnostics),
    )
}

fn artifact_path(input: ArtifactPathInput<'_>) -> String {
    let slug = input.fixture_id.replace('.', "-");
    let suffix = if input.renderer == RENDERER_METEORITE_DOM {
        "meteorite.html"
    } else {
        "subwayrat.txt"
    };
    format!("{ARTIFACT_ROOT}/{slug}.{suffix}")
}

fn html_escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
