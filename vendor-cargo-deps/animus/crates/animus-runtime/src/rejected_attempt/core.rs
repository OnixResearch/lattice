//! Pure rejected-attempt admission, projection, and receipt logic.

const REQUIRED_MATCH_COUNT: u32 = 1;
const MAX_PLANNING_ISSUES: usize = 8;
const REVIEW_ISSUE_SLOTS: usize = 3;
const VCS_ISSUE_SLOTS: usize = 2;
const SUMMARY_ISSUE_SLOTS: usize = 2;
const MAX_RECEIPT_ISSUES: usize = 2;
const FRESH_CONTEXT_PROFILE: &str = "fresh-context-task";
const RALPH_PROFILE: &str = "ralph-loop";
const OMITTED_RAW_DIFF: &str = "raw_diff";
const OMITTED_TRANSCRIPT: &str = "attempt_transcript";
const RECEIPT_HASH_DOMAIN: &str = "animus.rejected-attempt-receipt.v1";
const HASH_SEPARATOR: &str = "|";
const CLAIM_REVIEW_BOUND: &str = "review-evidence-bound";
const CLAIM_CURRENT_ATTEMPT: &str = "current-attempt-identity";
const CLAIM_BOUNDED_SUMMARY: &str = "bounded-redacted-summary";
const ALLOWED_CLAIMS: &[&str] = &[
    CLAIM_REVIEW_BOUND,
    CLAIM_CURRENT_ATTEMPT,
    CLAIM_BOUNDED_SUMMARY,
];
const UNSAFE_MARKERS: &[&str] = &["API_KEY", "/home/", "provider payload", "tool_output:"];

/// Plans safe rejected-attempt context from already-loaded facts.
///
/// # Errors
///
/// Returns all detected admission, freshness, classification, and bound issues.
///
/// # Panics
///
/// Panics if the internal issue bound or valid-plan invariants are incorrect.
pub fn plan_context(
    request: &super::data::Request,
    policy: &super::data::PlanningContract,
) -> Result<super::data::ContextPlan, Vec<super::data::Issue>> {
    let mut issues = Vec::with_capacity(MAX_PLANNING_ISSUES);
    issues.extend(validate_identities(request, policy));
    issues.extend(validate_review(request, policy).into_iter().flatten());
    issues.extend(validate_vcs(request, policy).into_iter().flatten());
    issues.extend(validate_summary(request, policy).into_iter().flatten());
    assert!(issues.len() <= MAX_PLANNING_ISSUES);

    if !issues.is_empty() {
        return Err(issues);
    }

    let raw_diff_disposition = classify_raw_diff(request, policy);
    let mut omissions = vec![OMITTED_TRANSCRIPT.to_string()];
    if raw_diff_disposition != super::data::RawDiffDisposition::Admitted {
        omissions.push(OMITTED_RAW_DIFF.to_string());
    }
    let plan = super::data::ContextPlan {
        task_id: request.task_id.clone(),
        run_id: request.run_id.clone(),
        attempt_id: request.attempt_id.clone(),
        handoff_id: request.review.handoff_id.clone(),
        revision_id: request.revision_id.clone(),
        summary_id: request.context.summary_id.clone(),
        redacted_summary: request.context.redacted_summary.clone(),
        raw_diff_disposition,
        context_profile: FRESH_CONTEXT_PROFILE.to_string(),
        loop_profile: RALPH_PROFILE.to_string(),
        omissions,
    };
    assert!(!plan.context_profile.is_empty());
    assert!(!plan.omissions.is_empty());
    Ok(plan)
}

/// Projects a metadata-only advisory receipt.
///
/// # Panics
///
/// Panics if the internal receipt identity or claim-roster invariants are incorrect.
#[must_use]
pub fn project_receipt(
    plan: &super::data::ContextPlan,
    policy: &super::data::PlanningContract,
) -> super::data::EvidenceReceipt {
    let disposition = raw_diff_disposition_label(plan.raw_diff_disposition);
    let mut hasher = blake3::Hasher::new();
    for part in [
        RECEIPT_HASH_DOMAIN,
        plan.task_id.as_str(),
        plan.run_id.as_str(),
        plan.attempt_id.as_str(),
        plan.revision_id.as_str(),
        plan.handoff_id.as_str(),
        plan.summary_id.as_str(),
        disposition,
        policy.policy_id.as_str(),
    ] {
        hasher.update(part.as_bytes());
        hasher.update(HASH_SEPARATOR.as_bytes());
    }

    let receipt = super::data::EvidenceReceipt {
        receipt_id: super::data::ReceiptIdentity::new(hasher.finalize().to_hex().to_string()),
        task_id: plan.task_id.clone(),
        run_id: plan.run_id.clone(),
        attempt_id: plan.attempt_id.clone(),
        revision_id: plan.revision_id.clone(),
        handoff_id: plan.handoff_id.clone(),
        summary_id: plan.summary_id.clone(),
        raw_diff_disposition: plan.raw_diff_disposition,
        claims: ALLOWED_CLAIMS.iter().map(ToString::to_string).collect(),
        non_claims: policy.required_non_claims.clone(),
    };
    assert!(!receipt.receipt_id.as_str().is_empty());
    assert_eq!(receipt.claims.len(), ALLOWED_CLAIMS.len());
    receipt
}

