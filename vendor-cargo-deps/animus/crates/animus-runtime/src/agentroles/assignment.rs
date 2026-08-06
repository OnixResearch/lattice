//! Pure role-bound model composition, assignment, capability, plan, and transition logic.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::separation::AgentRoleSeparationDecision;
use super::types::AGENT_ROLE_ASSIGNMENT_HASH_DOMAIN;
use super::types::AGENT_ROLE_CAPABILITY_HASH_DOMAIN;
use super::types::AGENT_ROLE_HANDOFF_HASH_DOMAIN;
use super::types::AGENT_ROLE_MODEL_PLAN_HASH_DOMAIN;
use super::types::AGENT_ROLE_PLAN_HASH_DOMAIN;
use super::types::AgentRoleError;
use super::types::AgentRoleIssue;
use super::types::AgentRoleProfile;
use super::validation::ValidationValue;
use super::validation::error;
use super::validation::hash_parts;
use super::validation::lowercase_blake3;
use super::validation::safe_label;

/// Capability contract used for semantic role requests.
pub const AGENT_ROLE_CAPABILITY_CONTRACT: &str = "animus.runtime";
/// Scoped role assignment ability.
pub const AGENT_ROLE_ASSIGN_ABILITY: &str = "assign";
/// Scoped role activation ability.
pub const AGENT_ROLE_ACTIVATE_ABILITY: &str = "activate";
/// Scoped role handoff ability.
pub const AGENT_ROLE_HANDOFF_ABILITY: &str = "handoff";

const FIELD_MODEL_PLAN: &str = "agent_role_model_plan";
const FIELD_ASSIGNMENT: &str = "agent_role_assignment";
const FIELD_CAPABILITY: &str = "agent_role_capability";
const FIELD_TRANSITION: &str = "agent_role_transition";
const FIELD_HANDOFF: &str = "agent_role_handoff";
const FIELD_TOOL_ACTION: &str = "agent_role_tool_action";
const AGENT_ROLE_ACTIVATION_HASH_DOMAIN: &str = "animus.agent-role-activation.v1";
const AGENT_ROLE_HANDOFF_AUTHORIZATION_HASH_DOMAIN: &str = "animus.agent-role-handoff-authorization.v1";
const AGENT_ROLE_TOOL_ACTION_HASH_DOMAIN: &str = "animus.agent-role-tool-action.v1";
const CAPABILITY_CEILING_COUNT: usize = 14;
const CAPABILITY_SET_COUNT: usize = CAPABILITY_CEILING_COUNT.saturating_add(1);
const MAX_HANDOFF_REFERENCES: usize = 64;
const REDACTION_MARKER: &str = "raw-role-context-omitted";

/// Model invocation plan after adding one semantic role as an independent ceiling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleBoundModelPlan {
    plan: crate::ModelInvocationPlan,
    role_profile_hash: String,
    binding_hash: String,
}

impl AgentRoleBoundModelPlan {
    /// Returns the existing per-model invocation plan.
    #[must_use]
    pub const fn plan(&self) -> &crate::ModelInvocationPlan {
        &self.plan
    }

    /// Returns the semantic role profile identity used as a ceiling.
    #[must_use]
    pub fn role_profile_hash(&self) -> &str {
        self.role_profile_hash.as_str()
    }

    /// Returns the role/model composition identity.
    #[must_use]
    pub fn binding_hash(&self) -> &str {
        self.binding_hash.as_str()
    }
}

/// Explicit role/model composition inputs and independently admitted non-model resources.
#[derive(Clone, Copy)]
pub struct AgentRoleModelCompositionInput<'a> {
    /// Selected semantic role profile.
    pub profile: &'a AgentRoleProfile,
    /// Existing per-model composition input.
    pub model: crate::ModelInvocationCompositionInput<'a>,
    /// Host-admitted output-contract labels.
    pub admitted_output_contracts: &'a [&'a str],
    /// Host-admitted named role-budget labels.
    pub admitted_budget_policies: &'a [&'a str],
    /// Host-admitted workspace scope labels.
    pub admitted_workspace_scopes: &'a [&'a str],
}

/// Composes one existing per-model invocation plan with a semantic role ceiling.
///
/// # Errors
///
/// Returns [`AgentRoleError`] when the selected model profile, target, policy,
/// prompts, skills, memory, tools, capabilities, executors, output contract,
/// workspace, or budget does not satisfy the role and stronger existing ceilings.
// r[impl agent_harness_runtime.agent_role_profiles.resource_composition]
pub fn compose_agent_role_model_plan(
    composition: AgentRoleModelCompositionInput<'_>,
) -> Result<AgentRoleBoundModelPlan, AgentRoleError> {
    validate_role_resource_admission(&composition)?;
    let profile = composition.profile;
    let input = composition.model;
    validate_role_model_binding(profile, &input)?;
    let model_targets = string_refs(profile.model_targets());
    let prompt_modes = string_refs(profile.prompt_modes());
    let prompt_sources = string_refs(profile.prompt_sources());
    let skills = string_refs(profile.skills());
    let memory_policy_hashes = string_refs(profile.memory_policy_hashes());
    let tools = string_refs(profile.expanded_tools());
    let tool_capabilities = string_refs(profile.tool_capabilities());
    let role_ceiling = crate::ModelResourceCeiling {
        ceiling_label: profile.role_id(),
        model_targets: model_targets.as_slice(),
        prompt_modes: prompt_modes.as_slice(),
        prompt_sources: prompt_sources.as_slice(),
        skills: skills.as_slice(),
        memory_policy_hashes: memory_policy_hashes.as_slice(),
        tools: tools.as_slice(),
        tool_capabilities: tool_capabilities.as_slice(),
    };
    let ceiling_count = input.authority.ceilings.len().saturating_add(1);
    let mut ceilings = Vec::with_capacity(ceiling_count);
    ceilings.extend_from_slice(input.authority.ceilings);
    ceilings.push(role_ceiling);
    let authority = crate::ModelInvocationAuthority {
        ceilings: ceilings.as_slice(),
        approved_tools: input.authority.approved_tools,
        policy_hash: input.authority.policy_hash,
    };
    let plan = crate::compose_model_invocation_plan(crate::ModelInvocationCompositionInput { authority, ..input })
        .map_err(|model_error| error(AgentRoleIssue::ResourceDenied, FIELD_MODEL_PLAN, model_error.detail()))?;
    let binding_hash = hash_parts(AGENT_ROLE_MODEL_PLAN_HASH_DOMAIN, &[
        profile.profile_hash().to_string(),
        plan.plan_hash().to_string(),
        plan.profile_hash().to_string(),
        plan.target().target_hash().to_string(),
        plan.prompt_plan_hash().to_string(),
        plan.skill_plan_hash().to_string(),
        plan.memory_plan_hash().to_string(),
        plan.tool_plan_hash().to_string(),
    ]);
    debug_assert_eq!(ceilings.len(), ceiling_count);
    debug_assert!(!binding_hash.is_empty());
    Ok(AgentRoleBoundModelPlan {
        plan,
        role_profile_hash: profile.profile_hash().to_string(),
        binding_hash,
    })
}

fn validate_role_model_binding(
    profile: &AgentRoleProfile,
    input: &crate::ModelInvocationCompositionInput<'_>,
) -> Result<(), AgentRoleError> {
    let is_model_profile_eligible = profile
        .eligible_model_profiles()
        .iter()
        .any(|profile_id| profile_id == input.resolved.profile().profile_id());
    if !is_model_profile_eligible {
        return Err(error(
            AgentRoleIssue::ResourceDenied,
            FIELD_MODEL_PLAN,
            "selected model runtime profile is not eligible for this semantic role",
        ));
    }
    if profile.policy_hash() != input.authority.policy_hash {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_MODEL_PLAN,
            "role policy identity differs from model composition policy",
        ));
    }
    debug_assert!(is_model_profile_eligible);
    debug_assert_eq!(profile.policy_hash(), input.authority.policy_hash);
    Ok(())
}

