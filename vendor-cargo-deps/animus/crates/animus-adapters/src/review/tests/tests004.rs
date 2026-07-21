const REVIEW_REQUEST_ID: &str = "done-review-alpha";
const REVIEW_SUBJECT_ID: &str = "adapt-hermes-done-review-lifecycle";
const REVIEW_OWNER: &str = "adapter-contracts";
const REVIEW_STAGE: &str = "post-completion";
const REVIEW_EVIDENCE_ID: &str = "focused-review-tests";
const REVIEW_EVIDENCE_LABEL: &str = "cargo-test-review";
const REVIEW_RAIL_ID: &str = "cairn-validation";
const REVIEW_RAIL_PROFILE: &str = "cairn-validate";
const REVIEW_ATTEMPT_ID: &str = "attempt-primary";
const REVIEWER_ROLE: &str = "independent-reviewer";
const REASONING_PROFILE: &str = "bounded-review";
const REVIEW_OUTPUT_CONTRACT: &str = "verdict-markdown";
const REVIEW_TIMEOUT_CLASS: &str = "short";
const REVIEW_ATTEMPT_BUDGET: u16 = 4;
const ZERO_REVIEW_ATTEMPT_BUDGET: u16 = 0;
const RAW_DIFF_MARKER: &str = "raw diff: +secret";
const REVIEW_RECEIPT_ISSUE: &str = "review-passed";

fn review_hash(label: &str) -> alloc::string::String {
    super::hash_text(label)
}

fn subject_draft() -> super::ReviewLifecycleSubjectDraft<'static> {
    super::ReviewLifecycleSubjectDraft {
        subject_id: REVIEW_SUBJECT_ID,
        kind: super::ReviewLifecycleSubjectKind::Change,
        owner_label: REVIEW_OWNER,
        stage_label: REVIEW_STAGE,
    }
}

fn evidence_draft(hash: &str) -> super::ReviewEvidenceDraft<'_> {
    super::ReviewEvidenceDraft {
        evidence_id: REVIEW_EVIDENCE_ID,
        kind: super::ReviewEvidenceKind::FocusedCheck,
        label: REVIEW_EVIDENCE_LABEL,
        evidence_hash: hash,
        item_count: SINGLE_FINDING_COUNT,
        redaction: super::ReviewEvidenceRedaction::SafeSummary,
    }
}

fn rail_draft() -> super::ReviewRailDescriptorDraft<'static> {
    super::ReviewRailDescriptorDraft {
        rail_id: REVIEW_RAIL_ID,
        profile_label: REVIEW_RAIL_PROFILE,
        requirement: super::ReviewRailRequirement::Required,
    }
}

fn rail_result_draft(hash: &str) -> super::ReviewRailResultDraft<'_> {
    super::ReviewRailResultDraft {
        rail_id: REVIEW_RAIL_ID,
        status: super::ReviewRailStatus::Passed,
        summary_hash: hash,
        issue_code: None,
    }
}

fn attempt_draft() -> super::ReviewAttemptPlanDraft<'static> {
    super::ReviewAttemptPlanDraft {
        attempt_id: REVIEW_ATTEMPT_ID,
        reviewer_role: REVIEWER_ROLE,
        reasoning_profile: REASONING_PROFILE,
        output_contract: REVIEW_OUTPUT_CONTRACT,
        timeout_class: REVIEW_TIMEOUT_CLASS,
        budget_units: REVIEW_ATTEMPT_BUDGET,
    }
}

fn request_draft<'a>(
    evidence: &'a [super::ReviewEvidenceDraft<'a>],
    rails: &'a [super::ReviewRailDescriptorDraft<'a>],
    rail_results: &'a [super::ReviewRailResultDraft<'a>],
    attempts: &'a [super::ReviewAttemptPlanDraft<'a>],
) -> super::ReviewLifecycleRequestDraft<'a> {
    super::ReviewLifecycleRequestDraft {
        request_id: REVIEW_REQUEST_ID,
        subject: subject_draft(),
        evidence,
        rails,
        rail_results,
        attempts,
        requires_independent_reviewer: true,
        output_contract: REVIEW_OUTPUT_CONTRACT,
    }
}

