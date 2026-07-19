use super::*;

pub(super) fn coverage_entry(fixture: &ShowcaseFixture) -> FixtureCoverageEntry {
    let renderer_entries = fixture
        .renderers
        .iter()
        .map(|entry| (entry.renderer, entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let has_tui_renderer = renderer_entries
        .get(&ShowcaseRenderer::Subwayrat)
        .is_some_and(|entry| entry.supported);
    let has_meteorite_renderer = renderer_entries
        .get(&ShowcaseRenderer::Meteorite)
        .is_some_and(|entry| entry.supported);
    let kind = match (has_tui_renderer, has_meteorite_renderer) {
        (true, true) => FixtureCoverageKind::Shared,
        (true, false) => FixtureCoverageKind::TuiOnly,
        (false, true) => FixtureCoverageKind::MeteoriteOnly,
        (false, false) => FixtureCoverageKind::Unsupported,
    };
    debug_assert!(renderer_entries.len() <= fixture.renderers.len());
    debug_assert!(fixture.id.trim().len() <= fixture.id.len());
    FixtureCoverageEntry {
        fixture_id: fixture.id.clone(),
        kind,
        gaps: coverage_gaps(&renderer_entries),
    }
}

fn coverage_gaps(
    support: &std::collections::BTreeMap<ShowcaseRenderer, &RendererSupport>,
) -> Vec<String> {
    [ShowcaseRenderer::Subwayrat, ShowcaseRenderer::Meteorite]
        .into_iter()
        .filter_map(|renderer| match support.get(&renderer) {
            Some(entry) if entry.supported => None,
            Some(entry) => Some(format!(
                "{}: {}",
                renderer.label(),
                entry
                    .reason
                    .clone()
                    .unwrap_or_else(|| "unsupported".to_string())
            )),
            None => Some(format!("{}: missing", renderer.label())),
        })
        .collect()
}
