use super::super::*;

pub(super) const POLICY_HASH: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const CAPABILITY_HASH: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const CATALOG_HASH: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
pub(super) const MEMORY_POLICY_HASH: &str =
    "9999999999999999999999999999999999999999999999999999999999999999";
pub(super) const PROMPT_HASH: &str =
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
pub(super) const SKILL_HASH: &str =
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
pub(super) const ALTERNATE_HASH: &str =
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
pub(super) const ZERO_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
pub(super) const PROVIDER_ID: &str = "mock";
pub(super) const PROVIDER_TRANSPORT_ID: &str = "mock-responses-v1";
pub(super) const MODEL_ID: &str = "mock-tools";
pub(super) const MODEL_DIALECT_ID: &str = "mock-tools-v1";
pub(super) const ALTERNATE_MODEL_DIALECT_ID: &str = "generic-responses-v1";
pub(super) const ALTERNATE_PROVIDER_ID: &str = "other";
pub(super) const ALTERNATE_PROVIDER_TRANSPORT_ID: &str = "other-responses-v1";
pub(super) const ALTERNATE_MODEL_ID: &str = "other-model";
pub(super) const ALTERNATE_TARGET_LABEL: &str = "other:other-model";
pub(super) const ROUTER_ROLE: &str = "balanced";
pub(super) const PROFILE_ID: &str = "mock-tools-profile";
pub(super) const PROMPT_MODE: &str = "standard";
pub(super) const MODEL_PROMPT: &str = "model-guidance";
pub(super) const MANDATORY_PROMPT: &str = "host-policy";
pub(super) const SKILL_NAME: &str = "proof-search";
pub(super) const TOOL_NAME: &str = "inspect_context";
pub(super) const TOOL_CAPABILITY: &str = "observe_read";
pub(super) const EXECUTOR_BINDING: &str = "executor.inspect";
pub(super) const BUDGET_ID: &str = "standard-budget";
pub(super) const EXPECTED_TWO: usize = 2;
pub(super) const HASH_HEX_LENGTH: usize = 64;
pub(super) const MODULE_FROZEN_HASH: &str =
    "1212121212121212121212121212121212121212121212121212121212121212";
pub(super) const MODULE_SOURCE: &str = "(define (proof-search-helper value) value)";
pub(super) const PROMPT_BUDGET_UNITS: u32 = 64;
pub(super) const PROFILE_CONTEXT_UNITS: u32 = 1_024;
pub(super) const TOOL_CALL_LIMIT: u32 = 8;
pub(super) const TOOL_PARALLEL_LIMIT: u16 = 2;
pub(super) const TOOL_VISIBLE_CHARS: u32 = 1_024;
pub(super) const TOOL_OUTPUT_BYTES: u64 = 4_096;
pub(super) const PROMPT_SOURCE_UNITS: u32 = 8;
pub(super) const HOST_PRIORITY: u16 = 10;
pub(super) const MODEL_PRIORITY: u16 = 20;

