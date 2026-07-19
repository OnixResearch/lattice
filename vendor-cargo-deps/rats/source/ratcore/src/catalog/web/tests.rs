use super::super::built_in_fixtures;
use super::*;

#[test]
fn built_in_browser_index_lists_meteorite_and_ratzilla() {
    let index = built_in_browser_showcase_index();

    assert!(index.is_valid());
    assert_eq!(index.entries.len(), EXPECTED_BROWSER_ENTRY_COUNT);
    assert!(index.entries.iter().any(|entry| {
        entry.package_name == PACKAGE_MET_SHOWCASE
            && entry.renderer_label == RENDERER_METEORITE_DOM
            && entry
                .supported_fixture_ids
                .contains(&FIXTURE_KNOWLEDGE_WORKFLOW.to_string())
            && entry
                .evidence_links
                .contains(&DOC_METEORITE_README.to_string())
    }));
    assert!(index.entries.iter().any(|entry| {
        entry.package_name == PACKAGE_RAT_RATZILLA
            && entry.renderer_label == RENDERER_RATZILLA_WEB
            && entry
                .supported_fixture_ids
                .contains(&FIXTURE_CHROME_SHELL.to_string())
            && entry
                .evidence_links
                .contains(&DOC_SUBWAYRAT_WEB.to_string())
    }));
}

#[test]
fn invalid_browser_index_reports_deterministic_diagnostics() {
    let duplicate = BrowserShowcaseEntry::new(
        "duplicate.browser",
        "Duplicate browser",
        RENDERER_RATZILLA_WEB,
        PACKAGE_RAT_RATZILLA,
        PATH_RAT_RATZILLA,
    )
    .with_commands(RATZILLA_LAUNCH_COMMAND, RATZILLA_BUNDLE_COMMAND)
    .with_validation_profile(PROFILE_SUBWAYRAT_WEB)
    .with_supported_fixture_ids(&[FIXTURE_DIALOG_BASIC])
    .with_evidence_links(&[DOC_SUBWAYRAT_WEB])
    .with_evidence_status(BrowserShowcaseEvidenceStatus::Complete);
    let missing_fields = BrowserShowcaseEntry::new(
        "duplicate.browser",
        "Missing fields",
        EMPTY_VALUE,
        PACKAGE_MET_SHOWCASE,
        PATH_MET_SHOWCASE,
    )
    .with_validation_profile(PROFILE_MET_SHOWCASE)
    .with_supported_fixture_ids(&["unknown.fixture"])
    .with_evidence_status(BrowserShowcaseEvidenceStatus::Complete);
    let unsupported_without_reason = BrowserShowcaseEntry::new(
        "unsupported.browser",
        "Unsupported browser",
        RENDERER_METEORITE_DOM,
        PACKAGE_MET_SHOWCASE,
        PATH_MET_SHOWCASE,
    )
    .with_commands(METEORITE_LAUNCH_COMMAND, METEORITE_BUNDLE_COMMAND)
    .with_validation_profile(PROFILE_MET_SHOWCASE)
    .with_supported_fixture_ids(&[FIXTURE_THEME_TOKENS])
    .with_evidence_status(BrowserShowcaseEvidenceStatus::Unsupported);

    let issues = validate_browser_showcase_index(
        &[duplicate, missing_fields, unsupported_without_reason],
        &built_in_fixtures(),
    );

    assert!(
        issues
            .iter()
            .any(|issue| issue.code == BrowserShowcaseIssueCode::DuplicateEntryId)
    );
    assert!(issues.iter().any(|issue| {
        issue.code == BrowserShowcaseIssueCode::MissingRequiredField
            && issue.field == FIELD_RENDERER_LABEL
    }));
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == BrowserShowcaseIssueCode::MissingCommand)
    );
    assert!(issues.iter().any(|issue| {
        issue.code == BrowserShowcaseIssueCode::CompleteEvidenceWithoutLinks
            && issue.field == FIELD_EVIDENCE_LINKS
    }));
    assert!(issues.iter().any(|issue| {
        issue.code == BrowserShowcaseIssueCode::InvalidFixtureReference
            && issue.message.contains("unknown.fixture")
    }));
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == BrowserShowcaseIssueCode::UnsupportedWithoutReason)
    );
}

#[test]
fn browser_portal_markdown_keeps_entry_facts_and_gaps_visible() {
    let index = built_in_browser_showcase_index();
    let markdown = render_browser_showcase_portal_markdown(&index.entries);

    assert!(markdown.contains(TITLE_METEORITE_WEB));
    assert!(markdown.contains(RENDERER_METEORITE_DOM));
    assert!(markdown.contains(PATH_MET_SHOWCASE));
    assert!(markdown.contains(FIXTURE_KNOWLEDGE_WORKFLOW));
    assert!(markdown.contains(PROFILE_SUBWAYRAT_WEB));
    assert!(markdown.contains(RATZILLA_LAUNCH_COMMAND));
    assert!(markdown.contains(METEORITE_SMOKE_GAP));
}

#[test]
fn fixture_navigation_uses_index_labels_and_evidence() {
    let index = built_in_browser_showcase_index();
    let entry = index
        .entries
        .iter()
        .find(|entry| entry.package_name == PACKAGE_RAT_RATZILLA)
        .expect("ratzilla browser entry is present");
    let navigation = browser_fixture_navigation_rows(entry, &built_in_fixtures());

    assert!(navigation.is_valid());
    assert!(navigation.rows.iter().any(|row| {
        row.fixture_id == FIXTURE_CHROME_SHELL
            && row.label == "Chrome shell"
            && row.renderer_label == RENDERER_RATZILLA_WEB
            && row.support_status == STATUS_COMPLETE
            && row.evidence_text.contains(DOC_SUBWAYRAT_WEB)
            && row.launch_context == RATZILLA_LAUNCH_COMMAND
    }));
}

#[test]
fn fixture_navigation_reports_unknown_fixture_without_panicking() {
    let entry = BrowserShowcaseEntry::new(
        "broken.browser",
        "Broken browser",
        RENDERER_RATZILLA_WEB,
        PACKAGE_RAT_RATZILLA,
        PATH_RAT_RATZILLA,
    )
    .with_commands(RATZILLA_LAUNCH_COMMAND, RATZILLA_BUNDLE_COMMAND)
    .with_validation_profile(PROFILE_SUBWAYRAT_WEB)
    .with_supported_fixture_ids(&["missing.fixture"]);

    let navigation = browser_fixture_navigation_rows(&entry, &built_in_fixtures());

    assert!(!navigation.is_valid());
    assert!(navigation.rows.is_empty());
    assert!(navigation.issues.iter().any(|issue| {
        issue.code == BrowserShowcaseIssueCode::InvalidFixtureReference
            && issue.field == FIELD_SUPPORTED_FIXTURE_IDS
            && issue.message.contains("missing.fixture")
    }));
}

const EXPECTED_BROWSER_ENTRY_COUNT: usize = 2;