fn string_refs(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

fn validate_role_resource_admission(composition: &AgentRoleModelCompositionInput<'_>) -> Result<(), AgentRoleError> {
    for (field, values) in [
        ("admitted_output_contracts", composition.admitted_output_contracts),
        ("admitted_budget_policies", composition.admitted_budget_policies),
        ("admitted_workspace_scopes", composition.admitted_workspace_scopes),
    ] {
        if values.is_empty() {
            return Err(error(
                AgentRoleIssue::ResourceDenied,
                FIELD_MODEL_PLAN,
                format!("{field} cannot be empty or ambient"),
            ));
        }
        let mut unique = std::collections::BTreeSet::new();
        for value in values {
            let label = safe_label(ValidationValue {
                field: FIELD_MODEL_PLAN,
                value,
            })?;
            if !unique.insert(label) {
                return Err(error(
                    AgentRoleIssue::DuplicateIdentity,
                    FIELD_MODEL_PLAN,
                    format!("{field} contains a duplicate identity"),
                ));
            }
        }
    }
    let is_output_admitted = composition.admitted_output_contracts.contains(&composition.profile.output_contract());
    let is_budget_admitted = composition.admitted_budget_policies.contains(&composition.profile.budget_policy_ref());
    let is_workspace_scope_admitted = composition
        .profile
        .workspace_scopes()
        .iter()
        .all(|scope| composition.admitted_workspace_scopes.contains(&scope.as_str()));
    if !is_output_admitted || !is_budget_admitted || !is_workspace_scope_admitted {
        return Err(error(
            AgentRoleIssue::ResourceDenied,
            FIELD_MODEL_PLAN,
            "role output contract, named budget, or workspace scope is not independently admitted",
        ));
    }
    debug_assert!(is_output_admitted);
    debug_assert!(is_budget_admitted && is_workspace_scope_admitted);
    Ok(())
}

/// Exact identities bound into role assignment and activation authorization.
#[derive(Clone, Copy)]
pub struct AgentRoleActivationResourceInput<'a> {
    /// Selected semantic role profile.
    pub profile: &'a AgentRoleProfile,
    /// Role-bound model composition.
    pub model_plan: &'a AgentRoleBoundModelPlan,
    /// Explicit separation decision.
    pub separation: &'a AgentRoleSeparationDecision,
    /// Pinned subject identity.
    pub subject_hash: &'a str,
    /// Pinned task identity.
    pub task_hash: &'a str,
    /// Actor identity.
    pub actor_hash: &'a str,
    /// Parent assignment or delegation lineage identity.
    pub parent_lineage_hash: &'a str,
    /// Run or thread identity.
    pub run_thread_hash: &'a str,
    /// Session identity.
    pub session_hash: &'a str,
    /// Workspace identity.
    pub workspace_hash: &'a str,
    /// Mutation-lineage identity.
    pub mutation_lineage_hash: &'a str,
    /// Admitted agent-definition identity.
    pub agent_definition_hash: &'a str,
    /// Verified grant-set identity.
    pub grant_set_hash: &'a str,
    /// Freshness identity supplied by the host.
    pub freshness_hash: &'a str,
    /// Exact output-contract identity.
    pub output_contract_hash: &'a str,
}

/// Computes the exact scoped resource authorized for assignment and activation.
///
/// # Errors
///
/// Returns [`AgentRoleError`] when any bound identity is malformed or the role,
/// model plan, separation decision, policy, or subject bindings disagree.
pub fn agent_role_activation_resource(input: AgentRoleActivationResourceInput<'_>) -> Result<String, AgentRoleError> {
    validate_activation_resource_input(&input)?;
    let binding_hash = hash_parts(AGENT_ROLE_ACTIVATION_HASH_DOMAIN, &[
        input.profile.role_id().to_string(),
        input.profile.profile_hash().to_string(),
        input.model_plan.binding_hash().to_string(),
        input.separation.decision_hash().to_string(),
        input.subject_hash.to_string(),
        input.task_hash.to_string(),
        input.actor_hash.to_string(),
        input.parent_lineage_hash.to_string(),
        input.run_thread_hash.to_string(),
        input.session_hash.to_string(),
        input.workspace_hash.to_string(),
        input.mutation_lineage_hash.to_string(),
        input.agent_definition_hash.to_string(),
        input.profile.policy_hash().to_string(),
        input.grant_set_hash.to_string(),
        input.freshness_hash.to_string(),
        input.output_contract_hash.to_string(),
    ]);
    debug_assert!(!binding_hash.is_empty());
    Ok(format!("role:{}:{}:{}", input.profile.role_id(), input.subject_hash, binding_hash))
}

fn validate_activation_resource_input(input: &AgentRoleActivationResourceInput<'_>) -> Result<(), AgentRoleError> {
    for (field, value) in [
        ("subject_hash", input.subject_hash),
        ("task_hash", input.task_hash),
        ("actor_hash", input.actor_hash),
        ("parent_lineage_hash", input.parent_lineage_hash),
        ("run_thread_hash", input.run_thread_hash),
        ("session_hash", input.session_hash),
        ("workspace_hash", input.workspace_hash),
        ("mutation_lineage_hash", input.mutation_lineage_hash),
        ("agent_definition_hash", input.agent_definition_hash),
        ("grant_set_hash", input.grant_set_hash),
        ("freshness_hash", input.freshness_hash),
        ("output_contract_hash", input.output_contract_hash),
    ] {
        lowercase_blake3(ValidationValue { field, value })?;
    }
    let is_binding_exact = input.model_plan.role_profile_hash() == input.profile.profile_hash()
        && input.model_plan.plan().policy_hash() == input.profile.policy_hash()
        && input.separation.subject_hash() == input.subject_hash
        && input.separation.policy_id() == input.profile.separation_policy_ref()
        && input.separation.passed();
    if !is_binding_exact {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_ASSIGNMENT,
            "activation resource inputs do not share one role, subject, policy, model, and separation binding",
        ));
    }
    debug_assert!(is_binding_exact);
    debug_assert!(input.separation.passed());
    Ok(())
}

/// Explicit role-assignment admission input.
#[derive(Clone, Copy)]
pub struct AgentRoleAssignmentRequest<'a> {
    /// Selected semantic role profile.
    pub profile: &'a AgentRoleProfile,
    /// Role-bound model composition.
    pub model_plan: &'a AgentRoleBoundModelPlan,
    /// Explicit separation decision for the proposed assignment.
    pub separation: &'a AgentRoleSeparationDecision,
    /// Pinned subject identity.
    pub subject_hash: &'a str,
    /// Pinned task identity.
    pub task_hash: &'a str,
    /// Actor identity.
    pub actor_hash: &'a str,
    /// Parent assignment or delegation lineage identity.
    pub parent_lineage_hash: &'a str,
    /// Run or thread identity.
    pub run_thread_hash: &'a str,
    /// Session identity.
    pub session_hash: &'a str,
    /// Workspace identity.
    pub workspace_hash: &'a str,
    /// Mutation-lineage identity.
    pub mutation_lineage_hash: &'a str,
    /// Admitted agent-definition identity.
    pub agent_definition_hash: &'a str,
    /// Verified grant-set identity.
    pub grant_set_hash: &'a str,
    /// Freshness identity supplied by the host.
    pub freshness_hash: &'a str,
    /// Exact output-contract identity.
    pub output_contract_hash: &'a str,
    /// Scoped role-assignment request.
    pub assignment_request: &'a animus_adapters::CapabilityRequest,
    /// Authorization receipt for exactly that assignment request.
    pub assignment_receipt: &'a animus_adapters::AuthorizationReceipt,
    /// Scoped role-activation request.
    pub activation_request: &'a animus_adapters::CapabilityRequest,
    /// Authorization receipt for exactly that activation request.
    pub activation_receipt: &'a animus_adapters::AuthorizationReceipt,
}

/// Admitted immutable semantic role assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleAssignment {
    role_id: String,
    role_class: super::AgentRoleClass,
    role_profile_hash: String,
    subject_hash: String,
    task_hash: String,
    actor_hash: String,
    parent_lineage_hash: String,
    run_thread_hash: String,
    session_hash: String,
    workspace_hash: String,
    mutation_lineage_hash: String,
    agent_definition_hash: String,
    model_plan_hash: String,
    policy_hash: String,
    grant_set_hash: String,
    separation_hash: String,
    freshness_hash: String,
    output_contract_hash: String,
    assignment_receipt_hash: String,
    activation_receipt_hash: String,
    assignment_hash: String,
}

impl AgentRoleAssignment {
    /// Returns the semantic role ID.
    #[must_use]
    pub fn role_id(&self) -> &str {
        self.role_id.as_str()
    }

    /// Returns the semantic role class.
    #[must_use]
    pub const fn role_class(&self) -> super::AgentRoleClass {
        self.role_class
    }

