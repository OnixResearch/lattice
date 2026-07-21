use super::super::*;
use super::support::{ALTERNATE_HASH, HASH_HEX_LENGTH};
use serde::Deserialize;

const GENERATED_JSON: &str =
    include_str!("../../../../../config/generated/model-runtime-profiles.json");

#[derive(Clone, Debug, Deserialize)]
struct GeneratedSet {
    schema: String,
    enablement: String,
    explicit_default_profile_id: String,
    budget_policies: Vec<GeneratedBudget>,
    toolsets: Vec<GeneratedToolset>,
    profiles: Vec<GeneratedProfile>,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedBudget {
    policy_id: String,
    max_context_units: u32,
    max_tool_calls: u32,
    max_parallel_tool_calls: u16,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedToolset {
    toolset_id: String,
    tool_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedProfile {
    profile_id: String,
    binding: GeneratedBinding,
    source_scope: String,
    trust_status: String,
    admission_status: String,
    project_profile_admitted: bool,
    prompt_mode_ref: String,
    prompt_sources: Vec<GeneratedPromptReference>,
    skills: Vec<GeneratedSkillReference>,
    memory_access_mode: String,
    memory_policy_hash: String,
    tools: Vec<GeneratedNamedReference>,
    toolsets: Vec<GeneratedNamedReference>,
    tool_calling_mode: String,
    context_budget_policy: String,
    tool_budget_policy: String,
    policy_hash: String,
    profile_hash: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedBinding {
    kind: String,
    provider_id: String,
    model_id: String,
    router_role: String,
    capability_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedPromptReference {
    label: String,
    required: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedSkillReference {
    name: String,
    posture: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GeneratedNamedReference {
    name: String,
    required: bool,
}

#[test]
fn generated_profile_export_matches_validated_runtime_dto_meaning() {
    // r[verify agent_harness_runtime.model_runtime_profiles.contract.nickel]
    let generated: GeneratedSet =
        serde_json::from_str(GENERATED_JSON).expect("generated profile JSON");
    let validated = validate_generated(&generated).expect("generated profile set validates");

    assert_eq!(validated.profiles().len(), generated.profiles.len());
    assert_eq!(
        validated.explicit_default_profile_id(),
        Some(generated.explicit_default_profile_id.as_str())
    );
    for (profile, row) in validated.profiles().iter().zip(generated.profiles.iter()) {
        assert_eq!(profile.profile_id(), row.profile_id);
        assert_eq!(profile.profile_hash(), row.profile_hash);
        assert_eq!(
            profile.memory_access_mode().as_str(),
            row.memory_access_mode
        );
        assert_eq!(profile.memory_policy_hash(), row.memory_policy_hash);
        assert_eq!(profile.profile_hash().len(), HASH_HEX_LENGTH);
    }
}

#[test]
fn drifted_generated_profile_identity_fails_closed() {
    let mut generated: GeneratedSet =
        serde_json::from_str(GENERATED_JSON).expect("generated profile JSON");
    generated.profiles[0].profile_hash = ALTERNATE_HASH.to_string();

    let error = validate_generated(&generated).expect_err("drifted generated profile hash");
    assert_eq!(
        error.issue(),
        ModelRuntimeProfileIssue::StaleProfileIdentity
    );
}

fn validate_generated(
    generated: &GeneratedSet,
) -> Result<ModelRuntimeProfileSet, ModelRuntimeProfileError> {
    let budgets: Vec<ModelRuntimeBudgetPolicyDraft<'_>> = generated
        .budget_policies
        .iter()
        .map(|policy| ModelRuntimeBudgetPolicyDraft {
            policy_id: policy.policy_id.as_str(),
            max_context_units: policy.max_context_units,
            max_tool_calls: policy.max_tool_calls,
            max_parallel_tool_calls: policy.max_parallel_tool_calls,
        })
        .collect();
    let tool_names: Vec<Vec<&str>> = generated
        .toolsets
        .iter()
        .map(|toolset| toolset.tool_names.iter().map(String::as_str).collect())
        .collect();
    let toolsets: Vec<ModelToolsetDraft<'_>> = generated
        .toolsets
        .iter()
        .enumerate()
        .map(|(index, toolset)| ModelToolsetDraft {
            toolset_id: toolset.toolset_id.as_str(),
            tool_names: tool_names[index].as_slice(),
        })
        .collect();
    let prompt_refs = generated_prompt_refs(generated);
    let skill_refs = generated_skill_refs(generated);
    let tool_refs = generated_named_refs(generated, |profile| profile.tools.as_slice());
    let toolset_refs = generated_named_refs(generated, |profile| profile.toolsets.as_slice());
    let profiles: Vec<ModelRuntimeProfileDraft<'_>> = generated
        .profiles
        .iter()
        .enumerate()
        .map(|(index, profile)| {
            generated_profile_draft(
                profile,
                GeneratedProfileResources {
                    prompt_sources: prompt_refs[index].as_slice(),
                    skills: skill_refs[index].as_slice(),
                    tools: tool_refs[index].as_slice(),
                    toolsets: toolset_refs[index].as_slice(),
                },
            )
        })
        .collect();
    ModelRuntimeProfileSet::validate(&ModelRuntimeProfileSetDraft {
        schema: generated.schema.as_str(),
        enablement: parse_enablement(generated.enablement.as_str()),
        explicit_default_profile_id: Some(generated.explicit_default_profile_id.as_str()),
        budget_policies: budgets.as_slice(),
        toolsets: toolsets.as_slice(),
        profiles: profiles.as_slice(),
    })
}

fn generated_prompt_refs(
    generated: &GeneratedSet,
) -> Vec<Vec<ModelPromptSourceReferenceDraft<'_>>> {
    generated
        .profiles
        .iter()
        .map(|profile| {
            profile
                .prompt_sources
                .iter()
                .map(|reference| ModelPromptSourceReferenceDraft {
                    label: reference.label.as_str(),
                    required: reference.required,
                })
                .collect()
        })
        .collect()
}

fn generated_skill_refs(generated: &GeneratedSet) -> Vec<Vec<ModelSkillReferenceDraft<'_>>> {
    generated
        .profiles
        .iter()
        .map(|profile| {
            profile
                .skills
                .iter()
                .map(|reference| ModelSkillReferenceDraft {
                    name: reference.name.as_str(),
                    posture: parse_skill_posture(reference.posture.as_str()),
                })
                .collect()
        })
        .collect()
}

fn generated_named_refs<'a, F>(
    generated: &'a GeneratedSet,
    select: F,
) -> Vec<Vec<ModelNamedResourceReferenceDraft<'a>>>
where
    F: Fn(&'a GeneratedProfile) -> &'a [GeneratedNamedReference],
{
    generated
        .profiles
        .iter()
        .map(|profile| {
            select(profile)
                .iter()
                .map(|reference| ModelNamedResourceReferenceDraft {
                    name: reference.name.as_str(),
                    required: reference.required,
                })
                .collect()
        })
        .collect()
}

#[derive(Clone, Copy)]
struct GeneratedProfileResources<'a> {
    prompt_sources: &'a [ModelPromptSourceReferenceDraft<'a>],
    skills: &'a [ModelSkillReferenceDraft<'a>],
    tools: &'a [ModelNamedResourceReferenceDraft<'a>],
    toolsets: &'a [ModelNamedResourceReferenceDraft<'a>],
}

