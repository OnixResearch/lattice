use super::*;

pub(super) fn coverage_entry_with_scope(
    fixture: &ShowcaseFixture,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> FixtureCoverageEntry {
    let cells_by_family = fixture
        .renderer_cells
        .iter()
        .map(|cell| (&cell.family, cell))
        .collect::<std::collections::BTreeMap<_, _>>();
    let cells = scope.members.iter().filter_map(|member| scoped_cell(member, &cells_by_family)).collect::<Vec<_>>();
    let kind = compatibility_kind(&cells);
    debug_assert!(cells.len() <= scope.members.len());
    debug_assert!(fixture.id.as_str().trim().len() <= fixture.id.as_str().len());
    FixtureCoverageEntry {
        fixture_id: fixture.id.clone(),
        kind,
        gaps: coverage_gaps(&cells),
        cells,
    }
}

fn scoped_cell(
    member: &crate::targeting_core::RendererSubject,
    cells_by_family: &std::collections::BTreeMap<&crate::targeting_core::RendererFamilyId, &FixtureRendererCell>,
) -> Option<FixtureCoverageCell> {
    let crate::targeting_core::RendererSubject::Family(family) = member else {
        return None;
    };
    let declared = cells_by_family.get(family);
    Some(FixtureCoverageCell {
        family: family.clone(),
        supported: declared.map(|cell| cell.supported),
        reason: declared
            .and_then(|cell| cell.reason.clone())
            .or_else(|| declared.is_none().then(|| "missing renderer declaration".to_string())),
    })
}

fn compatibility_kind(cells: &[FixtureCoverageCell]) -> FixtureCoverageKind {
    let has_subwayrat = family_is_supported(cells, crate::targeting_core::FAMILY_SUBWAYRAT);
    let has_meteorite = family_is_supported(cells, crate::targeting_core::FAMILY_METEORITE);
    match (has_subwayrat, has_meteorite) {
        (true, true) => FixtureCoverageKind::Shared,
        (true, false) => FixtureCoverageKind::TuiOnly,
        (false, true) => FixtureCoverageKind::MeteoriteOnly,
        (false, false) => FixtureCoverageKind::Unsupported,
    }
}

fn family_is_supported(cells: &[FixtureCoverageCell], family: &str) -> bool {
    cells.iter().find(|cell| cell.family == family).is_some_and(|cell| cell.supported == Some(true))
}

fn coverage_gaps(cells: &[FixtureCoverageCell]) -> Vec<String> {
    cells
        .iter()
        .filter(|cell| cell.supported != Some(true))
        .map(|cell| format!("{}: {}", cell.family, cell.reason.clone().unwrap_or_else(|| "unsupported".to_string())))
        .collect()
}
