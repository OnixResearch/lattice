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
use super::{
    BLAKE3_ALGORITHM, EVAL_FEEDBACK_HASH_DOMAIN, FIELD_CANDIDATE, FIELD_OUTPUT_FORMAT,
    FIELD_PLAN_SCOPE, FIELD_RECEIPT_LEAK, FIELD_RECEIPT_REDATION, FIELD_RESOURCE_LABELS,
    FIELD_RUN_COUNT, FIELD_SAMPLING, FIELD_SCAFFOLD_HASH, FIELD_TASK_SET, FIELD_TOOL_SURFACE_HASH,
    FIELD_TRUST_BOUNDARY_HASH, FIELD_VERIFIER_HASH, HASH_PART_SEPARATOR,
    data::{
        EvalFeedbackCandidate, EvalFeedbackDiagnostic, EvalFeedbackDiagnosticKind,
        EvalFeedbackFingerprint, EvalFeedbackFreshnessDecision, EvalFeedbackFreshnessPolicy,
        EvalFeedbackOutputFormat, EvalFeedbackPlan, EvalFeedbackReceipt, EvalFeedbackReceiptLeak,
        EvalFeedbackRedactionState, EvalFeedbackReviewDecision, EvalFeedbackReviewOutcome,
        EvalFeedbackTaskSetKind, EvalFeedbackValidationResult,
    },
};

const MIN_RUN_COUNT: u32 = 1;

/// Computes a domain-separated BLAKE3 fingerprint for evaluation feedback.
#[must_use]
pub fn blake3_eval_feedback_fingerprint(label: &str, bytes: &[u8]) -> EvalFeedbackFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EVAL_FEEDBACK_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    EvalFeedbackFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Validates an offline scaffold evaluation plan.
///
/// # Errors
///
/// Returns diagnostics for remote task sets, missing hashes, unsupported formats,
/// non-positive run counts, missing labels, and unsupported scope requests.
pub fn validate_eval_feedback_plan(plan: &EvalFeedbackPlan) -> EvalFeedbackValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&plan.task_set.label, FIELD_TASK_SET, &mut diagnostics);
    if plan.task_set.kind == EvalFeedbackTaskSetKind::Remote {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::RemoteTaskSet,
            FIELD_TASK_SET,
            "offline evaluation plans must use local task sets",
        ));
    }
    require_text(
        &plan.scaffold_revision_id,
        FIELD_SCAFFOLD_HASH,
        &mut diagnostics,
    );
    require_hash(&plan.scaffold_hash, FIELD_SCAFFOLD_HASH, &mut diagnostics);
    require_text(&plan.model_route, FIELD_TASK_SET, &mut diagnostics);
    require_text(&plan.verifier.label, FIELD_VERIFIER_HASH, &mut diagnostics);
    require_hash(&plan.verifier.hash, FIELD_VERIFIER_HASH, &mut diagnostics);
    require_hash(
        &plan.trust_boundary_hash,
        FIELD_TRUST_BOUNDARY_HASH,
        &mut diagnostics,
    );
    require_hash(
        &plan.tool_surface_hash,
        FIELD_TOOL_SURFACE_HASH,
        &mut diagnostics,
    );
    require_text(&plan.sampling_profile, FIELD_SAMPLING, &mut diagnostics);
    require_collection(
        &plan.resource_labels,
        FIELD_RESOURCE_LABELS,
        &mut diagnostics,
    );
    if plan.run_count < MIN_RUN_COUNT {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::NonPositiveRunCount,
            FIELD_RUN_COUNT,
            "offline evaluation run count must be positive",
        ));
    }
    if plan.output_format == EvalFeedbackOutputFormat::Unsupported {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::UnsupportedOutputFormat,
            FIELD_OUTPUT_FORMAT,
            "offline evaluation output format is unsupported",
        ));
    }
    for _scope in &plan.unsupported_scope_requests {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::UnsupportedScope,
            FIELD_PLAN_SCOPE,
            "offline feedback scope rejects RL training, live self-mutation, cloud runners, hidden tests, provider eval APIs, and public benchmark claims",
        ));
    }
    to_validation_result(diagnostics)
}

/// Validates redacted evaluation receipt metadata.
///
/// # Errors
///
/// Returns diagnostics when required metadata is missing, hashes are invalid,
/// redaction is unsafe, or leak classes appear.
pub fn validate_eval_feedback_receipt(
    receipt: &EvalFeedbackReceipt,
) -> EvalFeedbackValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&receipt.model_id, FIELD_TASK_SET, &mut diagnostics);
    require_text(&receipt.model_route, FIELD_TASK_SET, &mut diagnostics);
    require_hash(
        &receipt.scaffold_hash,
        FIELD_SCAFFOLD_HASH,
        &mut diagnostics,
    );
    require_hash(
        &receipt.trust_boundary_hash,
        FIELD_TRUST_BOUNDARY_HASH,
        &mut diagnostics,
    );
    require_hash(
        &receipt.tool_surface_hash,
        FIELD_TOOL_SURFACE_HASH,
        &mut diagnostics,
    );
    require_hash(
        &receipt.verifier_hash,
        FIELD_VERIFIER_HASH,
        &mut diagnostics,
    );
    require_text(
        &receipt.runtime_version_label,
        FIELD_TASK_SET,
        &mut diagnostics,
    );
    require_text(
        &receipt.context_window_class,
        FIELD_TASK_SET,
        &mut diagnostics,
    );
    require_text(&receipt.sampling_profile, FIELD_SAMPLING, &mut diagnostics);
    require_collection(&receipt.sampling_labels, FIELD_SAMPLING, &mut diagnostics);
    require_collection(
        &receipt.resource_labels,
        FIELD_RESOURCE_LABELS,
        &mut diagnostics,
    );
    if receipt.run_count < MIN_RUN_COUNT {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::NonPositiveRunCount,
            FIELD_RUN_COUNT,
            "evaluation receipt run count must be positive",
        ));
    }
    if receipt.redaction_state == EvalFeedbackRedactionState::Unsafe {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::ReceiptLeak,
            FIELD_RECEIPT_REDATION,
            "evaluation receipt redaction state is unsafe",
        ));
    }
    for leak in &receipt.leaks {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::ReceiptLeak,
            FIELD_RECEIPT_LEAK,
            leak_detail(*leak),
        ));
    }
    to_validation_result(diagnostics)
}

