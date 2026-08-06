use super::super::*;

pub(super) fn fixture_issues(fixtures: &[ShowcaseFixture]) -> Vec<FixtureIssue> {
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

fn single_fixture_issues(fixture: &ShowcaseFixture) -> Vec<FixtureIssue> {
    let mut issues = [
        malformed_fixture_issue(fixture),
        unsupported_family_issue(fixture),
        missing_state_issue(fixture),
        missing_renderer_issue(fixture),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    issues.extend(state_issues(fixture));
    issues
}

fn malformed_fixture_issue(fixture: &ShowcaseFixture) -> Option<FixtureIssue> {
    (fixture.id.as_str().trim().is_empty() || fixture.title.trim().is_empty()).then(|| {
        FixtureIssue::new(
            FixtureIssueCode::MalformedFixture,
            fixture.id.to_string(),
            "fixture ID and title are required",
        )
    })
}

fn unsupported_family_issue(fixture: &ShowcaseFixture) -> Option<FixtureIssue> {
    (!SUPPORTED_FAMILIES.contains(&fixture.family.as_str())).then(|| {
        FixtureIssue::new(
            FixtureIssueCode::UnsupportedFamily,
            fixture.id.clone(),
            format!("unsupported family `{}`", fixture.family),
        )
    })
}

fn missing_state_issue(fixture: &ShowcaseFixture) -> Option<FixtureIssue> {
    fixture.states.is_empty().then(|| {
        FixtureIssue::new(FixtureIssueCode::MissingState, fixture.id.clone(), "at least one fixture state is required")
    })
}

fn missing_renderer_issue(fixture: &ShowcaseFixture) -> Option<FixtureIssue> {
    fixture.renderer_cells.is_empty().then(|| {
        FixtureIssue::new(
            FixtureIssueCode::UnsupportedRenderer,
            fixture.id.clone(),
            "fixture must declare renderer support",
        )
    })
}

fn state_issues(fixture: &ShowcaseFixture) -> Vec<FixtureIssue> {
    fixture
        .states
        .iter()
        .filter(|state| {
            state.id.as_str().trim().is_empty()
                || state.label.trim().is_empty()
                || state.data.iter().any(|datum| datum.key.trim().is_empty())
        })
        .map(|_| {
            FixtureIssue::new(
                FixtureIssueCode::InvalidStateData,
                fixture.id.to_string(),
                "state IDs, labels, and data keys must be present",
            )
        })
        .collect()
}
