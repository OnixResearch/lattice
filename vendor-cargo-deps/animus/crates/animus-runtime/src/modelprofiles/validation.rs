//! Pure validation and canonical identity construction for model runtime profiles.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::types::BLAKE3_HEX_CHARS;
use super::types::HASH_SEPARATOR;
use super::types::MAX_BUDGET_POLICY_COUNT;
use super::types::MAX_CONTEXT_BUDGET_UNITS;
use super::types::MAX_PROFILE_COUNT;
use super::types::MAX_PROFILE_RESOURCE_REFS;
use super::types::MAX_SAFE_LABEL_CHARS;
use super::types::MAX_TOOL_CALL_BUDGET;
use super::types::MAX_TOOL_PARALLELISM;
use super::types::MAX_TOOLSET_COUNT;
use super::types::MODEL_RUNTIME_PROFILE_HASH_DOMAIN;
use super::types::MODEL_RUNTIME_PROFILE_SCHEMA;
use super::types::ModelNamedResourceReference;
use super::types::ModelNamedResourceReferenceDraft;
use super::types::ModelProfileSourceScope;
use super::types::ModelPromptSourceReference;
use super::types::ModelPromptSourceReferenceDraft;
use super::types::ModelRuntimeBinding;
use super::types::ModelRuntimeBindingDraft;
use super::types::ModelRuntimeBudgetPolicy;
use super::types::ModelRuntimeBudgetPolicyDraft;
use super::types::ModelRuntimeProfile;
use super::types::ModelRuntimeProfileDraft;
use super::types::ModelRuntimeProfileError;
use super::types::ModelRuntimeProfileIssue;
use super::types::ModelRuntimeProfileSet;
use super::types::ModelRuntimeProfileSetDraft;
use super::types::ModelSkillReference;
use super::types::ModelSkillReferenceDraft;
use super::types::ModelToolset;
use super::types::ModelToolsetDraft;
use super::types::NativeToolCallingMode;

const FIELD_SCHEMA: &str = "schema";
const FIELD_PROFILE_SET: &str = "profiles";
const FIELD_PROFILE_ID: &str = "profile_id";
const FIELD_PROFILE_BINDING: &str = "binding";
const FIELD_PROFILE_HASH: &str = "profile_hash";
const FIELD_POLICY_HASH: &str = "policy_hash";
const FIELD_DEFAULT_PROFILE: &str = "explicit_default_profile_id";
const FIELD_BUDGET_POLICY: &str = "budget_policy";
const FIELD_TOOLSET: &str = "toolset";
const FIELD_PROMPT_SOURCE: &str = "prompt_sources";
const FIELD_SKILL: &str = "skills";
const FIELD_MEMORY_POLICY: &str = "memory_policy_hash";
const FIELD_TOOL: &str = "tools";
const FIELD_TOOLSET_REFERENCE: &str = "toolsets";
const FIELD_PROJECT_ADMISSION: &str = "project_profile_admitted";
const EMPTY_PROFILE_HASH: &str = "";

