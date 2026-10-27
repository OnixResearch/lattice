// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const TASK_SET: &str = "tasks.local";
const SCAFFOLD_REVISION: &str = "revision.one";
const MODEL_ROUTE: &str = "route.local";
const VERIFIER: &str = "verifier.main";
const SAMPLING: &str = "temperature-zero";
const RESOURCE: &str = "timeout.short";
const MODEL_ID: &str = "local-model";
const CONTEXT_WINDOW: &str = "context.medium";
const RUNTIME_VERSION: &str = "runtime.v1";
const CANDIDATE_ID: &str = "candidate.one";
const HASH_LABEL: &str = "hash";
const RUN_COUNT: u32 = 3;

#[test]
fn plan_validation_accepts_local_plan_and_rejects_scope_failures() {
    // r[verify agent_harness_runtime.ornith_eval_feedback.plan.invalid]
    let valid = plan();
    assert!(validate_eval_feedback_plan(&valid).is_ok());

    let invalid = EvalFeedbackPlan {
        task_set: EvalFeedbackTaskSet {
            label: TASK_SET.to_string(),
            kind: EvalFeedbackTaskSetKind::Remote,
        },
        scaffold_hash: EvalFeedbackFingerprint::new(BLAKE3_ALGORITHM, String::new()),
        verifier: EvalFeedbackVerifierIdentity {
            label: VERIFIER.to_string(),
            hash: EvalFeedbackFingerprint::new("SHA256", "abc"),
        },
        output_format: EvalFeedbackOutputFormat::Unsupported,
        run_count: 0,
        unsupported_scope_requests: vec![
            EvalFeedbackScopeRequest::ProviderEvalApi,
            EvalFeedbackScopeRequest::PublicBenchmarkClaim,
        ],
        ..valid
    };
    let diagnostics = validate_eval_feedback_plan(&invalid).expect_err("invalid plan");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == EvalFeedbackDiagnosticKind::RemoteTaskSet })
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == EvalFeedbackDiagnosticKind::InvalidHash
            && diagnostic.field == FIELD_SCAFFOLD_HASH
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == EvalFeedbackDiagnosticKind::InvalidHash
            && diagnostic.field == FIELD_VERIFIER_HASH
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == EvalFeedbackDiagnosticKind::UnsupportedOutputFormat
    }));
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == EvalFeedbackDiagnosticKind::NonPositiveRunCount
        })
    );
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == EvalFeedbackDiagnosticKind::UnsupportedScope)
            .count(),
        invalid.unsupported_scope_requests.len()
    );
}

#[test]
fn receipt_validation_rejects_transcript_and_sensitive_leaks() {
    // r[verify agent_harness_runtime.ornith_eval_feedback.receipts.redacted]
    let safe = receipt(Vec::new(), EvalFeedbackRedactionState::Redacted);
    assert!(validate_eval_feedback_receipt(&safe).is_ok());

    let leaking = receipt(
        vec![
            EvalFeedbackReceiptLeak::RawPrompt,
            EvalFeedbackReceiptLeak::HiddenTest,
            EvalFeedbackReceiptLeak::ProviderPayload,
            EvalFeedbackReceiptLeak::Credential,
            EvalFeedbackReceiptLeak::FullPath,
            EvalFeedbackReceiptLeak::VerifierBody,
            EvalFeedbackReceiptLeak::UnboundedToolOutput,
        ],
        EvalFeedbackRedactionState::Unsafe,
    );
    let diagnostics = validate_eval_feedback_receipt(&leaking).expect_err("receipt leak");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == EvalFeedbackDiagnosticKind::ReceiptLeak })
    );
    assert!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == EvalFeedbackDiagnosticKind::ReceiptLeak)
            .count()
            > leaking.leaks.len()
    );
}

#[test]
fn feedback_review_never_auto_activates_or_changes_policy() {
    // r[verify agent_harness_runtime.ornith_eval_feedback.review.no_auto_accept]
    let approved = candidate(EvalFeedbackReviewOutcome::Approved);
    let decision = evaluate_feedback_review(&approved);
    assert!(decision.can_request_scaffold_admission);
    assert!(!decision.activates_scaffold);
    assert!(!decision.changes_runtime_selection_policy);

    for outcome in [
        EvalFeedbackReviewOutcome::Missing,
        EvalFeedbackReviewOutcome::Denied,
        EvalFeedbackReviewOutcome::Unavailable,
        EvalFeedbackReviewOutcome::StaleEvidence,
    ] {
        let decision = evaluate_feedback_review(&candidate(outcome));
        assert!(!decision.can_request_scaffold_admission);
        assert!(!decision.activates_scaffold);
        assert!(!decision.changes_runtime_selection_policy);
        assert!(decision.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == EvalFeedbackDiagnosticKind::ReviewNotAccepted
        }));
    }
}