/// Validates the bounded receipt claim surface.
///
/// # Errors
///
/// Returns an overclaim or missing-non-claim issue.
///
/// # Panics
///
/// Panics if the internal claim roster or validation issue bound is incorrect.
pub fn validate_receipt(
    receipt: &super::data::EvidenceReceipt,
    policy: &super::data::PlanningContract,
) -> Result<(), Vec<super::data::Issue>> {
    let mut issues = Vec::with_capacity(MAX_RECEIPT_ISSUES);
    let has_overclaim = receipt
        .claims
        .iter()
        .any(|claim| !ALLOWED_CLAIMS.contains(&claim.as_str()));
    if has_overclaim {
        issues.push(super::data::Issue::ReceiptOverclaim);
    }
    let is_non_claim_missing = policy
        .required_non_claims
        .iter()
        .any(|required| !receipt.non_claims.contains(required));
    if is_non_claim_missing {
        issues.push(super::data::Issue::MissingNonClaim);
    }
    assert!(!ALLOWED_CLAIMS.is_empty());
    assert!(issues.len() <= MAX_RECEIPT_ISSUES);

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

fn validate_identities(
    request: &super::data::Request,
    policy: &super::data::PlanningContract,
) -> Option<super::data::Issue> {
    let identities = [
        request.task_id.as_str(),
        request.run_id.as_str(),
        request.attempt_id.as_str(),
        request.revision_id.as_str(),
        request.workspace_snapshot_id.as_str(),
        request.content_id.as_str(),
        request.review.handoff_id.as_str(),
        request.context.summary_id.as_str(),
        policy.policy_id.as_str(),
    ];
    identities
        .iter()
        .any(|value| value.trim().is_empty())
        .then_some(super::data::Issue::MissingIdentity)
}

fn validate_review(
    request: &super::data::Request,
    policy: &super::data::PlanningContract,
) -> [Option<super::data::Issue>; REVIEW_ISSUE_SLOTS] {
    let review = &request.review;
    let is_review_denied = !review.validated
        || review.disposition != super::data::ReviewDisposition::Rejected
        || !policy.accepted_review_profiles.contains(&review.profile);
    let is_wrong_binding = review.task_id != request.task_id
        || review.run_id != request.run_id
        || review.attempt_id != request.attempt_id
        || review.revision_id != request.revision_id;
    let is_match_ambiguous = review.matching_attempts != REQUIRED_MATCH_COUNT;
    [
        is_review_denied.then_some(super::data::Issue::ReviewEvidenceDenied),
        is_wrong_binding.then_some(super::data::Issue::WrongTaskOrRun),
        is_match_ambiguous.then_some(super::data::Issue::AmbiguousAttempt),
    ]
}

fn validate_vcs(
    request: &super::data::Request,
    policy: &super::data::PlanningContract,
) -> [Option<super::data::Issue>; VCS_ISSUE_SLOTS] {
    let is_provider_denied = !policy
        .accepted_vcs_providers
        .contains(&request.vcs.provider);
    let is_observation_stale = !request.vcs.current
        || request.vcs.attempt_id != request.attempt_id
        || request.vcs.revision_id != request.revision_id
        || request.vcs.workspace_snapshot_id != request.workspace_snapshot_id
        || request.vcs.content_id != request.content_id;
    [
        is_provider_denied.then_some(super::data::Issue::VcsProviderDenied),
        is_observation_stale.then_some(super::data::Issue::StaleAttempt),
    ]
}

fn validate_summary(
    request: &super::data::Request,
    policy: &super::data::PlanningContract,
) -> [Option<super::data::Issue>; SUMMARY_ISSUE_SLOTS] {
    let summary = request.context.redacted_summary.as_str();
    let is_summary_unsafe = !request.context.summary_classification_safe
        || summary.trim().is_empty()
        || UNSAFE_MARKERS.iter().any(|marker| summary.contains(marker));
    let is_summary_within_budget =
        is_byte_length_within_bound(summary.len(), policy.max_summary_bytes);
    [
        is_summary_unsafe.then_some(super::data::Issue::UnsafeSummary),
        (!is_summary_within_budget).then_some(super::data::Issue::OversizedSummary),
    ]
}

fn classify_raw_diff(
    request: &super::data::Request,
    policy: &super::data::PlanningContract,
) -> super::data::RawDiffDisposition {
    let Some(raw_diff) = request.context.raw_diff.as_ref() else {
        return super::data::RawDiffDisposition::NotSupplied;
    };
    if !policy.allow_raw_diff || !request.context.raw_diff_capability_admitted {
        return super::data::RawDiffDisposition::DeniedByPolicy;
    }
    if !request.context.raw_diff_classification_safe {
        return super::data::RawDiffDisposition::DeniedByClassification;
    }
    if !is_byte_length_within_bound(raw_diff.len(), policy.max_raw_diff_bytes) {
        return super::data::RawDiffDisposition::DeniedByBudget;
    }
    super::data::RawDiffDisposition::Admitted
}

fn is_byte_length_within_bound(length_bytes: usize, max_bytes: u32) -> bool {
    match u32::try_from(length_bytes) {
        Ok(length_bytes) => length_bytes <= max_bytes,
        Err(_) => false,
    }
}

fn raw_diff_disposition_label(disposition: super::data::RawDiffDisposition) -> &'static str {
    match disposition {
        super::data::RawDiffDisposition::DeniedByPolicy => "denied_by_policy",
        super::data::RawDiffDisposition::DeniedByClassification => "denied_by_classification",
        super::data::RawDiffDisposition::DeniedByBudget => "denied_by_budget",
        super::data::RawDiffDisposition::Admitted => "admitted",
        super::data::RawDiffDisposition::NotSupplied => "not_supplied",
    }
}
