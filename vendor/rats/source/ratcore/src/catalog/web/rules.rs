const REQUIRED_TEXT_FIELD_COUNT: usize = 6;
const COMMAND_FIELD_COUNT: usize = 2;
const EXTRA_STATUS_ISSUE_BOUND: usize = 2;

/// Validates browser showcase entries against shared fixture data.
#[must_use]
pub fn validate_browser_showcase_index(
    entries: &[super::BrowserShowcaseEntry],
    fixtures: &[super::super::ShowcaseFixture],
) -> Vec<super::BrowserShowcaseIssue> {
    let mut issues = Vec::with_capacity(entries.len());
    let mut seen_ids = std::collections::BTreeSet::new();
    let fixture_ids = fixtures
        .iter()
        .map(|fixture| fixture.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    debug_assert!(issues.capacity() >= entries.len());
    debug_assert!(seen_ids.is_empty());
    for entry in entries {
        if !seen_ids.insert(entry.id.clone()) {
            issues.push(super::BrowserShowcaseIssue::new(
                super::BrowserShowcaseIssueCode::DuplicateEntryId,
                entry.id.clone(),
                super::FIELD_ID,
                "duplicate browser showcase entry ID",
            ));
        }
        issues.extend(single_entry_issues(entry, &fixture_ids));
    }
    issues
}

pub(super) fn unknown_fixture_issue(
    entry: &super::BrowserShowcaseEntry,
    fixture_id: &str,
) -> super::BrowserShowcaseIssue {
    super::BrowserShowcaseIssue::new(
        super::BrowserShowcaseIssueCode::InvalidFixtureReference,
        entry.id.clone(),
        super::FIELD_SUPPORTED_FIXTURE_IDS,
        format!("browser showcase entry references unknown fixture `{fixture_id}`"),
    )
}

fn single_entry_issues(
    entry: &super::BrowserShowcaseEntry,
    fixture_ids: &std::collections::BTreeSet<&str>,
) -> Vec<super::BrowserShowcaseIssue> {
    let issue_bound = REQUIRED_TEXT_FIELD_COUNT
        .saturating_add(COMMAND_FIELD_COUNT)
        .saturating_add(entry.supported_fixture_ids.len())
        .saturating_add(EXTRA_STATUS_ISSUE_BOUND);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    debug_assert!(entry.id.trim().len() <= entry.id.len());
    issues.extend(required_text_issues(entry));
    issues.extend(command_issues(entry));
    issues.extend(fixture_reference_issues(entry, fixture_ids));
    issues.extend(evidence_status_issues(entry));
    issues
}

fn evidence_status_issues(entry: &super::BrowserShowcaseEntry) -> Vec<super::BrowserShowcaseIssue> {
    let mut issues = Vec::with_capacity(EXTRA_STATUS_ISSUE_BOUND);
    debug_assert!(issues.capacity() >= EXTRA_STATUS_ISSUE_BOUND);
    debug_assert!(!entry.evidence_status.label().is_empty());
    if entry.evidence_status == super::BrowserShowcaseEvidenceStatus::Complete
        && entry.evidence_links.is_empty()
    {
        issues.push(super::BrowserShowcaseIssue::new(
            super::BrowserShowcaseIssueCode::CompleteEvidenceWithoutLinks,
            entry.id.clone(),
            super::FIELD_EVIDENCE_LINKS,
            "complete browser evidence requires at least one evidence link",
        ));
    }
    if entry.evidence_status == super::BrowserShowcaseEvidenceStatus::Unsupported
        && entry.unsupported_reasons.is_empty()
    {
        issues.push(super::BrowserShowcaseIssue::new(
            super::BrowserShowcaseIssueCode::UnsupportedWithoutReason,
            entry.id.clone(),
            super::FIELD_UNSUPPORTED_REASONS,
            "unsupported browser showcase entries require a visible reason",
        ));
    }
    issues
}

fn required_text_issues(entry: &super::BrowserShowcaseEntry) -> Vec<super::BrowserShowcaseIssue> {
    let required = [
        (super::FIELD_ID, entry.id.as_str()),
        (super::FIELD_TITLE, entry.title.as_str()),
        (super::FIELD_RENDERER_LABEL, entry.renderer_label.as_str()),
        (super::FIELD_PACKAGE_NAME, entry.package_name.as_str()),
        (super::FIELD_CRATE_PATH, entry.crate_path.as_str()),
        (
            super::FIELD_VALIDATION_PROFILE,
            entry.validation_profile.as_str(),
        ),
    ];
    debug_assert_eq!(required.len(), REQUIRED_TEXT_FIELD_COUNT);
    required
        .into_iter()
        .filter(|(_, value)| value.trim().is_empty())
        .map(|(field, _)| {
            super::BrowserShowcaseIssue::new(
                super::BrowserShowcaseIssueCode::MissingRequiredField,
                entry.id.clone(),
                field,
                format!("browser showcase entry requires `{field}`"),
            )
        })
        .collect()
}

fn command_issues(entry: &super::BrowserShowcaseEntry) -> Vec<super::BrowserShowcaseIssue> {
    let commands = [
        (super::FIELD_LAUNCH_COMMAND, entry.launch_command.as_str()),
        (super::FIELD_BUNDLE_COMMAND, entry.bundle_command.as_str()),
    ];
    debug_assert_eq!(commands.len(), COMMAND_FIELD_COUNT);
    commands
        .into_iter()
        .filter(|(_, command)| command.trim().is_empty())
        .map(|(field, _)| {
            super::BrowserShowcaseIssue::new(
                super::BrowserShowcaseIssueCode::MissingCommand,
                entry.id.clone(),
                field,
                format!("browser showcase entry requires `{field}`"),
            )
        })
        .collect()
}

fn fixture_reference_issues(
    entry: &super::BrowserShowcaseEntry,
    fixture_ids: &std::collections::BTreeSet<&str>,
) -> Vec<super::BrowserShowcaseIssue> {
    if entry.supported_fixture_ids.is_empty() {
        return vec![super::BrowserShowcaseIssue::new(
            super::BrowserShowcaseIssueCode::InvalidFixtureReference,
            entry.id.clone(),
            super::FIELD_SUPPORTED_FIXTURE_IDS,
            "browser showcase entry requires at least one shared fixture ID",
        )];
    }
    entry
        .supported_fixture_ids
        .iter()
        .filter(|fixture_id| !fixture_ids.contains(fixture_id.as_str()))
        .map(|fixture_id| unknown_fixture_issue(entry, fixture_id))
        .collect()
}