#[test]
fn freshness_detects_exact_match_stale_and_reduced_weight_evidence() {
    // r[verify agent_harness_runtime.ornith_eval_feedback.freshness.stale]
    let receipt = receipt(Vec::new(), EvalFeedbackRedactionState::Redacted);
    let current = policy(&receipt, false);
    assert_eq!(
        classify_eval_feedback_freshness(&receipt, &current),
        EvalFeedbackFreshnessDecision::Current
    );

    let stale_policy = EvalFeedbackFreshnessPolicy {
        scaffold_hash: fp("changed-scaffold"),
        ..current.clone()
    };
    assert_eq!(
        classify_eval_feedback_freshness(&receipt, &stale_policy),
        EvalFeedbackFreshnessDecision::Stale
    );

    let reduced_policy = EvalFeedbackFreshnessPolicy {
        verifier_hash: fp("changed-verifier"),
        reduce_weight_on_mismatch: true,
        ..current
    };
    assert_eq!(
        classify_eval_feedback_freshness(&receipt, &reduced_policy),
        EvalFeedbackFreshnessDecision::ReducedWeight
    );
}

#[test]
fn scope_rejects_live_self_improvement_claims() {
    // r[verify agent_harness_runtime.ornith_eval_feedback.scope.live_mutation_rejected]
    let invalid = EvalFeedbackPlan {
        unsupported_scope_requests: vec![
            EvalFeedbackScopeRequest::RlTraining,
            EvalFeedbackScopeRequest::LiveSelfMutation,
            EvalFeedbackScopeRequest::CloudBenchmarkRunner,
            EvalFeedbackScopeRequest::HiddenTestAccess,
        ],
        ..plan()
    };
    let diagnostics = validate_eval_feedback_plan(&invalid).expect_err("scope rejected");

    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == EvalFeedbackDiagnosticKind::UnsupportedScope)
            .count(),
        invalid.unsupported_scope_requests.len()
    );
}

fn candidate(review_outcome: EvalFeedbackReviewOutcome) -> EvalFeedbackCandidate {
    EvalFeedbackCandidate {
        candidate_id: CANDIDATE_ID.to_string(),
        kind: EvalFeedbackCandidateKind::Promote,
        scaffold_revision_id: SCAFFOLD_REVISION.to_string(),
        receipt_hash: fp(HASH_LABEL),
        review_outcome,
    }
}

fn fp(label: &str) -> EvalFeedbackFingerprint {
    blake3_eval_feedback_fingerprint(label, label.as_bytes())
}

fn plan() -> EvalFeedbackPlan {
    EvalFeedbackPlan {
        task_set: EvalFeedbackTaskSet {
            label: TASK_SET.to_string(),
            kind: EvalFeedbackTaskSetKind::Local,
        },
        scaffold_revision_id: SCAFFOLD_REVISION.to_string(),
        scaffold_hash: fp("scaffold"),
        model_route: MODEL_ROUTE.to_string(),
        verifier: EvalFeedbackVerifierIdentity {
            label: VERIFIER.to_string(),
            hash: fp("verifier"),
        },
        trust_boundary_hash: fp("trust-boundary"),
        tool_surface_hash: fp("tool-surface"),
        sampling_profile: SAMPLING.to_string(),
        resource_labels: vec![RESOURCE.to_string()],
        run_count: RUN_COUNT,
        output_format: EvalFeedbackOutputFormat::Jsonl,
        unsupported_scope_requests: Vec::new(),
    }
}

fn policy(
    receipt: &EvalFeedbackReceipt,
    reduce_weight_on_mismatch: bool,
) -> EvalFeedbackFreshnessPolicy {
    EvalFeedbackFreshnessPolicy {
        model_route: receipt.model_route.clone(),
        scaffold_hash: receipt.scaffold_hash.clone(),
        verifier_hash: receipt.verifier_hash.clone(),
        trust_boundary_hash: receipt.trust_boundary_hash.clone(),
        tool_surface_hash: receipt.tool_surface_hash.clone(),
        runtime_version_label: receipt.runtime_version_label.clone(),
        sampling_profile: receipt.sampling_profile.clone(),
        reduce_weight_on_mismatch,
    }
}

fn receipt(
    leaks: Vec<EvalFeedbackReceiptLeak>,
    redaction_state: EvalFeedbackRedactionState,
) -> EvalFeedbackReceipt {
    EvalFeedbackReceipt {
        model_id: MODEL_ID.to_string(),
        model_route: MODEL_ROUTE.to_string(),
        scaffold_hash: fp("scaffold"),
        trust_boundary_hash: fp("trust-boundary"),
        tool_surface_hash: fp("tool-surface"),
        verifier_hash: fp("verifier"),
        runtime_version_label: RUNTIME_VERSION.to_string(),
        context_window_class: CONTEXT_WINDOW.to_string(),
        sampling_profile: SAMPLING.to_string(),
        sampling_labels: vec![SAMPLING.to_string()],
        resource_labels: vec![RESOURCE.to_string()],
        run_count: RUN_COUNT,
        status: EvalFeedbackRunStatus::Passed,
        redaction_state,
        leaks,
    }
}
