/// Borrowed review report before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewReportDraft<'a> {
    /// Stable review run identity.
    pub run_id: &'a str,
    /// Source labels included in the reviewed change set.
    pub reviewed_source_labels: &'a [&'a str],
    /// Structured findings emitted by the reviewer.
    pub findings: &'a [super::issue::DiffReviewFindingDraft<'a>],
    /// Read-only policy denials observed during the run.
    pub policy_denials: &'a [super::DiffReviewDenialReason],
}

/// Validated structured review report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewReport {
    run_id: super::Text,
    reviewed_source_labels: super::Collection<super::Text>,
    findings: super::Collection<super::issue::DiffReviewFinding>,
    policy_denials: super::Collection<super::DiffReviewDenialReason>,
}

impl DiffReviewReport {
    /// Validates a review report and rejects duplicate or out-of-scope findings.
    ///
    /// # Errors
    ///
    /// Returns [`super::DiffReviewError`] when report fields are malformed, finding
    /// count exceeds bounds, findings are duplicated, or a finding references a
    /// source outside the reviewed change set.
    pub fn validate(draft: &DiffReviewReportDraft<'_>) -> Result<Self, super::DiffReviewError> {
        let run_id = super::check::required_text(super::RUN_ID_FIELD, draft.run_id)?;
        let reviewed_source_labels = super::check::text_collection(
            super::SOURCE_LABEL_FIELD,
            draft.reviewed_source_labels,
            super::check::CollectionPolicy {
                required: true,
                max_items: super::MAX_TARGET_LABELS,
            },
        )?;
        super::check::collection_size(super::check::CollectionSize {
            field: super::FINDING_ID_FIELD,
            actual_items: draft.findings.len(),
            max_items: super::MAX_REVIEW_FINDINGS,
        })?;
        let mut finding_ids = super::Collection::with_capacity(draft.findings.len());
        let mut findings = super::Collection::with_capacity(draft.findings.len());
        for finding_draft in draft.findings {
            let finding = super::issue::DiffReviewFinding::validate(finding_draft)?;
            super::check::reject_duplicate(super::FINDING_ID_FIELD, &mut finding_ids, finding.finding_id())?;
            finding_ids.push(super::Text::from(finding.finding_id()));
            validate_finding_scope(&finding, reviewed_source_labels.as_slice())?;
            findings.push(finding);
        }
        Ok(Self {
            run_id,
            reviewed_source_labels,
            findings,
            policy_denials: draft.policy_denials.to_vec(),
        })
    }

    /// Returns the stable review run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    /// Returns reviewed source labels.
    #[must_use]
    pub fn reviewed_source_labels(&self) -> &[super::Text] {
        self.reviewed_source_labels.as_slice()
    }

    /// Returns structured review findings.
    #[must_use]
    pub fn findings(&self) -> &[super::issue::DiffReviewFinding] {
        self.findings.as_slice()
    }

    /// Returns read-only policy denial reasons observed during the run.
    #[must_use]
    pub fn policy_denials(&self) -> &[super::DiffReviewDenialReason] {
        self.policy_denials.as_slice()
    }
}

/// Rerun state assigned to a review finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewFindingState {
    /// Finding remains present in the latest report.
    Open,
    /// Prior finding no longer appears in the latest report.
    Fixed,
    /// Prior finding was intentionally dismissed by host-supplied state.
    Dismissed,
    /// Finding first appears in the latest report.
    NewlyIntroduced,
}

/// Rerun state record for one finding identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewFindingStateRecord {
    finding_id: super::Text,
    state: DiffReviewFindingState,
}

impl DiffReviewFindingStateRecord {
    fn new(finding_id: &str, state: DiffReviewFindingState) -> Self {
        Self {
            finding_id: super::Text::from(finding_id),
            state,
        }
    }

    /// Returns the finding identity.
    #[must_use]
    pub fn finding_id(&self) -> &str {
        self.finding_id.as_str()
    }

    /// Returns the rerun state.
    #[must_use]
    pub const fn state(&self) -> DiffReviewFindingState {
        self.state
    }
}

/// Compares prior and current review reports into finding-state records.
///
/// # Errors
///
/// Returns [`super::DiffReviewError`] when dismissed finding identities are malformed
/// or duplicated.
///
/// # Panics
///
/// Panics if validated review reports contain more finding records than the
/// public review bound.
pub fn compare_diff_review_rerun(
    prior: &DiffReviewReport,
    current: &DiffReviewReport,
    dismissed_ids: &[&str],
) -> Result<super::Collection<DiffReviewFindingStateRecord>, super::DiffReviewError> {
    assert!(prior.findings().len() <= super::MAX_REVIEW_FINDINGS);
    assert!(current.findings().len() <= super::MAX_REVIEW_FINDINGS);
    let dismissed =
        super::check::text_collection(super::DISMISSED_ID_FIELD, dismissed_ids, super::check::CollectionPolicy {
            required: false,
            max_items: super::MAX_REVIEW_FINDINGS,
        })?;
    let mut records = super::Collection::with_capacity(prior.findings().len().saturating_add(current.findings().len()));
    for prior_finding in prior.findings() {
        let state = if contains_finding(current.findings(), prior_finding.finding_id()) {
            DiffReviewFindingState::Open
        } else if super::check::contains_text(dismissed.as_slice(), prior_finding.finding_id()) {
            DiffReviewFindingState::Dismissed
        } else {
            DiffReviewFindingState::Fixed
        };
        records.push(DiffReviewFindingStateRecord::new(prior_finding.finding_id(), state));
    }
    for current_finding in current.findings() {
        if !contains_finding(prior.findings(), current_finding.finding_id()) {
            records.push(DiffReviewFindingStateRecord::new(
                current_finding.finding_id(),
                DiffReviewFindingState::NewlyIntroduced,
            ));
        }
    }
    Ok(records)
}

fn validate_finding_scope(
    finding: &super::issue::DiffReviewFinding,
    source_labels: &[super::Text],
) -> Result<(), super::DiffReviewError> {
    if super::check::contains_text(source_labels, finding.span().source_label()) {
        Ok(())
    } else {
        Err(super::DiffReviewError::FindingOutsideReviewScope {
            finding_id: super::Text::from(finding.finding_id()),
            source_label: super::Text::from(finding.span().source_label()),
        })
    }
}

fn contains_finding(findings: &[super::issue::DiffReviewFinding], finding_id: &str) -> bool {
    findings.iter().any(|finding| finding.finding_id() == finding_id)
}
