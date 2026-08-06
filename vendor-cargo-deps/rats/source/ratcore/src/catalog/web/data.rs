/// Builds the built-in browser showcase index.
#[must_use]
pub fn built_in_browser_showcase_index() -> super::BrowserShowcaseIndex {
    let mut entries = built_in_browser_showcase_entries();
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    let issues = super::validate_browser_showcase_index(&entries, &super::super::built_in_fixtures());
    super::BrowserShowcaseIndex { entries, issues }
}

/// Builds built-in browser showcase entries before validation.
#[must_use]
pub fn built_in_browser_showcase_entries() -> Vec<super::BrowserShowcaseEntry> {
    vec![meteorite_entry(), ratzilla_entry()]
}

fn meteorite_entry() -> super::BrowserShowcaseEntry {
    super::BrowserShowcaseEntry::new(
        super::ENTRY_METEORITE_WEB,
        super::TITLE_METEORITE_WEB,
        super::RENDERER_METEORITE_DOM,
        super::PACKAGE_MET_SHOWCASE,
        super::PATH_MET_SHOWCASE,
    )
    .with_commands(super::METEORITE_LAUNCH_COMMAND, super::METEORITE_BUNDLE_COMMAND)
    .with_validation_profile(super::PROFILE_MET_SHOWCASE)
    .with_supported_fixture_ids(&[
        super::FIXTURE_DIALOG_BASIC,
        super::FIXTURE_CHROME_SHELL,
        super::FIXTURE_KNOWLEDGE_WORKFLOW,
        super::FIXTURE_THEME_TOKENS,
    ])
    .with_evidence_links(&[
        super::DOC_METEORITE_README,
        super::DOC_METEORITE_RUNTIME,
        super::DOC_WORKSPACE_VALIDATION,
    ])
    .with_evidence_status(super::BrowserShowcaseEvidenceStatus::Complete)
}

fn ratzilla_entry() -> super::BrowserShowcaseEntry {
    super::BrowserShowcaseEntry::new(
        super::ENTRY_RATZILLA_WEB,
        super::TITLE_RATZILLA_WEB,
        super::RENDERER_RATZILLA_WEB,
        super::PACKAGE_RAT_RATZILLA,
        super::PATH_RAT_RATZILLA,
    )
    .with_commands(super::RATZILLA_LAUNCH_COMMAND, super::RATZILLA_BUNDLE_COMMAND)
    .with_validation_profile(super::PROFILE_SUBWAYRAT_WEB)
    .with_supported_fixture_ids(&[
        super::FIXTURE_DIALOG_BASIC,
        super::FIXTURE_CHROME_SHELL,
        super::FIXTURE_THEME_TOKENS,
    ])
    .with_evidence_links(&[
        super::DOC_SUBWAYRAT_WEB,
        super::DOC_RATZILLA_INPUT,
        super::DOC_WORKSPACE_VALIDATION,
    ])
    .with_evidence_status(super::BrowserShowcaseEvidenceStatus::Complete)
    .with_unsupported_reasons(&[
        super::RATZILLA_GAP_KNOWLEDGE,
        super::RATZILLA_GAP_NATIVE,
        super::RATZILLA_GAP_INPUT,
        super::RATZILLA_GAP_SCOPE,
    ])
}