    /// Returns the role profile identity.
    #[must_use]
    pub fn role_profile_hash(&self) -> &str {
        self.role_profile_hash.as_str()
    }

    /// Returns the pinned subject identity.
    #[must_use]
    pub fn subject_hash(&self) -> &str {
        self.subject_hash.as_str()
    }

    /// Returns the actor identity.
    #[must_use]
    pub fn actor_hash(&self) -> &str {
        self.actor_hash.as_str()
    }

    /// Returns parent lineage identity.
    #[must_use]
    pub fn parent_lineage_hash(&self) -> &str {
        self.parent_lineage_hash.as_str()
    }

    /// Returns model-plan identity.
    #[must_use]
    pub fn model_plan_hash(&self) -> &str {
        self.model_plan_hash.as_str()
    }

    /// Returns active policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns verified grant-set identity.
    #[must_use]
    pub fn grant_set_hash(&self) -> &str {
        self.grant_set_hash.as_str()
    }

    /// Returns separation-decision identity.
    #[must_use]
    pub fn separation_hash(&self) -> &str {
        self.separation_hash.as_str()
    }

    /// Returns output-contract identity.
    #[must_use]
    pub fn output_contract_hash(&self) -> &str {
        self.output_contract_hash.as_str()
    }

    /// Returns immutable assignment identity.
    #[must_use]
    pub fn assignment_hash(&self) -> &str {
        self.assignment_hash.as_str()
    }
}

/// Admits an assignment only from an exact scoped activation authorization.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for missing or mismatched authority, stale model,
/// subject, policy, lineage, separation, freshness, or output bindings.
// r[impl agent_harness_runtime.agent_role_profiles.activation]
pub fn admit_agent_role_assignment(
    request: AgentRoleAssignmentRequest<'_>,
) -> Result<AgentRoleAssignment, AgentRoleError> {
    validate_assignment_hashes(&request)?;
    validate_assignment_authority(&request)?;
    validate_assignment_bindings(&request)?;
    let assignment_hash = hash_parts(AGENT_ROLE_ASSIGNMENT_HASH_DOMAIN, &[
        request.profile.role_id().to_string(),
        request.profile.profile_hash().to_string(),
        request.subject_hash.to_string(),
        request.task_hash.to_string(),
        request.actor_hash.to_string(),
        request.parent_lineage_hash.to_string(),
        request.run_thread_hash.to_string(),
        request.session_hash.to_string(),
        request.workspace_hash.to_string(),
        request.mutation_lineage_hash.to_string(),
        request.agent_definition_hash.to_string(),
        request.model_plan.binding_hash().to_string(),
        request.profile.policy_hash().to_string(),
        request.grant_set_hash.to_string(),
        request.separation.decision_hash().to_string(),
        request.freshness_hash.to_string(),
        request.output_contract_hash.to_string(),
        request.assignment_receipt.receipt_hash().to_string(),
        request.activation_receipt.receipt_hash().to_string(),
    ]);
    debug_assert!(request.activation_receipt.is_allowed());
    debug_assert!(!assignment_hash.is_empty());
    Ok(AgentRoleAssignment {
        role_id: request.profile.role_id().to_string(),
        role_class: request.profile.class(),
        role_profile_hash: request.profile.profile_hash().to_string(),
        subject_hash: request.subject_hash.to_string(),
        task_hash: request.task_hash.to_string(),
        actor_hash: request.actor_hash.to_string(),
        parent_lineage_hash: request.parent_lineage_hash.to_string(),
        run_thread_hash: request.run_thread_hash.to_string(),
        session_hash: request.session_hash.to_string(),
        workspace_hash: request.workspace_hash.to_string(),
        mutation_lineage_hash: request.mutation_lineage_hash.to_string(),
        agent_definition_hash: request.agent_definition_hash.to_string(),
        model_plan_hash: request.model_plan.binding_hash().to_string(),
        policy_hash: request.profile.policy_hash().to_string(),
        grant_set_hash: request.grant_set_hash.to_string(),
        separation_hash: request.separation.decision_hash().to_string(),
        freshness_hash: request.freshness_hash.to_string(),
        output_contract_hash: request.output_contract_hash.to_string(),
        assignment_receipt_hash: request.assignment_receipt.receipt_hash().to_string(),
        activation_receipt_hash: request.activation_receipt.receipt_hash().to_string(),
        assignment_hash,
    })
}

fn validate_assignment_bindings(request: &AgentRoleAssignmentRequest<'_>) -> Result<(), AgentRoleError> {
    let is_model_binding_exact = [
        request.model_plan.role_profile_hash() == request.profile.profile_hash(),
        request.model_plan.plan().policy_hash() == request.profile.policy_hash(),
    ]
    .into_iter()
    .all(|is_exact| is_exact);
    if !is_model_binding_exact {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_ASSIGNMENT,
            "role profile or policy differs from the role-bound model plan",
        ));
    }
    let is_separation_binding_exact = [
        request.separation.passed(),
        request.separation.subject_hash() == request.subject_hash,
        request.separation.policy_id() == request.profile.separation_policy_ref(),
    ]
    .into_iter()
    .all(|is_exact| is_exact);
    if !is_separation_binding_exact {
        return Err(error(
            AgentRoleIssue::SeparationDenied,
            FIELD_ASSIGNMENT,
            "required separation policy did not pass for this exact subject",
        ));
    }
    debug_assert!(is_model_binding_exact);
    debug_assert!(is_separation_binding_exact);
    Ok(())
}

fn validate_assignment_hashes(request: &AgentRoleAssignmentRequest<'_>) -> Result<(), AgentRoleError> {
    for (field, value) in [
        ("subject_hash", request.subject_hash),
        ("task_hash", request.task_hash),
        ("actor_hash", request.actor_hash),
        ("parent_lineage_hash", request.parent_lineage_hash),
        ("run_thread_hash", request.run_thread_hash),
        ("session_hash", request.session_hash),
        ("workspace_hash", request.workspace_hash),
        ("mutation_lineage_hash", request.mutation_lineage_hash),
        ("agent_definition_hash", request.agent_definition_hash),
        ("grant_set_hash", request.grant_set_hash),
        ("freshness_hash", request.freshness_hash),
        ("output_contract_hash", request.output_contract_hash),
    ] {
        lowercase_blake3(ValidationValue { field, value })?;
    }
    Ok(())
}

fn validate_assignment_authority(request: &AgentRoleAssignmentRequest<'_>) -> Result<(), AgentRoleError> {
    let resource_input = AgentRoleActivationResourceInput {
        profile: request.profile,
        model_plan: request.model_plan,
        separation: request.separation,
        subject_hash: request.subject_hash,
        task_hash: request.task_hash,
        actor_hash: request.actor_hash,
        parent_lineage_hash: request.parent_lineage_hash,
        run_thread_hash: request.run_thread_hash,
        session_hash: request.session_hash,
        workspace_hash: request.workspace_hash,
        mutation_lineage_hash: request.mutation_lineage_hash,
        agent_definition_hash: request.agent_definition_hash,
        grant_set_hash: request.grant_set_hash,
        freshness_hash: request.freshness_hash,
        output_contract_hash: request.output_contract_hash,
    };
    let expected_resource = agent_role_activation_resource(resource_input)?;
    let is_assignment_authorization_exact = role_authorization_is_exact(
        request.assignment_request,
        request.assignment_receipt,
        RoleAuthorizationExpectation {
            resource: expected_resource.as_str(),
            ability: AGENT_ROLE_ASSIGN_ABILITY,
        },
    );
    let is_activation_authorization_exact = role_authorization_is_exact(
        request.activation_request,
        request.activation_receipt,
        RoleAuthorizationExpectation {
            resource: expected_resource.as_str(),
            ability: AGENT_ROLE_ACTIVATE_ABILITY,
        },
    );
    if !is_assignment_authorization_exact || !is_activation_authorization_exact {
        return Err(error(
            AgentRoleIssue::AuthorityDenied,
            FIELD_ASSIGNMENT,
            "assignment requires exact allowed role-assignment and role-activation requests and receipts",
        ));
    }
    debug_assert!(is_assignment_authorization_exact);
    debug_assert!(is_activation_authorization_exact);
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) struct RoleAuthorizationExpectation<'a> {
    pub resource: &'a str,
    pub ability: &'a str,
}

