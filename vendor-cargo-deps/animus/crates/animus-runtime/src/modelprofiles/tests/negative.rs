use super::super::*;
use super::support::*;

const NO_LABELS: &[&str] = &[];
const MODEL_ONLY_PROMPTS: &[&str] = &[MODEL_PROMPT];

#[test]
fn contract_rejects_stale_duplicate_unsafe_and_unadmitted_profiles() {
    // r[verify agent_harness_runtime.model_runtime_profiles.contract.nickel]
    let skills = [ModelSkillReferenceDraft {
        name: SKILL_NAME,
        posture: ModelSkillResourcePosture::Selectable,
    }];
    let stale = composition_profile_draft(
        NativeToolCallingMode::Optional,
        &skills,
        &[ModelNamedResourceReferenceDraft {
            name: TOOL_NAME,
            required: true,
        }],
        ALTERNATE_HASH,
    );
    let stale_error = validate_set_result(&[stale], None).expect_err("stale profile hash");
    assert_eq!(stale_error.issue(), ModelRuntimeProfileIssue::StaleProfileIdentity);

    let provisional = composition_profile_draft(NativeToolCallingMode::Disabled, &skills, &[], ZERO_HASH);
    let valid_hash = blake3_model_runtime_profile_identity(&provisional).expect("valid profile material");
    let valid = composition_profile_draft(NativeToolCallingMode::Disabled, &skills, &[], &valid_hash);
    let duplicate_error = validate_set_result(&[valid, valid], None).expect_err("duplicate profile identity");
    assert_eq!(duplicate_error.issue(), ModelRuntimeProfileIssue::DuplicateIdentity);

    let malformed_memory = ModelRuntimeProfileDraft {
        memory_policy_hash: "not-a-hash",
        ..provisional
    };
    let memory_error = validate_set_result(&[malformed_memory], None).expect_err("malformed memory policy hash");
    assert_eq!(memory_error.issue(), ModelRuntimeProfileIssue::InvalidHash);

    let unsafe_profile = ModelRuntimeProfileDraft {
        profile_id: "unsafe profile",
        ..provisional
    };
    let unsafe_error = validate_set_result(&[unsafe_profile], None).expect_err("unsafe profile label");
    assert_eq!(unsafe_error.issue(), ModelRuntimeProfileIssue::UnsafeLabel);

    let unadmitted_project = ModelRuntimeProfileDraft {
        source_scope: ModelProfileSourceScope::Project,
        project_profile_admitted: false,
        ..provisional
    };
    let project_error = validate_set_result(&[unadmitted_project], None).expect_err("unadmitted project profile");
    assert_eq!(project_error.issue(), ModelRuntimeProfileIssue::UntrustedProfile);
}

#[test]
fn required_resolution_has_no_ambient_fallback_and_rejects_stale_capabilities() {
    // r[verify agent_harness_runtime.model_runtime_profiles.resolution.missing]
    let set = composition_profile_set(NativeToolCallingMode::Disabled, ModelSkillResourcePosture::MetadataOnly);
    let missing = resolve_model_runtime_profile(&set, ModelRuntimeTargetDraft {
        provider_id: "unknown",
        model_id: "unknown-model",
        router_role: None,
        capability_identity: CAPABILITY_HASH,
    })
    .expect_err("required mode must not infer a fallback");
    let stale = resolve_model_runtime_profile(&set, ModelRuntimeTargetDraft {
        capability_identity: ALTERNATE_HASH,
        ..target(None)
    })
    .expect_err("stale capability binding");

    assert_eq!(missing.issue(), ModelRuntimeProfileIssue::MissingProfile);
    assert_eq!(stale.issue(), ModelRuntimeProfileIssue::StaleBinding);
}