fn generated_profile_draft<'a>(
    profile: &'a GeneratedProfile,
    resources: GeneratedProfileResources<'a>,
) -> ModelRuntimeProfileDraft<'a> {
    ModelRuntimeProfileDraft {
        profile_id: profile.profile_id.as_str(),
        binding: parse_binding(&profile.binding),
        source_scope: parse_source_scope(profile.source_scope.as_str()),
        trust_status: parse_trust(profile.trust_status.as_str()),
        admission_status: parse_admission(profile.admission_status.as_str()),
        project_profile_admitted: profile.project_profile_admitted,
        prompt_mode_ref: profile.prompt_mode_ref.as_str(),
        prompt_sources: resources.prompt_sources,
        skills: resources.skills,
        memory_access_mode: parse_memory_access(profile.memory_access_mode.as_str()),
        memory_policy_hash: profile.memory_policy_hash.as_str(),
        tools: resources.tools,
        toolsets: resources.toolsets,
        tool_calling_mode: parse_tool_mode(profile.tool_calling_mode.as_str()),
        context_budget_policy: profile.context_budget_policy.as_str(),
        tool_budget_policy: profile.tool_budget_policy.as_str(),
        policy_hash: profile.policy_hash.as_str(),
        profile_hash: profile.profile_hash.as_str(),
    }
}

