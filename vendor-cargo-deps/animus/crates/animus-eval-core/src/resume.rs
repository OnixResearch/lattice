//! Content-addressed evaluation resume decisions.

use alloc::string::String;

use crate::{
    EvaluationPolicy, ResumeMismatchPolicy, hash_identity, validate_blake3_identity,
    validate_identity,
};

/// Every material identity that participates in a resume key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "the identity suffix makes every resume-key binding explicit"
)]
pub struct ResumeMaterial<'a> {
    /// Canonical task-data identity.
    pub task_data_identity: &'a str,
    /// Taskset identity.
    pub taskset_identity: &'a str,
    /// Recipe plan identity.
    pub recipe_identity: &'a str,
    /// Effective model identity.
    pub model_identity: &'a str,
    /// Harness identity.
    pub harness_identity: &'a str,
    /// Environment identity.
    pub environment_identity: &'a str,
    /// Tool policy identity.
    pub tool_identity: &'a str,
    /// Verifier identity.
    pub verifier_identity: &'a str,
    /// Rubric identity.
    pub rubric_identity: &'a str,
    /// Reward policy identity.
    pub reward_identity: &'a str,
    /// Sampling policy identity.
    pub sampling_identity: &'a str,
    /// Redaction policy identity.
    pub redaction_identity: &'a str,
    /// Output contract identity.
    pub output_identity: &'a str,
    /// Evaluation policy identity.
    pub policy_identity: &'a str,
}

/// Validated state claimed by prior resume evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeEvidenceState {
    /// Complete, terminal, successful, safe, and within budget.
    CompleteTerminalSuccess,
    /// Record is incomplete or non-terminal.
    Incomplete,
    /// Terminal work failed.
    Failed,
    /// A torn write was observed.
    Torn,
    /// Redaction or evidence-safety checks failed.
    Unsafe,
    /// Recorded resource use exceeded the current bound.
    OverBudget,
}

/// Prior terminal evidence supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeEvidence<'a> {
    /// Resume key claimed by the record.
    pub resume_key: &'a str,
    /// Evidence record identity.
    pub evidence_identity: &'a str,
    /// Output evidence identity.
    pub output_evidence_identity: &'a str,
    /// Explicit evidence state.
    pub state: ResumeEvidenceState,
}

/// Pure resume action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeAction {
    /// Keep one exact complete result.
    Keep,
    /// Rerun this work item.
    Rerun,
    /// Block until evidence conflict or corruption is resolved.
    Block,
    /// Redo all runs for the affected task.
    RedoWholeTask,
}

/// Deterministic resume decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumeDecision {
    /// Canonical current resume key.
    pub resume_key: String,
    /// Selected action.
    pub action: ResumeAction,
    /// Exact accepted evidence identity for a keep decision.
    pub accepted_evidence_identity: Option<String>,
    /// Stable reason label.
    pub reason: &'static str,
}

/// Resume validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeError {
    /// Current resume material is malformed.
    Material,
    /// Prior evidence identity is malformed.
    Evidence,
    /// Prior evidence is duplicated or conflicting.
    Conflict,
}

/// Build the full content-addressed resume key.
///
/// # Errors
/// Returns [`ResumeError::Material`] if any material identity is malformed.
pub fn build_resume_key(material: &ResumeMaterial<'_>) -> Result<String, ResumeError> {
    let values = material_values(material);
    for identity in values {
        validate_identity(identity).map_err(|_| ResumeError::Material)?;
    }
    Ok(hash_identity(
        "animus.native-evaluation.resume-key.v1",
        &values,
    ))
}

/// Evaluate caller-supplied resume evidence without reading or mutating storage.
///
/// # Errors
/// Returns a malformed-evidence or conflict class. Stale and incomplete evidence
/// becomes an explicit rerun, block, or redo decision according to policy.
pub fn evaluate_resume(
    material: &ResumeMaterial<'_>,
    evidence: &[ResumeEvidence<'_>],
    policy: &EvaluationPolicy,
) -> Result<ResumeDecision, ResumeError> {
    let resume_key = build_resume_key(material)?;
    if evidence.len() > 1 {
        return Err(ResumeError::Conflict);
    }
    let Some(record) = evidence.first() else {
        return Ok(ResumeDecision {
            resume_key,
            action: ResumeAction::Rerun,
            accepted_evidence_identity: None,
            reason: "missing",
        });
    };
    validate_blake3_identity(record.resume_key).map_err(|_| ResumeError::Evidence)?;
    validate_identity(record.evidence_identity).map_err(|_| ResumeError::Evidence)?;
    validate_identity(record.output_evidence_identity).map_err(|_| ResumeError::Evidence)?;
    if record.resume_key == resume_key
        && record.state == ResumeEvidenceState::CompleteTerminalSuccess
    {
        return Ok(ResumeDecision {
            resume_key,
            action: ResumeAction::Keep,
            accepted_evidence_identity: Some(record.evidence_identity.into()),
            reason: "exact_complete_terminal_match",
        });
    }
    let (action, reason) = match record.state {
        ResumeEvidenceState::Torn | ResumeEvidenceState::Unsafe => {
            (ResumeAction::Block, "unsafe_or_torn")
        }
        ResumeEvidenceState::OverBudget => mismatch_action(policy.resume_mismatch, "over_budget"),
        ResumeEvidenceState::Incomplete | ResumeEvidenceState::Failed => {
            mismatch_action(policy.resume_mismatch, "incomplete_or_failed")
        }
        ResumeEvidenceState::CompleteTerminalSuccess if record.resume_key != resume_key => {
            mismatch_action(policy.resume_mismatch, "identity_drift")
        }
        ResumeEvidenceState::CompleteTerminalSuccess => {
            mismatch_action(policy.resume_mismatch, "stale")
        }
    };
    Ok(ResumeDecision {
        resume_key,
        action,
        accepted_evidence_identity: None,
        reason,
    })
}

const RESUME_MATERIAL_FIELD_COUNT: usize = 14;

fn material_values<'a>(material: &'a ResumeMaterial<'a>) -> [&'a str; RESUME_MATERIAL_FIELD_COUNT] {
    [
        material.task_data_identity,
        material.taskset_identity,
        material.recipe_identity,
        material.model_identity,
        material.harness_identity,
        material.environment_identity,
        material.tool_identity,
        material.verifier_identity,
        material.rubric_identity,
        material.reward_identity,
        material.sampling_identity,
        material.redaction_identity,
        material.output_identity,
        material.policy_identity,
    ]
}

fn mismatch_action(
    policy: ResumeMismatchPolicy,
    reason: &'static str,
) -> (ResumeAction, &'static str) {
    let action = match policy {
        ResumeMismatchPolicy::Rerun => ResumeAction::Rerun,
        ResumeMismatchPolicy::Block => ResumeAction::Block,
        ResumeMismatchPolicy::RedoWholeTask => ResumeAction::RedoWholeTask,
    };
    (action, reason)
}
