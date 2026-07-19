use super::*;

/// Validates fixtures and computes renderer coverage.
#[must_use]
pub fn validate_showcase_fixtures(fixtures: &[ShowcaseFixture]) -> FixtureSetReport {
    let issues = fixture_issues(fixtures);
    let mut coverage = fixtures
        .iter()
        .map(super::metrics::coverage_entry)
        .collect::<Vec<_>>();
    coverage.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    FixtureSetReport { issues, coverage }
}

/// Returns fixtures supported by a renderer without silently omitting validation.
#[must_use]
pub fn fixtures_for_renderer(
    fixtures: &[ShowcaseFixture],
    renderer: ShowcaseRenderer,
) -> (Vec<ShowcaseFixture>, Vec<FixtureIssue>) {
    let outcome = validate_showcase_fixtures(fixtures);
    let matching_fixtures = fixtures
        .iter()
        .filter(|fixture| {
            fixture
                .renderers
                .iter()
                .any(|entry| entry.renderer == renderer && entry.supported)
        })
        .cloned()
        .collect();
    (matching_fixtures, outcome.issues)
}

/// Validates a release-facing cross-renderer fixture parity baseline.
#[must_use]
pub fn validate_fixture_parity_baseline(
    rows: &[FixtureParityRow],
    fixtures: &[ShowcaseFixture],
) -> FixtureParityReport {
    let fixture_status = validate_showcase_fixtures(fixtures);
    let fixture_map = fixtures
        .iter()
        .map(|fixture| (fixture.id.as_str(), fixture))
        .collect::<std::collections::BTreeMap<_, _>>();
    let rejected_fixture_ids = fixture_status
        .issues
        .iter()
        .map(|issue| issue.subject.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut issues = Vec::with_capacity(rows.len().saturating_add(fixture_status.issues.len()));
    let mut seen_rows = std::collections::BTreeSet::new();
    debug_assert!(fixture_map.len() <= fixtures.len());
    debug_assert!(issues.capacity() >= rows.len());
    for row in rows {
        if !seen_rows.insert(row.fixture_id.clone()) {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::ParityDuplicateRow,
                row.fixture_id.clone(),
                "duplicate fixture parity row",
            ));
        }
        issues.extend(single_parity_row_issues(
            row,
            fixture_map.get(row.fixture_id.as_str()).copied(),
            &rejected_fixture_ids,
        ));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    FixtureParityReport {
        issues,
        rows: sorted_rows,
    }
}

fn fixture_issues(fixtures: &[ShowcaseFixture]) -> Vec<FixtureIssue> {
    let mut issues = Vec::with_capacity(fixtures.len());
    let mut ids = std::collections::BTreeSet::new();
    for fixture in fixtures {
        if !ids.insert(fixture.id.clone()) {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::DuplicateFixtureId,
                fixture.id.clone(),
                "duplicate fixture ID",
            ));
        }
        issues.extend(single_fixture_issues(fixture));
    }
    issues
}

fn single_parity_row_issues(
    row: &FixtureParityRow,
    fixture: Option<&ShowcaseFixture>,
    rejected_fixture_ids: &std::collections::BTreeSet<&str>,
) -> Vec<FixtureIssue> {
    let issue_bound = row.renderers.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    if row.documentation_target.trim().is_empty() {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::ParityMissingDocumentationTarget,
            row.fixture_id.clone(),
            "fixture parity row requires a generated documentation target",
        ));
    }
    match fixture {
        Some(fixture) if rejected_fixture_ids.contains(fixture.id.as_str()) => {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::ParityRejectedFixture,
                row.fixture_id.clone(),
                "fixture parity row references a fixture rejected by catalog validation",
            ));
        }
        Some(fixture) => issues.extend(parity_staleness_issues(row, fixture)),
        None => issues.push(FixtureIssue::new(
            FixtureIssueCode::ParityUnknownFixture,
            row.fixture_id.clone(),
            "fixture parity row references an unknown fixture",
        )),
    }
    issues.extend(parity_renderer_issues(row));
    issues
}

