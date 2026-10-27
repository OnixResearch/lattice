use alloc::{string::String, vec::Vec};

use crate::{
    ChangeRequestBinding, ChangeRequestRef, CheckFailureObservation, DiffFileObservation,
    DiffObservation, Error, ForgeCapabilities, RepositoryRef, ReviewBounds, ReviewFinding,
    ReviewInlineAnchor, ReviewPolicy, ReviewSeverity, bounded_u32,
};

const COMPLEX_DIFF_HUNK_THRESHOLD: u32 = 1;
const COMPLEX_DIFF_FILE_THRESHOLD: usize = 1;

/// Input for pure code-review planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeReviewInput {
    /// Exact service binding.
    pub binding: ChangeRequestBinding,
    /// Reviewed repository.
    pub repository: RepositoryRef,
    /// Reviewed change request.
    pub change_request: ChangeRequestRef,
    /// Bounded diff metadata.
    pub diff: DiffObservation,
    /// Deterministic findings supplied by the host.
    pub deterministic_findings: Vec<ReviewFinding>,
    /// Failed check observations supplied by the host.
    pub ci_failures: Vec<CheckFailureObservation>,
    /// Review policy.
    pub policy: ReviewPolicy,
    /// Review bounds.
    pub bounds: ReviewBounds,
    /// Advertised forge capabilities.
    pub forge_capabilities: ForgeCapabilities,
}

/// Reason for a review escalation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewEscalationReason {
    /// Finding confidence is below the publication threshold.
    LowConfidence,
    /// No matching inline anchor was observed.
    MissingInlineAnchor,
    /// CI repair attempts are exhausted.
    RepairAttemptsExhausted,
}

/// Pure review effect intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewEffect {
    /// Request bounded advisory review analysis.
    RequestAnalysis {
        /// Exact reviewed commit identity.
        commit_identity: String,
    },
    /// Request bounded CI-failure analysis.
    RequestCiRepair {
        /// Trusted validation profile identity.
        profile_identity: String,
        /// BLAKE3 output evidence identity.
        output_identity: String,
    },
    /// Prepare one provider-neutral inline review comment.
    PrepareInlineComment {
        /// Finding to publish.
        finding: ReviewFinding,
        /// Exact observed anchor.
        anchor: ReviewInlineAnchor,
    },
    /// Prepare one provider-neutral summary.
    PrepareSummary {
        /// Number of included findings.
        finding_count: u32,
    },
    /// Escalate one bounded review condition.
    Escalate {
        /// Escalation reason.
        reason: ReviewEscalationReason,
        /// Optional finding that caused the escalation.
        finding: Option<ReviewFinding>,
    },
}

/// Deterministic code-review plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeReviewPlan {
    /// Exact plan binding.
    pub binding: ChangeRequestBinding,
    /// Reviewed repository identity.
    pub repository_identity: String,
    /// Reviewed change-request identity.
    pub change_request_identity: String,
    /// Validated findings.
    pub findings: Vec<ReviewFinding>,
    /// Ordered pure effect intents.
    pub effects: Vec<ReviewEffect>,
}

/// Plans one bounded code review without effects.
///
/// # Errors
///
/// Returns a typed error when input metadata, bounds, policy, or anchors are invalid.
pub fn plan_code_review(input: CodeReviewInput) -> Result<CodeReviewPlan, Error> {
    input.repository.validate()?;
    input.change_request.validate()?;
    if input.binding.repository_identity() != input.repository.identity
        || input.repository.default_branch != input.change_request.target_branch
    {
        return Err(Error::BindingMismatch);
    }
    input.bounds.validate()?;
    input.policy.validate(input.forge_capabilities)?;
    validate_diff(&input.diff, input.bounds, &input.change_request)?;
    validate_collection_bound(
        "review.findings",
        input.deterministic_findings.len(),
        input.bounds.max_findings,
    )?;
    validate_collection_bound(
        "review.ci_failures",
        input.ci_failures.len(),
        input.bounds.max_ci_failures,
    )?;

    let mut findings = Vec::with_capacity(input.deterministic_findings.len());
    let mut effects = Vec::new();
    if input.policy.request_analysis_for_complex_diff && is_complex_diff(&input.diff) {
        effects.push(ReviewEffect::RequestAnalysis {
            commit_identity: input.diff.commit_identity.clone(),
        });
    }

    for failure in &input.ci_failures {
        failure.validate()?;
        if failure.repair_attempts >= input.policy.max_repair_attempts {
            effects.push(ReviewEffect::Escalate {
                reason: ReviewEscalationReason::RepairAttemptsExhausted,
                finding: Some(ci_failure_finding()),
            });
        } else {
            effects.push(ReviewEffect::RequestCiRepair {
                profile_identity: failure.profile_identity.clone(),
                output_identity: failure.output_identity.clone(),
            });
        }
    }

    for finding in input.deterministic_findings {
        finding.validate()?;
        if finding.confidence < input.policy.minimum_comment_confidence {
            effects.push(ReviewEffect::Escalate {
                reason: ReviewEscalationReason::LowConfidence,
                finding: Some(finding.clone()),
            });
            findings.push(finding);
            continue;
        }

        if input.policy.publish_inline_comments && finding.file_identity.is_some() {
            match matching_inline_anchor(&input.diff, &finding) {
                Some(anchor) => effects.push(ReviewEffect::PrepareInlineComment {
                    finding: finding.clone(),
                    anchor,
                }),
                None if input.policy.allow_summary_fallback => {
                    effects.push(ReviewEffect::Escalate {
                        reason: ReviewEscalationReason::MissingInlineAnchor,
                        finding: Some(finding.clone()),
                    });
                }
                None => return Err(Error::MissingInlineAnchor),
            }
        }
        findings.push(finding);
    }

    if input.policy.allow_summary_fallback && !findings.is_empty() {
        effects.push(ReviewEffect::PrepareSummary {
            finding_count: bounded_u32(findings.len()),
        });
    }

    Ok(CodeReviewPlan {
        binding: input.binding,
        repository_identity: input.repository.identity,
        change_request_identity: input.change_request.identity,
        findings,
        effects,
    })
}

