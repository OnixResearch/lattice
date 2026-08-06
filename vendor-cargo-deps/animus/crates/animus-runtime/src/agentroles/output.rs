//! Assignment-bound role output admission and bounded claim classes.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::assignment::AgentRoleAssignment;
use super::assignment::AgentRoleCapabilityResolution;
use super::assignment::AgentRolePlan;
use super::assignment::RoleAuthorizationExpectation;
use super::assignment::role_authorization_is_exact;
use super::separation::AgentRoleSeparationDecision;
use super::types::AGENT_ROLE_OUTPUT_HASH_DOMAIN;
use super::types::AgentRoleClass;
use super::types::AgentRoleError;
use super::types::AgentRoleIssue;
use super::types::AgentRoleOutputKind;
use super::types::AgentRoleProfile;
use super::validation::ValidationValue;
use super::validation::error;
use super::validation::hash_parts;
use super::validation::lowercase_blake3;

/// Scoped implementation-candidate emission ability.
pub const AGENT_ROLE_EMIT_CANDIDATE_ABILITY: &str = "emit_candidate";
/// Scoped review-report emission ability.
pub const AGENT_ROLE_EMIT_REVIEW_ABILITY: &str = "emit_review";
/// Scoped verification-evidence emission ability.
pub const AGENT_ROLE_EMIT_VERIFICATION_ABILITY: &str = "emit_verification";
/// Scoped judgment-report emission ability.
pub const AGENT_ROLE_EMIT_JUDGMENT_ABILITY: &str = "emit_judgment";
/// Scoped judgment-selection ability.
pub const AGENT_ROLE_SELECT_ABILITY: &str = "select";
/// Scoped judgment-veto ability.
pub const AGENT_ROLE_VETO_ABILITY: &str = "veto";
/// Scoped coordination-report ability.
pub const AGENT_ROLE_EMIT_COORDINATION_ABILITY: &str = "emit_coordination";
/// Scoped observation-report ability.
pub const AGENT_ROLE_EMIT_OBSERVATION_ABILITY: &str = "emit_observation";

const FIELD_OUTPUT: &str = "agent_role_output";
const AGENT_ROLE_OUTPUT_AUTHORIZATION_HASH_DOMAIN: &str = "animus.agent-role-output-authorization.v1";
const REDACTION_MARKER: &str = "raw-role-output-omitted";

/// Role output status, including explicit non-success paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleOutputStatus {
    /// Bounded output was admitted.
    Admitted,
    /// Output was denied by policy or authority.
    Denied,
    /// Required service or evidence was unavailable.
    Unavailable,
    /// Role operation timed out.
    TimedOut,
    /// Role operation was cancelled.
    Cancelled,
    /// Output shape was malformed.
    Malformed,
    /// Evidence or candidates conflicted.
    Conflict,
    /// Role explicitly abstained.
    Abstained,
}

impl AgentRoleOutputStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::Malformed => "malformed",
            Self::Conflict => "conflict",
            Self::Abstained => "abstained",
        }
    }
}

/// Typed role output effect and claim boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleOutputEffect {
    /// Implementation candidate requiring separate mutation and completion gates.
    Candidate,
    /// Advisory review finding.
    Finding,
    /// Warning that can only tighten downstream behavior.
    Warn,
    /// Blocker that can only tighten downstream behavior.
    Block,
    /// Explicit tighten-only recommendation.
    Tighten,
    /// Supplied named-check evidence for a pinned subject.
    CheckEvidence,
    /// Rubric-bound ranking without authority grant.
    Rank,
    /// Narrow host-authorized candidate selection.
    Select,
    /// Narrow host-authorized veto.
    Veto,
    /// Rubric-bound tie.
    Tie,
    /// Explicit abstention.
    Abstain,
    /// Coordination metadata or bounded handoff request.
    Coordinate,
    /// Read-only observation.
    Observe,
}

