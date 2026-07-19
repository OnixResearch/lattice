/// Validates edge-case fixtures against accepted fixture IDs.
#[must_use]
pub fn validate(fixtures: &[super::Fixture], accepted_fixture_ids: &[&str]) -> super::Report {
    let mut issues = Vec::with_capacity(fixtures.len());
    let accepted = accepted_fixture_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut seen_fixtures = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= fixtures.len());
    debug_assert!(accepted.len() <= accepted_fixture_ids.len());
    for fixture in fixtures {
        if !seen_fixtures.insert(fixture.fixture_id.clone()) {
            issues.push(super::Issue::new(
                super::IssueCode::DuplicateFixtureId,
                fixture.fixture_id.clone(),
                "fixture_id",
                "duplicate edge fixture ID",
            ));
        }
        issues.extend(fixture_issues(fixture, &accepted));
    }
    let mut sorted = fixtures.to_vec();
    sorted.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    super::Report {
        issues,
        fixtures: sorted,
    }
}

fn fixture_issues(
    fixture: &super::Fixture,
    accepted: &std::collections::BTreeSet<&str>,
) -> Vec<super::Issue> {
    let issue_bound = fixture.states.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    if fixture.fixture_id.trim().is_empty() || fixture.coverage_reference.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingRequiredField,
            fixture.fixture_id.clone(),
            "fixture",
            "edge fixture ID and coverage reference are required",
        ));
    }
    if !accepted.contains(fixture.fixture_id.as_str()) {
        issues.push(super::Issue::new(
            super::IssueCode::StaleCoverageReference,
            fixture.fixture_id.clone(),
            "fixture_id",
            "edge fixture references an unknown or stale base fixture",
        ));
    }
    let mut seen_states = std::collections::BTreeSet::new();
    for state in &fixture.states {
        if !seen_states.insert(state.id.clone()) {
            issues.push(super::Issue::new(
                super::IssueCode::DuplicateStateId,
                fixture.fixture_id.clone(),
                state.id.clone(),
                "duplicate edge state ID",
            ));
        }
        issues.extend(state_issues(fixture, state));
    }
    issues
}

fn state_issues(fixture: &super::Fixture, state: &super::State) -> Vec<super::Issue> {
    let issue_bound = state.renderers.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    debug_assert!(state.diagnostics.len() <= issue_bound.saturating_add(state.diagnostics.len()));
    if state.id.trim().is_empty() || state.label.trim().is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingRequiredField,
            fixture.fixture_id.clone(),
            state.id.clone(),
            "edge state ID and label are required",
        ));
    }
    if requires_diagnostic(state.kind)
        && state
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.trim().is_empty())
    {
        issues.push(super::Issue::new(
            super::IssueCode::MissingDiagnostic,
            fixture.fixture_id.clone(),
            state.id.clone(),
            "invalid, malformed, and unsupported edge states require diagnostics",
        ));
    }
    if state.renderers.is_empty() {
        issues.push(super::Issue::new(
            super::IssueCode::MissingRendererSupport,
            fixture.fixture_id.clone(),
            state.id.clone(),
            "edge state requires renderer support metadata",
        ));
    }
    for renderer in &state.renderers {
        if !renderer.visible
            && renderer
                .reason
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            issues.push(super::Issue::new(
                super::IssueCode::MissingUnsupportedReason,
                fixture.fixture_id.clone(),
                renderer.renderer.label(),
                "unsupported edge renderer requires a reason",
            ));
        }
    }
    issues
}

fn requires_diagnostic(kind: super::Kind) -> bool {
    matches!(
        kind,
        super::KIND_INVALID | super::KIND_MALFORMED | super::KIND_UNSUPPORTED
    )
}
