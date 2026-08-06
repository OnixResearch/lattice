use super::*;

const FIXTURE_ID: &str = "dialog.basic";
const EXPECTED_PARITY_REJECTION_ISSUES: usize = 6;

#[test]
fn valid_shared_fixture_reports_shared_coverage() {
    let fixtures = vec![valid_fixture()];

    let outcome = validate_showcase_fixtures(&fixtures);

    assert!(outcome.is_valid());
    assert_eq!(outcome.coverage[0].kind, FixtureCoverageKind::Shared);
    assert!(outcome.coverage[0].gaps.is_empty());
}

#[test]
fn renderer_projection_returns_supported_fixtures_and_issues() {
    let fixtures = vec![valid_fixture()];

    let (supported, issues) = fixtures_for_renderer(&fixtures, ShowcaseRenderer::Subwayrat);

    assert_eq!(supported.len(), 1);
    assert!(issues.is_empty());
}

#[test]
fn invalid_fixture_reports_duplicate_malformed_and_unsupported() {
    let fixtures = vec![
        valid_fixture(),
        ShowcaseFixture::new(FIXTURE_ID, "unknown", "")
            .with_state(FixtureState::new("", "bad"))
            .with_renderer(RendererSupport::unsupported(ShowcaseRenderer::Meteorite, "")),
    ];

    let outcome = validate_showcase_fixtures(&fixtures);

    assert_issue(&outcome, FixtureIssueCode::DuplicateFixtureId);
    assert_issue(&outcome, FixtureIssueCode::MalformedFixture);
    assert_issue(&outcome, FixtureIssueCode::UnsupportedFamily);
    assert_issue(&outcome, FixtureIssueCode::InvalidStateData);
    assert_issue(&outcome, FixtureIssueCode::UnsupportedRenderer);
}

#[test]
fn fixture_family_cells_must_follow_scope_order() {
    let fixture = ShowcaseFixture::new("dialog.reordered", "dialog", "Reordered")
        .with_state(FixtureState::new("default", "Default"))
        .with_renderer_cell(FixtureRendererCell::supported("meteorite"))
        .with_renderer_cell(FixtureRendererCell::supported("subwayrat"));
    let outcome = validate_showcase_fixtures(&[fixture]);
    assert!(!outcome.is_valid());
    assert_issue(&outcome, FixtureIssueCode::InvalidRendererCell);
}

#[test]
fn unsupported_fixture_is_reported_in_coverage() {
    let fixture = ShowcaseFixture::new("web.only", "chrome", "Web only")
        .with_state(FixtureState::new("default", "Default"))
        .with_renderer(RendererSupport::unsupported(ShowcaseRenderer::Subwayrat, "requires browser layout"))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Meteorite));

    let outcome = validate_showcase_fixtures(&[fixture]);

    assert_eq!(outcome.coverage[0].kind, FixtureCoverageKind::MeteoriteOnly);
    assert_eq!(outcome.coverage[0].gaps.len(), 1);
}

#[test]
fn valid_parity_baseline_names_renderer_evidence() {
    let row = parity_row();

    let outcome = validate_fixture_parity_baseline(&[row], &[valid_fixture()]);

    assert!(outcome.is_valid());
    assert_eq!(outcome.rows[0].fixture_id, FIXTURE_ID);
    assert_eq!(outcome.rows[0].renderers.len(), 2);
}

