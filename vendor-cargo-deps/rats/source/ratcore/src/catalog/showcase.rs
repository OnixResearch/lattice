use super::*;

const FIXTURE_DIALOG_BASIC: &str = "dialog.basic";
const FIXTURE_CHROME_SHELL: &str = "chrome.shell";
const FIXTURE_KNOWLEDGE_WORKFLOW: &str = "knowledge.workflow";
const FIXTURE_THEME_TOKENS: &str = "theme.tokens";
const FAMILY_DIALOG: &str = "dialog";
const FAMILY_CHROME: &str = "chrome";
const FAMILY_KNOWLEDGE: &str = "knowledge";
const FAMILY_THEME: &str = "theme";
const DOC_DIALOG_BASIC: &str = "docs/generated/components/dialog-basic.md";
const DOC_CHROME_SHELL: &str = "docs/generated/components/chrome-shell.md";
const DOC_KNOWLEDGE_WORKFLOW: &str = "docs/generated/components/knowledge-workflow.md";
const DOC_THEME_TOKENS: &str = "docs/generated/components/theme-tokens.md";
const KNOWLEDGE_STATE_CAPTURE: &str = "capture";
const KNOWLEDGE_LABEL_CAPTURE_BACKLINKS: &str = "Capture and backlinks";
const KNOWLEDGE_DATA_GRAPH: &str = "graph";
const KNOWLEDGE_GRAPH_FACTS: &str = "doc -> other";
const KNOWLEDGE_DATA_AGENDA: &str = "agenda";
const KNOWLEDGE_AGENDA_FACTS: &str = "2026-06-11 Review";
const KNOWLEDGE_DATA_METADATA: &str = "metadata";
const KNOWLEDGE_METADATA_FACTS: &str = "documents=2,tags=1,backlinks=1";
const KNOWLEDGE_DATA_DIAGNOSTICS: &str = "diagnostics";
const KNOWLEDGE_DIAGNOSTIC_FACTS: &str = "valid workspace";
const KNOWLEDGE_CONTROL_SEARCH: &str = "search";
const KNOWLEDGE_CONTROL_SEARCH_LABEL: &str = "Search knowledge graph";
const KNOWLEDGE_CONTROL_CAPTURE: &str = "capture";
const KNOWLEDGE_CONTROL_CAPTURE_LABEL: &str = "Capture to Inbox";
const STATUS_SHARED: &str = "shared";
const STATUS_TUI_ONLY: &str = "tui-only";
const STATUS_METEORITE_ONLY: &str = "meteorite-only";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_SUPPORTED: &str = "supported";
const STATUS_EVIDENCE_COMPLETE: &str = "complete";
const STATUS_EVIDENCE_MISSING: &str = "missing";
const STATUS_EVIDENCE_UNSUPPORTED: &str = "unsupported";

/// Display-ready parity summary row shared by showcase renderers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParitySummaryRow {
    /// Fixture ID.
    pub fixture_id: String,
    /// Component family.
    pub family: String,
    /// Subwayrat support and evidence text.
    pub subwayrat: String,
    /// Meteorite support and evidence text.
    pub meteorite: String,
    /// Generated documentation target.
    pub documentation_target: String,
    /// Overall parity status.
    pub status: String,
}

/// Display-ready parity summary and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParitySummary {
    /// Rows sorted by fixture ID.
    pub rows: Vec<FixtureParitySummaryRow>,
    /// Display-ready edge states sorted by fixture and state ID.
    pub edge_states: Vec<EdgeDisplayRow>,
    /// Deterministic validation diagnostics.
    pub issues: Vec<FixtureIssue>,
    /// Deterministic edge-case validation diagnostics.
    pub edge_issues: Vec<Issue>,
}

impl FixtureParitySummary {
    /// Returns true when the summary has no diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty() && self.edge_issues.is_empty()
    }
}