pub(super) fn role_authorization_is_exact(
    request: &animus_adapters::CapabilityRequest,
    receipt: &animus_adapters::AuthorizationReceipt,
    expectation: RoleAuthorizationExpectation<'_>,
) -> bool {
    let expected_resource_hash = blake3::hash(expectation.resource.as_bytes()).to_hex().to_string();
    let is_request_exact = request.action() == animus_adapters::GuardedAction::AgentRole
        && request.contract() == AGENT_ROLE_CAPABILITY_CONTRACT
        && request.resource() == expectation.resource
        && request.ability() == expectation.ability;
    let is_receipt_exact = receipt.is_allowed()
        && receipt.action() == request.action()
        && receipt.contract() == request.contract()
        && receipt.ability() == request.ability()
        && receipt.resource_hash() == expected_resource_hash;
    is_request_exact && is_receipt_exact
}

/// Independent capability ceiling source for a semantic role assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleCapabilitySource {
    /// Verified grant and Nickel contract.
    VerifiedGrant,
    /// Host or session policy and caveats.
    HostPolicy,
    /// Parent delegation ceiling.
    ParentDelegation,
    /// Task or recipe scope.
    TaskScope,
    /// Admitted agent-definition ceiling.
    AgentDefinition,
    /// Semantic role profile ceiling.
    SemanticRole,
    /// Admitted per-model runtime profile ceiling.
    ModelProfile,
    /// Workspace and context scope.
    Workspace,
    /// Tool catalog and executor admission.
    ToolExecutor,
    /// Explicit approval state.
    Approval,
    /// Sandbox admission.
    Sandbox,
    /// Cancellation state.
    Cancellation,
    /// Timeout state.
    Timeout,
    /// Named finite budget admission.
    Budget,
}

impl AgentRoleCapabilitySource {
    /// Returns the stable source label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VerifiedGrant => "verified_grant",
            Self::HostPolicy => "host_policy",
            Self::ParentDelegation => "parent_delegation",
            Self::TaskScope => "task_scope",
            Self::AgentDefinition => "agent_definition",
            Self::SemanticRole => "semantic_role",
            Self::ModelProfile => "model_profile",
            Self::Workspace => "workspace",
            Self::ToolExecutor => "tool_executor",
            Self::Approval => "approval",
            Self::Sandbox => "sandbox",
            Self::Cancellation => "cancellation",
            Self::Timeout => "timeout",
            Self::Budget => "budget",
        }
    }
}

/// Borrowed independent capability ceiling sets.
#[derive(Clone, Copy)]
pub struct AgentRoleCapabilityCeilings<'a> {
    /// Verified grant and contract capabilities.
    pub verified_grants: &'a [crate::TypedSubagentCapability],
    /// Host/session policy capabilities.
    pub host_policy: &'a [crate::TypedSubagentCapability],
    /// Parent delegation capabilities.
    pub parent_delegation: &'a [crate::TypedSubagentCapability],
    /// Task/recipe capabilities.
    pub task_scope: &'a [crate::TypedSubagentCapability],
    /// Agent-definition capabilities.
    pub agent_definition: &'a [crate::TypedSubagentCapability],
    /// Model-profile capabilities.
    pub model_profile: &'a [crate::TypedSubagentCapability],
    /// Workspace/context capabilities.
    pub workspace: &'a [crate::TypedSubagentCapability],
    /// Tool catalog/executor capabilities.
    pub tool_executor: &'a [crate::TypedSubagentCapability],
    /// Explicitly approved capabilities.
    pub approval: &'a [crate::TypedSubagentCapability],
    /// Sandbox-admitted capabilities.
    pub sandbox: &'a [crate::TypedSubagentCapability],
    /// Capabilities still admitted by cancellation state.
    pub cancellation: &'a [crate::TypedSubagentCapability],
    /// Capabilities still admitted by timeout state.
    pub timeout: &'a [crate::TypedSubagentCapability],
    /// Capabilities admitted by named finite budgets.
    pub budget: &'a [crate::TypedSubagentCapability],
}

/// Pure exact-intersection input for one assignment.
#[derive(Clone, Copy)]
pub struct AgentRoleCapabilityResolutionInput<'a> {
    /// Exact admitted assignment.
    pub assignment: &'a AgentRoleAssignment,
    /// Exact role profile used by the assignment.
    pub profile: &'a AgentRoleProfile,
    /// Requested capabilities for this assignment only.
    pub requested: &'a [crate::TypedSubagentCapability],
    /// Typed capabilities represented by the profile ceiling.
    pub role_ceiling: &'a [crate::TypedSubagentCapability],
    /// Every stronger independent ceiling.
    pub ceilings: AgentRoleCapabilityCeilings<'a>,
}

/// One capability denial and its exact ceiling source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleCapabilityDenial {
    capability: crate::TypedSubagentCapability,
    source: AgentRoleCapabilitySource,
}

impl AgentRoleCapabilityDenial {
    /// Returns the denied typed capability.
    #[must_use]
    pub const fn capability(&self) -> &crate::TypedSubagentCapability {
        &self.capability
    }

    /// Returns the exact denying ceiling source.
    #[must_use]
    pub const fn source(&self) -> AgentRoleCapabilitySource {
        self.source
    }
}

/// Exact capability intersection for one immutable assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleCapabilityResolution {
    assignment_hash: String,
    effective: Vec<crate::TypedSubagentCapability>,
    denied: Vec<AgentRoleCapabilityDenial>,
    decision_hash: String,
}

impl AgentRoleCapabilityResolution {
    /// Returns assignment identity.
    #[must_use]
    pub fn assignment_hash(&self) -> &str {
        self.assignment_hash.as_str()
    }

    /// Returns capabilities admitted by every ceiling.
    #[must_use]
    pub fn effective(&self) -> &[crate::TypedSubagentCapability] {
        self.effective.as_slice()
    }

    /// Returns per-source denials.
    #[must_use]
    pub fn denied(&self) -> &[AgentRoleCapabilityDenial] {
        self.denied.as_slice()
    }

    /// Returns capability decision identity.
    #[must_use]
    pub fn decision_hash(&self) -> &str {
        self.decision_hash.as_str()
    }
}

/// Resolves role capabilities as an exact intersection; required labels never grant authority.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for stale assignment/profile bindings, malformed
/// capability sets, role-label spoofing, or a missing required capability.
// r[impl agent_harness_runtime.agent_role_profiles.capability_intersection]
pub fn resolve_agent_role_capabilities(
    input: AgentRoleCapabilityResolutionInput<'_>,
) -> Result<AgentRoleCapabilityResolution, AgentRoleError> {
    validate_capability_input(&input)?;
    let denial_slots = input.requested.len().saturating_mul(CAPABILITY_CEILING_COUNT);
    let mut effective = Vec::with_capacity(input.requested.len());
    let mut denied = Vec::with_capacity(denial_slots);
    for capability in input.requested {
        let missing = missing_capability_sources(&input, capability);
        if missing.is_empty() {
            effective.push(capability.clone());
        } else {
            denied.extend(missing.into_iter().map(|source| AgentRoleCapabilityDenial {
                capability: capability.clone(),
                source,
            }));
        }
    }
    for required in input.profile.required_capabilities() {
        if !effective.iter().any(|capability| capability.stable_label() == required) {
            return Err(error(
                AgentRoleIssue::MissingRequiredCapability,
                FIELD_CAPABILITY,
                "required role capability is absent from the final exact intersection",
            ));
        }
    }
    let mut hash_material = Vec::with_capacity(input.requested.len().saturating_add(1));
    hash_material.push(input.assignment.assignment_hash().to_string());
    hash_material.extend(
        effective
            .iter()
            .map(|capability| crate::subagent_capability_receipt_label(capability).capability_hash),
    );
    hash_material.extend(denied.iter().map(|entry| {
        format!(
            "{}:{}",
            entry.source.as_str(),
            crate::subagent_capability_receipt_label(&entry.capability).capability_hash
        )
    }));
    let decision_hash = hash_parts(AGENT_ROLE_CAPABILITY_HASH_DOMAIN, hash_material.as_slice());
    debug_assert!(effective.len() <= input.requested.len());
    debug_assert!(denied.len() <= denial_slots);
    Ok(AgentRoleCapabilityResolution {
        assignment_hash: input.assignment.assignment_hash().to_string(),
        effective,
        denied,
        decision_hash,
    })
}