const EMPTY_PROMPTS: &[ModelPromptSourceReferenceDraft<'static>] = &[];
const EMPTY_SKILLS: &[ModelSkillReferenceDraft<'static>] = &[];
const EMPTY_NAMED_REFS: &[ModelNamedResourceReferenceDraft<'static>] = &[];
const PROMPT_REFS: &[ModelPromptSourceReferenceDraft<'static>] =
    &[ModelPromptSourceReferenceDraft {
        label: MODEL_PROMPT,
        required: false,
    }];
const DIRECT_TOOL_REFS: &[ModelNamedResourceReferenceDraft<'static>] =
    &[ModelNamedResourceReferenceDraft {
        name: TOOL_NAME,
        required: true,
    }];
const SECTION_ORDER: &[animus_adapters::PromptSectionKind] = &[
    animus_adapters::PromptSectionKind::Policy,
    animus_adapters::PromptSectionKind::Context,
];
const TARGET_LABELS: &[&str] = &["mock:mock-tools"];
const PROMPT_MODES: &[&str] = &[PROMPT_MODE];
const PROMPT_LABELS: &[&str] = &[MANDATORY_PROMPT, MODEL_PROMPT];
const SKILL_LABELS: &[&str] = &[SKILL_NAME];
const MEMORY_POLICY_HASHES: &[&str] = &[MEMORY_POLICY_HASH];
const TOOL_LABELS: &[&str] = &[TOOL_NAME];
const TOOL_CAPABILITIES: &[&str] = &[TOOL_CAPABILITY];
const ARGUMENT_NAMES: &[&str] = &["query"];
const PROMPT_GUIDELINES: &[&str] = &["Use only bounded host-supplied context."];
const SKILL_TAGS: &[&str] = &["proof"];
const NO_RELATED_SKILLS: &[&str] = &[];

pub(super) fn composition_profile_set(
    mode: NativeToolCallingMode,
    skill_posture: ModelSkillResourcePosture,
) -> ModelRuntimeProfileSet {
    let skill_refs = [ModelSkillReferenceDraft {
        name: SKILL_NAME,
        posture: skill_posture,
    }];
    let tools = if mode == NativeToolCallingMode::Disabled {
        EMPTY_NAMED_REFS
    } else {
        DIRECT_TOOL_REFS
    };
    let provisional = composition_profile_draft(mode, &skill_refs, tools, ZERO_HASH);
    let profile_hash =
        blake3_model_runtime_profile_identity(&provisional).expect("valid profile hash material");
    let profile = composition_profile_draft(mode, &skill_refs, tools, &profile_hash);
    validate_set(&[profile], None)
}

pub(super) fn composition_profile_draft<'a>(
    mode: NativeToolCallingMode,
    skills: &'a [ModelSkillReferenceDraft<'a>],
    tools: &'a [ModelNamedResourceReferenceDraft<'a>],
    profile_hash: &'a str,
) -> ModelRuntimeProfileDraft<'a> {
    ModelRuntimeProfileDraft {
        profile_id: PROFILE_ID,
        dialect_id: MODEL_DIALECT_ID,
        binding: ModelRuntimeBindingDraft::Exact {
            provider_id: PROVIDER_ID,
            model_id: MODEL_ID,
            router_role: None,
            capability_identity: CAPABILITY_HASH,
        },
        source_scope: ModelProfileSourceScope::Host,
        trust_status: ModelProfileTrustStatus::Trusted,
        admission_status: ModelProfileAdmissionStatus::Admitted,
        project_profile_admitted: false,
        prompt_mode_ref: PROMPT_MODE,
        prompt_sources: PROMPT_REFS,
        skills,
        memory_access_mode: ModelMemoryAccessMode::ReadOnly,
        memory_policy_hash: MEMORY_POLICY_HASH,
        tools,
        toolsets: EMPTY_NAMED_REFS,
        tool_calling_mode: mode,
        context_budget_policy: BUDGET_ID,
        tool_budget_policy: BUDGET_ID,
        policy_hash: POLICY_HASH,
        profile_hash,
    }
}

pub(super) fn resolver_profile_set() -> ModelRuntimeProfileSet {
    const ROLE_PROFILE: &str = "mock-role-profile";
    const MODEL_PROFILE: &str = "mock-model-profile";
    const DEFAULT_PROFILE: &str = "explicit-default-profile";
    let role_binding = ModelRuntimeBindingDraft::Exact {
        provider_id: PROVIDER_ID,
        model_id: MODEL_ID,
        router_role: Some(ROUTER_ROLE),
        capability_identity: CAPABILITY_HASH,
    };
    let model_binding = ModelRuntimeBindingDraft::Exact {
        provider_id: PROVIDER_ID,
        model_id: MODEL_ID,
        router_role: None,
        capability_identity: CAPABILITY_HASH,
    };
    let role_provisional = resolver_profile_draft(ROLE_PROFILE, role_binding, ZERO_HASH);
    let model_provisional = resolver_profile_draft(MODEL_PROFILE, model_binding, ZERO_HASH);
    let default_provisional = resolver_profile_draft(
        DEFAULT_PROFILE,
        ModelRuntimeBindingDraft::Default,
        ZERO_HASH,
    );
    let role_hash = blake3_model_runtime_profile_identity(&role_provisional).expect("role hash");
    let model_hash = blake3_model_runtime_profile_identity(&model_provisional).expect("model hash");
    let default_hash =
        blake3_model_runtime_profile_identity(&default_provisional).expect("default hash");
    let profiles = [
        resolver_profile_draft(ROLE_PROFILE, role_binding, &role_hash),
        resolver_profile_draft(MODEL_PROFILE, model_binding, &model_hash),
        resolver_profile_draft(
            DEFAULT_PROFILE,
            ModelRuntimeBindingDraft::Default,
            &default_hash,
        ),
    ];
    validate_set(&profiles, Some(DEFAULT_PROFILE))
}

fn resolver_profile_draft<'a>(
    profile_id: &'a str,
    binding: ModelRuntimeBindingDraft<'a>,
    profile_hash: &'a str,
) -> ModelRuntimeProfileDraft<'a> {
    ModelRuntimeProfileDraft {
        profile_id,
        dialect_id: MODEL_DIALECT_ID,
        binding,
        source_scope: ModelProfileSourceScope::Host,
        trust_status: ModelProfileTrustStatus::Trusted,
        admission_status: ModelProfileAdmissionStatus::Admitted,
        project_profile_admitted: false,
        prompt_mode_ref: PROMPT_MODE,
        prompt_sources: EMPTY_PROMPTS,
        skills: EMPTY_SKILLS,
        memory_access_mode: ModelMemoryAccessMode::Disabled,
        memory_policy_hash: MEMORY_POLICY_HASH,
        tools: EMPTY_NAMED_REFS,
        toolsets: EMPTY_NAMED_REFS,
        tool_calling_mode: NativeToolCallingMode::Disabled,
        context_budget_policy: BUDGET_ID,
        tool_budget_policy: BUDGET_ID,
        policy_hash: POLICY_HASH,
        profile_hash,
    }
}

