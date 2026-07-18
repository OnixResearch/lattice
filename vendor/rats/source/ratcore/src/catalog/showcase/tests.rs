use super::*;

#[test]
fn built_in_summary_contains_shared_rows_and_evidence() {
    let summary = built_in_fixture_parity_summary();

    assert!(summary.is_valid());
    assert!(
        summary
            .rows
            .iter()
            .any(|row| row.fixture_id == FIXTURE_DIALOG_BASIC
                && row.status == STATUS_SHARED
                && row.subwayrat.contains(STATUS_EVIDENCE_COMPLETE)
                && row.meteorite.contains(STATUS_EVIDENCE_COMPLETE))
    );
    assert!(
        summary
            .rows
            .iter()
            .any(|row| row.fixture_id == FIXTURE_CHROME_SHELL
                && row.subwayrat.contains(STATUS_EVIDENCE_COMPLETE)
                && row.meteorite.contains(STATUS_EVIDENCE_COMPLETE))
    );
}

#[test]
fn built_in_summary_reports_knowledge_as_shared() {
    let summary = built_in_fixture_parity_summary();
    let knowledge = summary
        .rows
        .iter()
        .find(|row| row.fixture_id == FIXTURE_KNOWLEDGE_WORKFLOW)
        .expect("knowledge fixture is present");

    assert_eq!(knowledge.status, STATUS_SHARED);
    assert!(knowledge.subwayrat.contains(STATUS_EVIDENCE_COMPLETE));
    assert!(knowledge.meteorite.contains(STATUS_EVIDENCE_COMPLETE));
}

#[test]
fn invalid_summary_returns_diagnostics() {
    let invalid_rows = vec![
        FixtureParityRow::new("missing.fixture", FAMILY_DIALOG, "").with_renderer(
            FixtureParityRenderer::unsupported(ShowcaseRenderer::Subwayrat, ""),
        ),
    ];

    let summary = build_fixture_parity_summary(&invalid_rows, &built_in_fixtures());

    assert!(!summary.is_valid());
    assert!(
        summary
            .rows
            .iter()
            .any(|row| row.fixture_id == "missing.fixture")
    );
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.code == FixtureIssueCode::ParityUnknownFixture)
    );
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.code == FixtureIssueCode::ParityMissingRenderer)
    );
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.code == FixtureIssueCode::ParityMissingUnsupportedReason)
    );
}

#[test]
fn knowledge_fixture_contains_projection_fields() {
    let fixtures = built_in_fixtures();
    let knowledge = fixtures
        .iter()
        .find(|fixture| fixture.id == FIXTURE_KNOWLEDGE_WORKFLOW)
        .expect("knowledge fixture is present");
    let data = knowledge
        .states
        .first()
        .expect("knowledge state is present")
        .data
        .iter()
        .map(|datum| (datum.key.as_str(), datum.value.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();

    assert_eq!(data.get(KNOWLEDGE_DATA_GRAPH), Some(&KNOWLEDGE_GRAPH_FACTS));
    assert_eq!(
        data.get(KNOWLEDGE_DATA_AGENDA),
        Some(&KNOWLEDGE_AGENDA_FACTS)
    );
    assert_eq!(
        data.get(KNOWLEDGE_DATA_METADATA),
        Some(&KNOWLEDGE_METADATA_FACTS)
    );
    assert_eq!(
        data.get(KNOWLEDGE_DATA_DIAGNOSTICS),
        Some(&KNOWLEDGE_DIAGNOSTIC_FACTS)
    );
}
