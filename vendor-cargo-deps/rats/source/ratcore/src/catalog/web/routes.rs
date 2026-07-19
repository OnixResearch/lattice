/// Builds fixture navigation rows for one browser showcase entry.
#[must_use]
pub fn browser_fixture_navigation_rows(
    entry: &super::BrowserShowcaseEntry,
    fixtures: &[super::super::ShowcaseFixture],
) -> super::BrowserFixtureNavigation {
    let fixture_map = fixtures
        .iter()
        .map(|fixture| (fixture.id.as_str(), fixture))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut rows = Vec::with_capacity(entry.supported_fixture_ids.len());
    let mut issues = Vec::with_capacity(entry.supported_fixture_ids.len());
    debug_assert!(rows.capacity() >= entry.supported_fixture_ids.len());
    debug_assert!(issues.capacity() >= entry.supported_fixture_ids.len());
    for fixture_id in &entry.supported_fixture_ids {
        match fixture_map.get(fixture_id.as_str()) {
            Some(fixture) => rows.push(navigation_row(entry, fixture)),
            None => issues.push(super::rules::unknown_fixture_issue(entry, fixture_id)),
        }
    }
    rows.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    super::BrowserFixtureNavigation { rows, issues }
}

fn navigation_row(
    entry: &super::BrowserShowcaseEntry,
    fixture: &super::super::ShowcaseFixture,
) -> super::BrowserFixtureNavigationRow {
    super::BrowserFixtureNavigationRow {
        fixture_id: fixture.id.clone(),
        label: fixture.title.clone(),
        renderer_label: entry.renderer_label.clone(),
        support_status: entry.evidence_status.label().to_string(),
        evidence_text: evidence_text(entry),
        launch_context: entry.launch_command.clone(),
    }
}

fn evidence_text(entry: &super::BrowserShowcaseEntry) -> String {
    if !entry.evidence_links.is_empty() {
        entry.evidence_links.join(super::LIST_SEPARATOR)
    } else if !entry.unsupported_reasons.is_empty() {
        entry.unsupported_reasons.join(super::REASON_SEPARATOR)
    } else {
        entry.evidence_status.label().to_string()
    }
}