fn review_request() -> super::ReviewLifecycleRequest {
    let hash = review_hash(REVIEW_EVIDENCE_LABEL);
    let evidence = [evidence_draft(hash.as_str())];
    let rails = [rail_draft()];
    let rail_hash = review_hash(REVIEW_RAIL_ID);
    let rail_results = [rail_result_draft(rail_hash.as_str())];
    let attempts = [attempt_draft()];
    super::validate_review_lifecycle_request(&request_draft(
        &evidence,
        &rails,
        &rail_results,
        &attempts,
    ))
    .unwrap()
}

#[test]
fn done_review_lifecycle_request_validates_bounded_evidence() {
    // r[verify agent_harness_runtime.hermes_done_review.lifecycle]
    // r[verify agent_harness_runtime.hermes_done_review.evidence]
    let request = review_request();
    let dispatch = super::plan_review_lifecycle_dispatch(
        &request,
        super::ReviewServiceStatus::Available,
    );

    assert_eq!(request.request_id(), REVIEW_REQUEST_ID);
    assert_eq!(request.subject().subject_id(), REVIEW_SUBJECT_ID);
    assert_eq!(request.evidence().len(), SINGLE_FINDING_COUNT);
    assert_eq!(request.rails().len(), SINGLE_FINDING_COUNT);
    assert_eq!(request.rail_results().len(), SINGLE_FINDING_COUNT);
    assert_eq!(request.attempts().len(), SINGLE_FINDING_COUNT);
    assert_eq!(request.request_hash().len(), crate::SAFE_HASH_HEX_CHARS);
    assert_eq!(dispatch.status(), super::ReviewDispatchStatus::Ready);
}

#[test]
fn done_review_rejects_unsafe_evidence_and_zero_attempt_budget() {
    // r[verify agent_harness_runtime.hermes_done_review.evidence]
    // r[verify agent_harness_runtime.hermes_done_review.verification]
    let evidence_hash = review_hash(REVIEW_EVIDENCE_LABEL);
    let unsafe_evidence = [super::ReviewEvidenceDraft {
        label: RAW_DIFF_MARKER,
        ..evidence_draft(evidence_hash.as_str())
    }];
    let rail_hash = review_hash(REVIEW_RAIL_ID);
    let rail_results = [rail_result_draft(rail_hash.as_str())];
    let rails = [rail_draft()];
    let attempts = [attempt_draft()];
    let unsafe_error = super::validate_review_lifecycle_request(&request_draft(
        &unsafe_evidence,
        &rails,
        &rail_results,
        &attempts,
    ))
    .expect_err("raw diff marker must be rejected");
    let zero_budget_attempt = [super::ReviewAttemptPlanDraft {
        budget_units: ZERO_REVIEW_ATTEMPT_BUDGET,
        ..attempt_draft()
    }];
    let budget_error = super::validate_review_lifecycle_request(&request_draft(
        &[evidence_draft(evidence_hash.as_str())],
        &rails,
        &rail_results,
        &zero_budget_attempt,
    ))
    .expect_err("zero reviewer budget must fail");

    assert_eq!(unsafe_error.code(), super::ReviewDiagnosticCode::UnsafeMetadata);
    assert_eq!(
        budget_error.code(),
        super::ReviewDiagnosticCode::PromptAdmissionDenied
    );
}

#[test]
fn missing_reviewer_service_fails_closed_without_default_dispatch() {
    // r[verify agent_harness_runtime.hermes_done_review.host_owned]
    let request = review_request();
    let dispatch = super::plan_review_lifecycle_dispatch(
        &request,
        super::ReviewServiceStatus::Missing,
    );
    let outcome = super::reduce_review_lifecycle_outcome(&request, &dispatch, None);

    assert_eq!(
        dispatch.status(),
        super::ReviewDispatchStatus::ReviewerServiceMissing
    );
    assert_eq!(outcome.status(), super::ReviewLifecycleStatus::MissingService);
    assert_eq!(
        outcome.continuation(),
        super::ReviewContinuationIntent::BlockCompletion
    );
    assert!(outcome.report_hash().is_none());
}

