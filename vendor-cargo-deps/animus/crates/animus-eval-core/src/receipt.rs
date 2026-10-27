//! Metadata-only native evaluation receipt projection.

use alloc::{format, string::String, vec, vec::Vec};

use crate::{hash_identity, validate_identity};

/// Maximum metadata labels in one receipt.
pub const MAX_RECEIPT_LABELS: usize = 64;
/// Maximum bytes in one diagnostic label.
pub const MAX_RECEIPT_LABEL_BYTES: usize = 96;
const PROHIBITED_RECEIPT_LABEL_TOKEN_COUNT: usize = 12;
const PROHIBITED_RECEIPT_LABEL_TOKENS: [&str; PROHIBITED_RECEIPT_LABEL_TOKEN_COUNT] = [
    "prompt",
    "hidden_test",
    "reasoning",
    "tool_body",
    "provider_body",
    "credential",
    "secret",
    "environment_value",
    "physical_path",
    "transcript",
    "private_locator",
    "unbounded_log",
];

/// Evaluation receipt status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationReceiptStatus {
    /// Planning completed without effect dispatch.
    Planned,
    /// Supplied terminal episode evidence was validated.
    Observed,
    /// Evaluation remains blocked.
    Blocked,
    /// Evaluation failed under its admitted policy.
    Failed,
}

/// Meaning represented by a receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationReceiptClaim {
    /// A bounded metadata observation only.
    BoundedObservation,
    /// Invalid verifier-soundness claim.
    VerifierSoundness,
    /// Invalid benchmark claim.
    BenchmarkValidity,
    /// Invalid training-fitness claim.
    TrainingFitness,
    /// Invalid authoritative-completion claim.
    AuthoritativeCompletion,
    /// Invalid production-readiness claim.
    ProductionReadiness,
    /// Invalid release-readiness claim.
    ReleaseReadiness,
}

/// Borrowed receipt projection input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationReceiptDraft<'a> {
    /// Evaluation identity.
    pub evaluation_identity: &'a str,
    /// Taskset plan identity.
    pub taskset_identity: &'a str,
    /// Recipe plan identity.
    pub recipe_identity: &'a str,
    /// Optional episode identity.
    pub episode_identity: Option<&'a str>,
    /// Optional reward reduction identity.
    pub reward_identity: Option<&'a str>,
    /// Optional resume decision identity.
    pub resume_identity: Option<&'a str>,
    /// Policy identity.
    pub policy_identity: &'a str,
    /// Evaluation status.
    pub status: EvaluationReceiptStatus,
    /// Planned task count.
    pub task_count: u32,
    /// Planned run count.
    pub run_count: u32,
    /// Completed run count.
    pub completed_count: u32,
    /// Redacted-item count.
    pub redaction_count: u32,
    /// Bounded safe diagnostic labels.
    pub diagnostic_labels: &'a [&'a str],
    /// Claims requested by the caller.
    pub claims: &'a [EvaluationReceiptClaim],
}

/// Validated metadata-only evaluation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationReceipt {
    /// Canonical receipt identity.
    pub identity: String,
    /// Evaluation identity.
    pub evaluation_identity: String,
    /// Taskset plan identity.
    pub taskset_identity: String,
    /// Recipe plan identity.
    pub recipe_identity: String,
    /// Optional episode identity.
    pub episode_identity: Option<String>,
    /// Optional reward identity.
    pub reward_identity: Option<String>,
    /// Optional resume identity.
    pub resume_identity: Option<String>,
    /// Policy identity.
    pub policy_identity: String,
    /// Status.
    pub status: EvaluationReceiptStatus,
    /// Planned task count.
    pub task_count: u32,
    /// Planned run count.
    pub run_count: u32,
    /// Completed run count.
    pub completed_count: u32,
    /// Redacted-item count.
    pub redaction_count: u32,
    /// Safe diagnostic labels.
    pub diagnostic_labels: Vec<String>,
    /// Explicit non-claims.
    pub non_claims: Vec<&'static str>,
}