#[test]
fn parity_baseline_rejects_stale_unknown_and_unreasoned_rows() {
    let stale_row = FixtureParityRow::new(FIXTURE_ID, "chrome", "").with_renderer(FixtureParityRenderer::supported(
        ShowcaseRenderer::Subwayrat,
        FixtureParityEvidenceStatus::Complete,
    ));
    let unknown_row = FixtureParityRow::new("missing", "dialog", "docs/missing.md")
        .with_renderer(FixtureParityRenderer::unsupported(ShowcaseRenderer::Subwayrat, "not implemented"))
        .with_renderer(FixtureParityRenderer {
            renderer: ShowcaseRenderer::Meteorite,
            supported: false,
            evidence: FixtureParityEvidenceStatus::Unsupported,
            unsupported_reason: Some(String::new()),
        });

    let outcome = validate_fixture_parity_baseline(&[stale_row.clone(), stale_row, unknown_row], &[valid_fixture()]);

    assert!(!outcome.is_valid());
    assert!(outcome.issues.len() >= EXPECTED_PARITY_REJECTION_ISSUES);
    assert_issue_in(&outcome.issues, FixtureIssueCode::ParityDuplicateRow);
    assert_issue_in(&outcome.issues, FixtureIssueCode::ParityMissingRenderer);
    assert_issue_in(&outcome.issues, FixtureIssueCode::ParityMissingUnsupportedReason);
    assert_issue_in(&outcome.issues, FixtureIssueCode::ParityUnknownFixture);
    assert_issue_in(&outcome.issues, FixtureIssueCode::ParityStaleCoverage);
    assert_issue_in(&outcome.issues, FixtureIssueCode::ParityMissingDocumentationTarget);
}

#[test]
fn three_family_scope_keeps_supported_unsupported_and_missing_cells_visible() {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::RendererRequirementScope::new(
        "fixtures.experimental",
        crate::targeting_core::RendererSubjectKind::Family,
        crate::targeting_core::RendererScopeCompletion::Experimental,
    )
    .with_member(crate::targeting_core::RendererSubject::Family(ShowcaseRenderer::Subwayrat.family_id()))
    .with_member(crate::targeting_core::RendererSubject::Family(ShowcaseRenderer::Meteorite.family_id()))
    .with_member(crate::targeting_core::RendererSubject::Family(crate::targeting_core::RendererFamilyId::new("iced")));
    let complete = valid_fixture()
        .with_renderer_cell(FixtureRendererCell::unsupported("iced", "pilot has no maintained showcase shell"));
    let accepted = validate_showcase_fixtures_with_policy(&[complete.clone()], &registry, &scope);
    assert!(accepted.is_valid());
    assert_eq!(accepted.coverage[0].cells.len(), 3);
    assert_eq!(accepted.coverage[0].cells[2].family, "iced");
    assert_eq!(accepted.coverage[0].cells[2].supported, Some(false));
    assert!(accepted.coverage[0].cells[2].reason.is_some());
    let row = parity_row()
        .with_renderer_cell(FixtureParityCell::unsupported("iced", "pilot has no maintained showcase shell"));
    let summary = build_fixture_parity_summary_with_policy(&[row], &[complete], &registry, &scope);
    assert!(summary.is_valid(), "issues: {:?}", summary.issues);
    assert_eq!(summary.rows[0].renderer_cells.len(), 3);
    assert!(summary.rows[0].renderer_cells[2].text.contains("unsupported"));

    let missing = validate_showcase_fixtures_with_policy(&[valid_fixture()], &registry, &scope);
    assert!(!missing.is_valid());
    assert_issue(&missing, FixtureIssueCode::MissingRendererCell);
    assert_eq!(missing.coverage[0].cells[2].supported, None);
}

fn valid_fixture() -> ShowcaseFixture {
    ShowcaseFixture::new(FIXTURE_ID, "dialog", "Basic dialog")
        .with_state(FixtureState::new("default", "Default").with_data(FixtureDatum::new("message", "Hello")))
        .with_control(FixtureControl::new("toggle", "Toggle"))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Subwayrat))
        .with_renderer(RendererSupport::supported(ShowcaseRenderer::Meteorite))
}

fn parity_row() -> FixtureParityRow {
    FixtureParityRow::new(FIXTURE_ID, "dialog", "docs/generated/components/dialog-basic.md")
        .with_renderer(FixtureParityRenderer::supported(
            ShowcaseRenderer::Subwayrat,
            FixtureParityEvidenceStatus::Complete,
        ))
        .with_renderer(FixtureParityRenderer::supported(
            ShowcaseRenderer::Meteorite,
            FixtureParityEvidenceStatus::Complete,
        ))
}

fn assert_issue(report: &FixtureSetReport, code: FixtureIssueCode) {
    assert_issue_in(&report.issues, code);
}

fn assert_issue_in(issues: &[FixtureIssue], code: FixtureIssueCode) {
    assert!(issues.iter().any(|issue| issue.code == code));
}