fn parity_renderer_issues(row: &FixtureParityRow) -> Vec<FixtureIssue> {
    let mut issues = Vec::with_capacity(row.renderers.len().saturating_add(1));
    let renderer_map = row
        .renderers
        .iter()
        .map(|entry| (entry.renderer, entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    debug_assert!(renderer_map.len() <= row.renderers.len());
    for renderer in [ShowcaseRenderer::Subwayrat, ShowcaseRenderer::Meteorite] {
        if !renderer_map.contains_key(&renderer) {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::ParityMissingRenderer,
                row.fixture_id.clone(),
                format!(
                    "fixture parity row is missing `{}` renderer metadata",
                    renderer.label()
                ),
            ));
        }
    }
    for entry in &row.renderers {
        if !entry.supported
            && entry
                .unsupported_reason
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::ParityMissingUnsupportedReason,
                row.fixture_id.clone(),
                format!(
                    "unsupported `{}` parity renderer requires a reason",
                    entry.renderer.label()
                ),
            ));
        }
    }
    issues
}

fn parity_staleness_issues(row: &FixtureParityRow, fixture: &ShowcaseFixture) -> Vec<FixtureIssue> {
    let mut issues = Vec::with_capacity(row.renderers.len().saturating_add(1));
    debug_assert_eq!(row.fixture_id, fixture.id);
    debug_assert!(issues.capacity() > row.renderers.len());
    if row.family != fixture.family {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::ParityStaleCoverage,
            row.fixture_id.clone(),
            format!(
                "fixture parity family `{}` does not match catalog family `{}`",
                row.family, fixture.family
            ),
        ));
    }
    let fixture_renderers = fixture
        .renderers
        .iter()
        .map(|entry| (entry.renderer, entry.supported))
        .collect::<std::collections::BTreeMap<_, _>>();
    for parity_renderer in &row.renderers {
        if fixture_renderers.get(&parity_renderer.renderer) != Some(&parity_renderer.supported) {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::ParityStaleCoverage,
                row.fixture_id.clone(),
                format!(
                    "fixture parity renderer `{}` no longer matches catalog coverage",
                    parity_renderer.renderer.label()
                ),
            ));
        }
    }
    issues
}

fn single_fixture_issues(fixture: &ShowcaseFixture) -> Vec<FixtureIssue> {
    let issue_bound = fixture
        .states
        .len()
        .saturating_add(fixture.renderers.len())
        .saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    debug_assert!(fixture.id.trim().len() <= fixture.id.len());
    if fixture.id.trim().is_empty() || fixture.title.trim().is_empty() {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::MalformedFixture,
            fixture.id.clone(),
            "fixture ID and title are required",
        ));
    }
    if !SUPPORTED_FAMILIES.contains(&fixture.family.as_str()) {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::UnsupportedFamily,
            fixture.id.clone(),
            format!("unsupported family `{}`", fixture.family),
        ));
    }
    if fixture.states.is_empty() {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::MissingState,
            fixture.id.clone(),
            "at least one fixture state is required",
        ));
    }
    for state in &fixture.states {
        if state.id.trim().is_empty()
            || state.label.trim().is_empty()
            || state.data.iter().any(|datum| datum.key.trim().is_empty())
        {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::InvalidStateData,
                fixture.id.clone(),
                "state IDs, labels, and data keys must be present",
            ));
        }
    }
    issues.extend(renderer_issues(fixture));
    issues
}

fn renderer_issues(fixture: &ShowcaseFixture) -> Vec<FixtureIssue> {
    let issue_bound = fixture.renderers.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    debug_assert!(fixture.renderers.len() <= issue_bound);
    if fixture.renderers.is_empty() {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::UnsupportedRenderer,
            fixture.id.clone(),
            "fixture must declare renderer support",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for support in &fixture.renderers {
        if !seen.insert(support.renderer) {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::UnsupportedRenderer,
                fixture.id.clone(),
                format!("duplicate renderer `{}`", support.renderer.label()),
            ));
        }
        if !support.supported
            && support
                .reason
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::UnsupportedRenderer,
                fixture.id.clone(),
                "unsupported renderer declarations require a reason",
            ));
        }
    }
    issues
}
