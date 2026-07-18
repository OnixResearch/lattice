const ACTION_SAVE: &str = "file.save";
const ACTION_OPEN: &str = "file.open";
const ACTION_SOURCE: &str = "builtin";
const ACTION_SAVE_KEY: &str = "s";
const ACTION_OPEN_KEY: &str = "o";
const ACTION_MODIFIER: &str = "Ctrl";
const FIXTURE_DIALOG: &str = "dialog.basic";
const FIXTURE_FAMILY: &str = "dialog";
const FIXTURE_STATE: &str = "default";
const FIXTURE_STATE_LABEL: &str = "Default";
const FIXTURE_DATUM_KEY: &str = "message";
const FIXTURE_DATUM_VALUE: &str = "Hello";

#[test]
fn valid_action_and_fixture_inputs_project_without_issues() {
    let registry =
        crate::action_registry::build_registry(vec![action(ACTION_SAVE), action(ACTION_OPEN)]);
    assert!(registry.is_valid(), "issues: {:?}", registry.issues);
    assert_eq!(registry.snapshot.actions().len(), 2);

    let fixtures = vec![fixture(FIXTURE_DIALOG)];
    let fixture_outcome = crate::showcase_fixtures::validate_showcase_fixtures(&fixtures);
    assert!(
        fixture_outcome.is_valid(),
        "issues: {:?}",
        fixture_outcome.issues
    );
    assert_eq!(
        fixture_outcome.coverage[0].kind,
        crate::showcase_fixtures::FixtureCoverageKind::Shared
    );
}

#[test]
fn duplicate_action_id_reports_stable_issue_and_keeps_unrelated_action() {
    let registry = crate::action_registry::build_registry(vec![
        action(ACTION_SAVE),
        crate::action_registry::ActionDescriptor::new(ACTION_SAVE, "Save Copy", source()),
        action(ACTION_OPEN),
    ]);

    assert!(!registry.is_valid());
    assert!(registry.snapshot.action(ACTION_SAVE).is_none());
    assert!(registry.snapshot.action(ACTION_OPEN).is_some());
    let issue = registry
        .issues
        .iter()
        .find(|issue| issue.code == crate::action_registry::ActionRegistryIssueCode::DuplicateId)
        .expect("duplicate ID issue is reported");
    assert_eq!(issue.action_ids, vec![ACTION_SAVE.to_string()]);
    assert_eq!(issue.field.as_deref(), Some("id"));
}

#[test]
fn malformed_fixture_reports_diagnostics_without_coverage_loss() {
    let fixtures = vec![
        fixture(FIXTURE_DIALOG),
        crate::showcase_fixtures::ShowcaseFixture::new(FIXTURE_DIALOG, "unknown", "")
            .with_state(crate::showcase_fixtures::FixtureState::new(
                "",
                FIXTURE_STATE_LABEL,
            ))
            .with_renderer(crate::showcase_fixtures::RendererSupport::unsupported(
                crate::showcase_fixtures::ShowcaseRenderer::Meteorite,
                "",
            )),
    ];

    let fixture_outcome = crate::showcase_fixtures::validate_showcase_fixtures(&fixtures);
    assert!(!fixture_outcome.is_valid());
    assert!(
        fixture_outcome
            .coverage
            .iter()
            .any(|entry| entry.fixture_id == FIXTURE_DIALOG)
    );
    assert_issue(
        &fixture_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::DuplicateFixtureId,
    );
    assert_issue(
        &fixture_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::MalformedFixture,
    );
    assert_issue(
        &fixture_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::UnsupportedFamily,
    );
    assert_issue(
        &fixture_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::InvalidStateData,
    );
    assert_issue(
        &fixture_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::UnsupportedRenderer,
    );
}

#[test]
fn stale_parity_row_reports_renderer_and_documentation_invariants() {
    let row = crate::showcase_fixtures::FixtureParityRow::new(FIXTURE_DIALOG, "chrome", "")
        .with_renderer(crate::showcase_fixtures::FixtureParityRenderer::supported(
            crate::showcase_fixtures::ShowcaseRenderer::Subwayrat,
            crate::showcase_fixtures::FixtureParityEvidenceStatus::Complete,
        ));

    let parity_outcome = crate::showcase_fixtures::validate_fixture_parity_baseline(
        &[row],
        &[fixture(FIXTURE_DIALOG)],
    );

    assert!(!parity_outcome.is_valid());
    assert_eq!(parity_outcome.rows[0].fixture_id, FIXTURE_DIALOG);
    assert_issue(
        &parity_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::ParityMissingDocumentationTarget,
    );
    assert_issue(
        &parity_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::ParityMissingRenderer,
    );
    assert_issue(
        &parity_outcome.issues,
        crate::showcase_fixtures::FixtureIssueCode::ParityStaleCoverage,
    );
}

fn source() -> crate::action_registry::ActionSource {
    crate::action_registry::ActionSource::new(ACTION_SOURCE)
}

fn action(id: &str) -> crate::action_registry::ActionDescriptor {
    let key = if id == ACTION_OPEN {
        ACTION_OPEN_KEY
    } else {
        ACTION_SAVE_KEY
    };
    crate::action_registry::ActionDescriptor::new(id, id, source()).with_shortcut(
        crate::action_registry::ActionShortcut::global(key, vec![ACTION_MODIFIER.to_string()]),
    )
}

fn fixture(id: &str) -> crate::showcase_fixtures::ShowcaseFixture {
    crate::showcase_fixtures::ShowcaseFixture::new(id, FIXTURE_FAMILY, "Basic dialog")
        .with_state(
            crate::showcase_fixtures::FixtureState::new(FIXTURE_STATE, FIXTURE_STATE_LABEL)
                .with_data(crate::showcase_fixtures::FixtureDatum::new(
                    FIXTURE_DATUM_KEY,
                    FIXTURE_DATUM_VALUE,
                )),
        )
        .with_renderer(crate::showcase_fixtures::RendererSupport::supported(
            crate::showcase_fixtures::ShowcaseRenderer::Subwayrat,
        ))
        .with_renderer(crate::showcase_fixtures::RendererSupport::supported(
            crate::showcase_fixtures::ShowcaseRenderer::Meteorite,
        ))
}

fn assert_issue(
    issues: &[crate::showcase_fixtures::FixtureIssue],
    code: crate::showcase_fixtures::FixtureIssueCode,
) {
    assert!(issues.iter().any(|issue| issue.code == code));
}
