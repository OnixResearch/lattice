use super::*;

const SUMMARY_BOUND_BYTES: u32 = 128;
const DIFF_BOUND_BYTES: usize = 256;
const OVERSIZED_DIFF_BYTES: usize = 257;
const SINGLE_MATCH: u32 = 1;
const AMBIGUOUS_MATCHES: u32 = 2;
const NON_CLAIM_LIFECYCLE: &str = "does-not-grant-lifecycle-authority";
const NON_CLAIM_RETRY: &str = "does-not-authorize-retry";
const NON_CLAIM_CORRECTNESS: &str = "does-not-prove-later-correction";

const _: () = assert!(OVERSIZED_DIFF_BYTES > DIFF_BOUND_BYTES);

#[test]
fn valid_cairn_handoff_projects_bounded_summary_and_metadata_receipt() {
    // r[agent_harness_runtime.rejected_attempt.boundary]
    // r[agent_harness_runtime.rejected_attempt.planning]
    // r[agent_harness_runtime.rejected_attempt.evidence]
    let plan = plan_context(&request(), &policy()).expect("valid handoff");
    assert_eq!(plan.redacted_summary, "check failed: stale projection");
    assert_eq!(
        plan.raw_diff_disposition,
        RawDiffDisposition::DeniedByPolicy
    );
    assert_eq!(plan.context_profile, "fresh-context-task");
    assert_eq!(plan.loop_profile, "ralph-loop");

    let receipt = project_receipt(&plan, &policy());
    assert!(validate_receipt(&receipt, &policy()).is_ok());
    assert!(!receipt.receipt_id.as_str().is_empty());
    assert_eq!(receipt.non_claims.len(), policy().required_non_claims.len());
}

#[test]
fn current_jujutsu_record_is_vcs_neutral_and_deterministic() {
    // r[agent_harness_runtime.rejected_attempt.model]
    let first = plan_context(&request(), &policy()).expect("first plan");
    let second = plan_context(&request(), &policy()).expect("second plan");
    assert_eq!(first, second);
    assert_eq!(request().vcs.provider, VcsProvider::Jujutsu);
    assert_eq!(
        project_receipt(&first, &policy()).receipt_id,
        project_receipt(&second, &policy()).receipt_id
    );
}

#[test]
fn missing_review_wrong_task_and_ambiguous_match_fail_closed() {
    // r[agent_harness_runtime.rejected_attempt.boundary]
    // r[agent_harness_runtime.rejected_attempt.verification]
    let mut input = request();
    input.review.validated = false;
    input.review.task_id = TaskIdentity::new("task.other");
    input.review.matching_attempts = AMBIGUOUS_MATCHES;
    let issues = plan_context(&input, &policy()).expect_err("must deny");
    assert!(issues.contains(&Issue::ReviewEvidenceDenied));
    assert!(issues.contains(&Issue::WrongTaskOrRun));
    assert!(issues.contains(&Issue::AmbiguousAttempt));
}

#[test]
fn blank_identity_and_denied_vcs_provider_fail_closed() {
    let mut input = request();
    input.run_id = RunIdentity::new("");
    input.vcs.provider = VcsProvider::PolicyDeclared;
    let issues = plan_context(&input, &policy()).expect_err("must deny");
    assert!(issues.contains(&Issue::MissingIdentity));
    assert!(issues.contains(&Issue::VcsProviderDenied));
}

#[test]
fn stale_revision_unsafe_summary_and_oversized_summary_fail_closed() {
    // r[agent_harness_runtime.rejected_attempt.policy]
    // r[agent_harness_runtime.rejected_attempt.verification]
    let mut input = request();
    input.vcs.current = false;
    input.vcs.content_id = ContentIdentity::new("content.changed");
    let summary_bound_bytes = usize::try_from(SUMMARY_BOUND_BYTES).expect("test bound fits");
    input.context.redacted_summary = format!("API_KEY {}", "x".repeat(summary_bound_bytes));
    input.context.summary_classification_safe = false;
    let issues = plan_context(&input, &policy()).expect_err("must deny");
    assert!(issues.contains(&Issue::StaleAttempt));
    assert!(issues.contains(&Issue::UnsafeSummary));
    assert!(issues.contains(&Issue::OversizedSummary));
}

