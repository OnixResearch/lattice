//! Pure role-catalog validation and canonical identity construction.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::types::AGENT_ROLE_CATALOG_SCHEMA;
use super::types::AGENT_ROLE_PROFILE_HASH_DOMAIN;
use super::types::AgentRoleBudgetPolicy;
use super::types::AgentRoleBudgetPolicyDraft;
use super::types::AgentRoleCatalog;
use super::types::AgentRoleCatalogDraft;
use super::types::AgentRoleClass;
use super::types::AgentRoleContextPosture;
use super::types::AgentRoleError;
use super::types::AgentRoleEvidencePosture;
use super::types::AgentRoleIssue;
use super::types::AgentRoleMutationPosture;
use super::types::AgentRoleOutputKind;
use super::types::AgentRoleProfile;
use super::types::AgentRoleProfileDraft;
use super::types::AgentRoleSeparationDimension;
use super::types::AgentRoleSeparationPolicy;
use super::types::AgentRoleSeparationPolicyDraft;
use super::types::AgentRoleToolset;
use super::types::AgentRoleToolsetDraft;
use super::types::BLAKE3_HEX_CHARS;
use super::types::HASH_SEPARATOR;
use super::types::MAX_ROLE_ACTION_BUDGET;
use super::types::MAX_ROLE_CONTEXT_BUDGET;
use super::types::MAX_ROLE_LABEL_CHARS;
use super::types::MAX_ROLE_OUTPUT_BUDGET;
use super::types::MAX_ROLE_POLICIES;
use super::types::MAX_ROLE_PROFILES;
use super::types::MAX_ROLE_PURPOSE_CHARS;
use super::types::MAX_ROLE_REFERENCES;

const FIELD_SCHEMA: &str = "agent_role_schema";
const FIELD_PROFILE: &str = "agent_role_profile";
const FIELD_PROFILE_HASH: &str = "agent_role_profile_hash";
const FIELD_CAPABILITY: &str = "agent_role_capability";
const FIELD_CONTRACT: &str = "agent_role_contract";
const FIELD_TOOLSET: &str = "agent_role_toolset";
const FIELD_BUDGET: &str = "agent_role_budget";
const FIELD_SEPARATION: &str = "agent_role_separation";
const FIELD_CLASS: &str = "agent_role_class";
const BASELINE_IMPLEMENTER: &str = "implementer";
const BASELINE_REVIEWER: &str = "reviewer";
const BASELINE_VERIFIER: &str = "verifier";
const BASELINE_JUDGE: &str = "judge";
const FORBIDDEN_PURPOSE_MARKERS: &[&str] = &[
    "RAW_PROMPT",
    "HIDDEN_REASONING",
    "PROVIDER_PAYLOAD",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "/home/",
];

impl AgentRoleCatalog {
    /// Validates an already loaded role catalog without filesystem or Nickel authority.
    ///
    /// # Errors
    ///
    /// Returns [`AgentRoleError`] for malformed, duplicate, contradictory, stale,
    /// untrusted, or dangling catalog data.
    // r[impl agent_harness_runtime.agent_role_profiles.contract]
    // r[impl agent_harness_runtime.agent_role_profiles.semantic_boundary]
    pub fn validate(draft: &AgentRoleCatalogDraft<'_>) -> Result<Self, AgentRoleError> {
        validate_catalog_shape(draft)?;
        let known_capabilities = unique_labels(FIELD_CAPABILITY, draft.known_capabilities)?;
        let input_contracts = unique_labels(FIELD_CONTRACT, draft.input_contracts)?;
        let output_contracts = unique_labels(FIELD_CONTRACT, draft.output_contracts)?;
        let evidence_contracts = unique_labels(FIELD_CONTRACT, draft.evidence_contracts)?;
        let budget_policies = validate_budgets(draft.budget_policies)?;
        let toolsets = validate_toolsets(draft.toolsets)?;
        let separation_policies = validate_separation_policies(draft.separation_policies)?;
        let profiles = validate_profiles(draft.profiles, ProfileReferenceContext {
            known_capabilities: &known_capabilities,
            input_contracts: &input_contracts,
            output_contracts: &output_contracts,
            evidence_contracts: &evidence_contracts,
            budgets: &budget_policies,
            toolsets: &toolsets,
            separation_policies: &separation_policies,
        })?;
        validate_baseline_profiles(&profiles)?;
        debug_assert_eq!(profiles.len(), draft.profiles.len());
        debug_assert!(!known_capabilities.is_empty());
        Ok(Self {
            known_capabilities,
            input_contracts,
            output_contracts,
            evidence_contracts,
            budget_policies,
            toolsets,
            separation_policies,
            profiles,
        })
    }
}