fn validate_capability_input(input: &AgentRoleCapabilityResolutionInput<'_>) -> Result<(), AgentRoleError> {
    if input.assignment.role_profile_hash() != input.profile.profile_hash() {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_CAPABILITY,
            "capability resolution profile differs from assignment",
        ));
    }
    for capabilities in all_capability_sets(input) {
        ensure_unique_capabilities(capabilities)?;
    }
    let mut role_labels =
        input.role_ceiling.iter().map(crate::TypedSubagentCapability::stable_label).collect::<Vec<_>>();
    role_labels.sort_unstable();
    let mut profile_labels = input.profile.capability_ceiling().iter().map(String::as_str).collect::<Vec<_>>();
    profile_labels.sort_unstable();
    if role_labels != profile_labels {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_CAPABILITY,
            "typed semantic role ceiling does not exactly match the role profile",
        ));
    }
    debug_assert_eq!(role_labels, profile_labels);
    debug_assert_eq!(input.assignment.role_profile_hash(), input.profile.profile_hash());
    Ok(())
}

fn all_capability_sets<'a>(
    input: &'a AgentRoleCapabilityResolutionInput<'a>,
) -> [&'a [crate::TypedSubagentCapability]; CAPABILITY_SET_COUNT] {
    [
        input.requested,
        input.role_ceiling,
        input.ceilings.verified_grants,
        input.ceilings.host_policy,
        input.ceilings.parent_delegation,
        input.ceilings.task_scope,
        input.ceilings.agent_definition,
        input.ceilings.model_profile,
        input.ceilings.workspace,
        input.ceilings.tool_executor,
        input.ceilings.approval,
        input.ceilings.sandbox,
        input.ceilings.cancellation,
        input.ceilings.timeout,
        input.ceilings.budget,
    ]
}

fn ensure_unique_capabilities(capabilities: &[crate::TypedSubagentCapability]) -> Result<(), AgentRoleError> {
    if capabilities.is_empty() {
        return Err(error(
            AgentRoleIssue::InvalidCollection,
            FIELD_CAPABILITY,
            "capability ceiling cannot be empty or ambient",
        ));
    }
    let mut labels = std::collections::BTreeSet::new();
    for capability in capabilities {
        let label = safe_label(ValidationValue {
            field: FIELD_CAPABILITY,
            value: capability.stable_label(),
        })?;
        if label == "all" || label == "wildcard" || !labels.insert(label) {
            return Err(error(
                AgentRoleIssue::DuplicateIdentity,
                FIELD_CAPABILITY,
                "capability labels must be exact, unique, and non-wildcard",
            ));
        }
    }
    debug_assert_eq!(labels.len(), capabilities.len());
    debug_assert!(!labels.contains("all") && !labels.contains("wildcard"));
    Ok(())
}

fn missing_capability_sources(
    input: &AgentRoleCapabilityResolutionInput<'_>,
    capability: &crate::TypedSubagentCapability,
) -> Vec<AgentRoleCapabilitySource> {
    let candidates = [
        (AgentRoleCapabilitySource::VerifiedGrant, input.ceilings.verified_grants),
        (AgentRoleCapabilitySource::HostPolicy, input.ceilings.host_policy),
        (AgentRoleCapabilitySource::ParentDelegation, input.ceilings.parent_delegation),
        (AgentRoleCapabilitySource::TaskScope, input.ceilings.task_scope),
        (AgentRoleCapabilitySource::AgentDefinition, input.ceilings.agent_definition),
        (AgentRoleCapabilitySource::SemanticRole, input.role_ceiling),
        (AgentRoleCapabilitySource::ModelProfile, input.ceilings.model_profile),
        (AgentRoleCapabilitySource::Workspace, input.ceilings.workspace),
        (AgentRoleCapabilitySource::ToolExecutor, input.ceilings.tool_executor),
        (AgentRoleCapabilitySource::Approval, input.ceilings.approval),
        (AgentRoleCapabilitySource::Sandbox, input.ceilings.sandbox),
        (AgentRoleCapabilitySource::Cancellation, input.ceilings.cancellation),
        (AgentRoleCapabilitySource::Timeout, input.ceilings.timeout),
        (AgentRoleCapabilitySource::Budget, input.ceilings.budget),
    ];
    let missing = candidates
        .iter()
        .filter_map(|(source, ceiling)| (!ceiling.contains(capability)).then_some(*source))
        .collect::<Vec<_>>();
    debug_assert!(missing.len() <= CAPABILITY_CEILING_COUNT);
    debug_assert!(missing.iter().all(|source| candidates.iter().any(|(candidate, _)| candidate == source)));
    missing
}

/// Admits one capability only under the resolution's exact assignment identity.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for a cross-assignment or denied capability use.
pub fn admit_agent_role_capability_use(
    assignment: &AgentRoleAssignment,
    resolution: &AgentRoleCapabilityResolution,
    capability: &crate::TypedSubagentCapability,
) -> Result<(), AgentRoleError> {
    if assignment.assignment_hash() != resolution.assignment_hash() {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_CAPABILITY,
            "capability decision belongs to another assignment",
        ));
    }
    if !resolution.effective().contains(capability) {
        return Err(error(
            AgentRoleIssue::CapabilityDenied,
            FIELD_CAPABILITY,
            "capability was not admitted by every assignment ceiling",
        ));
    }
    Ok(())
}

/// Immutable role plan binding assignment, model, capabilities, separation, and output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRolePlan {
    assignment_hash: String,
    role_profile_hash: String,
    subject_hash: String,
    actor_hash: String,
    parent_lineage_hash: String,
    model_plan_hash: String,
    prompt_plan_hash: String,
    skill_plan_hash: String,
    memory_plan_hash: String,
    tool_plan_hash: String,
    capability_decision_hash: String,
    policy_hash: String,
    grant_set_hash: String,
    separation_hash: String,
    budget_policy_ref: String,
    output_contract_hash: String,
    plan_hash: String,
}

impl AgentRolePlan {
    /// Returns assignment identity.
    #[must_use]
    pub fn assignment_hash(&self) -> &str {
        self.assignment_hash.as_str()
    }

    /// Returns semantic role profile identity.
    #[must_use]
    pub fn role_profile_hash(&self) -> &str {
        self.role_profile_hash.as_str()
    }

    /// Returns subject identity.
    #[must_use]
    pub fn subject_hash(&self) -> &str {
        self.subject_hash.as_str()
    }

    /// Returns model role-binding identity.
    #[must_use]
    pub fn model_plan_hash(&self) -> &str {
        self.model_plan_hash.as_str()
    }

    /// Returns capability decision identity.
    #[must_use]
    pub fn capability_decision_hash(&self) -> &str {
        self.capability_decision_hash.as_str()
    }

    /// Returns immutable role-plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }
}

/// Exact admitted inputs for one immutable semantic role plan.
#[derive(Clone, Copy)]
pub struct AgentRolePlanInput<'a> {
    /// Exact admitted assignment.
    pub assignment: &'a AgentRoleAssignment,
    /// Exact semantic role profile.
    pub profile: &'a AgentRoleProfile,
    /// Exact role-bound model plan.
    pub model_plan: &'a AgentRoleBoundModelPlan,
    /// Exact capability intersection.
    pub capabilities: &'a AgentRoleCapabilityResolution,
    /// Exact separation decision.
    pub separation: &'a AgentRoleSeparationDecision,
}

