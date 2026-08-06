//! Metadata-only semantic role receipts.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::assignment::AgentRoleAssignment;
use super::assignment::AgentRoleCapabilityResolution;
use super::assignment::AgentRoleHandoff;
use super::assignment::AgentRolePlan;
use super::output::AgentRoleOutput;
use super::separation::AgentRoleSeparationDecision;
use super::types::AGENT_ROLE_RECEIPT_HASH_DOMAIN;
use super::types::AgentRoleClass;
use super::types::AgentRoleError;
use super::types::AgentRoleIssue;
use super::types::AgentRoleProfile;
use super::validation::error;
use super::validation::hash_parts;

const FIELD_RECEIPT: &str = "agent_role_receipt";
const REDACTION_MARKER: &str = "raw-role-context-and-output-omitted";

/// Metadata-only semantic role receipt class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleReceiptKind {
    /// Role assignment admission.
    Assignment,
    /// Capability intersection decision.
    CapabilityDecision,
    /// Separation-of-duty decision.
    SeparationDecision,
    /// Immutable role plan.
    RolePlan,
    /// Bounded handoff.
    Handoff,
    /// Admitted role output.
    Output,
}

impl AgentRoleReceiptKind {
    /// Returns the stable receipt-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assignment => "assignment",
            Self::CapabilityDecision => "capability_decision",
            Self::SeparationDecision => "separation_decision",
            Self::RolePlan => "role_plan",
            Self::Handoff => "handoff",
            Self::Output => "output",
        }
    }
}

/// Forbidden durable role material candidates.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AgentRoleReceiptSensitiveDraft<'a> {
    /// Raw prompt or role instruction.
    pub raw_prompt_or_instruction: Option<&'a str>,
    /// Hidden reasoning.
    pub hidden_reasoning: Option<&'a str>,
    /// Provider payload.
    pub provider_payload: Option<&'a str>,
    /// Credential or token.
    pub credential_or_token: Option<&'a str>,
    /// Private or full path.
    pub private_path: Option<&'a str>,
    /// Raw verifier/check body.
    pub verifier_body: Option<&'a str>,
    /// Hidden test or oracle.
    pub hidden_test_or_oracle: Option<&'a str>,
    /// Raw diff or file body.
    pub raw_diff_or_file: Option<&'a str>,
    /// Tool arguments or output.
    pub tool_body: Option<&'a str>,
    /// Judge rationale.
    pub judge_rationale: Option<&'a str>,
    /// Raw or unbounded transcript.
    pub transcript: Option<&'a str>,
}

/// Explicit source references for one role receipt projection.
#[derive(Clone, Copy)]
pub struct AgentRoleReceiptInput<'a> {
    /// Receipt class.
    pub kind: AgentRoleReceiptKind,
    /// Exact assignment.
    pub assignment: &'a AgentRoleAssignment,
    /// Exact role profile.
    pub profile: &'a AgentRoleProfile,
    /// Capability decision when relevant.
    pub capability_resolution: Option<&'a AgentRoleCapabilityResolution>,
    /// Separation decision when relevant.
    pub separation: Option<&'a AgentRoleSeparationDecision>,
    /// Immutable role plan when relevant.
    pub role_plan: Option<&'a AgentRolePlan>,
    /// Bounded handoff when relevant.
    pub handoff: Option<&'a AgentRoleHandoff>,
    /// Admitted role output when relevant.
    pub output: Option<&'a AgentRoleOutput>,
}

/// Safe bounded semantic role evidence receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleReceipt {
    kind: AgentRoleReceiptKind,
    role_id: String,
    role_class: AgentRoleClass,
    role_profile_hash: String,
    assignment_hash: String,
    subject_hash: String,
    actor_hash: String,
    model_plan_hash: String,
    policy_hash: String,
    grant_set_hash: String,
    separation_hash: String,
    capability_decision_hash: Option<String>,
    role_plan_hash: Option<String>,
    handoff_hash: Option<String>,
    output_hash: Option<String>,
    effective_capability_count: u32,
    denied_capability_count: u32,
    separation_passed: Option<bool>,
    redaction_marker: &'static str,
    receipt_hash: String,
}