#[derive(Clone, Copy)]
pub(super) struct ModelProfileField(pub &'static str);

impl ModelRuntimeProfileSet {
    /// Validates host-loaded profile-set data without reading files or evaluating Nickel.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRuntimeProfileError`] for malformed, duplicate, stale, or
    /// dangling configuration.
    // r[impl agent_harness_runtime.model_runtime_profiles.contract]
    pub fn validate(draft: &ModelRuntimeProfileSetDraft<'_>) -> Result<Self, ModelRuntimeProfileError> {
        validate_set_shape(draft)?;
        let budget_policies = validate_budget_policies(draft.budget_policies)?;
        let toolsets = validate_toolsets(draft.toolsets)?;
        let profiles = validate_profiles(draft.profiles)?;
        let explicit_default_profile_id =
            validate_default_profile(draft.explicit_default_profile_id, profiles.as_slice())?;
        validate_profile_references(profiles.as_slice(), budget_policies.as_slice(), toolsets.as_slice())?;
        Ok(Self {
            enablement: draft.enablement,
            explicit_default_profile_id,
            budget_policies,
            toolsets,
            profiles,
        })
    }
}

/// Computes the canonical BLAKE3 content identity for one profile draft.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] when profile fields are malformed.
pub fn blake3_model_runtime_profile_identity(
    draft: &ModelRuntimeProfileDraft<'_>,
) -> Result<String, ModelRuntimeProfileError> {
    let profile = validate_profile_fields(draft, false)?;
    Ok(profile_content_hash(&profile))
}

fn validate_set_shape(draft: &ModelRuntimeProfileSetDraft<'_>) -> Result<(), ModelRuntimeProfileError> {
    if draft.schema != MODEL_RUNTIME_PROFILE_SCHEMA {
        return Err(error(
            ModelRuntimeProfileIssue::InvalidSchema,
            FIELD_SCHEMA,
            "unsupported model runtime profile schema",
        ));
    }
    if draft.profiles.is_empty() || draft.profiles.len() > MAX_PROFILE_COUNT {
        return Err(error(
            ModelRuntimeProfileIssue::CollectionTooLarge,
            FIELD_PROFILE_SET,
            "profile count is empty or exceeds the profile bound",
        ));
    }
    if draft.budget_policies.is_empty() || draft.budget_policies.len() > MAX_BUDGET_POLICY_COUNT {
        return Err(error(
            ModelRuntimeProfileIssue::CollectionTooLarge,
            FIELD_BUDGET_POLICY,
            "budget policy count is empty or exceeds the policy bound",
        ));
    }
    if draft.toolsets.len() > MAX_TOOLSET_COUNT {
        return Err(error(
            ModelRuntimeProfileIssue::CollectionTooLarge,
            FIELD_TOOLSET,
            "toolset count exceeds the toolset bound",
        ));
    }
    assert_eq!(draft.schema, MODEL_RUNTIME_PROFILE_SCHEMA);
    assert!(!draft.profiles.is_empty());
    Ok(())
}

fn validate_budget_policies(
    drafts: &[ModelRuntimeBudgetPolicyDraft<'_>],
) -> Result<Vec<ModelRuntimeBudgetPolicy>, ModelRuntimeProfileError> {
    let mut policies = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let policy_id = safe_label(ModelProfileField(FIELD_BUDGET_POLICY), draft.policy_id)?;
        validate_budget_values(draft)?;
        if policies.iter().any(|policy: &ModelRuntimeBudgetPolicy| policy.policy_id == policy_id) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_BUDGET_POLICY,
                "duplicate budget policy identity",
            ));
        }
        policies.push(ModelRuntimeBudgetPolicy {
            policy_id,
            max_context_units: draft.max_context_units,
            max_tool_calls: draft.max_tool_calls,
            max_parallel_tool_calls: draft.max_parallel_tool_calls,
        });
    }
    assert_eq!(policies.len(), drafts.len());
    assert!(policies.iter().all(|policy| !policy.policy_id.is_empty()));
    Ok(policies)
}

fn validate_budget_values(draft: &ModelRuntimeBudgetPolicyDraft<'_>) -> Result<(), ModelRuntimeProfileError> {
    let parallel_calls = u32::from(draft.max_parallel_tool_calls);
    let is_budget_valid = draft.max_context_units > u32::default()
        && draft.max_context_units <= MAX_CONTEXT_BUDGET_UNITS
        && draft.max_tool_calls > u32::default()
        && draft.max_tool_calls <= MAX_TOOL_CALL_BUDGET
        && draft.max_parallel_tool_calls > u16::default()
        && draft.max_parallel_tool_calls <= MAX_TOOL_PARALLELISM
        && parallel_calls <= draft.max_tool_calls;
    if !is_budget_valid {
        return Err(error(
            ModelRuntimeProfileIssue::InvalidBudget,
            FIELD_BUDGET_POLICY,
            "budget values must be positive, bounded, and internally ordered",
        ));
    }
    Ok(())
}