/// Builds the built-in showcase fixtures used by parity showcase surfaces.
#[must_use]
pub fn built_in_fixtures() -> Vec<ShowcaseFixture> {
    vec![
        shared_fixture(SharedFixtureSpec {
            id: FIXTURE_DIALOG_BASIC,
            family: FAMILY_DIALOG,
            title: "Basic dialog",
        }),
        shared_fixture(SharedFixtureSpec {
            id: FIXTURE_CHROME_SHELL,
            family: FAMILY_CHROME,
            title: "Chrome shell",
        }),
        ShowcaseFixture::new(
            FIXTURE_KNOWLEDGE_WORKFLOW,
            FAMILY_KNOWLEDGE,
            "Knowledge workflow",
        )
        .with_state(
            FixtureState::new(KNOWLEDGE_STATE_CAPTURE, KNOWLEDGE_LABEL_CAPTURE_BACKLINKS)
                .with_data(FixtureDatum::new(
                    KNOWLEDGE_DATA_GRAPH,
                    KNOWLEDGE_GRAPH_FACTS,
                ))
                .with_data(FixtureDatum::new(
                    KNOWLEDGE_DATA_AGENDA,
                    KNOWLEDGE_AGENDA_FACTS,
                ))
                .with_data(FixtureDatum::new(
                    KNOWLEDGE_DATA_METADATA,
                    KNOWLEDGE_METADATA_FACTS,
                ))
                .with_data(FixtureDatum::new(
                    KNOWLEDGE_DATA_DIAGNOSTICS,
                    KNOWLEDGE_DIAGNOSTIC_FACTS,
                )),
        )
        .with_control(FixtureControl::new(
            KNOWLEDGE_CONTROL_SEARCH,
            KNOWLEDGE_CONTROL_SEARCH_LABEL,
        ))
        .with_control(FixtureControl::new(
            KNOWLEDGE_CONTROL_CAPTURE,
            KNOWLEDGE_CONTROL_CAPTURE_LABEL,
        ))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Subwayrat))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Meteorite)),
        shared_fixture(SharedFixtureSpec {
            id: FIXTURE_THEME_TOKENS,
            family: FAMILY_THEME,
            title: "Theme tokens",
        }),
    ]
}

/// Builds the built-in fixture parity rows used by showcase surfaces.
#[must_use]
pub fn built_in_fixture_parity_rows() -> Vec<FixtureParityRow> {
    vec![
        shared_parity_row(SharedParitySpec {
            id: FIXTURE_DIALOG_BASIC,
            family: FAMILY_DIALOG,
            documentation_target: DOC_DIALOG_BASIC,
        }),
        shared_parity_row(SharedParitySpec {
            id: FIXTURE_CHROME_SHELL,
            family: FAMILY_CHROME,
            documentation_target: DOC_CHROME_SHELL,
        }),
        FixtureParityRow::new(
            FIXTURE_KNOWLEDGE_WORKFLOW,
            FAMILY_KNOWLEDGE,
            DOC_KNOWLEDGE_WORKFLOW,
        )
        .with_renderer(FixtureParityRenderer::supported(
            ShowcaseRenderer::Subwayrat,
            FixtureParityEvidenceStatus::Complete,
        ))
        .with_renderer(FixtureParityRenderer::supported(
            ShowcaseRenderer::Meteorite,
            FixtureParityEvidenceStatus::Complete,
        )),
        shared_parity_row(SharedParitySpec {
            id: FIXTURE_THEME_TOKENS,
            family: FAMILY_THEME,
            documentation_target: DOC_THEME_TOKENS,
        }),
    ]
}

/// Builds a display-ready parity summary from rows and fixture metadata.
#[must_use]
pub fn build_fixture_parity_summary(
    rows: &[FixtureParityRow],
    fixtures: &[ShowcaseFixture],
) -> FixtureParitySummary {
    let parity_status = validate_fixture_parity_baseline(rows, fixtures);
    let mut summary_rows = parity_status
        .rows
        .iter()
        .map(summary_row)
        .collect::<Vec<FixtureParitySummaryRow>>();
    summary_rows.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    FixtureParitySummary {
        rows: summary_rows,
        edge_states: Vec::new(),
        issues: parity_status.issues,
        edge_issues: Vec::new(),
    }
}