#[test]
fn every_authority_ceiling_can_deny_model_prompt_skill_memory_or_tool_requests() {
    // r[verify agent_harness_runtime.model_runtime_profiles.composition.ceilings]
    let model_denied = compose_case(ComposeCaseInput {
        ceiling: ModelResourceCeiling {
            model_targets: NO_LABELS,
            ..ceiling()
        },
        posture: ModelSkillResourcePosture::MetadataOnly,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: false,
    })
    .expect_err("model target denied");
    let prompt_denied = compose_case(ComposeCaseInput {
        ceiling: ModelResourceCeiling {
            prompt_sources: MODEL_ONLY_PROMPTS,
            ..ceiling()
        },
        posture: ModelSkillResourcePosture::MetadataOnly,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: false,
    })
    .expect_err("mandatory prompt denied");
    let skill_denied = compose_case(ComposeCaseInput {
        ceiling: ModelResourceCeiling {
            skills: NO_LABELS,
            ..ceiling()
        },
        posture: ModelSkillResourcePosture::RequiredSelected,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: false,
    })
    .expect_err("required skill denied");
    let memory_denied = compose_case(ComposeCaseInput {
        ceiling: ModelResourceCeiling {
            memory_policy_hashes: NO_LABELS,
            ..ceiling()
        },
        posture: ModelSkillResourcePosture::MetadataOnly,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: false,
    })
    .expect_err("memory policy denied");
    let tool_denied = compose_case(ComposeCaseInput {
        ceiling: ModelResourceCeiling {
            tools: NO_LABELS,
            ..ceiling()
        },
        posture: ModelSkillResourcePosture::MetadataOnly,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: false,
    })
    .expect_err("required tool denied");

    assert_eq!(model_denied.issue(), ModelRuntimeProfileIssue::AuthorityDenied);
    assert_eq!(prompt_denied.issue(), ModelRuntimeProfileIssue::PromptDenied);
    assert_eq!(skill_denied.issue(), ModelRuntimeProfileIssue::SkillDenied);
    assert_eq!(memory_denied.issue(), ModelRuntimeProfileIssue::MemoryDenied);
    assert_eq!(tool_denied.issue(), ModelRuntimeProfileIssue::ToolDenied);
}

#[test]
fn required_skill_and_tool_modes_fail_when_existing_admission_is_absent() {
    // r[verify agent_harness_runtime.model_runtime_profiles.skills.required_denied]
    // r[verify agent_harness_runtime.model_runtime_profiles.tools.unsupported]
    let missing_skill = compose_case(ComposeCaseInput {
        ceiling: ceiling(),
        posture: ModelSkillResourcePosture::RequiredSelected,
        native_tool_calls: true,
        include_skill: false,
        stale_skill_binding: false,
    })
    .expect_err("required selected skill missing");
    let stale_skill = compose_case(ComposeCaseInput {
        ceiling: ceiling(),
        posture: ModelSkillResourcePosture::Selectable,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: true,
    })
    .expect_err("cross-profile selected skill body");
    let unsupported_tools =
        composed_plan(NativeToolCallingMode::CapabilityRequired, ModelSkillResourcePosture::MetadataOnly, false)
            .expect_err("required native tool calls unsupported");

    assert_eq!(missing_skill.issue(), ModelRuntimeProfileIssue::SkillDenied);
    assert_eq!(stale_skill.issue(), ModelRuntimeProfileIssue::StaleBinding);
    assert_eq!(unsupported_tools.issue(), ModelRuntimeProfileIssue::ToolDenied);
}

#[test]
fn memory_access_requires_an_exact_existing_policy_and_cannot_widen_controls() {
    // r[verify agent_harness_runtime.model_runtime_profiles.memory.denied]
    let input = ComposeCaseInput {
        ceiling: ceiling(),
        posture: ModelSkillResourcePosture::MetadataOnly,
        native_tool_calls: true,
        include_skill: true,
        stale_skill_binding: false,
    };
    let missing = compose_case_with_memory(input, MemoryPolicyFixture::Missing).expect_err("missing memory policy");
    let disabled =
        compose_case_with_memory(input, MemoryPolicyFixture::Disabled).expect_err("memory policy use disabled");
    let stale = compose_case_with_memory(input, MemoryPolicyFixture::Stale).expect_err("stale memory policy identity");

    assert_eq!(missing.issue(), ModelRuntimeProfileIssue::MemoryDenied);
    assert_eq!(disabled.issue(), ModelRuntimeProfileIssue::MemoryDenied);
    assert_eq!(stale.issue(), ModelRuntimeProfileIssue::StaleBinding);
}