/// Receipt validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptError {
    /// An identity is malformed.
    Identity,
    /// Counts are inconsistent or labels exceed their bounds.
    Bound,
    /// A diagnostic resembles private content or a physical locator.
    Leak,
    /// A requested claim exceeds metadata evidence authority.
    Overclaim,
}

/// Project a deterministic metadata-only receipt.
///
/// # Errors
/// Returns an identity, bound, leak, or overclaim class. No success identity is
/// emitted for rejected drafts.
pub fn project_evaluation_receipt(
    draft: &EvaluationReceiptDraft<'_>,
) -> Result<EvaluationReceipt, ReceiptError> {
    for identity in [
        Some(draft.evaluation_identity),
        Some(draft.taskset_identity),
        Some(draft.recipe_identity),
        draft.episode_identity,
        draft.reward_identity,
        draft.resume_identity,
        Some(draft.policy_identity),
    ]
    .into_iter()
    .flatten()
    {
        validate_identity(identity).map_err(|_| ReceiptError::Identity)?;
    }
    if draft.completed_count > draft.run_count || draft.diagnostic_labels.len() > MAX_RECEIPT_LABELS
    {
        return Err(ReceiptError::Bound);
    }
    for label in draft.diagnostic_labels {
        validate_receipt_label(label)?;
    }
    if draft
        .claims
        .iter()
        .any(|claim| *claim != EvaluationReceiptClaim::BoundedObservation)
    {
        return Err(ReceiptError::Overclaim);
    }

    let counts = format!(
        "{}:{}:{}:{}",
        draft.task_count, draft.run_count, draft.completed_count, draft.redaction_count
    );
    let labels = draft.diagnostic_labels.join(",");
    let identity = hash_identity(
        "animus.native-evaluation.receipt.v1",
        &[
            draft.evaluation_identity,
            draft.taskset_identity,
            draft.recipe_identity,
            draft.episode_identity.unwrap_or("none"),
            draft.reward_identity.unwrap_or("none"),
            draft.resume_identity.unwrap_or("none"),
            draft.policy_identity,
            status_label(draft.status),
            &counts,
            &labels,
        ],
    );
    Ok(EvaluationReceipt {
        identity,
        evaluation_identity: draft.evaluation_identity.into(),
        taskset_identity: draft.taskset_identity.into(),
        recipe_identity: draft.recipe_identity.into(),
        episode_identity: draft.episode_identity.map(String::from),
        reward_identity: draft.reward_identity.map(String::from),
        resume_identity: draft.resume_identity.map(String::from),
        policy_identity: draft.policy_identity.into(),
        status: draft.status,
        task_count: draft.task_count,
        run_count: draft.run_count,
        completed_count: draft.completed_count,
        redaction_count: draft.redaction_count,
        diagnostic_labels: draft
            .diagnostic_labels
            .iter()
            .map(|label| String::from(*label))
            .collect(),
        non_claims: vec![
            "not-task-correctness",
            "not-trace-completeness",
            "not-verifier-soundness",
            "not-benchmark-or-training-fitness",
            "not-authoritative-completion",
            "not-production-or-release-readiness",
        ],
    })
}

fn validate_receipt_label(label: &str) -> Result<(), ReceiptError> {
    if label.is_empty() || label.len() > MAX_RECEIPT_LABEL_BYTES {
        return Err(ReceiptError::Bound);
    }
    if !label.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
    }) {
        return Err(ReceiptError::Leak);
    }
    if PROHIBITED_RECEIPT_LABEL_TOKENS
        .iter()
        .any(|token| label.contains(token))
    {
        return Err(ReceiptError::Leak);
    }
    Ok(())
}

fn status_label(status: EvaluationReceiptStatus) -> &'static str {
    match status {
        EvaluationReceiptStatus::Planned => "planned",
        EvaluationReceiptStatus::Observed => "observed",
        EvaluationReceiptStatus::Blocked => "blocked",
        EvaluationReceiptStatus::Failed => "failed",
    }
}