fn validate_toolsets(drafts: &[ModelToolsetDraft<'_>]) -> Result<Vec<ModelToolset>, ModelRuntimeProfileError> {
    let mut toolsets = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let toolset_id = safe_label(ModelProfileField(FIELD_TOOLSET), draft.toolset_id)?;
        if draft.tool_names.is_empty() || draft.tool_names.len() > MAX_PROFILE_RESOURCE_REFS {
            return Err(error(
                ModelRuntimeProfileIssue::CollectionTooLarge,
                FIELD_TOOLSET,
                "toolset members are empty or exceed the resource bound",
            ));
        }
        let tool_names = unique_labels(ModelProfileField(FIELD_TOOLSET), draft.tool_names)?;
        if toolsets.iter().any(|toolset: &ModelToolset| toolset.toolset_id == toolset_id) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_TOOLSET,
                "duplicate toolset identity",
            ));
        }
        toolsets.push(ModelToolset { toolset_id, tool_names });
    }
    assert_eq!(toolsets.len(), drafts.len());
    assert!(toolsets.iter().all(|toolset| !toolset.toolset_id.is_empty()));
    Ok(toolsets)
}

fn validate_profiles(
    drafts: &[ModelRuntimeProfileDraft<'_>],
) -> Result<Vec<ModelRuntimeProfile>, ModelRuntimeProfileError> {
    let mut profiles = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let profile = validate_profile_fields(draft, true)?;
        if profiles.iter().any(|candidate: &ModelRuntimeProfile| candidate.profile_id == profile.profile_id) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_PROFILE_ID,
                "duplicate profile identity",
            ));
        }
        if profiles.iter().any(|candidate| profile_bindings_overlap(&candidate.binding, &profile.binding)) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_PROFILE_BINDING,
                "duplicate profile binding",
            ));
        }
        profiles.push(profile);
    }
    assert_eq!(profiles.len(), drafts.len());
    assert!(profiles.iter().all(|profile| !profile.profile_id.is_empty()));
    Ok(profiles)
}

fn validate_profile_fields(
    draft: &ModelRuntimeProfileDraft<'_>,
    verify_profile_hash: bool,
) -> Result<ModelRuntimeProfile, ModelRuntimeProfileError> {
    validate_profile_collection_bounds(draft)?;
    validate_project_profile_admission(draft)?;
    let profile = ModelRuntimeProfile {
        profile_id: safe_label(ModelProfileField(FIELD_PROFILE_ID), draft.profile_id)?,
        binding: validate_binding(draft.binding)?,
        source_scope: draft.source_scope,
        trust_status: draft.trust_status,
        admission_status: draft.admission_status,
        project_profile_admitted: draft.project_profile_admitted,
        prompt_mode_ref: safe_label(ModelProfileField("prompt_mode_ref"), draft.prompt_mode_ref)?,
        prompt_sources: validate_prompt_refs(draft.prompt_sources)?,
        skills: validate_skill_refs(draft.skills)?,
        memory_access_mode: draft.memory_access_mode,
        memory_policy_hash: lowercase_blake3(ModelProfileField(FIELD_MEMORY_POLICY), draft.memory_policy_hash)?,
        tools: validate_named_refs(ModelProfileField(FIELD_TOOL), draft.tools)?,
        toolsets: validate_named_refs(ModelProfileField(FIELD_TOOLSET_REFERENCE), draft.toolsets)?,
        tool_calling_mode: draft.tool_calling_mode,
        context_budget_policy: safe_label(ModelProfileField("context_budget_policy"), draft.context_budget_policy)?,
        tool_budget_policy: safe_label(ModelProfileField("tool_budget_policy"), draft.tool_budget_policy)?,
        policy_hash: lowercase_blake3(ModelProfileField(FIELD_POLICY_HASH), draft.policy_hash)?,
        profile_hash: if verify_profile_hash {
            lowercase_blake3(ModelProfileField(FIELD_PROFILE_HASH), draft.profile_hash)?
        } else {
            String::from(EMPTY_PROFILE_HASH)
        },
    };
    validate_disabled_tools(&profile)?;
    if verify_profile_hash {
        let expected_profile_hash = profile_content_hash(&profile);
        if profile.profile_hash != expected_profile_hash {
            return Err(error(
                ModelRuntimeProfileIssue::StaleProfileIdentity,
                FIELD_PROFILE_HASH,
                format!("profile content identity does not match canonical fields; expected {expected_profile_hash}"),
            ));
        }
    }
    assert!(!profile.profile_id.is_empty());
    assert!(!profile.policy_hash.is_empty());
    Ok(profile)
}