fn validate_set(
    profiles: &[ModelRuntimeProfileDraft<'_>],
    default_profile_id: Option<&str>,
) -> ModelRuntimeProfileSet {
    validate_set_result(profiles, default_profile_id).expect("valid profile set")
}

pub(super) fn validate_set_result(
    profiles: &[ModelRuntimeProfileDraft<'_>],
    default_profile_id: Option<&str>,
) -> Result<ModelRuntimeProfileSet, ModelRuntimeProfileError> {
    let budgets = [ModelRuntimeBudgetPolicyDraft {
        policy_id: BUDGET_ID,
        max_context_units: PROFILE_CONTEXT_UNITS,
        max_tool_calls: TOOL_CALL_LIMIT,
        max_parallel_tool_calls: TOOL_PARALLEL_LIMIT,
    }];
    ModelRuntimeProfileSet::validate(&ModelRuntimeProfileSetDraft {
        schema: MODEL_RUNTIME_PROFILE_SCHEMA,
        enablement: ModelProfileEnablementPolicy::Required,
        explicit_default_profile_id: default_profile_id,
        budget_policies: &budgets,
        toolsets: &[],
        profiles,
    })
}

pub(super) fn target(router_role: Option<&str>) -> ModelRuntimeTargetDraft<'_> {
    ModelRuntimeTargetDraft {
        provider_id: PROVIDER_ID,
        provider_transport_id: PROVIDER_TRANSPORT_ID,
        model_id: MODEL_ID,
        model_dialect_id: MODEL_DIALECT_ID,
        router_role,
        capability_identity: CAPABILITY_HASH,
    }
}

pub(super) fn resolved_composition_profile(
    set: &ModelRuntimeProfileSet,
) -> ResolvedModelRuntimeProfile {
    match resolve_model_runtime_profile(set, target(None)).expect("profile resolution") {
        ModelRuntimeProfileResolution::Resolved(resolved) => *resolved,
        ModelRuntimeProfileResolution::CompatibilityDisabled => {
            panic!("required fixture must resolve")
        }
    }
}

pub(super) fn prompt_sources() -> Vec<animus_adapters::PromptSource> {
    vec![
        animus_adapters::PromptSource::validate(&animus_adapters::PromptSourceDraft {
            kind: animus_adapters::PromptSourceKind::PolicyProfile,
            scope: animus_adapters::PromptResourceScope::Host,
            trust_status: animus_adapters::PromptSourceTrustStatus::Trusted,
            admission_status: animus_adapters::PromptSourceAdmissionStatus::Admitted,
            requirement: animus_adapters::PromptSourceRequirement::Required,
            priority: animus_adapters::PromptSourcePriority::new(HOST_PRIORITY),
            budget_class: animus_adapters::PromptBudgetClass::Tiny,
            redaction_class: animus_adapters::PromptRedactionClass::Public,
            label: MANDATORY_PROMPT,
            section: animus_adapters::PromptSectionKind::Policy,
            body_hash: PROMPT_HASH,
            requested_units: PROMPT_SOURCE_UNITS,
        })
        .expect("mandatory prompt source"),
        animus_adapters::PromptSource::validate(&animus_adapters::PromptSourceDraft {
            kind: animus_adapters::PromptSourceKind::HostContext,
            scope: animus_adapters::PromptResourceScope::Host,
            trust_status: animus_adapters::PromptSourceTrustStatus::Trusted,
            admission_status: animus_adapters::PromptSourceAdmissionStatus::Admitted,
            requirement: animus_adapters::PromptSourceRequirement::Optional,
            priority: animus_adapters::PromptSourcePriority::new(MODEL_PRIORITY),
            budget_class: animus_adapters::PromptBudgetClass::Normal,
            redaction_class: animus_adapters::PromptRedactionClass::Public,
            label: MODEL_PROMPT,
            section: animus_adapters::PromptSectionKind::Context,
            body_hash: ALTERNATE_HASH,
            requested_units: PROMPT_SOURCE_UNITS,
        })
        .expect("model prompt source"),
    ]
}

pub(super) fn prompt_policy() -> animus_adapters::PromptAssemblyPolicy {
    animus_adapters::PromptAssemblyPolicy::validate(&animus_adapters::PromptAssemblyPolicyDraft {
        mode: animus_adapters::PromptAssemblyMode::AdmittedSources,
        section_order: SECTION_ORDER,
        section_budget_units: PROMPT_BUDGET_UNITS,
        policy_hash: POLICY_HASH,
    })
    .expect("prompt policy")
}

pub(super) fn skill_manifest(
    trust_status: animus_adapters::SkillManifestTrustStatus,
) -> animus_adapters::SkillManifest {
    animus_adapters::SkillManifest::validate(&animus_adapters::SkillManifestDraft {
        name: Some(SKILL_NAME),
        description: Some("Search bounded proof families."),
        version: Some("1.0.0"),
        category: Some("reasoning"),
        tags: SKILL_TAGS,
        related_skills: NO_RELATED_SKILLS,
        provenance: Some("host-reviewed"),
        trust_status: Some(trust_status.as_str()),
        content_identity: Some(SKILL_HASH),
    })
    .expect("skill manifest")
}

pub(super) fn loaded_skill_body(
    manifest: &animus_adapters::SkillManifest,
) -> animus_adapters::SkillLoadedBody {
    let body = [animus_adapters::SkillBodyDraft {
        name: SKILL_NAME,
        content_identity: SKILL_HASH,
        body: "Frame, diversify, audit, and validate.",
    }];
    animus_adapters::build_skill_disclosure_context(animus_adapters::SkillDisclosureRequest {
        manifests: core::slice::from_ref(manifest),
        loaded_bodies: &body,
        selected_name: Some(SKILL_NAME),
    })
    .selected_body()
    .cloned()
    .expect("trusted matching skill body")
}

pub(super) fn skill_module_descriptor(
    manifest_identity: &str,
) -> animus_adapter_rlm_kernel::SteelSkillModuleDescriptor {
    let source_identity =
        animus_adapter_rlm_kernel::steel_skill_module_source_identity(MODULE_SOURCE);
    animus_adapter_rlm_kernel::SteelSkillModuleDescriptor::validate(
        &animus_adapter_rlm_kernel::SteelSkillModuleDescriptorDraft {
            schema: animus_adapter_rlm_kernel::STEEL_SKILL_MODULE_SCHEMA,
            name: SKILL_NAME,
            version: "v1",
            manifest_identity,
            frozen_source_identity: MODULE_FROZEN_HASH,
            source_identity: source_identity.as_str(),
            source: MODULE_SOURCE,
            exports: &["proof-search-helper"],
            dependencies: &[],
            capability_requests: &[],
            trust: animus_adapter_rlm_kernel::SteelSkillModuleTrust::Trusted,
            bounds: animus_adapter_rlm_kernel::SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        },
    )
    .expect("skill module descriptor")
}

pub(super) fn capability_matrix(tool_calls: bool) -> crate::ProviderModelCapabilityMatrix {
    crate::ProviderModelCapabilityMatrix {
        schema_identity: "animus.model-capability-matrix.v1".to_string(),
        provider_id: PROVIDER_ID.to_string(),
        model_id: MODEL_ID.to_string(),
        roles: vec![animus_adapters::CanonicalModelRole::User],
        content_parts: vec![crate::ProviderModelContentPartClass::Text],
        tool_calls,
        structured_outputs: Vec::new(),
        stream_channels: Vec::new(),
        reasoning_controls: Vec::new(),
        context_budget: PROFILE_CONTEXT_UNITS,
        output_budget: PROFILE_CONTEXT_UNITS,
        privacy: crate::ProviderPrivacyPosture::LocalOnly,
    }
}

pub(super) fn capability_decision() -> crate::ModelCapabilityNegotiationDecision {
    crate::ModelCapabilityNegotiationDecision {
        status: crate::ModelCapabilityDecisionStatus::Admitted,
        provider_id: PROVIDER_ID.to_string(),
        model_id: MODEL_ID.to_string(),
        evidence: vec!["capability-matrix-admitted".to_string()],
        downgrades: Vec::new(),
        issues: Vec::new(),
    }
}

pub(super) fn memory_policy() -> animus_adapters::OptInMemoryPolicy {
    let controls = animus_adapters::MemoryScopeControls {
        user: animus_adapters::MemoryOptInControl::Enabled,
        project: animus_adapters::MemoryOptInControl::Enabled,
        thread: animus_adapters::MemoryOptInControl::Enabled,
    };
    animus_adapters::OptInMemoryPolicy::validate(&animus_adapters::OptInMemoryPolicyDraft {
        use_controls: controls,
        generation_controls: controls,
        policy_hash: MEMORY_POLICY_HASH,
    })
    .expect("memory policy")
}

pub(super) fn tool_catalog() -> crate::tool_executor_kit::ToolCatalog {
    let metadata =
        animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
            name: Some(TOOL_NAME),
            label: Some("Inspect context"),
            description: Some("Read bounded host-supplied context."),
            parameter_schema_identity: Some("schema:animus.tool.input.context.v1"),
            prompt_snippet: Some("Inspect bounded context."),
            prompt_guidelines: PROMPT_GUIDELINES,
            source_identity: Some("animus.tool.inspect-context.v1"),
            provenance: Some("host-reviewed"),
            capability_labels: TOOL_CAPABILITIES,
            result_metadata_shape: Some("schema:animus.tool.output.context.v1"),
            status: animus_adapters::tool::DefinitionStatus::Active,
        })
        .expect("tool metadata");
    let output_policy = crate::tool_executor_kit::ToolExecutorOutputPolicy::new(
        TOOL_VISIBLE_CHARS,
        TOOL_OUTPUT_BYTES,
        true,
    )
    .expect("bounded output policy");
    let definition = crate::tool_executor_kit::ToolDefinition::from_draft(
        crate::tool_executor_kit::ToolDefinitionDraft {
            metadata,
            executor_binding: EXECUTOR_BINDING,
            argument_names: ARGUMENT_NAMES,
            mutation_posture: crate::tool_executor_kit::ToolMutationPosture::ReadOnly,
            output_policy,
            approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved,
        },
    )
    .expect("executor-bound tool");
    crate::tool_executor_kit::ToolCatalog::new(vec![definition]).expect("tool catalog")
}

