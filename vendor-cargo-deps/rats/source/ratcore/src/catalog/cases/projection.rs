const FIXTURE_DIALOG: &str = "dialog.basic";
const FIXTURE_CHROME: &str = "chrome.shell";
const COVERAGE_DOC: &str = "docs/showcase-edge-coverage.md";
const STATE_DEFAULT: &str = "default";
const STATE_EMPTY: &str = "empty";
const STATE_INVALID: &str = "invalid-input";
const STATE_OVERSIZED: &str = "oversized-content";
const DIAGNOSTIC_EMPTY: &str = "empty message renders placeholder text";
const DIAGNOSTIC_INVALID: &str = "input is rejected before rendering";
const DIAGNOSTIC_OVERSIZED: &str = "oversized content names clipping or responsive behavior";
const METEORITE_OVERSIZED_REASON: &str = "web shell uses responsive layout instead of terminal clipping";

/// Display-ready edge-case row shared by showcase renderers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDisplayRow {
    /// Fixture ID from the shared catalog.
    pub fixture_id: super::super::FixtureId,
    /// Stable edge state ID.
    pub state_id: super::super::FixtureStateId,
    /// Display label.
    pub label: String,
    /// Edge state kind.
    pub kind: String,
    /// Subwayrat visibility and reason text.
    pub subwayrat: String,
    /// Meteorite visibility and reason text.
    pub meteorite: String,
    /// Expected diagnostics joined for display.
    pub diagnostics: String,
}

/// Display-ready edge-case rows and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDisplay {
    /// Rows sorted by fixture and state ID.
    pub rows: Vec<EdgeDisplayRow>,
    /// Deterministic validation diagnostics.
    pub issues: Vec<super::Issue>,
}

impl EdgeDisplay {
    /// Returns true when the edge-case display has no diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Builds the built-in edge-case fixtures used by showcase parity surfaces.
#[must_use]
pub fn built_in_edge_fixtures() -> Vec<super::Fixture> {
    vec![
        super::Fixture::new(FIXTURE_DIALOG, COVERAGE_DOC)
            .with_state(shared_state(StateSpec {
                id: STATE_DEFAULT,
                label: "Default",
                kind: super::Kind::HappyPath,
            }))
            .with_state(
                shared_state(StateSpec {
                    id: STATE_EMPTY,
                    label: "Empty",
                    kind: super::Kind::Empty,
                })
                .with_diagnostic(DIAGNOSTIC_EMPTY),
            )
            .with_state(
                shared_state(StateSpec {
                    id: STATE_INVALID,
                    label: "Invalid input",
                    kind: super::Kind::Invalid,
                })
                .with_diagnostic(DIAGNOSTIC_INVALID),
            ),
        super::Fixture::new(FIXTURE_CHROME, COVERAGE_DOC).with_state(
            super::State::new(STATE_OVERSIZED, "Oversized content", super::Kind::Oversized)
                .with_diagnostic(DIAGNOSTIC_OVERSIZED)
                .with_renderer(super::Renderer::visible(super::super::ShowcaseRenderer::Subwayrat))
                .with_renderer(super::Renderer::unsupported(
                    super::super::ShowcaseRenderer::Meteorite,
                    METEORITE_OVERSIZED_REASON,
                )),
        ),
    ]
}

/// Builds display-ready edge-case rows from catalog entries.
#[must_use]
pub fn build_edge_display(
    fixtures: &[super::Fixture],
    accepted_fixture_ids: &[super::super::FixtureId],
) -> EdgeDisplay {
    let validation = super::validate(fixtures, accepted_fixture_ids);
    let mut rows = validation.fixtures.iter().flat_map(fixture_rows).collect::<Vec<_>>();
    rows.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id).then(left.state_id.cmp(&right.state_id)));
    EdgeDisplay {
        rows,
        issues: validation.issues,
    }
}

/// Builds the built-in edge display used by showcase renderers.
#[must_use]
pub fn built_in_edge_display() -> EdgeDisplay {
    let fixture_ids = super::super::built_in_fixtures().iter().map(|fixture| fixture.id.clone()).collect::<Vec<_>>();
    build_edge_display(&built_in_edge_fixtures(), &fixture_ids)
}

struct StateSpec {
    id: &'static str,
    label: &'static str,
    kind: super::Kind,
}

fn shared_state(spec: StateSpec) -> super::State {
    super::State::new(spec.id, spec.label, spec.kind)
        .with_renderer(super::Renderer::visible(super::super::ShowcaseRenderer::Subwayrat))
        .with_renderer(super::Renderer::visible(super::super::ShowcaseRenderer::Meteorite))
}

fn fixture_rows(fixture: &super::Fixture) -> impl Iterator<Item = EdgeDisplayRow> + '_ {
    fixture.states.iter().map(|state| EdgeDisplayRow {
        fixture_id: fixture.fixture_id.clone(),
        state_id: state.id.clone(),
        label: state.label.clone(),
        kind: kind_label(state.kind).to_string(),
        subwayrat: renderer_text(state, super::super::ShowcaseRenderer::Subwayrat),
        meteorite: renderer_text(state, super::super::ShowcaseRenderer::Meteorite),
        diagnostics: diagnostic_text(state),
    })
}

fn renderer_text(state: &super::State, renderer: super::super::ShowcaseRenderer) -> String {
    state.renderers.iter().find(|entry| entry.renderer == renderer).map_or_else(
        || "missing renderer metadata".to_string(),
        |entry| {
            if entry.visible {
                "visible".to_string()
            } else {
                format!("unsupported: {}", entry.reason.as_deref().unwrap_or("reason missing"))
            }
        },
    )
}

fn diagnostic_text(state: &super::State) -> String {
    if state.diagnostics.is_empty() {
        "none".to_string()
    } else {
        state.diagnostics.join("; ")
    }
}

fn kind_label(kind: super::Kind) -> &'static str {
    match kind {
        super::Kind::HappyPath => "happy-path",
        super::Kind::Empty => "empty",
        super::Kind::Boundary => "boundary",
        super::Kind::Disabled => "disabled",
        super::Kind::Invalid => "invalid",
        super::Kind::Malformed => "malformed",
        super::Kind::Oversized => "oversized",
        super::Kind::Unsupported => "unsupported",
    }
}

#[cfg(test)]
mod tests {
    const EXPECTED_BUILT_IN_ROWS: usize = 4;

    #[test]
    fn built_in_rows_include_invalid_and_unsupported_visibility() {
        let display = super::built_in_edge_display();

        assert!(display.is_valid(), "issues: {:?}", display.issues);
        assert_eq!(display.rows.len(), EXPECTED_BUILT_IN_ROWS);
        assert!(
            display
                .rows
                .iter()
                .any(|row| row.state_id == super::STATE_INVALID && row.diagnostics.contains("rejected"))
        );
        assert!(display.rows.iter().any(|row| row.meteorite.contains("responsive layout")));
    }

    #[test]
    fn malformed_rows_keep_diagnostics_visible() {
        let display = super::build_edge_display(&[super::super::Fixture::new("missing.fixture", "")], &[
            crate::showcase_fixtures::FixtureId::new(super::FIXTURE_DIALOG),
        ]);

        assert!(!display.is_valid());
        assert!(display.rows.is_empty());
        assert!(!display.issues.is_empty());
    }
}