/// Builds the built-in parity summary used by showcase renderers.
#[must_use]
pub fn built_in_fixture_parity_summary() -> FixtureParitySummary {
    let mut summary =
        build_fixture_parity_summary(&built_in_fixture_parity_rows(), &built_in_fixtures());
    let edge_display = built_in_edge_display();
    summary.edge_states = edge_display.rows;
    summary.edge_issues = edge_display.issues;
    summary
}

struct SharedFixtureSpec {
    id: &'static str,
    family: &'static str,
    title: &'static str,
}

fn shared_fixture(spec: SharedFixtureSpec) -> ShowcaseFixture {
    ShowcaseFixture::new(spec.id, spec.family, spec.title)
        .with_state(FixtureState::new("default", "Default"))
        .with_control(FixtureControl::new("inspect", "Inspect fixture evidence"))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Subwayrat))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Meteorite))
}

struct SharedParitySpec {
    id: &'static str,
    family: &'static str,
    documentation_target: &'static str,
}

fn shared_parity_row(spec: SharedParitySpec) -> FixtureParityRow {
    FixtureParityRow::new(spec.id, spec.family, spec.documentation_target)
        .with_renderer(FixtureParityRenderer::supported(
            ShowcaseRenderer::Subwayrat,
            FixtureParityEvidenceStatus::Complete,
        ))
        .with_renderer(FixtureParityRenderer::supported(
            ShowcaseRenderer::Meteorite,
            FixtureParityEvidenceStatus::Complete,
        ))
}

fn summary_row(row: &FixtureParityRow) -> FixtureParitySummaryRow {
    FixtureParitySummaryRow {
        fixture_id: row.fixture_id.clone(),
        family: row.family.clone(),
        subwayrat: renderer_summary(row, ShowcaseRenderer::Subwayrat),
        meteorite: renderer_summary(row, ShowcaseRenderer::Meteorite),
        documentation_target: row.documentation_target.clone(),
        status: parity_status(row).to_string(),
    }
}

fn renderer_summary(row: &FixtureParityRow, renderer: ShowcaseRenderer) -> String {
    row.renderers
        .iter()
        .find(|entry| entry.renderer == renderer)
        .map_or_else(
            || "missing".to_string(),
            |entry| {
                if entry.supported {
                    format!("{STATUS_SUPPORTED} ({})", evidence_status(entry.evidence))
                } else {
                    format!(
                        "{STATUS_UNSUPPORTED}: {}",
                        entry
                            .unsupported_reason
                            .as_deref()
                            .unwrap_or("reason missing")
                    )
                }
            },
        )
}

fn parity_status(row: &FixtureParityRow) -> &'static str {
    let has_subwayrat_renderer = renderer_supported(row, ShowcaseRenderer::Subwayrat);
    let has_meteorite_renderer = renderer_supported(row, ShowcaseRenderer::Meteorite);
    match (has_subwayrat_renderer, has_meteorite_renderer) {
        (true, true) => STATUS_SHARED,
        (true, false) => STATUS_TUI_ONLY,
        (false, true) => STATUS_METEORITE_ONLY,
        (false, false) => STATUS_UNSUPPORTED,
    }
}

fn renderer_supported(row: &FixtureParityRow, renderer: ShowcaseRenderer) -> bool {
    row.renderers
        .iter()
        .any(|entry| entry.renderer == renderer && entry.supported)
}

fn evidence_status(status: FixtureParityEvidenceStatus) -> &'static str {
    match status {
        FixtureParityEvidenceStatus::Complete => STATUS_EVIDENCE_COMPLETE,
        FixtureParityEvidenceStatus::Missing => STATUS_EVIDENCE_MISSING,
        FixtureParityEvidenceStatus::Unsupported => STATUS_EVIDENCE_UNSUPPORTED,
    }
}

#[cfg(test)]
#[path = "showcase/tests.rs"]
mod tests;