#[test]
fn stale_retry_cross_target_resume_and_carried_identity_are_rejected() {
    let (_resolved, plan) = composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable, true)
        .expect("admitted invocation plan");
    let stale_snapshot = ModelInvocationIdentitySnapshot {
        prompt_plan_hash: ALTERNATE_HASH,
        ..ModelInvocationIdentitySnapshot::from(&plan)
    };
    let stale = evaluate_model_invocation_transition(ModelInvocationTransitionRequest {
        kind: ModelInvocationTransitionKind::SameTargetRetry,
        current_plan: &plan,
        next_target_hash: plan.target().target_hash(),
        current_snapshot: stale_snapshot,
    })
    .expect_err("changed prompt identity");
    let stale_memory = evaluate_model_invocation_transition(ModelInvocationTransitionRequest {
        kind: ModelInvocationTransitionKind::SameTargetRetry,
        current_plan: &plan,
        next_target_hash: plan.target().target_hash(),
        current_snapshot: ModelInvocationIdentitySnapshot {
            memory_plan_hash: ALTERNATE_HASH,
            ..ModelInvocationIdentitySnapshot::from(&plan)
        },
    })
    .expect_err("changed memory-plan identity");
    let cross_target = evaluate_model_invocation_transition(ModelInvocationTransitionRequest {
        kind: ModelInvocationTransitionKind::Resume,
        current_plan: &plan,
        next_target_hash: ALTERNATE_HASH,
        current_snapshot: ModelInvocationIdentitySnapshot::from(&plan),
    })
    .expect_err("cross-target resume");

    assert_eq!(stale.issue(), ModelRuntimeProfileIssue::StaleBinding);
    assert_eq!(stale_memory.issue(), ModelRuntimeProfileIssue::StaleBinding);
    assert_eq!(cross_target.issue(), ModelRuntimeProfileIssue::RecompositionRequired);
}

#[test]
fn tool_calls_reject_unadvertised_prompt_emulated_and_stale_bindings() {
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.tool_call]
    let (_resolved, plan) =
        composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::MetadataOnly, true)
            .expect("admitted invocation plan");
    let unadvertised = validate_model_tool_call_binding(&plan, ModelToolCallBindingDraft {
        tool_name: "unknown-tool",
        ..tool_call_draft(&plan)
    })
    .expect_err("unadvertised tool");
    let emulated = validate_model_tool_call_binding(&plan, ModelToolCallBindingDraft {
        prompt_emulated: true,
        ..tool_call_draft(&plan)
    })
    .expect_err("prompt-emulated call");
    let stale = validate_model_tool_call_binding(&plan, ModelToolCallBindingDraft {
        profile_hash: ALTERNATE_HASH,
        ..tool_call_draft(&plan)
    })
    .expect_err("stale profile call");
    let stale_memory = validate_model_tool_call_binding(&plan, ModelToolCallBindingDraft {
        memory_plan_hash: ALTERNATE_HASH,
        ..tool_call_draft(&plan)
    })
    .expect_err("stale memory plan call");
    let stale_catalog = validate_model_tool_call_binding(&plan, ModelToolCallBindingDraft {
        tool_catalog_identity: ALTERNATE_HASH,
        ..tool_call_draft(&plan)
    })
    .expect_err("stale tool catalog call");

    assert_eq!(unadvertised.issue(), ModelRuntimeProfileIssue::ToolDenied);
    assert_eq!(emulated.issue(), ModelRuntimeProfileIssue::ToolDenied);
    assert_eq!(stale.issue(), ModelRuntimeProfileIssue::StaleBinding);
    assert_eq!(stale_memory.issue(), ModelRuntimeProfileIssue::StaleBinding);
    assert_eq!(stale_catalog.issue(), ModelRuntimeProfileIssue::StaleBinding);
}

