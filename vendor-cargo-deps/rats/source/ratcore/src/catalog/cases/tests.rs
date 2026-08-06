const FIXTURE_DIALOG: &str = "dialog.basic";
const FIXTURE_UNKNOWN: &str = "dialog.unknown";
const COVERAGE_DOC: &str = "docs/showcase-fixture-coverage.md";
const STATE_DEFAULT: &str = "default";
const STATE_EMPTY: &str = "empty";
const STATE_INVALID: &str = "invalid-input";
const STATE_UNSUPPORTED: &str = "unsupported-web";
const DIAGNOSTIC_INVALID: &str = "input is rejected before rendering";
const UNSUPPORTED_REASON: &str = "requires browser-only measurement";
const EXPECTED_NEGATIVE_ISSUES: usize = 6;

#[test]
fn valid_rows_name_visible_states() {
    let fixtures = vec![valid_fixture()];

    let outcome = super::validate(&fixtures, &[super::super::FixtureId::new(FIXTURE_DIALOG)]);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.fixtures[0].fixture_id, FIXTURE_DIALOG);
    assert_eq!(outcome.fixtures[0].states.len(), 2);
}

#[test]
fn malformed_rows_fail_with_deterministic_diagnostics() {
    let duplicate = super::Fixture::new(FIXTURE_DIALOG, COVERAGE_DOC)
        .with_state(super::State::new("", "", super::Kind::Invalid))
        .with_state(super::State::new(STATE_INVALID, "Invalid", super::Kind::Invalid))
        .with_state(super::State::new(STATE_INVALID, "Invalid duplicate", super::Kind::Malformed));
    let stale = super::Fixture::new(FIXTURE_UNKNOWN, "").with_state(
        super::State::new(STATE_UNSUPPORTED, "Unsupported", super::Kind::Unsupported).with_renderer(super::Renderer {
            renderer: crate::showcase_fixtures::ShowcaseRenderer::Meteorite,
            visible: false,
            reason: Some(String::new()),
        }),
    );

    let outcome =
        super::validate(&[duplicate.clone(), duplicate, stale], &[super::super::FixtureId::new(FIXTURE_DIALOG)]);

    assert!(!outcome.is_valid());
    assert!(outcome.issues.len() >= EXPECTED_NEGATIVE_ISSUES);
    assert_issue(&outcome.issues, super::IssueCode::DuplicateFixtureId);
    assert_issue(&outcome.issues, super::IssueCode::DuplicateStateId);
    assert_issue(&outcome.issues, super::IssueCode::MissingRequiredField);
    assert_issue(&outcome.issues, super::IssueCode::MissingDiagnostic);
    assert_issue(&outcome.issues, super::IssueCode::MissingRendererSupport);
    assert_issue(&outcome.issues, super::IssueCode::MissingUnsupportedReason);
    assert_issue(&outcome.issues, super::IssueCode::StaleCoverageReference);
}

fn valid_fixture() -> super::Fixture {
    super::Fixture::new(FIXTURE_DIALOG, COVERAGE_DOC)
        .with_state(
            super::State::new(STATE_DEFAULT, "Default", super::Kind::HappyPath)
                .with_renderer(super::Renderer::visible(crate::showcase_fixtures::ShowcaseRenderer::Subwayrat))
                .with_renderer(super::Renderer::visible(crate::showcase_fixtures::ShowcaseRenderer::Meteorite)),
        )
        .with_state(
            super::State::new(STATE_EMPTY, "Empty", super::Kind::Empty)
                .with_diagnostic(DIAGNOSTIC_INVALID)
                .with_renderer(super::Renderer::visible(crate::showcase_fixtures::ShowcaseRenderer::Subwayrat))
                .with_renderer(super::Renderer::unsupported(
                    crate::showcase_fixtures::ShowcaseRenderer::Meteorite,
                    UNSUPPORTED_REASON,
                )),
        )
}

fn assert_issue(issues: &[super::Issue], code: super::IssueCode) {
    assert!(issues.iter().any(|issue| issue.code == code));
}