impl AgentRoleOutputEffect {
    /// Returns the stable effect label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Finding => "finding",
            Self::Warn => "warn",
            Self::Block => "block",
            Self::Tighten => "tighten",
            Self::CheckEvidence => "check_evidence",
            Self::Rank => "rank",
            Self::Select => "select",
            Self::Veto => "veto",
            Self::Tie => "tie",
            Self::Abstain => "abstain",
            Self::Coordinate => "coordinate",
            Self::Observe => "observe",
        }
    }
}

/// Independent admission state supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleAdmissionState {
    /// The resource was not independently admitted.
    NotAdmitted,
    /// The resource was independently admitted.
    Admitted,
}

impl AgentRoleAdmissionState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NotAdmitted => "not_admitted",
            Self::Admitted => "admitted",
        }
    }

    const fn is_admitted(self) -> bool {
        matches!(self, Self::Admitted)
    }
}

/// Deterministic policy state for the pinned subject or candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleDeterministicPolicyState {
    /// No deterministic denial was supplied.
    Clear,
    /// A deterministic denial exists and cannot be weakened.
    Denied,
}

impl AgentRoleDeterministicPolicyState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Denied => "denied",
        }
    }
}

/// Independent output-admission facts bound into the authorization resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleOutputAdmission {
    /// Deterministic policy result.
    pub deterministic_policy: AgentRoleDeterministicPolicyState,
    /// Supplied host evidence admission.
    pub host_evidence: AgentRoleAdmissionState,
    /// Named verifier/check admission.
    pub verifier: AgentRoleAdmissionState,
    /// Typed rubric admission.
    pub rubric: AgentRoleAdmissionState,
}

impl Default for AgentRoleOutputAdmission {
    fn default() -> Self {
        Self {
            deterministic_policy: AgentRoleDeterministicPolicyState::Clear,
            host_evidence: AgentRoleAdmissionState::NotAdmitted,
            verifier: AgentRoleAdmissionState::NotAdmitted,
            rubric: AgentRoleAdmissionState::NotAdmitted,
        }
    }
}

/// Forbidden claim classes that role output must never carry as authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleOutputClaim {
    /// Attempted capability or authority grant.
    AuthorityGrant,
    /// Attempted mutation of the pinned subject or candidate.
    SubjectMutation,
    /// Attempted authoritative completion claim.
    AuthoritativeCompletion,
    /// Claimed verifier soundness.
    VerifierSoundness,
    /// Claimed independence beyond checked dimensions.
    UnboundedIndependence,
}