#[test]
fn raw_diff_requires_policy_capability_classification_and_budget() {
    // r[agent_harness_runtime.rejected_attempt.policy]
    let mut input = request();
    let mut admitted_policy = policy();
    admitted_policy.allow_raw_diff = true;

    input.context.raw_diff_capability_admitted = true;
    input.context.raw_diff_classification_safe = false;
    let classified_plan = plan_context(&input, &admitted_policy).expect("summary valid");
    assert_eq!(
        classified_plan.raw_diff_disposition,
        RawDiffDisposition::DeniedByClassification
    );

    input.context.raw_diff_classification_safe = true;
    input.context.raw_diff = Some(vec![b'x'; OVERSIZED_DIFF_BYTES]);
    let budget_denied_plan = plan_context(&input, &admitted_policy).expect("summary valid");
    assert_eq!(
        budget_denied_plan.raw_diff_disposition,
        RawDiffDisposition::DeniedByBudget
    );
}

#[test]
fn receipt_rejects_completion_overclaims_and_weakened_non_claims() {
    // r[agent_harness_runtime.rejected_attempt.evidence]
    // r[agent_harness_runtime.rejected_attempt.verification]
    let plan = plan_context(&request(), &policy()).expect("valid handoff");
    let mut receipt = project_receipt(&plan, &policy());
    receipt.claims.push("task-complete".to_string());
    receipt.non_claims.clear();
    let issues = validate_receipt(&receipt, &policy()).expect_err("must deny");
    assert!(issues.contains(&Issue::ReceiptOverclaim));
    assert!(issues.contains(&Issue::MissingNonClaim));
}

fn policy() -> PlanningContract {
    PlanningContract {
        policy_id: "rejected-attempt.default.v1".to_string(),
        accepted_review_profiles: vec![ReviewEvidenceProfile::CairnRevisionHandoff],
        accepted_vcs_providers: vec![VcsProvider::Jujutsu, VcsProvider::Git],
        max_summary_bytes: SUMMARY_BOUND_BYTES,
        max_raw_diff_bytes: u32::try_from(DIFF_BOUND_BYTES).expect("test bound fits"),
        allow_raw_diff: false,
        required_non_claims: vec![
            NON_CLAIM_LIFECYCLE.to_string(),
            NON_CLAIM_RETRY.to_string(),
            NON_CLAIM_CORRECTNESS.to_string(),
        ],
    }
}

#[derive(Clone)]
struct FixtureIdentities {
    task: TaskIdentity,
    run: RunIdentity,
    attempt: AttemptIdentity,
    revision: RevisionIdentity,
    workspace: WorkspaceSnapshotIdentity,
    content: ContentIdentity,
}

fn request() -> Request {
    let identities = identities();
    let request = Request {
        task_id: identities.task.clone(),
        run_id: identities.run.clone(),
        attempt_id: identities.attempt.clone(),
        revision_id: identities.revision.clone(),
        workspace_snapshot_id: identities.workspace.clone(),
        content_id: identities.content.clone(),
        review: review_facts(&identities),
        vcs: vcs_facts(&identities),
        context: context_facts(),
    };
    assert_eq!(request.review.task_id, request.task_id);
    assert_eq!(request.vcs.content_id, request.content_id);
    request
}

fn identities() -> FixtureIdentities {
    FixtureIdentities {
        task: TaskIdentity::new("task.one"),
        run: RunIdentity::new("run.one"),
        attempt: AttemptIdentity::new("attempt.one"),
        revision: RevisionIdentity::new("revision.one"),
        workspace: WorkspaceSnapshotIdentity::new("workspace.one"),
        content: ContentIdentity::new("content.one"),
    }
}

fn review_facts(identities: &FixtureIdentities) -> ReviewHandoffFacts {
    ReviewHandoffFacts {
        handoff_id: ReviewHandoffIdentity::new("handoff.one"),
        profile: ReviewEvidenceProfile::CairnRevisionHandoff,
        task_id: identities.task.clone(),
        run_id: identities.run.clone(),
        attempt_id: identities.attempt.clone(),
        revision_id: identities.revision.clone(),
        disposition: ReviewDisposition::Rejected,
        matching_attempts: SINGLE_MATCH,
        validated: true,
    }
}

fn vcs_facts(identities: &FixtureIdentities) -> VcsFacts {
    VcsFacts {
        provider: VcsProvider::Jujutsu,
        attempt_id: identities.attempt.clone(),
        revision_id: identities.revision.clone(),
        workspace_snapshot_id: identities.workspace.clone(),
        content_id: identities.content.clone(),
        current: true,
    }
}

fn context_facts() -> RejectionContextFacts {
    RejectionContextFacts {
        summary_id: SummaryIdentity::new("summary.one"),
        redacted_summary: "check failed: stale projection".to_string(),
        summary_classification_safe: true,
        raw_diff: Some(b"redacted metadata diff".to_vec()),
        raw_diff_classification_safe: true,
        raw_diff_capability_admitted: false,
    }
}