/// Computes canonical BLAKE3 identity from one structurally valid role draft.
///
/// # Errors
///
/// Returns [`AgentRoleError`] when profile fields are malformed.
pub fn blake3_agent_role_profile_identity(draft: &AgentRoleProfileDraft<'_>) -> Result<String, AgentRoleError> {
    validate_profile_shape(draft)?;
    Ok(profile_draft_hash(draft))
}

fn validate_catalog_shape(draft: &AgentRoleCatalogDraft<'_>) -> Result<(), AgentRoleError> {
    if draft.schema != AGENT_ROLE_CATALOG_SCHEMA {
        return Err(error(AgentRoleIssue::InvalidSchema, FIELD_SCHEMA, "unsupported agent-role catalog schema"));
    }
    validate_nonempty_bound(CollectionBound {
        item_count: draft.profiles.len(),
        maximum_count: MAX_ROLE_PROFILES,
        field: FIELD_PROFILE,
    })?;
    validate_nonempty_bound(CollectionBound {
        item_count: draft.known_capabilities.len(),
        maximum_count: MAX_ROLE_REFERENCES,
        field: FIELD_CAPABILITY,
    })?;
    validate_nonempty_bound(CollectionBound {
        item_count: draft.budget_policies.len(),
        maximum_count: MAX_ROLE_POLICIES,
        field: FIELD_BUDGET,
    })?;
    validate_nonempty_bound(CollectionBound {
        item_count: draft.separation_policies.len(),
        maximum_count: MAX_ROLE_POLICIES,
        field: FIELD_SEPARATION,
    })?;
    debug_assert_eq!(draft.schema, AGENT_ROLE_CATALOG_SCHEMA);
    debug_assert!(!draft.profiles.is_empty());
    Ok(())
}

#[derive(Clone, Copy)]
struct CollectionBound {
    item_count: usize,
    maximum_count: usize,
    field: &'static str,
}

fn validate_nonempty_bound(bound: CollectionBound) -> Result<(), AgentRoleError> {
    if bound.item_count == usize::default() || bound.item_count > bound.maximum_count {
        return Err(error(
            AgentRoleIssue::InvalidCollection,
            bound.field,
            "collection must be non-empty and within its finite bound",
        ));
    }
    Ok(())
}

fn validate_budgets(drafts: &[AgentRoleBudgetPolicyDraft<'_>]) -> Result<Vec<AgentRoleBudgetPolicy>, AgentRoleError> {
    let mut policies = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let policy_id = safe_label(ValidationValue {
            field: FIELD_BUDGET,
            value: draft.policy_id,
        })?;
        let is_action_budget_valid = draft.max_actions > u32::default() && draft.max_actions <= MAX_ROLE_ACTION_BUDGET;
        let is_output_budget_valid = draft.max_outputs > u16::default() && draft.max_outputs <= MAX_ROLE_OUTPUT_BUDGET;
        let is_context_budget_valid =
            draft.max_context_units > u32::default() && draft.max_context_units <= MAX_ROLE_CONTEXT_BUDGET;
        if !is_action_budget_valid || !is_output_budget_valid || !is_context_budget_valid {
            return Err(error(
                AgentRoleIssue::InvalidCollection,
                FIELD_BUDGET,
                "role budget values must be positive and finitely bounded",
            ));
        }
        if policies.iter().any(|policy: &AgentRoleBudgetPolicy| policy.policy_id == policy_id) {
            return Err(duplicate(FIELD_BUDGET));
        }
        policies.push(AgentRoleBudgetPolicy {
            policy_id,
            max_actions: draft.max_actions,
            max_outputs: draft.max_outputs,
            max_context_units: draft.max_context_units,
        });
    }
    debug_assert_eq!(policies.len(), drafts.len());
    debug_assert!(policies.iter().all(|policy| !policy.policy_id.is_empty()));
    Ok(policies)
}