#[test]
fn reviewer_verdicts_reduce_to_safe_continuation_intents() {
    // r[verify agent_harness_runtime.hermes_done_review.outcomes]
    let request = review_request();
    let dispatch = super::plan_review_lifecycle_dispatch(
        &request,
        super::ReviewServiceStatus::Available,
    );
    let pass = super::parse_review_report(&draft("VERDICT: PASS\n\n## Findings\n")).unwrap();
    let warn = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## Findings\n- [class=docs scope=review route=policy-guidance promoted=no] Add note.\n",
    ))
    .unwrap();
    let fail = super::parse_review_report(&draft(
        "VERDICT: FAIL\n\n## Findings\n- [class=safety scope=review route=deterministic-check promoted=no] Fix rail.\n",
    ))
    .unwrap();
    let malformed = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## Findings\n- [class=docs scope=review promoted=no] Missing route.\n",
    ))
    .unwrap();

    let pass_outcome = super::reduce_review_lifecycle_outcome(&request, &dispatch, Some(&pass));
    let warn_outcome = super::reduce_review_lifecycle_outcome(&request, &dispatch, Some(&warn));
    let fail_outcome = super::reduce_review_lifecycle_outcome(&request, &dispatch, Some(&fail));
    let malformed_outcome =
        super::reduce_review_lifecycle_outcome(&request, &dispatch, Some(&malformed));

    assert_eq!(pass_outcome.status(), super::ReviewLifecycleStatus::Passed);
    assert_eq!(
        pass_outcome.continuation(),
        super::ReviewContinuationIntent::AcknowledgeCompletion
    );
    assert_eq!(warn_outcome.status(), super::ReviewLifecycleStatus::Warning);
    assert_eq!(
        warn_outcome.continuation(),
        super::ReviewContinuationIntent::ManualFollowUp
    );
    assert_eq!(fail_outcome.status(), super::ReviewLifecycleStatus::Failed);
    assert_eq!(
        fail_outcome.continuation(),
        super::ReviewContinuationIntent::BlockCompletion
    );
    assert_eq!(
        malformed_outcome.status(),
        super::ReviewLifecycleStatus::MalformedOutput
    );
}

#[test]
fn done_review_receipts_are_hash_backed_and_reject_leaks() {
    // r[verify agent_harness_runtime.hermes_done_review.verification]
    let request = review_request();
    let evidence_hash = request.evidence()[ZERO_COUNT].evidence_hash();
    let receipt = super::validate_review_lifecycle_receipt(&super::ReviewLifecycleReceiptDraft {
        request_hash: request.request_hash(),
        status: super::ReviewLifecycleStatus::Passed,
        continuation: super::ReviewContinuationIntent::AcknowledgeCompletion,
        issue_code: Some(REVIEW_RECEIPT_ISSUE),
        evidence_hashes: &[evidence_hash],
    })
    .unwrap();
    let leaking = super::validate_review_lifecycle_receipt(&super::ReviewLifecycleReceiptDraft {
        request_hash: request.request_hash(),
        status: super::ReviewLifecycleStatus::Failed,
        continuation: super::ReviewContinuationIntent::BlockCompletion,
        issue_code: Some("provider payload leaked"),
        evidence_hashes: &[evidence_hash],
    })
    .expect_err("receipt issue code must be safe");

    assert_eq!(receipt.request_hash(), request.request_hash());
    assert_eq!(receipt.receipt_hash().len(), crate::SAFE_HASH_HEX_CHARS);
    assert_eq!(receipt.evidence_hashes().len(), SINGLE_FINDING_COUNT);
    assert_eq!(leaking.code(), super::ReviewDiagnosticCode::UnsafeMetadata);
}