/// Evaluates whether feedback can request scaffold admission.
#[must_use]
pub fn evaluate_feedback_review(candidate: &EvalFeedbackCandidate) -> EvalFeedbackReviewDecision {
    let mut diagnostics = Vec::new();
    require_candidate_shape(candidate, &mut diagnostics);
    let approved =
        candidate.review_outcome == EvalFeedbackReviewOutcome::Approved && diagnostics.is_empty();
    if candidate.review_outcome != EvalFeedbackReviewOutcome::Approved {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::ReviewNotAccepted,
            FIELD_CANDIDATE,
            "feedback candidate is non-authoritative until host review approves fresh evidence",
        ));
    }
    EvalFeedbackReviewDecision {
        can_request_scaffold_admission: approved,
        activates_scaffold: false,
        changes_runtime_selection_policy: false,
        diagnostics,
    }
}

/// Classifies receipt freshness against current evaluation policy.
#[must_use]
pub fn classify_eval_feedback_freshness(
    receipt: &EvalFeedbackReceipt,
    policy: &EvalFeedbackFreshnessPolicy,
) -> EvalFeedbackFreshnessDecision {
    let matches = receipt.model_route == policy.model_route
        && receipt.scaffold_hash == policy.scaffold_hash
        && receipt.verifier_hash == policy.verifier_hash
        && receipt.trust_boundary_hash == policy.trust_boundary_hash
        && receipt.tool_surface_hash == policy.tool_surface_hash
        && receipt.runtime_version_label == policy.runtime_version_label
        && receipt.sampling_profile == policy.sampling_profile;
    if matches {
        EvalFeedbackFreshnessDecision::Current
    } else if policy.reduce_weight_on_mismatch {
        EvalFeedbackFreshnessDecision::ReducedWeight
    } else {
        EvalFeedbackFreshnessDecision::Stale
    }
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn leak_detail(leak: EvalFeedbackReceiptLeak) -> &'static str {
    match leak {
        EvalFeedbackReceiptLeak::RawPrompt => "receipt contains raw prompt text",
        EvalFeedbackReceiptLeak::HiddenTest => "receipt contains hidden test content",
        EvalFeedbackReceiptLeak::ProviderPayload => "receipt contains provider payload",
        EvalFeedbackReceiptLeak::Credential => "receipt contains credential-looking value",
        EvalFeedbackReceiptLeak::FullPath => "receipt contains full path dump",
        EvalFeedbackReceiptLeak::VerifierBody => "receipt contains verifier body",
        EvalFeedbackReceiptLeak::UnboundedToolOutput => "receipt contains unbounded tool output",
    }
}

fn require_candidate_shape(
    candidate: &EvalFeedbackCandidate,
    diagnostics: &mut Vec<EvalFeedbackDiagnostic>,
) {
    require_text(&candidate.candidate_id, FIELD_CANDIDATE, diagnostics);
    require_text(
        &candidate.scaffold_revision_id,
        FIELD_CANDIDATE,
        diagnostics,
    );
    require_hash(&candidate.receipt_hash, FIELD_CANDIDATE, diagnostics);
}

fn require_collection(
    values: &[String],
    field: &str,
    diagnostics: &mut Vec<EvalFeedbackDiagnostic>,
) {
    if values.is_empty() || values.iter().any(|value| blank(value)) {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::EmptyCollection,
            field,
            "required collection is empty or contains a blank entry",
        ));
    }
}

fn require_hash(
    fingerprint: &EvalFeedbackFingerprint,
    field: &str,
    diagnostics: &mut Vec<EvalFeedbackDiagnostic>,
) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM || blank(&fingerprint.value_hex) {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::InvalidHash,
            field,
            "evaluation feedback hashes must use non-empty BLAKE3 fingerprints",
        ));
    }
}

fn require_text(text: &str, field: &str, diagnostics: &mut Vec<EvalFeedbackDiagnostic>) {
    if blank(text) {
        diagnostics.push(EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::MissingField,
            field,
            "required field is missing or blank",
        ));
    }
}

fn to_validation_result(diagnostics: Vec<EvalFeedbackDiagnostic>) -> EvalFeedbackValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