pub(super) fn executor_bindings() -> Vec<crate::tool_executor_kit::ToolExecutorBinding> {
    vec![
        crate::tool_executor_kit::ToolExecutorBinding::new(
            EXECUTOR_BINDING,
            crate::tool_executor_kit::ToolExecutorReadiness::Ready,
        )
        .expect("ready executor binding"),
    ]
}

pub(super) fn ceiling() -> ModelResourceCeiling<'static> {
    ModelResourceCeiling {
        ceiling_label: "host-policy",
        model_targets: TARGET_LABELS,
        prompt_modes: PROMPT_MODES,
        prompt_sources: PROMPT_LABELS,
        skills: SKILL_LABELS,
        memory_policy_hashes: MEMORY_POLICY_HASHES,
        tools: TOOL_LABELS,
        tool_capabilities: TOOL_CAPABILITIES,
    }
}

pub(crate) fn admitted_dispatch_plan() -> crate::ModelDispatchPlan {
    let request =
        animus_adapters::ModelRequest::new("bounded test prompt").expect("valid model request");
    let registry = crate::RuntimeCapabilityRegistry::compatibility_model_tool()
        .expect("compatibility capability registry");
    crate::plan_model_dispatch(crate::ModelDispatchPlanRequest {
        request: &request,
        required_capability: crate::RuntimeCapabilityDescriptorDraft::default_model(),
        registry: &registry,
        policy: crate::ModelDispatchPolicyDraft::compatibility_defaults(),
    })
}