impl AgentRoleReceipt {
    /// Returns receipt class.
    #[must_use]
    pub const fn kind(&self) -> AgentRoleReceiptKind {
        self.kind
    }

    /// Returns semantic role ID.
    #[must_use]
    pub fn role_id(&self) -> &str {
        self.role_id.as_str()
    }

    /// Returns semantic role class.
    #[must_use]
    pub const fn role_class(&self) -> AgentRoleClass {
        self.role_class
    }

    /// Returns assignment identity.
    #[must_use]
    pub fn assignment_hash(&self) -> &str {
        self.assignment_hash.as_str()
    }

    /// Returns effective capability count.
    #[must_use]
    pub const fn effective_capability_count(&self) -> u32 {
        self.effective_capability_count
    }

    /// Returns metadata-only redaction marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Projects bounded metadata-only evidence from role decisions.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for mismatched sources, missing kind-specific data,
/// or any forbidden raw, hidden, credential, path, verifier, tool, diff, rationale,
/// or transcript material.
// r[impl agent_harness_runtime.agent_role_profiles.receipts]
pub fn project_agent_role_receipt(
    input: AgentRoleReceiptInput<'_>,
    sensitive: AgentRoleReceiptSensitiveDraft<'_>,
) -> Result<AgentRoleReceipt, AgentRoleError> {
    reject_sensitive_receipt(sensitive)?;
    validate_receipt_sources(&input)?;
    let capability_decision_hash = input.capability_resolution.map(|resolution| resolution.decision_hash().to_string());
    let role_plan_hash = input.role_plan.map(|plan| plan.plan_hash().to_string());
    let handoff_hash = input.handoff.map(|handoff| handoff.handoff_hash().to_string());
    let output_hash = input.output.map(|output| output.output_hash().to_string());
    let (effective_capability_count, denied_capability_count) = receipt_capability_counts(&input)?;
    let separation_passed = input.separation.map(AgentRoleSeparationDecision::passed);
    let receipt_hash = hash_parts(AGENT_ROLE_RECEIPT_HASH_DOMAIN, &[
        input.kind.as_str().to_string(),
        input.profile.role_id().to_string(),
        input.profile.class().as_str().to_string(),
        input.profile.profile_hash().to_string(),
        input.assignment.assignment_hash().to_string(),
        input.assignment.subject_hash().to_string(),
        input.assignment.actor_hash().to_string(),
        input.assignment.model_plan_hash().to_string(),
        input.assignment.policy_hash().to_string(),
        input.assignment.grant_set_hash().to_string(),
        input.assignment.separation_hash().to_string(),
        capability_decision_hash.as_deref().unwrap_or("none").to_string(),
        role_plan_hash.as_deref().unwrap_or("none").to_string(),
        handoff_hash.as_deref().unwrap_or("none").to_string(),
        output_hash.as_deref().unwrap_or("none").to_string(),
        effective_capability_count.to_string(),
        denied_capability_count.to_string(),
        separation_passed.map_or("none".to_string(), |passed| passed.to_string()),
        REDACTION_MARKER.to_string(),
    ]);
    debug_assert!(!receipt_hash.is_empty());
    debug_assert_eq!(input.assignment.role_profile_hash(), input.profile.profile_hash());
    Ok(AgentRoleReceipt {
        kind: input.kind,
        role_id: input.profile.role_id().to_string(),
        role_class: input.profile.class(),
        role_profile_hash: input.profile.profile_hash().to_string(),
        assignment_hash: input.assignment.assignment_hash().to_string(),
        subject_hash: input.assignment.subject_hash().to_string(),
        actor_hash: input.assignment.actor_hash().to_string(),
        model_plan_hash: input.assignment.model_plan_hash().to_string(),
        policy_hash: input.assignment.policy_hash().to_string(),
        grant_set_hash: input.assignment.grant_set_hash().to_string(),
        separation_hash: input.assignment.separation_hash().to_string(),
        capability_decision_hash,
        role_plan_hash,
        handoff_hash,
        output_hash,
        effective_capability_count,
        denied_capability_count,
        separation_passed,
        redaction_marker: REDACTION_MARKER,
        receipt_hash,
    })
}

fn receipt_capability_counts(input: &AgentRoleReceiptInput<'_>) -> Result<(u32, u32), AgentRoleError> {
    let effective_capability_count = fixed_width_capability_count(
        input.capability_resolution.map_or(usize::default(), |resolution| resolution.effective().len()),
        "effective capability count exceeds its fixed-width receipt bound",
    )?;
    let denied_capability_count = fixed_width_capability_count(
        input.capability_resolution.map_or(usize::default(), |resolution| resolution.denied().len()),
        "denied capability count exceeds its fixed-width receipt bound",
    )?;
    debug_assert!(input.capability_resolution.is_some() || effective_capability_count == 0);
    debug_assert!(input.capability_resolution.is_some() || denied_capability_count == 0);
    Ok((effective_capability_count, denied_capability_count))
}

fn fixed_width_capability_count(item_count: usize, overflow_detail: &'static str) -> Result<u32, AgentRoleError> {
    u32::try_from(item_count).map_err(|_| error(AgentRoleIssue::InvalidCollection, FIELD_RECEIPT, overflow_detail))
}

fn reject_sensitive_receipt(sensitive: AgentRoleReceiptSensitiveDraft<'_>) -> Result<(), AgentRoleError> {
    let has_leak = [
        sensitive.raw_prompt_or_instruction,
        sensitive.hidden_reasoning,
        sensitive.provider_payload,
        sensitive.credential_or_token,
        sensitive.private_path,
        sensitive.verifier_body,
        sensitive.hidden_test_or_oracle,
        sensitive.raw_diff_or_file,
        sensitive.tool_body,
        sensitive.judge_rationale,
        sensitive.transcript,
    ]
    .iter()
    .any(Option::is_some);
    if has_leak {
        return Err(error(
            AgentRoleIssue::ReceiptLeak,
            FIELD_RECEIPT,
            "role receipt contains forbidden raw or hidden material",
        ));
    }
    debug_assert!(!has_leak);
    debug_assert!(sensitive.raw_prompt_or_instruction.is_none());
    Ok(())
}

fn validate_receipt_sources(input: &AgentRoleReceiptInput<'_>) -> Result<(), AgentRoleError> {
    if input.assignment.role_profile_hash() != input.profile.profile_hash() {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_RECEIPT,
            "receipt role profile differs from assignment",
        ));
    }
    let has_binding_mismatch = [
        input
            .capability_resolution
            .is_some_and(|resolution| resolution.assignment_hash() != input.assignment.assignment_hash()),
        input.role_plan.is_some_and(|plan| plan.assignment_hash() != input.assignment.assignment_hash()),
        input.handoff.is_some_and(|handoff| handoff.subject_hash() != input.assignment.subject_hash()),
        input.output.is_some_and(|output| output.assignment_hash() != input.assignment.assignment_hash()),
    ]
    .into_iter()
    .any(|is_mismatch| is_mismatch);
    if has_binding_mismatch {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_RECEIPT,
            "receipt source belongs to another assignment or subject",
        ));
    }
    let is_required_source_present = match input.kind {
        AgentRoleReceiptKind::Assignment => true,
        AgentRoleReceiptKind::CapabilityDecision => input.capability_resolution.is_some(),
        AgentRoleReceiptKind::SeparationDecision => input.separation.is_some(),
        AgentRoleReceiptKind::RolePlan => input.role_plan.is_some(),
        AgentRoleReceiptKind::Handoff => input.handoff.is_some(),
        AgentRoleReceiptKind::Output => input.output.is_some(),
    };
    if !is_required_source_present {
        return Err(error(
            AgentRoleIssue::MissingReference,
            FIELD_RECEIPT,
            "receipt kind lacks its required typed source",
        ));
    }
    debug_assert!(is_required_source_present);
    debug_assert_eq!(input.assignment.role_profile_hash(), input.profile.profile_hash());
    Ok(())
}