/// Forbidden raw material classes that must never enter a durable role output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleOutputLeak<'a> {
    /// Raw prompt or role instructions.
    RawPromptOrInstructions(&'a str),
    /// Hidden reasoning.
    HiddenReasoning(&'a str),
    /// Provider request or response payload.
    ProviderPayload(&'a str),
    /// Credential or token.
    Credential(&'a str),
    /// Private or full path.
    PrivatePath(&'a str),
    /// Raw verifier/check body.
    VerifierBody(&'a str),
    /// Hidden test or oracle body.
    HiddenTestOrOracle(&'a str),
    /// Raw diff or file body.
    RawDiffOrFile(&'a str),
    /// Tool arguments or output.
    ToolBody(&'a str),
    /// Judge rationale body.
    JudgeRationale(&'a str),
    /// Unbounded transcript.
    Transcript(&'a str),
}

/// Exact role output metadata bound into scoped output authorization.
#[derive(Clone, Copy)]
pub struct AgentRoleOutputResourceInput<'a> {
    /// Exact admitted assignment.
    pub assignment: &'a AgentRoleAssignment,
    /// Exact semantic role profile.
    pub profile: &'a AgentRoleProfile,
    /// Exact immutable role plan.
    pub role_plan: &'a AgentRolePlan,
    /// Exact capability intersection.
    pub capability_resolution: &'a AgentRoleCapabilityResolution,
    /// Exact separation decision.
    pub separation: &'a AgentRoleSeparationDecision,
    /// Output status.
    pub status: AgentRoleOutputStatus,
    /// Typed output effect.
    pub effect: AgentRoleOutputEffect,
    /// Safe evidence summary identity.
    pub evidence_hash: &'a str,
    /// Verifier/check identity when relevant.
    pub verifier_hash: Option<&'a str>,
    /// Rubric identity when relevant.
    pub rubric_hash: Option<&'a str>,
    /// Independent policy, evidence, verifier, and rubric admission facts.
    pub admission: AgentRoleOutputAdmission,
}

/// Computes the exact role output resource requiring separate authorization.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for malformed evidence identities or cross-plan,
/// cross-assignment, cross-subject, or failed-separation inputs.
pub fn agent_role_output_resource(input: AgentRoleOutputResourceInput<'_>) -> Result<String, AgentRoleError> {
    validate_output_resource_input(&input)?;
    let binding_hash = hash_parts(AGENT_ROLE_OUTPUT_AUTHORIZATION_HASH_DOMAIN, &[
        input.assignment.assignment_hash().to_string(),
        input.assignment.subject_hash().to_string(),
        input.profile.profile_hash().to_string(),
        input.role_plan.plan_hash().to_string(),
        input.assignment.model_plan_hash().to_string(),
        input.capability_resolution.decision_hash().to_string(),
        input.assignment.policy_hash().to_string(),
        input.separation.decision_hash().to_string(),
        input.assignment.output_contract_hash().to_string(),
        input.status.as_str().to_string(),
        input.effect.as_str().to_string(),
        input.evidence_hash.to_string(),
        input.verifier_hash.unwrap_or("none").to_string(),
        input.rubric_hash.unwrap_or("none").to_string(),
        input.admission.deterministic_policy.as_str().to_string(),
        input.admission.host_evidence.as_str().to_string(),
        input.admission.verifier.as_str().to_string(),
        input.admission.rubric.as_str().to_string(),
    ]);
    debug_assert!(!binding_hash.is_empty());
    Ok(format!("role:{}:{}:{}", input.profile.role_id(), input.assignment.subject_hash(), binding_hash))
}

/// Explicit role output admission input.
#[derive(Clone, Copy)]
pub struct AgentRoleOutputDraft<'a> {
    /// Exact admitted assignment.
    pub assignment: &'a AgentRoleAssignment,
    /// Exact semantic role profile.
    pub profile: &'a AgentRoleProfile,
    /// Exact immutable role plan.
    pub role_plan: &'a AgentRolePlan,
    /// Exact capability intersection.
    pub capability_resolution: &'a AgentRoleCapabilityResolution,
    /// Exact separation decision.
    pub separation: &'a AgentRoleSeparationDecision,
    /// Output status.
    pub status: AgentRoleOutputStatus,
    /// Typed output effect.
    pub effect: AgentRoleOutputEffect,
    /// Safe evidence summary identity.
    pub evidence_hash: &'a str,
    /// Verifier/check identity when required.
    pub verifier_hash: Option<&'a str>,
    /// Rubric or validated composite-rubric identity when required.
    pub rubric_hash: Option<&'a str>,
    /// Independent policy, evidence, verifier, and rubric admission facts.
    pub admission: AgentRoleOutputAdmission,
    /// Forbidden overclaim classes; every non-empty value fails closed.
    pub claims: &'a [AgentRoleOutputClaim],
    /// Separate output capability request.
    pub output_request: &'a animus_adapters::CapabilityRequest,
    /// Authorization receipt for exactly that output request.
    pub output_receipt: &'a animus_adapters::AuthorizationReceipt,
    /// Forbidden raw material candidates; every non-empty value fails closed.
    pub sensitive: &'a [AgentRoleOutputLeak<'a>],
}

/// Admitted metadata-only role output envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleOutput {
    assignment_hash: String,
    subject_hash: String,
    role_profile_hash: String,
    role_plan_hash: String,
    model_plan_hash: String,
    capability_decision_hash: String,
    policy_hash: String,
    separation_hash: String,
    output_contract_hash: String,
    output_kind: AgentRoleOutputKind,
    status: AgentRoleOutputStatus,
    effect: AgentRoleOutputEffect,
    evidence_hash: String,
    verifier_hash: Option<String>,
    rubric_hash: Option<String>,
    redaction_marker: &'static str,
    output_hash: String,
}

impl AgentRoleOutput {
    /// Returns assignment identity.
    #[must_use]
    pub fn assignment_hash(&self) -> &str {
        self.assignment_hash.as_str()
    }

    /// Returns pinned subject identity.
    #[must_use]
    pub fn subject_hash(&self) -> &str {
        self.subject_hash.as_str()
    }

    /// Returns role output kind.
    #[must_use]
    pub const fn output_kind(&self) -> AgentRoleOutputKind {
        self.output_kind
    }

    /// Returns output status.
    #[must_use]
    pub const fn status(&self) -> AgentRoleOutputStatus {
        self.status
    }

    /// Returns bounded output effect.
    #[must_use]
    pub const fn effect(&self) -> AgentRoleOutputEffect {
        self.effect
    }

    /// Returns metadata-only redaction marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns output identity.
    #[must_use]
    pub fn output_hash(&self) -> &str {
        self.output_hash.as_str()
    }
}

/// Admits one role output without converting it into authority or completion.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for stale bindings, class-incompatible effects,
/// missing output authority, deterministic-denial override, mutation, overclaim,
/// or forbidden raw material.
// r[impl agent_harness_runtime.agent_role_profiles.outputs]
pub fn admit_agent_role_output(draft: AgentRoleOutputDraft<'_>) -> Result<AgentRoleOutput, AgentRoleError> {
    validate_output_bindings(&draft)?;
    validate_output_class(&draft)?;
    validate_output_claims(&draft)?;
    validate_output_sensitive(draft.sensitive)?;
    validate_output_authority(&draft)?;
    let output_hash = hash_parts(AGENT_ROLE_OUTPUT_HASH_DOMAIN, &[
        draft.assignment.assignment_hash().to_string(),
        draft.assignment.subject_hash().to_string(),
        draft.profile.profile_hash().to_string(),
        draft.role_plan.plan_hash().to_string(),
        draft.assignment.model_plan_hash().to_string(),
        draft.capability_resolution.decision_hash().to_string(),
        draft.assignment.policy_hash().to_string(),
        draft.separation.decision_hash().to_string(),
        draft.assignment.output_contract_hash().to_string(),
        draft.profile.output_kind().as_str().to_string(),
        draft.status.as_str().to_string(),
        draft.effect.as_str().to_string(),
        draft.evidence_hash.to_string(),
        draft.verifier_hash.unwrap_or("none").to_string(),
        draft.rubric_hash.unwrap_or("none").to_string(),
        REDACTION_MARKER.to_string(),
        draft.output_receipt.receipt_hash().to_string(),
    ]);
    debug_assert!(!output_hash.is_empty());
    debug_assert!(draft.claims.is_empty());
    Ok(AgentRoleOutput {
        assignment_hash: draft.assignment.assignment_hash().to_string(),
        subject_hash: draft.assignment.subject_hash().to_string(),
        role_profile_hash: draft.profile.profile_hash().to_string(),
        role_plan_hash: draft.role_plan.plan_hash().to_string(),
        model_plan_hash: draft.assignment.model_plan_hash().to_string(),
        capability_decision_hash: draft.capability_resolution.decision_hash().to_string(),
        policy_hash: draft.assignment.policy_hash().to_string(),
        separation_hash: draft.separation.decision_hash().to_string(),
        output_contract_hash: draft.assignment.output_contract_hash().to_string(),
        output_kind: draft.profile.output_kind(),
        status: draft.status,
        effect: draft.effect,
        evidence_hash: draft.evidence_hash.to_string(),
        verifier_hash: draft.verifier_hash.map(str::to_string),
        rubric_hash: draft.rubric_hash.map(str::to_string),
        redaction_marker: REDACTION_MARKER,
        output_hash,
    })
}

fn validate_output_bindings(draft: &AgentRoleOutputDraft<'_>) -> Result<(), AgentRoleError> {
    agent_role_output_resource(output_resource_input(draft)).map(|_| ())
}

fn output_resource_input<'a>(draft: &AgentRoleOutputDraft<'a>) -> AgentRoleOutputResourceInput<'a> {
    AgentRoleOutputResourceInput {
        assignment: draft.assignment,
        profile: draft.profile,
        role_plan: draft.role_plan,
        capability_resolution: draft.capability_resolution,
        separation: draft.separation,
        status: draft.status,
        effect: draft.effect,
        evidence_hash: draft.evidence_hash,
        verifier_hash: draft.verifier_hash,
        rubric_hash: draft.rubric_hash,
        admission: draft.admission,
    }
}

fn validate_output_resource_input(input: &AgentRoleOutputResourceInput<'_>) -> Result<(), AgentRoleError> {
    lowercase_blake3(ValidationValue {
        field: "evidence_hash",
        value: input.evidence_hash,
    })?;
    for hash in [input.verifier_hash, input.rubric_hash].into_iter().flatten() {
        lowercase_blake3(ValidationValue {
            field: FIELD_OUTPUT,
            value: hash,
        })?;
    }
    let is_exact = input.assignment.role_profile_hash() == input.profile.profile_hash()
        && input.assignment.assignment_hash() == input.role_plan.assignment_hash()
        && input.assignment.assignment_hash() == input.capability_resolution.assignment_hash()
        && input.assignment.separation_hash() == input.separation.decision_hash()
        && input.assignment.subject_hash() == input.separation.subject_hash();
    if !is_exact || !input.separation.passed() {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_OUTPUT,
            "role output inputs belong to different assignments or separation decisions",
        ));
    }
    debug_assert!(is_exact);
    debug_assert!(input.separation.passed());
    Ok(())
}

fn validate_output_class(draft: &AgentRoleOutputDraft<'_>) -> Result<(), AgentRoleError> {
    let is_effect_allowed = match draft.profile.class() {
        AgentRoleClass::Implementation => draft.effect == AgentRoleOutputEffect::Candidate,
        AgentRoleClass::Review => matches!(
            draft.effect,
            AgentRoleOutputEffect::Finding
                | AgentRoleOutputEffect::Warn
                | AgentRoleOutputEffect::Block
                | AgentRoleOutputEffect::Tighten
                | AgentRoleOutputEffect::Abstain
        ),
        AgentRoleClass::Verification => {
            matches!(draft.effect, AgentRoleOutputEffect::CheckEvidence | AgentRoleOutputEffect::Abstain)
        }
        AgentRoleClass::Judgment => matches!(
            draft.effect,
            AgentRoleOutputEffect::Rank
                | AgentRoleOutputEffect::Select
                | AgentRoleOutputEffect::Veto
                | AgentRoleOutputEffect::Tie
                | AgentRoleOutputEffect::Abstain
        ),
        AgentRoleClass::Coordination => draft.effect == AgentRoleOutputEffect::Coordinate,
        AgentRoleClass::Observation => {
            matches!(draft.effect, AgentRoleOutputEffect::Observe | AgentRoleOutputEffect::Abstain)
        }
    };
    if !is_effect_allowed {
        return Err(error(
            AgentRoleIssue::OutputDenied,
            FIELD_OUTPUT,
            "role output effect contradicts its semantic class",
        ));
    }
    let is_check_evidence =
        draft.profile.class() == AgentRoleClass::Verification && draft.effect == AgentRoleOutputEffect::CheckEvidence;
    if is_check_evidence
        && (draft.verifier_hash.is_none()
            || !draft.admission.host_evidence.is_admitted()
            || !draft.admission.verifier.is_admitted())
    {
        return Err(error(
            AgentRoleIssue::MissingReference,
            FIELD_OUTPUT,
            "verification evidence requires an admitted named verifier and independently supplied host evidence",
        ));
    }
    let is_judgment =
        draft.profile.class() == AgentRoleClass::Judgment && draft.effect != AgentRoleOutputEffect::Abstain;
    if is_judgment && (draft.rubric_hash.is_none() || !draft.admission.rubric.is_admitted()) {
        return Err(error(
            AgentRoleIssue::MissingReference,
            FIELD_OUTPUT,
            "judgment output requires one independently admitted typed rubric identity",
        ));
    }
    debug_assert!(is_effect_allowed);
    debug_assert!(!is_judgment || draft.rubric_hash.is_some());
    Ok(())
}

fn validate_output_claims(draft: &AgentRoleOutputDraft<'_>) -> Result<(), AgentRoleError> {
    if !draft.claims.is_empty() {
        return Err(error(
            AgentRoleIssue::OutputDenied,
            FIELD_OUTPUT,
            "role output attempted authority, mutation, completion, soundness, or independence overclaim",
        ));
    }
    if draft.admission.deterministic_policy == AgentRoleDeterministicPolicyState::Denied
        && matches!(draft.effect, AgentRoleOutputEffect::Select | AgentRoleOutputEffect::Rank)
    {
        return Err(error(
            AgentRoleIssue::OutputDenied,
            FIELD_OUTPUT,
            "judgment cannot override or suppress a deterministic denial",
        ));
    }
    let is_status_abstained = draft.status == AgentRoleOutputStatus::Abstained;
    let is_effect_abstain = draft.effect == AgentRoleOutputEffect::Abstain;
    if is_status_abstained != is_effect_abstain {
        return Err(error(
            AgentRoleIssue::OutputDenied,
            FIELD_OUTPUT,
            "abstention status and effect must be explicit and consistent",
        ));
    }
    debug_assert!(draft.claims.is_empty());
    debug_assert_eq!(is_status_abstained, is_effect_abstain);
    Ok(())
}

fn validate_output_sensitive(sensitive: &[AgentRoleOutputLeak<'_>]) -> Result<(), AgentRoleError> {
    if !sensitive.is_empty() {
        return Err(error(
            AgentRoleIssue::ReceiptLeak,
            FIELD_OUTPUT,
            "role output contains forbidden raw or hidden material",
        ));
    }
    Ok(())
}

fn validate_output_authority(draft: &AgentRoleOutputDraft<'_>) -> Result<(), AgentRoleError> {
    let expected_resource = agent_role_output_resource(output_resource_input(draft))?;
    let expected_ability = output_ability(draft.profile.class(), draft.effect);
    if !role_authorization_is_exact(draft.output_request, draft.output_receipt, RoleAuthorizationExpectation {
        resource: expected_resource.as_str(),
        ability: expected_ability,
    }) {
        return Err(error(
            AgentRoleIssue::AuthorityDenied,
            FIELD_OUTPUT,
            "role output requires its own exact scoped capability authorization",
        ));
    }
    Ok(())
}

const fn output_ability(class: AgentRoleClass, effect: AgentRoleOutputEffect) -> &'static str {
    match (class, effect) {
        (AgentRoleClass::Implementation, _) => AGENT_ROLE_EMIT_CANDIDATE_ABILITY,
        (AgentRoleClass::Review, _) => AGENT_ROLE_EMIT_REVIEW_ABILITY,
        (AgentRoleClass::Verification, _) => AGENT_ROLE_EMIT_VERIFICATION_ABILITY,
        (AgentRoleClass::Judgment, AgentRoleOutputEffect::Select) => AGENT_ROLE_SELECT_ABILITY,
        (AgentRoleClass::Judgment, AgentRoleOutputEffect::Veto) => AGENT_ROLE_VETO_ABILITY,
        (AgentRoleClass::Judgment, _) => AGENT_ROLE_EMIT_JUDGMENT_ABILITY,
        (AgentRoleClass::Coordination, _) => AGENT_ROLE_EMIT_COORDINATION_ABILITY,
        (AgentRoleClass::Observation, _) => AGENT_ROLE_EMIT_OBSERVATION_ABILITY,
    }
}