pub(crate) fn composed_plan(
    mode: NativeToolCallingMode,
    posture: ModelSkillResourcePosture,
    native_tool_calls: bool,
) -> Result<(ResolvedModelRuntimeProfile, ModelInvocationPlan), ModelRuntimeProfileError> {
    let set = composition_profile_set(mode, posture);
    let resolved = resolved_composition_profile(&set);
    let prompts = prompt_sources();
    let prompt_policy = prompt_policy();
    let manifest = skill_manifest(animus_adapters::SkillManifestTrustStatus::Trusted);
    let body = loaded_skill_body(&manifest);
    let skill_resources = [ModelSkillCompositionResource {
        manifest: &manifest,
        selected_body: Some(&body),
        selection_profile_hash: Some(resolved.profile().profile_hash()),
        selection_target_hash: Some(resolved.target().target_hash()),
        invocation_admitted: true,
        dependencies_admitted: true,
        budget_admitted: true,
    }];
    let memory_policy = memory_policy();
    let catalog = tool_catalog();
    let executors = executor_bindings();
    let capabilities = capability_matrix(native_tool_calls);
    let capability_decision = capability_decision();
    let ceilings = [ceiling()];
    let plan = compose_model_invocation_plan(ModelInvocationCompositionInput {
        resolved: &resolved,
        capabilities: &capabilities,
        capability_decision: &capability_decision,
        prompt_sources: &prompts,
        prompt_policy: &prompt_policy,
        skills: &skill_resources,
        memory_policy: Some(&memory_policy),
        tool_catalog: &catalog,
        executor_bindings: &executors,
        tool_catalog_identity: CATALOG_HASH,
        authority: ModelInvocationAuthority {
            ceilings: &ceilings,
            approved_tools: &[],
            policy_hash: POLICY_HASH,
        },
    })?;
    Ok((resolved, plan))
}

