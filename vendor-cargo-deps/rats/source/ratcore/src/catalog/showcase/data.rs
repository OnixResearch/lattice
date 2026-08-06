use super::*;

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
        ShowcaseFixture::new(FIXTURE_KNOWLEDGE_WORKFLOW, FAMILY_KNOWLEDGE, "Knowledge workflow")
            .with_state(
                FixtureState::new(KNOWLEDGE_STATE_CAPTURE, KNOWLEDGE_LABEL_CAPTURE_BACKLINKS)
                    .with_data(FixtureDatum::new(KNOWLEDGE_DATA_GRAPH, KNOWLEDGE_GRAPH_FACTS))
                    .with_data(FixtureDatum::new(KNOWLEDGE_DATA_AGENDA, KNOWLEDGE_AGENDA_FACTS))
                    .with_data(FixtureDatum::new(KNOWLEDGE_DATA_METADATA, KNOWLEDGE_METADATA_FACTS))
                    .with_data(FixtureDatum::new(KNOWLEDGE_DATA_DIAGNOSTICS, KNOWLEDGE_DIAGNOSTIC_FACTS)),
            )
            .with_control(FixtureControl::new(KNOWLEDGE_CONTROL_SEARCH, KNOWLEDGE_CONTROL_SEARCH_LABEL))
            .with_control(FixtureControl::new(KNOWLEDGE_CONTROL_CAPTURE, KNOWLEDGE_CONTROL_CAPTURE_LABEL))
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
        FixtureParityRow::new(FIXTURE_KNOWLEDGE_WORKFLOW, FAMILY_KNOWLEDGE, DOC_KNOWLEDGE_WORKFLOW)
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
