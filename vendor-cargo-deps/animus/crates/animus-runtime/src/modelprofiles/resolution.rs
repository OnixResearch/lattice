//! Deterministic exact-tier model runtime profile resolution.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::types::{
    MODEL_RUNTIME_RESOLUTION_HASH_DOMAIN, ModelNamedResourceReference, ModelProfileAdmissionStatus,
    ModelProfileEnablementPolicy, ModelProfileSourceScope, ModelProfileSpecificity,
    ModelProfileTrustStatus, ModelRuntimeBinding, ModelRuntimeBudgetPolicy, ModelRuntimeProfile,
    ModelRuntimeProfileError, ModelRuntimeProfileIssue, ModelRuntimeProfileResolution,
    ModelRuntimeProfileSet, ModelRuntimeTarget, ModelRuntimeTargetDraft,
    ResolvedModelRuntimeProfile,
};
use super::validation::{ModelProfileField, blake3_parts, error, lowercase_blake3, safe_label};

const TARGET_HASH_DOMAIN: &str = "animus.model-runtime-target.v1";
const FIELD_PROFILE: &str = "profile";
const FIELD_PROVIDER_TRANSPORT_ID: &str = "provider_transport_id";
const FIELD_MODEL_DIALECT_ID: &str = "model_dialect_id";
const FIELD_CAPABILITY_IDENTITY: &str = "capability_identity";
const FIELD_BUDGET_POLICY: &str = "budget_policy";
const FIELD_TOOLSET: &str = "toolset";

/// Resolves one selected provider/model through fixed exact specificity tiers.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] for a malformed target or when required
/// mode encounters an ambiguous, missing, stale, untrusted, or unadmitted profile.
///
/// # Panics
///
/// Panics if the resolved value does not preserve the selected validated profile
/// identity or if canonical resolution identity construction returns empty text.
// r[impl agent_harness_runtime.model_runtime_profiles.resolution]
pub fn resolve_model_runtime_profile(
    profile_set: &ModelRuntimeProfileSet,
    target_draft: ModelRuntimeTargetDraft<'_>,
) -> Result<ModelRuntimeProfileResolution, ModelRuntimeProfileError> {
    if profile_set.enablement() == ModelProfileEnablementPolicy::CompatibilityDisabled {
        return Ok(ModelRuntimeProfileResolution::CompatibilityDisabled);
    }
    let target = validate_target(target_draft)?;
    let (profile, specificity) = select_profile(profile_set, &target)?;
    validate_selected_profile(profile, &target)?;
    let context_budget = resolve_budget(profile_set, profile.context_budget_policy())?;
    let tool_budget = resolve_budget(profile_set, profile.tool_budget_policy())?;
    let expanded_tools = expand_tool_references(profile_set, profile)?;
    let resolution_hash = resolution_hash(ResolutionHashInput {
        profile,
        target: &target,
        specificity,
        context_budget: &context_budget,
        tool_budget: &tool_budget,
        tools: expanded_tools.as_slice(),
    });
    let resolved = ResolvedModelRuntimeProfile {
        profile: profile.clone(),
        target,
        specificity,
        context_budget,
        tool_budget,
        expanded_tools,
        resolution_hash,
    };
    assert_eq!(resolved.profile().profile_id(), profile.profile_id());
    assert!(!resolved.resolution_hash().is_empty());
    Ok(ModelRuntimeProfileResolution::Resolved(Box::new(resolved)))
}