fn validate_diff(
    diff: &DiffObservation,
    bounds: ReviewBounds,
    change_request: &ChangeRequestRef,
) -> Result<(), Error> {
    crate::validate_identity("diff.commit_identity", &diff.commit_identity)?;
    if diff.commit_identity != change_request.commit_identity {
        return Err(Error::BindingMismatch);
    }
    validate_collection_bound("diff.files", diff.files.len(), bounds.max_files)?;
    validate_count("diff.hunks", diff.total_hunk_count, bounds.max_hunks)?;
    validate_count(
        "diff.changed_lines",
        diff.total_added_line_count
            .saturating_add(diff.total_deleted_line_count),
        bounds.max_changed_lines,
    )?;
    let mut observed_hunks = 0_u32;
    let mut observed_added_lines = 0_u32;
    let mut observed_deleted_lines = 0_u32;
    for file in &diff.files {
        validate_diff_file(file, bounds, &diff.commit_identity)?;
        observed_hunks = observed_hunks.saturating_add(file.hunk_count);
        observed_added_lines = observed_added_lines.saturating_add(file.added_line_count);
        observed_deleted_lines = observed_deleted_lines.saturating_add(file.deleted_line_count);
    }
    if observed_hunks != diff.total_hunk_count
        || observed_added_lines != diff.total_added_line_count
        || observed_deleted_lines != diff.total_deleted_line_count
    {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

fn validate_diff_file(
    file: &DiffFileObservation,
    bounds: ReviewBounds,
    commit_identity: &str,
) -> Result<(), Error> {
    crate::validate_identity("diff_file.file_identity", &file.file_identity)?;
    validate_count("diff_file.hunks", file.hunk_count, bounds.max_hunks)?;
    validate_collection_bound(
        "diff_file.inline_anchors",
        file.inline_anchors.len(),
        bounds.max_findings,
    )?;
    for anchor in &file.inline_anchors {
        anchor.validate()?;
        if anchor.file_identity != file.file_identity || anchor.commit_identity != commit_identity {
            return Err(Error::BindingMismatch);
        }
    }
    Ok(())
}

fn is_complex_diff(diff: &DiffObservation) -> bool {
    diff.total_hunk_count > COMPLEX_DIFF_HUNK_THRESHOLD
        || diff.files.len() > COMPLEX_DIFF_FILE_THRESHOLD
}

fn matching_inline_anchor(
    diff: &DiffObservation,
    finding: &ReviewFinding,
) -> Option<ReviewInlineAnchor> {
    let file_identity = finding.file_identity.as_ref()?;
    let line_range = finding.line_range?;
    let side = finding.side?;
    let file = diff
        .files
        .iter()
        .find(|file| &file.file_identity == file_identity)?;
    file.inline_anchors
        .iter()
        .find(|anchor| {
            let expected_start = (line_range.start != line_range.end).then_some(line_range.start);
            anchor.line == line_range.end
                && anchor.start_line == expected_start
                && anchor.side == side
                && anchor.commit_identity == diff.commit_identity
        })
        .cloned()
}

fn ci_failure_finding() -> ReviewFinding {
    ReviewFinding {
        source: crate::ReviewFindingSource::HumanEscalation,
        severity: ReviewSeverity::Blocking,
        category: String::from("ci-failure"),
        message: String::from("The validation repair budget is exhausted."),
        file_identity: None,
        line_range: None,
        side: None,
        confidence: crate::MAX_REVIEW_CONFIDENCE,
    }
}

fn validate_collection_bound(
    field: &'static str,
    actual: usize,
    maximum: u32,
) -> Result<(), Error> {
    validate_count(field, bounded_u32(actual), maximum)
}

fn validate_count(field: &'static str, actual: u32, maximum: u32) -> Result<(), Error> {
    if actual > maximum {
        return Err(Error::BoundExceeded {
            field,
            actual,
            maximum,
        });
    }
    Ok(())
}