fn validate_toolsets(drafts: &[AgentRoleToolsetDraft<'_>]) -> Result<Vec<AgentRoleToolset>, AgentRoleError> {
    if drafts.len() > MAX_ROLE_POLICIES {
        return Err(error(AgentRoleIssue::InvalidCollection, FIELD_TOOLSET, "toolset count exceeds its finite bound"));
    }
    let mut toolsets = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let toolset_id = safe_label(ValidationValue {
            field: FIELD_TOOLSET,
            value: draft.toolset_id,
        })?;
        validate_nonempty_bound(CollectionBound {
            item_count: draft.tool_names.len(),
            maximum_count: MAX_ROLE_REFERENCES,
            field: FIELD_TOOLSET,
        })?;
        let tool_names = unique_labels(FIELD_TOOLSET, draft.tool_names)?;
        if toolsets.iter().any(|toolset: &AgentRoleToolset| toolset.toolset_id == toolset_id) {
            return Err(duplicate(FIELD_TOOLSET));
        }
        toolsets.push(AgentRoleToolset { toolset_id, tool_names });
    }
    debug_assert_eq!(toolsets.len(), drafts.len());
    debug_assert!(toolsets.iter().all(|toolset| !toolset.toolset_id.is_empty()));
    Ok(toolsets)
}

fn validate_separation_policies(
    drafts: &[AgentRoleSeparationPolicyDraft<'_>],
) -> Result<Vec<AgentRoleSeparationPolicy>, AgentRoleError> {
    let mut policies = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let policy_id = safe_label(ValidationValue {
            field: FIELD_SEPARATION,
            value: draft.policy_id,
        })?;
        if draft.required_dimensions.is_empty() {
            return Err(error(
                AgentRoleIssue::ClassInvariant,
                FIELD_SEPARATION,
                "separation policy must check at least one explicit dimension",
            ));
        }
        let mut required_dimensions = draft.required_dimensions.to_vec();
        required_dimensions.sort_unstable();
        if required_dimensions.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(duplicate(FIELD_SEPARATION));
        }
        if policies.iter().any(|policy: &AgentRoleSeparationPolicy| policy.policy_id == policy_id) {
            return Err(duplicate(FIELD_SEPARATION));
        }
        policies.push(AgentRoleSeparationPolicy {
            policy_id,
            required_dimensions,
        });
    }
    debug_assert_eq!(policies.len(), drafts.len());
    debug_assert!(policies.iter().all(|policy| !policy.policy_id.is_empty()));
    Ok(policies)
}

#[derive(Clone, Copy)]
struct ProfileReferenceContext<'a> {
    known_capabilities: &'a [String],
    input_contracts: &'a [String],
    output_contracts: &'a [String],
    evidence_contracts: &'a [String],
    budgets: &'a [AgentRoleBudgetPolicy],
    toolsets: &'a [AgentRoleToolset],
    separation_policies: &'a [AgentRoleSeparationPolicy],
}