fn parse_binding(binding: &GeneratedBinding) -> ModelRuntimeBindingDraft<'_> {
    match binding.kind.as_str() {
        "exact" => ModelRuntimeBindingDraft::Exact {
            provider_id: binding.provider_id.as_str(),
            model_id: binding.model_id.as_str(),
            router_role: (!binding.router_role.is_empty()).then_some(binding.router_role.as_str()),
            capability_identity: binding.capability_identity.as_str(),
        },
        "default" => ModelRuntimeBindingDraft::Default,
        other => panic!("unsupported generated binding: {other}"),
    }
}

fn parse_enablement(value: &str) -> ModelProfileEnablementPolicy {
    match value {
        "required" => ModelProfileEnablementPolicy::Required,
        "compatibility_disabled" => ModelProfileEnablementPolicy::CompatibilityDisabled,
        other => panic!("unsupported generated enablement: {other}"),
    }
}

fn parse_source_scope(value: &str) -> ModelProfileSourceScope {
    match value {
        "host" => ModelProfileSourceScope::Host,
        "project" => ModelProfileSourceScope::Project,
        other => panic!("unsupported generated source scope: {other}"),
    }
}

fn parse_trust(value: &str) -> ModelProfileTrustStatus {
    match value {
        "trusted" => ModelProfileTrustStatus::Trusted,
        "untrusted" => ModelProfileTrustStatus::Untrusted,
        other => panic!("unsupported generated trust status: {other}"),
    }
}

fn parse_admission(value: &str) -> ModelProfileAdmissionStatus {
    match value {
        "admitted" => ModelProfileAdmissionStatus::Admitted,
        "denied" => ModelProfileAdmissionStatus::Denied,
        other => panic!("unsupported generated admission status: {other}"),
    }
}

fn parse_skill_posture(value: &str) -> ModelSkillResourcePosture {
    match value {
        "metadata_only" => ModelSkillResourcePosture::MetadataOnly,
        "selectable" => ModelSkillResourcePosture::Selectable,
        "required_selected" => ModelSkillResourcePosture::RequiredSelected,
        other => panic!("unsupported generated skill posture: {other}"),
    }
}

fn parse_memory_access(value: &str) -> ModelMemoryAccessMode {
    match value {
        "disabled" => ModelMemoryAccessMode::Disabled,
        "read_only" => ModelMemoryAccessMode::ReadOnly,
        "read_write" => ModelMemoryAccessMode::ReadWrite,
        other => panic!("unsupported generated memory access mode: {other}"),
    }
}

fn parse_tool_mode(value: &str) -> NativeToolCallingMode {
    match value {
        "disabled" => NativeToolCallingMode::Disabled,
        "optional" => NativeToolCallingMode::Optional,
        "capability_required" => NativeToolCallingMode::CapabilityRequired,
        other => panic!("unsupported generated tool mode: {other}"),
    }
}