fn validate_profile_collection_bounds(draft: &ModelRuntimeProfileDraft<'_>) -> Result<(), ModelRuntimeProfileError> {
    for (field, count) in [
        (FIELD_PROMPT_SOURCE, draft.prompt_sources.len()),
        (FIELD_SKILL, draft.skills.len()),
        (FIELD_TOOL, draft.tools.len()),
        (FIELD_TOOLSET_REFERENCE, draft.toolsets.len()),
    ] {
        if count > MAX_PROFILE_RESOURCE_REFS {
            return Err(error(
                ModelRuntimeProfileIssue::CollectionTooLarge,
                field,
                "profile resource references exceed the resource bound",
            ));
        }
    }
    Ok(())
}

fn validate_project_profile_admission(draft: &ModelRuntimeProfileDraft<'_>) -> Result<(), ModelRuntimeProfileError> {
    if draft.source_scope == ModelProfileSourceScope::Project && !draft.project_profile_admitted {
        return Err(error(
            ModelRuntimeProfileIssue::UntrustedProfile,
            FIELD_PROJECT_ADMISSION,
            "project profile lacks explicit host admission",
        ));
    }
    Ok(())
}

fn validate_binding(draft: ModelRuntimeBindingDraft<'_>) -> Result<ModelRuntimeBinding, ModelRuntimeProfileError> {
    match draft {
        ModelRuntimeBindingDraft::Exact {
            provider_id,
            model_id,
            router_role,
            capability_identity,
        } => Ok(ModelRuntimeBinding::Exact {
            provider_id: safe_label(ModelProfileField("provider_id"), provider_id)?,
            model_id: safe_label(ModelProfileField("model_id"), model_id)?,
            router_role: router_role.map(|role| safe_label(ModelProfileField("router_role"), role)).transpose()?,
            capability_identity: lowercase_blake3(ModelProfileField("capability_identity"), capability_identity)?,
        }),
        ModelRuntimeBindingDraft::Default => Ok(ModelRuntimeBinding::Default),
    }
}

fn validate_prompt_refs(
    drafts: &[ModelPromptSourceReferenceDraft<'_>],
) -> Result<Vec<ModelPromptSourceReference>, ModelRuntimeProfileError> {
    let mut references = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let label = safe_label(ModelProfileField(FIELD_PROMPT_SOURCE), draft.label)?;
        if references.iter().any(|item: &ModelPromptSourceReference| item.label == label) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_PROMPT_SOURCE,
                "duplicate prompt-source reference",
            ));
        }
        references.push(ModelPromptSourceReference {
            label,
            required: draft.required,
        });
    }
    assert_eq!(references.len(), drafts.len());
    assert!(references.iter().all(|reference| !reference.label.is_empty()));
    Ok(references)
}

fn validate_skill_refs(
    drafts: &[ModelSkillReferenceDraft<'_>],
) -> Result<Vec<ModelSkillReference>, ModelRuntimeProfileError> {
    let mut references = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let name = safe_label(ModelProfileField(FIELD_SKILL), draft.name)?;
        if references.iter().any(|item: &ModelSkillReference| item.name == name) {
            return Err(error(ModelRuntimeProfileIssue::DuplicateIdentity, FIELD_SKILL, "duplicate skill reference"));
        }
        references.push(ModelSkillReference {
            name,
            posture: draft.posture,
        });
    }
    assert_eq!(references.len(), drafts.len());
    assert!(references.iter().all(|reference| !reference.name.is_empty()));
    Ok(references)
}

fn validate_named_refs(
    field: ModelProfileField,
    drafts: &[ModelNamedResourceReferenceDraft<'_>],
) -> Result<Vec<ModelNamedResourceReference>, ModelRuntimeProfileError> {
    let mut references = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let name = safe_label(field, draft.name)?;
        if references.iter().any(|item: &ModelNamedResourceReference| item.name == name) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                field.0,
                "duplicate named resource reference",
            ));
        }
        references.push(ModelNamedResourceReference {
            name,
            required: draft.required,
        });
    }
    assert_eq!(references.len(), drafts.len());
    assert!(references.iter().all(|reference| !reference.name.is_empty()));
    Ok(references)
}