/// Builds one immutable role plan after every independent admission succeeds.
///
/// # Errors
///
/// Returns [`AgentRoleError`] when assignment, model, capability, separation, or
/// output identities do not match exactly.
pub fn build_agent_role_plan(input: AgentRolePlanInput<'_>) -> Result<AgentRolePlan, AgentRoleError> {
    let AgentRolePlanInput {
        assignment,
        profile,
        model_plan,
        capabilities,
        separation,
    } = input;
    let is_binding_exact = [
        assignment.role_profile_hash() == profile.profile_hash(),
        assignment.model_plan_hash() == model_plan.binding_hash(),
        assignment.assignment_hash() == capabilities.assignment_hash(),
        assignment.separation_hash() == separation.decision_hash(),
        assignment.subject_hash() == separation.subject_hash(),
    ]
    .into_iter()
    .all(|is_exact| is_exact);
    if !is_binding_exact {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_ASSIGNMENT,
            "role-plan inputs belong to different immutable assignments",
        ));
    }
    let plan = model_plan.plan();
    let hash_material = [
        assignment.assignment_hash().to_string(),
        profile.profile_hash().to_string(),
        assignment.subject_hash().to_string(),
        assignment.actor_hash().to_string(),
        assignment.parent_lineage_hash().to_string(),
        model_plan.binding_hash().to_string(),
        plan.prompt_plan_hash().to_string(),
        plan.skill_plan_hash().to_string(),
        plan.memory_plan_hash().to_string(),
        plan.tool_plan_hash().to_string(),
        capabilities.decision_hash().to_string(),
        assignment.policy_hash().to_string(),
        assignment.grant_set_hash().to_string(),
        separation.decision_hash().to_string(),
        profile.budget_policy_ref().to_string(),
        assignment.output_contract_hash().to_string(),
    ];
    let plan_hash = hash_parts(AGENT_ROLE_PLAN_HASH_DOMAIN, &hash_material);
    debug_assert_eq!(assignment.role_profile_hash(), profile.profile_hash());
    debug_assert!(!plan_hash.is_empty());
    Ok(AgentRolePlan {
        assignment_hash: assignment.assignment_hash().to_string(),
        role_profile_hash: profile.profile_hash().to_string(),
        subject_hash: assignment.subject_hash().to_string(),
        actor_hash: assignment.actor_hash().to_string(),
        parent_lineage_hash: assignment.parent_lineage_hash().to_string(),
        model_plan_hash: model_plan.binding_hash().to_string(),
        prompt_plan_hash: plan.prompt_plan_hash().to_string(),
        skill_plan_hash: plan.skill_plan_hash().to_string(),
        memory_plan_hash: plan.memory_plan_hash().to_string(),
        tool_plan_hash: plan.tool_plan_hash().to_string(),
        capability_decision_hash: capabilities.decision_hash().to_string(),
        policy_hash: assignment.policy_hash().to_string(),
        grant_set_hash: assignment.grant_set_hash().to_string(),
        separation_hash: separation.decision_hash().to_string(),
        budget_policy_ref: profile.budget_policy_ref().to_string(),
        output_contract_hash: assignment.output_contract_hash().to_string(),
        plan_hash,
    })
}

/// Returned native tool call after binding it to one immutable role plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleToolActionBinding {
    tool_name: String,
    role_plan_hash: String,
    binding_hash: String,
}

impl AgentRoleToolActionBinding {
    /// Returns exact native tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns immutable role-plan identity.
    #[must_use]
    pub fn role_plan_hash(&self) -> &str {
        self.role_plan_hash.as_str()
    }

    /// Returns role-bound tool-action identity.
    #[must_use]
    pub fn binding_hash(&self) -> &str {
        self.binding_hash.as_str()
    }
}

/// Exact admitted inputs for one returned native tool action.
#[derive(Clone, Copy)]
pub struct AgentRoleToolActionInput<'a> {
    /// Exact admitted assignment.
    pub assignment: &'a AgentRoleAssignment,
    /// Exact immutable role plan.
    pub role_plan: &'a AgentRolePlan,
    /// Exact role-bound model plan.
    pub model_plan: &'a AgentRoleBoundModelPlan,
    /// Exact capability intersection.
    pub capabilities: &'a AgentRoleCapabilityResolution,
    /// Exact typed tool capability.
    pub capability: &'a crate::TypedSubagentCapability,
    /// Already validated native model tool call.
    pub model_call: &'a crate::ModelToolCallBinding,
}

/// Binds an already validated native model tool call to exact role authority.
///
/// Existing model tool-call and executor-kit validation remain independently
/// required; this function only adds assignment, role-plan, and capability binding.
///
/// # Errors
///
/// Returns [`AgentRoleError`] when any assignment, role/model plan, capability
/// decision, tool capability, or returned model call belongs to another plan.
pub fn bind_agent_role_tool_action(
    input: AgentRoleToolActionInput<'_>,
) -> Result<AgentRoleToolActionBinding, AgentRoleError> {
    let AgentRoleToolActionInput {
        assignment,
        role_plan,
        model_plan,
        capabilities,
        capability,
        model_call,
    } = input;
    let is_plan_binding_exact = role_plan.assignment_hash() == assignment.assignment_hash()
        && role_plan.model_plan_hash() == model_plan.binding_hash()
        && role_plan.capability_decision_hash() == capabilities.decision_hash()
        && model_call.invocation_plan_hash() == model_plan.plan().plan_hash();
    if !is_plan_binding_exact {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_TOOL_ACTION,
            "returned native tool call belongs to another assignment, role, model, or capability plan",
        ));
    }
    admit_agent_role_capability_use(assignment, capabilities, capability)?;
    let is_tool_capability_exact = matches!(
        capability,
        crate::TypedSubagentCapability::ToolUse { tool_label, .. }
            if tool_label == model_call.tool_name()
    );
    if !is_tool_capability_exact {
        return Err(error(
            AgentRoleIssue::CapabilityDenied,
            FIELD_TOOL_ACTION,
            "effective role capability does not authorize the returned native tool name",
        ));
    }
    let binding_hash = hash_parts(AGENT_ROLE_TOOL_ACTION_HASH_DOMAIN, &[
        assignment.assignment_hash().to_string(),
        role_plan.plan_hash().to_string(),
        model_plan.binding_hash().to_string(),
        model_call.invocation_plan_hash().to_string(),
        model_call.tool_name().to_string(),
        capabilities.decision_hash().to_string(),
    ]);
    debug_assert!(!binding_hash.is_empty());
    Ok(AgentRoleToolActionBinding {
        tool_name: model_call.tool_name().to_string(),
        role_plan_hash: role_plan.plan_hash().to_string(),
        binding_hash,
    })
}

/// Immutable role-plan identity snapshot supplied at transition boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRolePlanSnapshot<'a> {
    /// Assignment identity.
    pub assignment_hash: &'a str,
    /// Semantic role profile identity.
    pub role_profile_hash: &'a str,
    /// Subject identity.
    pub subject_hash: &'a str,
    /// Actor identity.
    pub actor_hash: &'a str,
    /// Parent lineage identity.
    pub parent_lineage_hash: &'a str,
    /// Role-bound model-plan identity.
    pub model_plan_hash: &'a str,
    /// Prompt-plan identity.
    pub prompt_plan_hash: &'a str,
    /// Skill-plan identity.
    pub skill_plan_hash: &'a str,
    /// Memory-plan identity.
    pub memory_plan_hash: &'a str,
    /// Tool-plan identity.
    pub tool_plan_hash: &'a str,
    /// Capability-decision identity.
    pub capability_decision_hash: &'a str,
    /// Policy identity.
    pub policy_hash: &'a str,
    /// Grant-set identity.
    pub grant_set_hash: &'a str,
    /// Separation-decision identity.
    pub separation_hash: &'a str,
    /// Budget-policy reference.
    pub budget_policy_ref: &'a str,
    /// Output-contract identity.
    pub output_contract_hash: &'a str,
    /// Full role-plan identity.
    pub plan_hash: &'a str,
}

impl<'a> From<&'a AgentRolePlan> for AgentRolePlanSnapshot<'a> {
    fn from(plan: &'a AgentRolePlan) -> Self {
        Self {
            assignment_hash: plan.assignment_hash.as_str(),
            role_profile_hash: plan.role_profile_hash.as_str(),
            subject_hash: plan.subject_hash.as_str(),
            actor_hash: plan.actor_hash.as_str(),
            parent_lineage_hash: plan.parent_lineage_hash.as_str(),
            model_plan_hash: plan.model_plan_hash.as_str(),
            prompt_plan_hash: plan.prompt_plan_hash.as_str(),
            skill_plan_hash: plan.skill_plan_hash.as_str(),
            memory_plan_hash: plan.memory_plan_hash.as_str(),
            tool_plan_hash: plan.tool_plan_hash.as_str(),
            capability_decision_hash: plan.capability_decision_hash.as_str(),
            policy_hash: plan.policy_hash.as_str(),
            grant_set_hash: plan.grant_set_hash.as_str(),
            separation_hash: plan.separation_hash.as_str(),
            budget_policy_ref: plan.budget_policy_ref.as_str(),
            output_contract_hash: plan.output_contract_hash.as_str(),
            plan_hash: plan.plan_hash.as_str(),
        }
    }
}

/// Role transition class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleTransitionKind {
    /// Change semantic role and recompose all role-specific state.
    RoleChange,
    /// Hand work to another assignment and recompose.
    Handoff,
    /// Keep role semantics but recompose for a fallback model.
    ModelFallback,
    /// Resume the exact immutable role plan.
    Resume,
    /// Replay the exact historical role plan without upgrading it.
    Replay,
    /// Cancel the current role plan.
    Cancel,
}

