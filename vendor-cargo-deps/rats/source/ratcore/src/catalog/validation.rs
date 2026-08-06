use super::*;

#[path = "validation/policy.rs"]
mod policy;
#[path = "validation/shape.rs"]
mod shape;

/// Validates fixtures against the maintained Subwayrat/Meteorite family scope.
#[must_use]
pub fn validate_showcase_fixtures(fixtures: &[ShowcaseFixture]) -> FixtureSetReport {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::FixtureParityFamilies,
    );
    validate_showcase_fixtures_with_policy(fixtures, &registry, &scope)
}

/// Validates fixtures and computes ordered coverage for an explicit family scope.
#[must_use]
pub fn validate_showcase_fixtures_with_policy(
    fixtures: &[ShowcaseFixture],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> FixtureSetReport {
    FixtureSetReport {
        issues: fixture_policy_issues(fixtures, registry, scope),
        coverage: scoped_coverage(fixtures, registry, scope),
    }
}

fn fixture_policy_issues(
    fixtures: &[ShowcaseFixture],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<FixtureIssue> {
    let policy_issues = policy::registry_scope_issues(registry, scope);
    let mut issues = shape::fixture_issues(fixtures);
    if policy_issues.is_empty() {
        for fixture in fixtures {
            issues.extend(policy::fixture_cell_issues(fixture, registry, scope));
        }
    } else {
        issues.extend(policy_issues);
    }
    issues
}

fn scoped_coverage(
    fixtures: &[ShowcaseFixture],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<FixtureCoverageEntry> {
    if !crate::targeting_core::validate_renderer_scope(registry, scope).is_valid() {
        return Vec::new();
    }
    let mut coverage = fixtures
        .iter()
        .map(|fixture| super::metrics::coverage_entry_with_scope(fixture, scope))
        .collect::<Vec<_>>();
    coverage.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    coverage
}

/// Returns fixtures supported by a renderer without silently omitting validation.
#[must_use]
pub fn fixtures_for_renderer(
    fixtures: &[ShowcaseFixture],
    renderer: ShowcaseRenderer,
) -> (Vec<ShowcaseFixture>, Vec<FixtureIssue>) {
    let outcome = validate_showcase_fixtures(fixtures);
    let family = renderer.family_id();
    let matching_fixtures = fixtures
        .iter()
        .filter(|fixture| fixture.renderer_cells.iter().any(|entry| entry.family == family && entry.supported))
        .cloned()
        .collect();
    (matching_fixtures, outcome.issues)
}

/// Validates a release-facing baseline against the maintained default family scope.
#[must_use]
pub fn validate_fixture_parity_baseline(
    rows: &[FixtureParityRow],
    fixtures: &[ShowcaseFixture],
) -> FixtureParityReport {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::FixtureParityFamilies,
    );
    validate_fixture_parity_baseline_with_policy(rows, fixtures, &registry, &scope)
}

/// Validates a fixture parity baseline against an explicit renderer family scope.
#[must_use]
pub fn validate_fixture_parity_baseline_with_policy(
    rows: &[FixtureParityRow],
    fixtures: &[ShowcaseFixture],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> FixtureParityReport {
    let fixture_status = validate_showcase_fixtures_with_policy(fixtures, registry, scope);
    let fixture_map =
        fixtures.iter().map(|fixture| (&fixture.id, fixture)).collect::<std::collections::BTreeMap<_, _>>();
    let rejected = rejected_fixture_ids(&fixture_status.issues);
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
            fixture_map.get(&row.fixture_id).copied(),
            &rejected,
            registry,
            scope,
        ));
    }
    issues.extend(fixture_status.issues);
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    FixtureParityReport {
        issues,
        rows: sorted_rows,
    }
}

fn rejected_fixture_ids(issues: &[FixtureIssue]) -> std::collections::BTreeSet<FixtureId> {
    issues
        .iter()
        .filter(|issue| {
            !matches!(
                issue.code,
                FixtureIssueCode::InvalidRendererPolicy
                    | FixtureIssueCode::MissingRendererCell
                    | FixtureIssueCode::InvalidRendererCell
            )
        })
        .map(|issue| FixtureId::new(&issue.subject))
        .collect()
}

fn single_parity_row_issues(
    row: &FixtureParityRow,
    fixture: Option<&ShowcaseFixture>,
    rejected: &std::collections::BTreeSet<FixtureId>,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<FixtureIssue> {
    let mut issues = documentation_target_issues(row);
    issues.extend(fixture_reference_issues(row, fixture, rejected));
    if crate::targeting_core::validate_renderer_scope(registry, scope).is_valid() {
        issues.extend(policy::parity_cell_issues(row, registry, scope));
    }
    issues
}

fn fixture_reference_issues(
    row: &FixtureParityRow,
    fixture: Option<&ShowcaseFixture>,
    rejected: &std::collections::BTreeSet<FixtureId>,
) -> Vec<FixtureIssue> {
    match fixture {
        Some(fixture) if rejected.contains(&fixture.id) => vec![FixtureIssue::new(
            FixtureIssueCode::ParityRejectedFixture,
            row.fixture_id.clone(),
            "fixture parity row references a fixture rejected by catalog validation",
        )],
        Some(fixture) => parity_staleness_issues(row, fixture),
        None => vec![FixtureIssue::new(
            FixtureIssueCode::ParityUnknownFixture,
            row.fixture_id.clone(),
            "fixture parity row references an unknown fixture",
        )],
    }
}

fn documentation_target_issues(row: &FixtureParityRow) -> Vec<FixtureIssue> {
    if row.documentation_target.trim().is_empty() {
        vec![FixtureIssue::new(
            FixtureIssueCode::ParityMissingDocumentationTarget,
            row.fixture_id.clone(),
            "fixture parity row requires a generated documentation target",
        )]
    } else {
        Vec::new()
    }
}

fn parity_staleness_issues(row: &FixtureParityRow, fixture: &ShowcaseFixture) -> Vec<FixtureIssue> {
    let mut issues = Vec::with_capacity(row.renderer_cells.len().saturating_add(1));
    debug_assert_eq!(row.fixture_id, fixture.id);
    debug_assert!(issues.capacity() > row.renderer_cells.len());
    if row.family != fixture.family {
        issues.push(FixtureIssue::new(
            FixtureIssueCode::ParityStaleCoverage,
            row.fixture_id.clone(),
            format!("fixture parity family `{}` does not match catalog family `{}`", row.family, fixture.family),
        ));
    }
    let fixture_cells = fixture
        .renderer_cells
        .iter()
        .map(|entry| (&entry.family, entry.supported))
        .collect::<std::collections::BTreeMap<_, _>>();
    for parity_cell in &row.renderer_cells {
        if fixture_cells.get(&parity_cell.family) != Some(&parity_cell.supported) {
            issues.push(FixtureIssue::new(
                FixtureIssueCode::ParityStaleCoverage,
                row.fixture_id.clone(),
                format!("fixture parity renderer `{}` no longer matches catalog coverage", parity_cell.family),
            ));
        }
    }
    issues
}