fn validate_disabled_tools(profile: &ModelRuntimeProfile) -> Result<(), ModelRuntimeProfileError> {
    if profile.tool_calling_mode == NativeToolCallingMode::Disabled
        && (!profile.tools.is_empty() || !profile.toolsets.is_empty())
    {
        return Err(error(
            ModelRuntimeProfileIssue::ToolDenied,
            FIELD_TOOL,
            "disabled tool-calling mode cannot request tools or toolsets",
        ));
    }
    Ok(())
}

fn validate_default_profile(
    default_profile_id: Option<&str>,
    profiles: &[ModelRuntimeProfile],
) -> Result<Option<String>, ModelRuntimeProfileError> {
    assert!(profiles.len() <= MAX_PROFILE_COUNT);
    assert!(profiles.iter().all(|profile| !profile.profile_id.is_empty()));
    let default_profiles =
        profiles.iter().filter(|profile| matches!(profile.binding, ModelRuntimeBinding::Default)).count();
    match default_profile_id {
        None if default_profiles == usize::default() => Ok(None),
        None => Err(error(
            ModelRuntimeProfileIssue::MissingReference,
            FIELD_DEFAULT_PROFILE,
            "default binding must be explicitly named",
        )),
        Some(default_id) => {
            let normalized = safe_label(ModelProfileField(FIELD_DEFAULT_PROFILE), default_id)?;
            let matching = profiles.iter().filter(|profile| {
                profile.profile_id == normalized && matches!(profile.binding, ModelRuntimeBinding::Default)
            });
            if default_profiles != 1 || matching.count() != 1 {
                return Err(error(
                    ModelRuntimeProfileIssue::MissingReference,
                    FIELD_DEFAULT_PROFILE,
                    "explicit default must name the single default binding",
                ));
            }
            Ok(Some(normalized))
        }
    }
}

fn validate_profile_references(
    profiles: &[ModelRuntimeProfile],
    budgets: &[ModelRuntimeBudgetPolicy],
    toolsets: &[ModelToolset],
) -> Result<(), ModelRuntimeProfileError> {
    for profile in profiles {
        if !budgets.iter().any(|policy| policy.policy_id == profile.context_budget_policy)
            || !budgets.iter().any(|policy| policy.policy_id == profile.tool_budget_policy)
        {
            return Err(error(
                ModelRuntimeProfileIssue::MissingReference,
                FIELD_BUDGET_POLICY,
                "profile references an unknown budget policy",
            ));
        }
        if profile
            .toolsets
            .iter()
            .any(|reference| !toolsets.iter().any(|toolset| toolset.toolset_id == reference.name))
        {
            return Err(error(
                ModelRuntimeProfileIssue::MissingReference,
                FIELD_TOOLSET_REFERENCE,
                "profile references an unknown toolset",
            ));
        }
    }
    assert!(!profiles.is_empty());
    assert!(!budgets.is_empty());
    Ok(())
}

fn profile_bindings_overlap(left: &ModelRuntimeBinding, right: &ModelRuntimeBinding) -> bool {
    match (left, right) {
        (ModelRuntimeBinding::Default, ModelRuntimeBinding::Default) => true,
        (
            ModelRuntimeBinding::Exact {
                provider_id: left_provider,
                model_id: left_model,
                router_role: left_role,
                ..
            },
            ModelRuntimeBinding::Exact {
                provider_id: right_provider,
                model_id: right_model,
                router_role: right_role,
                ..
            },
        ) => left_provider == right_provider && left_model == right_model && left_role == right_role,
        (ModelRuntimeBinding::Exact { .. }, ModelRuntimeBinding::Default)
        | (ModelRuntimeBinding::Default, ModelRuntimeBinding::Exact { .. }) => false,
    }
}

pub(super) fn profile_content_hash(profile: &ModelRuntimeProfile) -> String {
    assert!(!profile.profile_id.is_empty());
    assert!(!profile.policy_hash.is_empty());
    let mut parts = vec![
        MODEL_RUNTIME_PROFILE_SCHEMA.to_string(),
        profile.profile_id.clone(),
        binding_material(&profile.binding),
        profile.source_scope.as_str().to_string(),
        profile.trust_status.as_str().to_string(),
        profile.admission_status.as_str().to_string(),
        profile.project_profile_admitted.to_string(),
        profile.prompt_mode_ref.clone(),
        profile.memory_access_mode.as_str().to_string(),
        profile.memory_policy_hash.clone(),
        profile.tool_calling_mode.as_str().to_string(),
        profile.context_budget_policy.clone(),
        profile.tool_budget_policy.clone(),
        profile.policy_hash.clone(),
    ];
    parts.extend(prompt_material(profile.prompt_sources.as_slice()));
    parts.extend(skill_material(profile.skills.as_slice()));
    parts.extend(named_material("tool", profile.tools.as_slice()));
    parts.extend(named_material("toolset", profile.toolsets.as_slice()));
    blake3_parts(MODEL_RUNTIME_PROFILE_HASH_DOMAIN, parts.as_slice())
}