/// Transition request over one current role plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleTransitionRequest<'a> {
    /// Transition class.
    pub kind: AgentRoleTransitionKind,
    /// Current immutable role plan.
    pub current_plan: &'a AgentRolePlan,
    /// Explicit current identity snapshot.
    pub current_snapshot: AgentRolePlanSnapshot<'a>,
    /// Next role profile identity for role change or handoff.
    pub next_role_profile_hash: Option<&'a str>,
    /// Next model target identity for model fallback.
    pub next_model_target_hash: Option<&'a str>,
}

/// Safe transition decision before role, model, or tool effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentRoleTransitionDecision {
    /// Reuse the exact immutable plan.
    ReuseExact {
        /// Exact role-plan identity.
        plan_hash: String,
    },
    /// Drop role-specific state and compose a new semantic role.
    RecomposeRole {
        /// New semantic role profile identity.
        role_profile_hash: String,
    },
    /// Drop model-specific state and compose the role for another model target.
    RecomposeModel {
        /// New model target identity.
        target_hash: String,
    },
    /// Current plan is cancelled and cannot influence later work.
    Cancelled {
        /// Cancelled role-plan identity.
        plan_hash: String,
    },
}

/// Evaluates role transitions without carrying capability or context state.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for stale snapshots, missing next identities, or
/// attempts to silently reuse a plan across role or model boundaries.
// r[impl agent_harness_runtime.agent_role_profiles.transition_binding]
pub fn evaluate_agent_role_transition(
    request: &AgentRoleTransitionRequest<'_>,
) -> Result<AgentRoleTransitionDecision, AgentRoleError> {
    validate_transition_snapshot(request)?;
    debug_assert!(snapshot_matches_plan(&request.current_snapshot, request.current_plan));
    debug_assert!(!request.current_plan.plan_hash().is_empty());
    match request.kind {
        AgentRoleTransitionKind::Resume | AgentRoleTransitionKind::Replay => {
            if request.next_role_profile_hash.is_some() || request.next_model_target_hash.is_some() {
                return Err(error(
                    AgentRoleIssue::TransitionDenied,
                    FIELD_TRANSITION,
                    "resume or replay cannot silently change role or model identity",
                ));
            }
            Ok(AgentRoleTransitionDecision::ReuseExact {
                plan_hash: request.current_plan.plan_hash().to_string(),
            })
        }
        AgentRoleTransitionKind::RoleChange | AgentRoleTransitionKind::Handoff => {
            let next_role_profile_hash = request.next_role_profile_hash.ok_or_else(|| {
                error(
                    AgentRoleIssue::TransitionDenied,
                    FIELD_TRANSITION,
                    "role change or handoff requires a next role profile identity",
                )
            })?;
            lowercase_blake3(ValidationValue {
                field: "next_role_profile_hash",
                value: next_role_profile_hash,
            })?;
            if next_role_profile_hash == request.current_plan.role_profile_hash() {
                return Err(error(
                    AgentRoleIssue::TransitionDenied,
                    FIELD_TRANSITION,
                    "role change must identify a different immutable profile",
                ));
            }
            Ok(AgentRoleTransitionDecision::RecomposeRole {
                role_profile_hash: next_role_profile_hash.to_string(),
            })
        }
        AgentRoleTransitionKind::ModelFallback => {
            let target_hash = request.next_model_target_hash.ok_or_else(|| {
                error(
                    AgentRoleIssue::TransitionDenied,
                    FIELD_TRANSITION,
                    "model fallback requires a fresh target identity",
                )
            })?;
            lowercase_blake3(ValidationValue {
                field: "next_model_target_hash",
                value: target_hash,
            })?;
            Ok(AgentRoleTransitionDecision::RecomposeModel {
                target_hash: target_hash.to_string(),
            })
        }
        AgentRoleTransitionKind::Cancel => Ok(AgentRoleTransitionDecision::Cancelled {
            plan_hash: request.current_plan.plan_hash().to_string(),
        }),
    }
}

fn validate_transition_snapshot(request: &AgentRoleTransitionRequest<'_>) -> Result<(), AgentRoleError> {
    validate_role_snapshot(&request.current_snapshot)?;
    if !snapshot_matches_plan(&request.current_snapshot, request.current_plan) {
        return Err(error(
            AgentRoleIssue::TransitionDenied,
            FIELD_TRANSITION,
            "role transition snapshot is stale and requires explicit replanning",
        ));
    }
    debug_assert!(snapshot_matches_plan(&request.current_snapshot, request.current_plan));
    debug_assert_eq!(request.current_snapshot.plan_hash, request.current_plan.plan_hash());
    Ok(())
}

fn validate_role_snapshot(snapshot: &AgentRolePlanSnapshot<'_>) -> Result<(), AgentRoleError> {
    for (field, value) in [
        ("assignment_hash", snapshot.assignment_hash),
        ("role_profile_hash", snapshot.role_profile_hash),
        ("subject_hash", snapshot.subject_hash),
        ("actor_hash", snapshot.actor_hash),
        ("parent_lineage_hash", snapshot.parent_lineage_hash),
        ("model_plan_hash", snapshot.model_plan_hash),
        ("prompt_plan_hash", snapshot.prompt_plan_hash),
        ("skill_plan_hash", snapshot.skill_plan_hash),
        ("memory_plan_hash", snapshot.memory_plan_hash),
        ("tool_plan_hash", snapshot.tool_plan_hash),
        ("capability_decision_hash", snapshot.capability_decision_hash),
        ("policy_hash", snapshot.policy_hash),
        ("grant_set_hash", snapshot.grant_set_hash),
        ("separation_hash", snapshot.separation_hash),
        ("output_contract_hash", snapshot.output_contract_hash),
        ("plan_hash", snapshot.plan_hash),
    ] {
        lowercase_blake3(ValidationValue { field, value })?;
    }
    safe_label(ValidationValue {
        field: "budget_policy_ref",
        value: snapshot.budget_policy_ref,
    })?;
    debug_assert!(!snapshot.plan_hash.is_empty());
    debug_assert!(!snapshot.budget_policy_ref.is_empty());
    Ok(())
}

fn snapshot_matches_plan(snapshot: &AgentRolePlanSnapshot<'_>, plan: &AgentRolePlan) -> bool {
    snapshot.assignment_hash == plan.assignment_hash
        && snapshot.role_profile_hash == plan.role_profile_hash
        && snapshot.subject_hash == plan.subject_hash
        && snapshot.actor_hash == plan.actor_hash
        && snapshot.parent_lineage_hash == plan.parent_lineage_hash
        && snapshot.model_plan_hash == plan.model_plan_hash
        && snapshot.prompt_plan_hash == plan.prompt_plan_hash
        && snapshot.skill_plan_hash == plan.skill_plan_hash
        && snapshot.memory_plan_hash == plan.memory_plan_hash
        && snapshot.tool_plan_hash == plan.tool_plan_hash
        && snapshot.capability_decision_hash == plan.capability_decision_hash
        && snapshot.policy_hash == plan.policy_hash
        && snapshot.grant_set_hash == plan.grant_set_hash
        && snapshot.separation_hash == plan.separation_hash
        && snapshot.budget_policy_ref == plan.budget_policy_ref
        && snapshot.output_contract_hash == plan.output_contract_hash
        && snapshot.plan_hash == plan.plan_hash
}

/// Forbidden handoff material that must remain absent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AgentRoleHandoffSensitiveDraft<'a> {
    /// Raw transcript body.
    pub raw_transcript: Option<&'a str>,
    /// Hidden reasoning.
    pub hidden_reasoning: Option<&'a str>,
    /// Credential or token.
    pub credential: Option<&'a str>,
    /// Raw tool arguments or output.
    pub tool_body: Option<&'a str>,
    /// Inherited grant body.
    pub inherited_grant: Option<&'a str>,
    /// Pending tool call body.
    pub pending_tool_call: Option<&'a str>,
}

/// Exact metadata bound into scoped handoff authorization.
#[derive(Clone, Copy)]
pub struct AgentRoleHandoffResourceInput<'a> {
    /// Current immutable role plan.
    pub current_plan: &'a AgentRolePlan,
    /// Target role profile identity.
    pub target_role_profile_hash: &'a str,
    /// Safe bounded evidence references.
    pub evidence_refs: &'a [&'a str],
    /// Safe unresolved question or intent labels.
    pub unresolved_intents: &'a [&'a str],
    /// Safe requested next-action labels.
    pub requested_actions: &'a [&'a str],
}

