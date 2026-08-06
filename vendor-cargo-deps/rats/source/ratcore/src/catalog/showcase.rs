use super::*;

#[path = "showcase/data.rs"]
mod data;
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

pub use data::built_in_fixture_parity_rows;
pub use data::built_in_fixtures;

const STATUS_SHARED: &str = "shared";
const STATUS_TUI_ONLY: &str = "tui-only";
const STATUS_METEORITE_ONLY: &str = "meteorite-only";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_SUPPORTED: &str = "supported";
const STATUS_EVIDENCE_COMPLETE: &str = "complete";
const STATUS_EVIDENCE_MISSING: &str = "missing";
const STATUS_EVIDENCE_UNSUPPORTED: &str = "unsupported";

/// One ordered display-ready renderer family summary cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParitySummaryCell {
    /// Nominal renderer family identity.
    pub family: crate::targeting_core::RendererFamilyId,
    /// Support, evidence, or gap summary.
    pub text: String,
}

/// Display-ready parity summary row shared by showcase renderers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParitySummaryRow {
    /// Fixture ID.
    pub fixture_id: FixtureId,
    /// Component family.
    pub family: String,
    /// Subwayrat support and evidence text.
    pub subwayrat: String,
    /// Meteorite support and evidence text.
    pub meteorite: String,
    /// Ordered source-of-truth renderer family summaries.
    pub renderer_cells: Vec<FixtureParitySummaryCell>,
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

/// Builds a display-ready parity summary for the maintained default family scope.
#[must_use]
pub fn build_fixture_parity_summary(rows: &[FixtureParityRow], fixtures: &[ShowcaseFixture]) -> FixtureParitySummary {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::FixtureParityFamilies,
    );
    build_fixture_parity_summary_with_policy(rows, fixtures, &registry, &scope)
}

/// Builds a display-ready parity summary for an explicit renderer family scope.
#[must_use]
pub fn build_fixture_parity_summary_with_policy(
    rows: &[FixtureParityRow],
    fixtures: &[ShowcaseFixture],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> FixtureParitySummary {
    let parity_status = validate_fixture_parity_baseline_with_policy(rows, fixtures, registry, scope);
    let mut summary_rows = parity_status
        .rows
        .iter()
        .map(|row| summary_row(row, scope))
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
    let mut summary = build_fixture_parity_summary(&built_in_fixture_parity_rows(), &built_in_fixtures());
    let edge_display = built_in_edge_display();
    summary.edge_states = edge_display.rows;
    summary.edge_issues = edge_display.issues;
    summary
}

fn summary_row(
    row: &FixtureParityRow,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> FixtureParitySummaryRow {
    FixtureParitySummaryRow {
        fixture_id: row.fixture_id.clone(),
        family: row.family.clone(),
        subwayrat: renderer_summary(row, &ShowcaseRenderer::Subwayrat.family_id()),
        meteorite: renderer_summary(row, &ShowcaseRenderer::Meteorite.family_id()),
        renderer_cells: scope.members.iter().filter_map(|member| summary_cell(row, member)).collect(),
        documentation_target: row.documentation_target.clone(),
        status: parity_status(row).to_string(),
    }
}

fn summary_cell(
    row: &FixtureParityRow,
    member: &crate::targeting_core::RendererSubject,
) -> Option<FixtureParitySummaryCell> {
    let crate::targeting_core::RendererSubject::Family(family) = member else {
        return None;
    };
    Some(FixtureParitySummaryCell {
        family: family.clone(),
        text: renderer_summary(row, family),
    })
}

fn renderer_summary(row: &FixtureParityRow, family: &crate::targeting_core::RendererFamilyId) -> String {
    row.renderer_cells.iter().find(|entry| entry.family == *family).map_or_else(
        || "missing".to_string(),
        |entry| {
            if entry.supported {
                format!("{STATUS_SUPPORTED} ({})", evidence_status(entry.evidence))
            } else {
                format!("{STATUS_UNSUPPORTED}: {}", entry.unsupported_reason.as_deref().unwrap_or("reason missing"))
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
    let family = renderer.family_id();
    row.renderer_cells.iter().any(|entry| entry.family == family && entry.supported)
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