fn binding_material(binding: &ModelRuntimeBinding) -> String {
    match binding {
        ModelRuntimeBinding::Default => String::from("default"),
        ModelRuntimeBinding::Exact {
            provider_id,
            model_id,
            router_role,
            capability_identity,
        } => [
            "exact",
            provider_id.as_str(),
            model_id.as_str(),
            router_role.as_deref().unwrap_or("none"),
            capability_identity.as_str(),
        ]
        .join(":"),
    }
}

fn prompt_material(refs: &[ModelPromptSourceReference]) -> Vec<String> {
    refs.iter().map(|reference| format!("prompt:{}:{}", reference.label, reference.required)).collect()
}

fn skill_material(refs: &[ModelSkillReference]) -> Vec<String> {
    refs.iter()
        .map(|reference| format!("skill:{}:{}", reference.name, reference.posture.as_str()))
        .collect()
}

fn named_material(class: &str, refs: &[ModelNamedResourceReference]) -> Vec<String> {
    refs.iter().map(|reference| format!("{class}:{}:{}", reference.name, reference.required)).collect()
}

pub(super) fn blake3_parts(domain: &str, parts: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn safe_label(field: ModelProfileField, value: &str) -> Result<String, ModelRuntimeProfileError> {
    let normalized = value.trim();
    let is_character_set_safe = normalized.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '-' | '_' | '.' | ':')
    });
    let is_endpoint_shape_safe = normalized
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && normalized
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    let is_label_empty = normalized.is_empty();
    let is_label_too_long = normalized.chars().count() > MAX_SAFE_LABEL_CHARS;
    if is_label_empty {
        return Err(unsafe_label_error(field));
    }
    if is_label_too_long {
        return Err(unsafe_label_error(field));
    }
    if !is_character_set_safe {
        return Err(unsafe_label_error(field));
    }
    if !is_endpoint_shape_safe {
        return Err(unsafe_label_error(field));
    }
    assert!(is_character_set_safe);
    assert!(is_endpoint_shape_safe);
    Ok(normalized.to_string())
}

fn unsafe_label_error(field: ModelProfileField) -> ModelRuntimeProfileError {
    error(ModelRuntimeProfileIssue::UnsafeLabel, field.0, "label must be bounded lowercase ASCII metadata")
}

pub(super) fn lowercase_blake3(field: ModelProfileField, value: &str) -> Result<String, ModelRuntimeProfileError> {
    if value.len() != BLAKE3_HEX_CHARS {
        return Err(invalid_hash_error(field));
    }
    let is_hash_text_valid = value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_hash_text_valid {
        return Err(invalid_hash_error(field));
    }
    assert_eq!(value.len(), BLAKE3_HEX_CHARS);
    assert!(is_hash_text_valid);
    Ok(value.to_string())
}

fn invalid_hash_error(field: ModelProfileField) -> ModelRuntimeProfileError {
    error(ModelRuntimeProfileIssue::InvalidHash, field.0, "identity must be lowercase BLAKE3 hexadecimal")
}

fn unique_labels(field: ModelProfileField, values: &[&str]) -> Result<Vec<String>, ModelRuntimeProfileError> {
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let label = safe_label(field, value)?;
        if normalized.contains(&label) {
            return Err(error(ModelRuntimeProfileIssue::DuplicateIdentity, field.0, "duplicate normalized label"));
        }
        normalized.push(label);
    }
    Ok(normalized)
}

pub(super) fn error(
    issue: ModelRuntimeProfileIssue,
    field: &'static str,
    detail: impl Into<String>,
) -> ModelRuntimeProfileError {
    ModelRuntimeProfileError::new(issue, field, detail)
}