#[test]
fn receipt_projection_rejects_every_sensitive_material_class() {
    // r[verify agent_harness_runtime.model_runtime_profiles.receipts.rejects_leaks]
    let (resolved, plan) = composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable, true)
        .expect("admitted invocation plan");
    let candidates = [
        ModelInvocationReceiptSensitiveDraft {
            raw_prompt: Some("raw"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            skill_body: Some("body"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            memory_body: Some("memory"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            script_or_asset: Some("script"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            tool_arguments: Some("arguments"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            tool_output: Some("output"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            provider_payload: Some("payload"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            credential: Some("credential"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            environment_value: Some("value"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            full_path: Some("/private/path"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            endpoint: Some("https://private.invalid"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            hidden_reasoning: Some("reasoning"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
        ModelInvocationReceiptSensitiveDraft {
            unbounded_trace: Some("trace"),
            ..ModelInvocationReceiptSensitiveDraft::default()
        },
    ];

    for candidate in candidates {
        let error =
            project_model_invocation_receipt(&resolved, &plan, candidate).expect_err("sensitive receipt material");
        assert_eq!(error.issue(), ModelRuntimeProfileIssue::ReceiptLeak);
    }
}

#[test]
fn dispatch_binding_does_not_cross_required_and_compatibility_modes() {
    let dispatch = admitted_dispatch_plan();
    let (_resolved, plan) =
        composed_plan(NativeToolCallingMode::Disabled, ModelSkillResourcePosture::MetadataOnly, false)
            .expect("admitted invocation plan");
    let missing = bind_model_dispatch_plan(&dispatch, ModelProfileEnablementPolicy::Required, None)
        .expect_err("required mode missing plan");
    let carried = bind_model_dispatch_plan(&dispatch, ModelProfileEnablementPolicy::CompatibilityDisabled, Some(&plan))
        .expect_err("compatibility mode carrying profile");

    assert_eq!(missing.issue(), ModelRuntimeProfileIssue::MissingProfile);
    assert_eq!(carried.issue(), ModelRuntimeProfileIssue::StaleBinding);
}

#[derive(Clone, Copy)]
struct ComposeCaseInput {
    ceiling: ModelResourceCeiling<'static>,
    posture: ModelSkillResourcePosture,
    native_tool_calls: bool,
    include_skill: bool,
    stale_skill_binding: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MemoryPolicyFixture {
    Enabled,
    Disabled,
    Missing,
    Stale,
}

fn compose_case(input: ComposeCaseInput) -> Result<ModelInvocationPlan, ModelRuntimeProfileError> {
    compose_case_with_memory(input, MemoryPolicyFixture::Enabled)
}

fn compose_case_with_memory(
    input: ComposeCaseInput,
    memory_fixture: MemoryPolicyFixture,
) -> Result<ModelInvocationPlan, ModelRuntimeProfileError> {
    let set = composition_profile_set(NativeToolCallingMode::Optional, input.posture);
    let resolved = resolved_composition_profile(&set);
    let prompts = prompt_sources();
    let prompt_policy = prompt_policy();
    let manifest = skill_manifest(animus_adapters::SkillManifestTrustStatus::Trusted);
    let body = loaded_skill_body(&manifest);
    let skill_resources = [ModelSkillCompositionResource {
        manifest: &manifest,
        selected_body: Some(&body),
        selection_profile_hash: Some(resolved.profile().profile_hash()),
        selection_target_hash: Some(if input.stale_skill_binding {
            ALTERNATE_HASH
        } else {
            resolved.target().target_hash()
        }),
        invocation_admitted: true,
        dependencies_admitted: true,
        budget_admitted: true,
    }];
    let selected_skills = if input.include_skill {
        skill_resources.as_slice()
    } else {
        &[]
    };
    let memory_policy = memory_policy_fixture(memory_fixture);
    let catalog = tool_catalog();
    let executors = executor_bindings();
    let capabilities = capability_matrix(input.native_tool_calls);
    let capability_decision = capability_decision();
    let ceilings = [input.ceiling];
    compose_model_invocation_plan(ModelInvocationCompositionInput {
        resolved: &resolved,
        capabilities: &capabilities,
        capability_decision: &capability_decision,
        prompt_sources: &prompts,
        prompt_policy: &prompt_policy,
        skills: selected_skills,
        memory_policy: memory_policy.as_ref(),
        tool_catalog: &catalog,
        executor_bindings: &executors,
        tool_catalog_identity: CATALOG_HASH,
        authority: ModelInvocationAuthority {
            ceilings: &ceilings,
            approved_tools: &[],
            policy_hash: POLICY_HASH,
        },
    })
}

fn memory_policy_fixture(fixture: MemoryPolicyFixture) -> Option<animus_adapters::OptInMemoryPolicy> {
    if fixture == MemoryPolicyFixture::Missing {
        return None;
    }
    let control = if fixture == MemoryPolicyFixture::Disabled {
        animus_adapters::MemoryOptInControl::Disabled
    } else {
        animus_adapters::MemoryOptInControl::Enabled
    };
    let controls = animus_adapters::MemoryScopeControls {
        user: control,
        project: control,
        thread: control,
    };
    let policy_hash = if fixture == MemoryPolicyFixture::Stale {
        ALTERNATE_HASH
    } else {
        MEMORY_POLICY_HASH
    };
    Some(
        animus_adapters::OptInMemoryPolicy::validate(&animus_adapters::OptInMemoryPolicyDraft {
            use_controls: controls,
            generation_controls: controls,
            policy_hash,
        })
        .expect("memory fixture policy"),
    )
}