fn validate_profiles(
    drafts: &[AgentRoleProfileDraft<'_>],
    references: ProfileReferenceContext<'_>,
) -> Result<Vec<AgentRoleProfile>, AgentRoleError> {
    let mut profiles = Vec::with_capacity(drafts.len());
    for draft in drafts {
        validate_profile_shape(draft)?;
        validate_profile_references(draft, references)?;
        let expected_hash = profile_draft_hash(draft);
        lowercase_blake3(ValidationValue {
            field: FIELD_PROFILE_HASH,
            value: draft.profile_hash,
        })?;
        if expected_hash != draft.profile_hash {
            return Err(error(
                AgentRoleIssue::StaleIdentity,
                FIELD_PROFILE_HASH,
                format!("role profile content identity is stale; expected {expected_hash}"),
            ));
        }
        let profile = validated_profile(draft, references.toolsets)?;
        if profiles.iter().any(|candidate: &AgentRoleProfile| candidate.role_id == profile.role_id) {
            return Err(duplicate(FIELD_PROFILE));
        }
        profiles.push(profile);
    }
    validate_class_separation(&profiles, references.separation_policies)?;
    debug_assert_eq!(profiles.len(), drafts.len());
    debug_assert!(profiles.iter().all(|profile| !profile.role_id.is_empty()));
    Ok(profiles)
}

fn validate_profile_shape(draft: &AgentRoleProfileDraft<'_>) -> Result<(), AgentRoleError> {
    safe_label(ValidationValue {
        field: FIELD_PROFILE,
        value: draft.role_id,
    })?;
    safe_purpose(draft.purpose)?;
    lowercase_blake3(ValidationValue {
        field: "agent_role_policy_hash",
        value: draft.policy_hash,
    })?;
    for policy_hash in draft.memory_policy_hashes {
        lowercase_blake3(ValidationValue {
            field: "agent_role_memory_policy_hash",
            value: policy_hash,
        })?;
    }
    for count in [
        draft.required_capabilities.len(),
        draft.capability_ceiling.len(),
        draft.eligible_model_profiles.len(),
        draft.model_targets.len(),
        draft.prompt_modes.len(),
        draft.prompt_sources.len(),
        draft.skills.len(),
        draft.memory_policy_hashes.len(),
        draft.tools.len(),
        draft.toolsets.len(),
        draft.tool_capabilities.len(),
        draft.workspace_scopes.len(),
    ] {
        if count > MAX_ROLE_REFERENCES {
            return Err(error(
                AgentRoleIssue::InvalidCollection,
                FIELD_PROFILE,
                "role resource reference count exceeds its finite bound",
            ));
        }
    }
    validate_required_collections(draft)?;
    validate_class_invariants(draft)?;
    if draft.source_scope == crate::ModelProfileSourceScope::Project && !draft.project_profile_admitted {
        return Err(error(
            AgentRoleIssue::UntrustedProfile,
            FIELD_PROFILE,
            "project role profile lacks explicit host admission",
        ));
    }
    if draft.trust_status != crate::ModelProfileTrustStatus::Trusted
        || draft.admission_status != crate::ModelProfileAdmissionStatus::Admitted
    {
        return Err(error(
            AgentRoleIssue::UntrustedProfile,
            FIELD_PROFILE,
            "role profile must be trusted and admitted",
        ));
    }
    debug_assert!(!draft.role_id.is_empty());
    debug_assert!(!draft.policy_hash.is_empty());
    Ok(())
}

fn validate_required_collections(draft: &AgentRoleProfileDraft<'_>) -> Result<(), AgentRoleError> {
    for (field, values) in [
        (FIELD_CAPABILITY, draft.capability_ceiling),
        ("eligible_model_profiles", draft.eligible_model_profiles),
        ("model_targets", draft.model_targets),
        ("prompt_modes", draft.prompt_modes),
        ("prompt_sources", draft.prompt_sources),
        ("workspace_scopes", draft.workspace_scopes),
    ] {
        validate_nonempty_bound(CollectionBound {
            item_count: values.len(),
            maximum_count: MAX_ROLE_REFERENCES,
            field,
        })?;
        unique_labels(field, values)?;
    }
    for (field, values) in [
        (FIELD_CAPABILITY, draft.required_capabilities),
        ("skills", draft.skills),
        ("tools", draft.tools),
        (FIELD_TOOLSET, draft.toolsets),
        ("tool_capabilities", draft.tool_capabilities),
    ] {
        if values.len() > MAX_ROLE_REFERENCES {
            return Err(error(
                AgentRoleIssue::InvalidCollection,
                field,
                "role reference count exceeds its finite bound",
            ));
        }
        unique_optional_labels(field, values)?;
    }
    debug_assert!(!draft.capability_ceiling.is_empty());
    debug_assert!(!draft.eligible_model_profiles.is_empty());
    Ok(())
}

fn validate_class_invariants(draft: &AgentRoleProfileDraft<'_>) -> Result<(), AgentRoleError> {
    let is_valid = match draft.class {
        AgentRoleClass::Implementation => {
            draft.mutation_posture == AgentRoleMutationPosture::ScopedIntent
                && draft.context_posture == AgentRoleContextPosture::CleanRecompose
                && draft.evidence_posture == AgentRoleEvidencePosture::Candidate
                && draft.output_kind == AgentRoleOutputKind::ImplementationCandidate
        }
        AgentRoleClass::Review => {
            draft.mutation_posture == AgentRoleMutationPosture::SubjectReadOnly
                && draft.context_posture == AgentRoleContextPosture::CleanRecompose
                && draft.evidence_posture == AgentRoleEvidencePosture::Findings
                && draft.output_kind == AgentRoleOutputKind::ReviewReport
        }
        AgentRoleClass::Verification => {
            draft.mutation_posture == AgentRoleMutationPosture::VerificationServiceOnly
                && draft.context_posture == AgentRoleContextPosture::CleanRecompose
                && draft.evidence_posture == AgentRoleEvidencePosture::CheckEvidence
                && draft.output_kind == AgentRoleOutputKind::VerificationReport
        }
        AgentRoleClass::Judgment => {
            draft.mutation_posture == AgentRoleMutationPosture::None
                && draft.context_posture == AgentRoleContextPosture::CleanRecompose
                && draft.evidence_posture == AgentRoleEvidencePosture::Judgment
                && draft.output_kind == AgentRoleOutputKind::JudgmentReport
        }
        AgentRoleClass::Coordination => {
            draft.mutation_posture == AgentRoleMutationPosture::None
                && draft.context_posture == AgentRoleContextPosture::BoundedShared
                && draft.evidence_posture == AgentRoleEvidencePosture::Coordination
                && draft.output_kind == AgentRoleOutputKind::CoordinationReport
        }
        AgentRoleClass::Observation => {
            draft.mutation_posture == AgentRoleMutationPosture::None
                && draft.context_posture == AgentRoleContextPosture::ReadOnlyObservation
                && draft.evidence_posture == AgentRoleEvidencePosture::Observation
                && draft.output_kind == AgentRoleOutputKind::ObservationReport
        }
    };
    if !is_valid {
        return Err(error(
            AgentRoleIssue::ClassInvariant,
            FIELD_CLASS,
            "semantic role class has contradictory mutation, context, evidence, or output posture",
        ));
    }
    debug_assert!(is_valid);
    debug_assert!(!draft.class.as_str().is_empty());
    Ok(())
}

fn validate_profile_references(
    draft: &AgentRoleProfileDraft<'_>,
    references: ProfileReferenceContext<'_>,
) -> Result<(), AgentRoleError> {
    if draft.required_capabilities.iter().any(|required| !draft.capability_ceiling.contains(required)) {
        return Err(error(
            AgentRoleIssue::ClassInvariant,
            FIELD_CAPABILITY,
            "required role capabilities must also appear in the role ceiling",
        ));
    }
    if draft.capability_ceiling.iter().any(|label| !contains_string(references.known_capabilities, label)) {
        return Err(missing(FIELD_CAPABILITY));
    }
    if !contains_string(references.input_contracts, draft.input_contract)
        || !contains_string(references.output_contracts, draft.output_contract)
        || !contains_string(references.evidence_contracts, draft.evidence_contract)
    {
        return Err(missing(FIELD_CONTRACT));
    }
    if !references.budgets.iter().any(|policy| policy.policy_id() == draft.budget_policy_ref) {
        return Err(missing(FIELD_BUDGET));
    }
    if !references
        .separation_policies
        .iter()
        .any(|policy| policy.policy_id() == draft.separation_policy_ref)
    {
        return Err(missing(FIELD_SEPARATION));
    }
    if draft
        .toolsets
        .iter()
        .any(|reference| !references.toolsets.iter().any(|toolset| toolset.toolset_id() == *reference))
    {
        return Err(missing(FIELD_TOOLSET));
    }
    debug_assert!(draft.required_capabilities.iter().all(|required| draft.capability_ceiling.contains(required)));
    debug_assert!(
        references
            .separation_policies
            .iter()
            .any(|policy| policy.policy_id() == draft.separation_policy_ref)
    );
    Ok(())
}

struct ValidatedProfileMetadata {
    role_id: String,
    purpose: String,
    input_contract: String,
    output_contract: String,
    evidence_contract: String,
    separation_policy_ref: String,
    budget_policy_ref: String,
    policy_hash: String,
    profile_hash: String,
}

fn validate_profile_metadata(draft: &AgentRoleProfileDraft<'_>) -> Result<ValidatedProfileMetadata, AgentRoleError> {
    let metadata = ValidatedProfileMetadata {
        role_id: safe_label(ValidationValue {
            field: FIELD_PROFILE,
            value: draft.role_id,
        })?,
        purpose: safe_purpose(draft.purpose)?,
        input_contract: safe_label(ValidationValue {
            field: FIELD_CONTRACT,
            value: draft.input_contract,
        })?,
        output_contract: safe_label(ValidationValue {
            field: FIELD_CONTRACT,
            value: draft.output_contract,
        })?,
        evidence_contract: safe_label(ValidationValue {
            field: FIELD_CONTRACT,
            value: draft.evidence_contract,
        })?,
        separation_policy_ref: safe_label(ValidationValue {
            field: FIELD_SEPARATION,
            value: draft.separation_policy_ref,
        })?,
        budget_policy_ref: safe_label(ValidationValue {
            field: FIELD_BUDGET,
            value: draft.budget_policy_ref,
        })?,
        policy_hash: lowercase_blake3(ValidationValue {
            field: "agent_role_policy_hash",
            value: draft.policy_hash,
        })?,
        profile_hash: lowercase_blake3(ValidationValue {
            field: FIELD_PROFILE_HASH,
            value: draft.profile_hash,
        })?,
    };
    debug_assert!(!metadata.role_id.is_empty());
    debug_assert!(!metadata.output_contract.is_empty());
    Ok(metadata)
}

fn validated_profile(
    draft: &AgentRoleProfileDraft<'_>,
    toolsets: &[AgentRoleToolset],
) -> Result<AgentRoleProfile, AgentRoleError> {
    let metadata = validate_profile_metadata(draft)?;
    let tools = unique_optional_labels("tools", draft.tools)?;
    let toolset_refs = unique_optional_labels(FIELD_TOOLSET, draft.toolsets)?;
    let expanded_tools = expand_tools(&tools, &toolset_refs, toolsets)?;
    debug_assert!(!draft.role_id.is_empty());
    debug_assert!(!draft.output_contract.is_empty());
    Ok(AgentRoleProfile {
        role_id: metadata.role_id,
        class: draft.class,
        purpose: metadata.purpose,
        source_scope: draft.source_scope,
        trust_status: draft.trust_status,
        admission_status: draft.admission_status,
        project_profile_admitted: draft.project_profile_admitted,
        required_capabilities: unique_optional_labels(FIELD_CAPABILITY, draft.required_capabilities)?,
        capability_ceiling: unique_labels(FIELD_CAPABILITY, draft.capability_ceiling)?,
        eligible_model_profiles: unique_labels("eligible_model_profiles", draft.eligible_model_profiles)?,
        model_targets: unique_labels("model_targets", draft.model_targets)?,
        prompt_modes: unique_labels("prompt_modes", draft.prompt_modes)?,
        prompt_sources: unique_labels("prompt_sources", draft.prompt_sources)?,
        skills: unique_optional_labels("skills", draft.skills)?,
        memory_policy_hashes: draft
            .memory_policy_hashes
            .iter()
            .map(|hash| {
                lowercase_blake3(ValidationValue {
                    field: "agent_role_memory_policy_hash",
                    value: hash,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        tools,
        toolsets: toolset_refs,
        expanded_tools,
        tool_capabilities: unique_optional_labels("tool_capabilities", draft.tool_capabilities)?,
        workspace_scopes: unique_labels("workspace_scopes", draft.workspace_scopes)?,
        mutation_posture: draft.mutation_posture,
        context_posture: draft.context_posture,
        evidence_posture: draft.evidence_posture,
        output_kind: draft.output_kind,
        input_contract: metadata.input_contract,
        output_contract: metadata.output_contract,
        evidence_contract: metadata.evidence_contract,
        separation_policy_ref: metadata.separation_policy_ref,
        budget_policy_ref: metadata.budget_policy_ref,
        policy_hash: metadata.policy_hash,
        profile_hash: metadata.profile_hash,
    })
}

fn expand_tools(
    direct_tools: &[String],
    toolset_refs: &[String],
    toolsets: &[AgentRoleToolset],
) -> Result<Vec<String>, AgentRoleError> {
    let additional_count = toolset_refs.iter().try_fold(usize::default(), |total, reference| {
        let toolset = toolsets
            .iter()
            .find(|toolset| toolset.toolset_id() == reference)
            .ok_or_else(|| missing(FIELD_TOOLSET))?;
        Ok::<usize, AgentRoleError>(total.saturating_add(toolset.tool_names().len()))
    })?;
    let expanded_tool_count = direct_tools.len().saturating_add(additional_count);
    let mut expanded = Vec::with_capacity(expanded_tool_count);
    expanded.extend(direct_tools.iter().cloned());
    for reference in toolset_refs {
        let toolset = toolsets
            .iter()
            .find(|toolset| toolset.toolset_id() == reference)
            .ok_or_else(|| missing(FIELD_TOOLSET))?;
        for tool_name in toolset.tool_names() {
            if !expanded.contains(tool_name) {
                expanded.push(tool_name.clone());
            }
        }
    }
    expanded.sort();
    debug_assert!(expanded.len() <= expanded_tool_count);
    debug_assert!(expanded.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(expanded)
}

fn validate_class_separation(
    profiles: &[AgentRoleProfile],
    policies: &[AgentRoleSeparationPolicy],
) -> Result<(), AgentRoleError> {
    for profile in profiles {
        let policy = policies
            .iter()
            .find(|policy| policy.policy_id() == profile.separation_policy_ref)
            .ok_or_else(|| missing(FIELD_SEPARATION))?;
        if matches!(profile.class, AgentRoleClass::Review | AgentRoleClass::Verification | AgentRoleClass::Judgment)
            && (!policy.requires(AgentRoleSeparationDimension::SubjectImmutable)
                || (!policy.requires(AgentRoleSeparationDimension::Assignment)
                    && !policy.requires(AgentRoleSeparationDimension::MutationLineage)
                    && !policy.requires(AgentRoleSeparationDimension::Actor)))
        {
            return Err(error(
                AgentRoleIssue::ClassInvariant,
                FIELD_SEPARATION,
                "independent evidence roles require subject immutability and an explicit distinct dimension",
            ));
        }
    }
    debug_assert!(!policies.is_empty());
    Ok(())
}

fn validate_baseline_profiles(profiles: &[AgentRoleProfile]) -> Result<(), AgentRoleError> {
    for (role_id, expected_class) in [
        (BASELINE_IMPLEMENTER, AgentRoleClass::Implementation),
        (BASELINE_REVIEWER, AgentRoleClass::Review),
        (BASELINE_VERIFIER, AgentRoleClass::Verification),
        (BASELINE_JUDGE, AgentRoleClass::Judgment),
    ] {
        let profile =
            profiles.iter().find(|profile| profile.role_id() == role_id).ok_or_else(|| missing(FIELD_PROFILE))?;
        if profile.class() != expected_class {
            return Err(error(
                AgentRoleIssue::ClassInvariant,
                FIELD_CLASS,
                "baseline role identity has the wrong semantic class",
            ));
        }
    }
    debug_assert!(profiles.len() >= 4);
    debug_assert!(profiles.iter().any(|profile| profile.role_id() == BASELINE_JUDGE));
    Ok(())
}

fn profile_draft_hash(draft: &AgentRoleProfileDraft<'_>) -> String {
    let mut parts = Vec::with_capacity(MAX_ROLE_REFERENCES);
    parts.extend([
        AGENT_ROLE_CATALOG_SCHEMA.to_string(),
        draft.role_id.to_string(),
        draft.class.as_str().to_string(),
        draft.purpose.trim().to_string(),
        draft.source_scope.as_str().to_string(),
        draft.trust_status.as_str().to_string(),
        draft.admission_status.as_str().to_string(),
        draft.project_profile_admitted.to_string(),
        draft.mutation_posture.as_str().to_string(),
        draft.context_posture.as_str().to_string(),
        draft.evidence_posture.as_str().to_string(),
        draft.output_kind.as_str().to_string(),
        draft.input_contract.to_string(),
        draft.output_contract.to_string(),
        draft.evidence_contract.to_string(),
        draft.separation_policy_ref.to_string(),
        draft.budget_policy_ref.to_string(),
        draft.policy_hash.to_string(),
    ]);
    for (prefix, values) in [
        ("required", draft.required_capabilities),
        ("ceiling", draft.capability_ceiling),
        ("model_profile", draft.eligible_model_profiles),
        ("model_target", draft.model_targets),
        ("prompt_mode", draft.prompt_modes),
        ("prompt", draft.prompt_sources),
        ("skill", draft.skills),
        ("memory", draft.memory_policy_hashes),
        ("tool", draft.tools),
        ("toolset", draft.toolsets),
        ("tool_capability", draft.tool_capabilities),
        ("workspace", draft.workspace_scopes),
    ] {
        parts.extend(values.iter().map(|value| format!("{prefix}:{value}")));
    }
    let profile_hash = hash_parts(AGENT_ROLE_PROFILE_HASH_DOMAIN, parts.as_slice());
    debug_assert!(!parts.is_empty());
    debug_assert_eq!(profile_hash.len(), BLAKE3_HEX_CHARS);
    profile_hash
}

pub(super) fn hash_parts(domain: &str, parts: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[derive(Clone, Copy)]
pub(super) struct ValidationValue<'a> {
    pub field: &'static str,
    pub value: &'a str,
}

pub(super) fn safe_label(input: ValidationValue<'_>) -> Result<String, AgentRoleError> {
    let normalized = input.value.trim();
    let is_within_label_bound = !normalized.is_empty() && normalized.chars().count() <= MAX_ROLE_LABEL_CHARS;
    let is_character_set_valid = normalized.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '-' | '_' | '.' | ':')
    });
    let is_endpoint_valid = normalized
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && normalized
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    if !is_within_label_bound || !is_character_set_valid || !is_endpoint_valid {
        return Err(error(
            AgentRoleIssue::UnsafeLabel,
            input.field,
            "role labels must be bounded lowercase ASCII metadata",
        ));
    }
    debug_assert!(is_within_label_bound);
    debug_assert!(is_character_set_valid && is_endpoint_valid);
    Ok(normalized.to_string())
}

fn safe_purpose(value: &str) -> Result<String, AgentRoleError> {
    let normalized = value.trim();
    let is_within_purpose_bound = !normalized.is_empty() && normalized.chars().count() <= MAX_ROLE_PURPOSE_CHARS;
    let has_forbidden_marker = FORBIDDEN_PURPOSE_MARKERS.iter().any(|marker| normalized.contains(marker));
    if !is_within_purpose_bound || has_forbidden_marker {
        return Err(error(
            AgentRoleIssue::UnsafeLabel,
            FIELD_PROFILE,
            "role purpose must be bounded safe descriptive text",
        ));
    }
    Ok(normalized.to_string())
}

pub(super) fn lowercase_blake3(input: ValidationValue<'_>) -> Result<String, AgentRoleError> {
    let is_valid = input.value.len() == BLAKE3_HEX_CHARS
        && input.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        return Err(error(AgentRoleIssue::InvalidHash, input.field, "identity must be lowercase BLAKE3 hexadecimal"));
    }
    Ok(input.value.to_string())
}

fn unique_labels(field: &'static str, values: &[&str]) -> Result<Vec<String>, AgentRoleError> {
    validate_nonempty_bound(CollectionBound {
        item_count: values.len(),
        maximum_count: MAX_ROLE_REFERENCES,
        field,
    })?;
    unique_optional_labels(field, values)
}

fn unique_optional_labels(field: &'static str, values: &[&str]) -> Result<Vec<String>, AgentRoleError> {
    let mut labels = Vec::with_capacity(values.len());
    for value in values {
        let label = safe_label(ValidationValue { field, value })?;
        if labels.contains(&label) {
            return Err(duplicate(field));
        }
        labels.push(label);
    }
    Ok(labels)
}

fn contains_string(values: &[String], expected: &str) -> bool {
    values.iter().any(|value| value == expected)
}

pub(super) fn error(issue: AgentRoleIssue, field: &'static str, detail: impl Into<String>) -> AgentRoleError {
    AgentRoleError::new(issue, field, detail)
}

fn duplicate(field: &'static str) -> AgentRoleError {
    error(AgentRoleIssue::DuplicateIdentity, field, "normalized role identity is duplicated")
}

fn missing(field: &'static str) -> AgentRoleError {
    error(AgentRoleIssue::MissingReference, field, "role profile references an unknown catalog identity")
}