pub(crate) fn role_composed_plan(
    profile: &crate::agent_roles::AgentRoleProfile,
) -> Result<crate::agent_roles::AgentRoleBoundModelPlan, crate::agent_roles::AgentRoleError> {
    let output_contracts = [profile.output_contract()];
    let budget_policies = [profile.budget_policy_ref()];
    let workspace_scopes = profile
        .workspace_scopes()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    role_composed_plan_with_admission(
        profile,
        &output_contracts,
        &budget_policies,
        workspace_scopes.as_slice(),
    )
}

pub(crate) fn role_composed_plan_with_admission(
    profile: &crate::agent_roles::AgentRoleProfile,
    admitted_output_contracts: &[&str],
    admitted_budget_policies: &[&str],
    admitted_workspace_scopes: &[&str],
) -> Result<crate::agent_roles::AgentRoleBoundModelPlan, crate::agent_roles::AgentRoleError> {
    let set = composition_profile_set(
        NativeToolCallingMode::CapabilityRequired,
        ModelSkillResourcePosture::MetadataOnly,
    );
    let resolved = resolved_composition_profile(&set);
    let prompts = prompt_sources();
    let prompt_policy = prompt_policy();
    let manifest = skill_manifest(animus_adapters::SkillManifestTrustStatus::Trusted);
    let skill_resources = [ModelSkillCompositionResource {
        manifest: &manifest,
        selected_body: None,
        selection_profile_hash: None,
        selection_target_hash: None,
        invocation_admitted: true,
        dependencies_admitted: true,
        budget_admitted: true,
    }];
    let memory_policy = memory_policy();
    let catalog = tool_catalog();
    let executors = executor_bindings();
    let capabilities = capability_matrix(true);
    let capability_decision = capability_decision();
    let ceilings = [ceiling()];
    crate::agent_roles::compose_agent_role_model_plan(
        crate::agent_roles::AgentRoleModelCompositionInput {
            profile,
            model: ModelInvocationCompositionInput {
                resolved: &resolved,
                capabilities: &capabilities,
                capability_decision: &capability_decision,
                prompt_sources: &prompts,
                prompt_policy: &prompt_policy,
                skills: &skill_resources,
                memory_policy: Some(&memory_policy),
                tool_catalog: &catalog,
                executor_bindings: &executors,
                tool_catalog_identity: CATALOG_HASH,
                authority: ModelInvocationAuthority {
                    ceilings: &ceilings,
                    approved_tools: &[],
                    policy_hash: POLICY_HASH,
                },
            },
            admitted_output_contracts,
            admitted_budget_policies,
            admitted_workspace_scopes,
        },
    )
}

pub(crate) fn tool_call_draft(plan: &ModelInvocationPlan) -> ModelToolCallBindingDraft<'_> {
    ModelToolCallBindingDraft {
        tool_name: TOOL_NAME,
        invocation_plan_hash: plan.plan_hash(),
        target_hash: plan.target().target_hash(),
        profile_hash: plan.profile_hash(),
        prompt_plan_hash: plan.prompt_plan_hash(),
        skill_plan_hash: plan.skill_plan_hash(),
        memory_plan_hash: plan.memory_plan_hash(),
        capability_plan_hash: plan.capability_plan_hash(),
        tool_plan_hash: plan.tool_plan_hash(),
        tool_catalog_identity: plan.tool_catalog_identity(),
        policy_hash: plan.policy_hash(),
        prompt_emulated: false,
    }
}