/// Computes the exact scoped resource authorized for a metadata-only handoff.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for same-role handoff, malformed identities,
/// duplicate/unsafe labels, or over-bound metadata.
pub fn agent_role_handoff_resource(input: AgentRoleHandoffResourceInput<'_>) -> Result<String, AgentRoleError> {
    lowercase_blake3(ValidationValue {
        field: "target_role_profile_hash",
        value: input.target_role_profile_hash,
    })?;
    if input.target_role_profile_hash == input.current_plan.role_profile_hash() {
        return Err(error(
            AgentRoleIssue::TransitionDenied,
            FIELD_HANDOFF,
            "handoff target must use a distinct role profile identity",
        ));
    }
    let evidence_refs = validate_handoff_labels(input.evidence_refs)?;
    let unresolved_intents = validate_handoff_labels(input.unresolved_intents)?;
    let requested_actions = validate_handoff_labels(input.requested_actions)?;
    let mut material = vec![
        input.current_plan.plan_hash().to_string(),
        input.current_plan.assignment_hash().to_string(),
        input.current_plan.subject_hash().to_string(),
        input.target_role_profile_hash.to_string(),
    ];
    material.extend(evidence_refs.iter().map(|value| format!("evidence:{value}")));
    material.extend(unresolved_intents.iter().map(|value| format!("intent:{value}")));
    material.extend(requested_actions.iter().map(|value| format!("action:{value}")));
    let binding_hash = hash_parts(AGENT_ROLE_HANDOFF_AUTHORIZATION_HASH_DOMAIN, &material);
    debug_assert!(!binding_hash.is_empty());
    Ok(format!(
        "role:{}:{}:{}",
        input.current_plan.role_profile_hash(),
        input.current_plan.subject_hash(),
        binding_hash
    ))
}

/// Bounded role handoff draft.
#[derive(Clone, Copy)]
pub struct AgentRoleHandoffDraft<'a> {
    /// Current immutable role plan.
    pub current_plan: &'a AgentRolePlan,
    /// Target role profile identity.
    pub target_role_profile_hash: &'a str,
    /// Safe bounded evidence references.
    pub evidence_refs: &'a [&'a str],
    /// Safe unresolved question or intent labels.
    pub unresolved_intents: &'a [&'a str],
    /// Safe requested next-action labels.
    pub requested_actions: &'a [&'a str],
    /// Scoped handoff capability request.
    pub handoff_request: &'a animus_adapters::CapabilityRequest,
    /// Exact allowed receipt for the handoff request.
    pub handoff_receipt: &'a animus_adapters::AuthorizationReceipt,
    /// Forbidden raw or inherited material candidates.
    pub sensitive: AgentRoleHandoffSensitiveDraft<'a>,
}

/// Validated metadata-only role handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleHandoff {
    source_plan_hash: String,
    subject_hash: String,
    target_role_profile_hash: String,
    evidence_refs: Vec<String>,
    unresolved_intents: Vec<String>,
    requested_actions: Vec<String>,
    redaction_marker: &'static str,
    handoff_hash: String,
}

impl AgentRoleHandoff {
    /// Returns source role-plan identity.
    #[must_use]
    pub fn source_plan_hash(&self) -> &str {
        self.source_plan_hash.as_str()
    }

    /// Returns pinned subject identity.
    #[must_use]
    pub fn subject_hash(&self) -> &str {
        self.subject_hash.as_str()
    }

    /// Returns target role profile identity.
    #[must_use]
    pub fn target_role_profile_hash(&self) -> &str {
        self.target_role_profile_hash.as_str()
    }

    /// Returns bounded evidence references.
    #[must_use]
    pub fn evidence_refs(&self) -> &[String] {
        self.evidence_refs.as_slice()
    }

    /// Returns the metadata-only redaction marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns handoff identity.
    #[must_use]
    pub fn handoff_hash(&self) -> &str {
        self.handoff_hash.as_str()
    }
}

/// Validates a handoff as bounded evidence references without inherited authority.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for stale role identity, unbounded collections,
/// unsafe labels, or any raw transcript, reasoning, credential, grant, or call.
// r[impl agent_harness_runtime.agent_role_profiles.transition_binding.handoff]
pub fn validate_agent_role_handoff(draft: AgentRoleHandoffDraft<'_>) -> Result<AgentRoleHandoff, AgentRoleError> {
    let expected_resource = agent_role_handoff_resource(AgentRoleHandoffResourceInput {
        current_plan: draft.current_plan,
        target_role_profile_hash: draft.target_role_profile_hash,
        evidence_refs: draft.evidence_refs,
        unresolved_intents: draft.unresolved_intents,
        requested_actions: draft.requested_actions,
    })?;
    if !role_authorization_is_exact(draft.handoff_request, draft.handoff_receipt, RoleAuthorizationExpectation {
        resource: expected_resource.as_str(),
        ability: AGENT_ROLE_HANDOFF_ABILITY,
    }) {
        return Err(error(
            AgentRoleIssue::AuthorityDenied,
            FIELD_HANDOFF,
            "handoff requires its own exact allowed scoped capability receipt",
        ));
    }
    reject_handoff_sensitive(draft.sensitive)?;
    let evidence_refs = validate_handoff_labels(draft.evidence_refs)?;
    let unresolved_intents = validate_handoff_labels(draft.unresolved_intents)?;
    let requested_actions = validate_handoff_labels(draft.requested_actions)?;
    let mut hash_material = Vec::with_capacity(MAX_HANDOFF_REFERENCES);
    hash_material.extend([
        draft.current_plan.plan_hash().to_string(),
        draft.current_plan.subject_hash().to_string(),
        draft.target_role_profile_hash.to_string(),
        REDACTION_MARKER.to_string(),
        draft.handoff_receipt.receipt_hash().to_string(),
    ]);
    hash_material.extend(evidence_refs.iter().map(|value| format!("evidence:{value}")));
    hash_material.extend(unresolved_intents.iter().map(|value| format!("intent:{value}")));
    hash_material.extend(requested_actions.iter().map(|value| format!("action:{value}")));
    let handoff_hash = hash_parts(AGENT_ROLE_HANDOFF_HASH_DOMAIN, hash_material.as_slice());
    debug_assert!(!handoff_hash.is_empty());
    debug_assert!(evidence_refs.len() <= MAX_HANDOFF_REFERENCES);
    Ok(AgentRoleHandoff {
        source_plan_hash: draft.current_plan.plan_hash().to_string(),
        subject_hash: draft.current_plan.subject_hash().to_string(),
        target_role_profile_hash: draft.target_role_profile_hash.to_string(),
        evidence_refs,
        unresolved_intents,
        requested_actions,
        redaction_marker: REDACTION_MARKER,
        handoff_hash,
    })
}

fn reject_handoff_sensitive(sensitive: AgentRoleHandoffSensitiveDraft<'_>) -> Result<(), AgentRoleError> {
    let has_sensitive_material = [
        sensitive.raw_transcript,
        sensitive.hidden_reasoning,
        sensitive.credential,
        sensitive.tool_body,
        sensitive.inherited_grant,
        sensitive.pending_tool_call,
    ]
    .iter()
    .any(Option::is_some);
    if has_sensitive_material {
        return Err(error(
            AgentRoleIssue::ReceiptLeak,
            FIELD_HANDOFF,
            "handoff contains raw context or inherited authority",
        ));
    }
    debug_assert!(!has_sensitive_material);
    debug_assert!(sensitive.raw_transcript.is_none());
    Ok(())
}

fn validate_handoff_labels(values: &[&str]) -> Result<Vec<String>, AgentRoleError> {
    if values.len() > MAX_HANDOFF_REFERENCES {
        return Err(error(
            AgentRoleIssue::InvalidCollection,
            FIELD_HANDOFF,
            "handoff reference count exceeds its finite bound",
        ));
    }
    let mut labels = Vec::with_capacity(values.len());
    for value in values {
        let label = safe_label(ValidationValue {
            field: FIELD_HANDOFF,
            value,
        })?;
        if labels.contains(&label) {
            return Err(error(AgentRoleIssue::DuplicateIdentity, FIELD_HANDOFF, "handoff labels must be unique"));
        }
        labels.push(label);
    }
    debug_assert!(labels.len() <= MAX_HANDOFF_REFERENCES);
    debug_assert_eq!(labels.len(), values.len());
    Ok(labels)
}