// r[impl animus.dialect.target]
#[cfg_attr(
    animus_octet,
    allow(
        tigerstyle::numeric_units,
        reason = "provider transport identifiers are labels, not numeric ports or quantities"
    )
)]
fn validate_target(
    draft: ModelRuntimeTargetDraft<'_>,
) -> Result<ModelRuntimeTarget, ModelRuntimeProfileError> {
    let provider_id = safe_label(ModelProfileField("provider_id"), draft.provider_id)?;
    let provider_transport_id = safe_label(
        ModelProfileField(FIELD_PROVIDER_TRANSPORT_ID),
        draft.provider_transport_id,
    )?;
    let model_id = safe_label(ModelProfileField("model_id"), draft.model_id)?;
    let model_dialect_id = safe_label(
        ModelProfileField(FIELD_MODEL_DIALECT_ID),
        draft.model_dialect_id,
    )?;
    let router_role = draft
        .router_role
        .map(|role| safe_label(ModelProfileField("router_role"), role))
        .transpose()?;
    let capability_identity = lowercase_blake3(
        ModelProfileField(FIELD_CAPABILITY_IDENTITY),
        draft.capability_identity,
    )?;
    let target_hash = blake3_parts(
        TARGET_HASH_DOMAIN,
        &[
            provider_id.clone(),
            provider_transport_id.clone(),
            model_id.clone(),
            model_dialect_id.clone(),
            router_role.clone().unwrap_or_else(|| String::from("none")),
            capability_identity.clone(),
        ],
    );
    let target = ModelRuntimeTarget {
        provider_id,
        provider_transport_id,
        model_id,
        model_dialect_id,
        router_role,
        capability_identity,
        target_hash,
    };
    assert!(!target.provider_id().is_empty());
    assert!(!target.target_hash().is_empty());
    Ok(target)
}

fn select_profile<'a>(
    profile_set: &'a ModelRuntimeProfileSet,
    target: &ModelRuntimeTarget,
) -> Result<(&'a ModelRuntimeProfile, ModelProfileSpecificity), ModelRuntimeProfileError> {
    assert!(!profile_set.profiles().is_empty());
    assert_eq!(
        profile_set.enablement(),
        ModelProfileEnablementPolicy::Required
    );
    if let Some(router_role) = target.router_role() {
        let role_matches: Vec<&ModelRuntimeProfile> = profile_set
            .profiles()
            .iter()
            .filter(|profile| exact_binding_matches(profile, target, Some(router_role)))
            .collect();
        if let Some(profile) = one_match(role_matches.as_slice())? {
            return Ok((profile, ModelProfileSpecificity::ExactTargetAndRole));
        }
    }
    let target_matches: Vec<&ModelRuntimeProfile> = profile_set
        .profiles()
        .iter()
        .filter(|profile| exact_binding_matches(profile, target, None))
        .collect();
    if let Some(profile) = one_match(target_matches.as_slice())? {
        return Ok((profile, ModelProfileSpecificity::ExactTarget));
    }
    select_explicit_default(profile_set)
}

fn one_match<'a>(
    matches: &[&'a ModelRuntimeProfile],
) -> Result<Option<&'a ModelRuntimeProfile>, ModelRuntimeProfileError> {
    match matches {
        [] => Ok(None),
        [profile] => Ok(Some(*profile)),
        [_, _, ..] => Err(error(
            ModelRuntimeProfileIssue::AmbiguousProfile,
            FIELD_PROFILE,
            "multiple profiles matched one exact specificity tier",
        )),
    }
}

fn exact_binding_matches(
    profile: &ModelRuntimeProfile,
    target: &ModelRuntimeTarget,
    router_role: Option<&str>,
) -> bool {
    matches!(
        profile.binding(),
        ModelRuntimeBinding::Exact {
            provider_id,
            model_id,
            router_role: profile_role,
            ..
        } if provider_id == target.provider_id()
            && model_id == target.model_id()
            && profile_role.as_deref() == router_role
    )
}

fn select_explicit_default(
    profile_set: &ModelRuntimeProfileSet,
) -> Result<(&ModelRuntimeProfile, ModelProfileSpecificity), ModelRuntimeProfileError> {
    let default_id = profile_set.explicit_default_profile_id().ok_or_else(|| {
        error(
            ModelRuntimeProfileIssue::MissingProfile,
            FIELD_PROFILE,
            "required mode has no exact binding or explicit default",
        )
    })?;
    let profile = profile_set
        .profiles()
        .iter()
        .find(|profile| {
            profile.profile_id() == default_id
                && matches!(profile.binding(), ModelRuntimeBinding::Default)
        })
        .ok_or_else(|| {
            error(
                ModelRuntimeProfileIssue::MissingProfile,
                FIELD_PROFILE,
                "explicit default profile is unavailable",
            )
        })?;
    assert_eq!(profile.profile_id(), default_id);
    assert!(matches!(profile.binding(), ModelRuntimeBinding::Default));
    Ok((profile, ModelProfileSpecificity::ExplicitDefault))
}

// r[impl animus.dialect.ownership]
// r[impl animus.dialect.ownership.dialect]
fn validate_selected_profile(
    profile: &ModelRuntimeProfile,
    target: &ModelRuntimeTarget,
) -> Result<(), ModelRuntimeProfileError> {
    if profile.trust_status() != ModelProfileTrustStatus::Trusted
        || profile.admission_status() != ModelProfileAdmissionStatus::Admitted
        || (profile.source_scope() == ModelProfileSourceScope::Project
            && !profile.project_profile_admitted())
    {
        return Err(error(
            ModelRuntimeProfileIssue::UntrustedProfile,
            FIELD_PROFILE,
            "selected profile is not trusted and admitted",
        ));
    }
    if let ModelRuntimeBinding::Exact {
        capability_identity,
        ..
    } = profile.binding()
        && capability_identity != target.capability_identity()
    {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_CAPABILITY_IDENTITY,
            "selected exact profile has stale model-capability identity",
        ));
    }
    if profile.dialect_id() != target.model_dialect_id() {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_MODEL_DIALECT_ID,
            "selected profile dialect does not match the exact target dialect",
        ));
    }
    assert_eq!(profile.trust_status(), ModelProfileTrustStatus::Trusted);
    assert_eq!(
        profile.admission_status(),
        ModelProfileAdmissionStatus::Admitted
    );
    Ok(())
}

fn resolve_budget(
    profile_set: &ModelRuntimeProfileSet,
    policy_id: &str,
) -> Result<ModelRuntimeBudgetPolicy, ModelRuntimeProfileError> {
    profile_set
        .budget_policies()
        .iter()
        .find(|policy| policy.policy_id() == policy_id)
        .cloned()
        .ok_or_else(|| {
            error(
                ModelRuntimeProfileIssue::MissingReference,
                FIELD_BUDGET_POLICY,
                "resolved profile references a missing budget policy",
            )
        })
}

fn expand_tool_references(
    profile_set: &ModelRuntimeProfileSet,
    profile: &ModelRuntimeProfile,
) -> Result<Vec<ModelNamedResourceReference>, ModelRuntimeProfileError> {
    let mut expanded = profile.tools().to_vec();
    for reference in profile.toolsets() {
        let toolset = profile_set
            .toolsets()
            .iter()
            .find(|toolset| toolset.toolset_id() == reference.name())
            .ok_or_else(|| {
                error(
                    ModelRuntimeProfileIssue::MissingReference,
                    FIELD_TOOLSET,
                    "resolved profile references a missing toolset",
                )
            })?;
        for tool_name in toolset.tool_names() {
            if let Some(new_reference) =
                merge_tool_reference(expanded.as_mut_slice(), tool_name, reference.required())
            {
                expanded.push(new_reference);
            }
        }
    }
    expanded.sort_by(|left, right| left.name.cmp(&right.name));
    assert!(expanded.iter().all(|reference| !reference.name.is_empty()));
    assert!(expanded.windows(2).all(|pair| pair[0].name <= pair[1].name));
    Ok(expanded)
}

fn merge_tool_reference(
    references: &mut [ModelNamedResourceReference],
    tool_name: &str,
    required: bool,
) -> Option<ModelNamedResourceReference> {
    if let Some(existing) = references
        .iter_mut()
        .find(|reference| reference.name == tool_name)
    {
        existing.required |= required;
        return None;
    }
    Some(ModelNamedResourceReference {
        name: tool_name.to_string(),
        required,
    })
}

#[derive(Clone, Copy)]
struct ResolutionHashInput<'a> {
    profile: &'a ModelRuntimeProfile,
    target: &'a ModelRuntimeTarget,
    specificity: ModelProfileSpecificity,
    context_budget: &'a ModelRuntimeBudgetPolicy,
    tool_budget: &'a ModelRuntimeBudgetPolicy,
    tools: &'a [ModelNamedResourceReference],
}

fn resolution_hash(input: ResolutionHashInput<'_>) -> String {
    let mut parts = vec![
        input.profile.profile_hash().to_string(),
        input.target.target_hash().to_string(),
        input.specificity.as_str().to_string(),
        input.context_budget.policy_id().to_string(),
        input.tool_budget.policy_id().to_string(),
    ];
    for tool in input.tools {
        parts.push(format!("tool:{}:{}", tool.name(), tool.required()));
    }
    blake3_parts(MODEL_RUNTIME_RESOLUTION_HASH_DOMAIN, parts.as_slice())
}
