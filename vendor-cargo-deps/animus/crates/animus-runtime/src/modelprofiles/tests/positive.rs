use super::super::*;
use super::support::*;

const CONTRIBUTION_LIMIT: u16 = 4;
const OMISSION_LIMIT: u16 = 4;
const CONTRIBUTION_COST: u32 = 1;

fn assert_contribution_binding(plan: &ModelInvocationPlan, alternate_plan: &ModelInvocationPlan) {
    // r[verify agent_harness_runtime.request_context_contributions.composition.bound]
    // r[verify agent_harness_runtime.request_context_contributions.composition.drift]
    let provider_request =
        crate::request_context::ProviderRequestIdentity::new("provider-request-a");
    let contribution_binding =
        crate::request_context::binding_from_model_invocation(plan, provider_request.clone());
    let payload = crate::request_context::blake3_payload_identity(b"mandatory context");
    let contribution =
        crate::request_context::validate_contribution(crate::request_context::ContributionDraft {
            contribution_id: "mandatory-context",
            source: crate::request_context::ContributionSource::Prompt,
            trust: crate::request_context::ContributionTrust::Trusted,
            posture: crate::request_context::ContributionPosture::Mandatory,
            priority: 0,
            budget_cost: CONTRIBUTION_COST,
            payload_identity: payload.as_str(),
            deduplication_key: "mandatory-context",
            supersedes: &[],
            conflict_group: None,
            lifetime: crate::request_context::ContributionLifetime::CurrentRequest,
            sensitivity: crate::request_context::ContributionSensitivity::RequestLocal,
            safe_label: "mandatory-context",
        })
        .expect("model contribution must validate");
    let contribution_plan = crate::request_context::reduce_contributions(
        &[contribution],
        &contribution_binding,
        crate::request_context::ContributionPolicy {
            max_contributions: CONTRIBUTION_LIMIT,
            context_budget: PROFILE_CONTEXT_UNITS,
            max_omissions: OMISSION_LIMIT,
        },
    )
    .expect("contribution plan must bind model composition");
    assert_eq!(
        crate::request_context::validate_model_invocation_binding(
            &contribution_plan,
            plan,
            provider_request.clone(),
        ),
        Ok(())
    );
    assert_eq!(
        crate::request_context::validate_model_invocation_binding(
            &contribution_plan,
            alternate_plan,
            provider_request,
        ),
        Err(crate::request_context::ContributionIssue::InvocationDrift)
    );
}

#[test]
fn explicit_disclosure_selects_only_the_matching_skill_module() {
    // r[verify agent_harness_runtime.steel_skill_modules.lifecycle]
    let manifest = skill_manifest(animus_adapters::SkillManifestTrustStatus::Trusted);
    let body = loaded_skill_body(&manifest);
    let disclosed = ModelSkillPlanEntry::test_fixture(
        SKILL_NAME,
        manifest.content_identity().as_str(),
        Some(body),
    );
    let module = skill_module_descriptor(manifest.content_identity().as_str());
    let authority = animus_adapter_rlm_kernel::SteelSkillModuleAuthority {
        session: &[],
        profile: &[],
        skill: &[],
        host: &[],
    };
    let plan = select_steel_skill_modules(&[disclosed], &[module], authority)
        .expect("explicit matching disclosure must select the module");
    assert_eq!(plan.ordered_modules().len(), 1);
    assert_eq!(plan.ordered_modules()[0].name(), SKILL_NAME);
}

#[test]
fn profile_contract_and_identity_validate_deterministically() {
    // r[verify agent_harness_runtime.model_runtime_profiles.contract]
    let first = composition_profile_set(
        NativeToolCallingMode::Optional,
        ModelSkillResourcePosture::Selectable,
    );
    let second = composition_profile_set(
        NativeToolCallingMode::Optional,
        ModelSkillResourcePosture::Selectable,
    );

    assert_eq!(first, second);
    assert_eq!(first.profiles().len(), 1);
    assert_eq!(first.profiles()[0].profile_hash().len(), HASH_HEX_LENGTH);
}

#[test]
fn resolver_uses_role_model_and_explicit_default_specificity() {
    // r[verify agent_harness_runtime.model_runtime_profiles.resolution.role_refinement]
    let set = resolver_profile_set();
    let role = resolve_model_runtime_profile(&set, target(Some(ROUTER_ROLE)))
        .expect("role-refined resolution");
    let model = resolve_model_runtime_profile(&set, target(None)).expect("model resolution");
    let fallback = resolve_model_runtime_profile(
        &set,
        ModelRuntimeTargetDraft {
            provider_id: "other",
            provider_transport_id: ALTERNATE_PROVIDER_TRANSPORT_ID,
            model_id: "other-model",
            model_dialect_id: MODEL_DIALECT_ID,
            router_role: None,
            capability_identity: CAPABILITY_HASH,
        },
    )
    .expect("explicit default resolution");

    assert_resolved(
        role,
        "mock-role-profile",
        ModelProfileSpecificity::ExactTargetAndRole,
    );
    assert_resolved(
        model,
        "mock-model-profile",
        ModelProfileSpecificity::ExactTarget,
    );
    assert_resolved(
        fallback,
        "explicit-default-profile",
        ModelProfileSpecificity::ExplicitDefault,
    );
}

#[test]
fn composition_preserves_mandatory_prompt_progressive_skill_and_exact_tool() {
    // r[verify agent_harness_runtime.model_runtime_profiles.composition.ceilings]
    // r[verify agent_harness_runtime.model_runtime_profiles.skills.distinct]
    // r[verify agent_harness_runtime.model_runtime_profiles.memory.existing_policy]
    // r[verify agent_harness_runtime.model_runtime_profiles.prompts.distinct]
    // r[verify agent_harness_runtime.model_runtime_profiles.tools.modes]
    let (_resolved, plan) = composed_plan(
        NativeToolCallingMode::Optional,
        ModelSkillResourcePosture::Selectable,
        true,
    )
    .expect("admitted invocation plan");

    assert_eq!(plan.prompt_assembly().sections().len(), EXPECTED_TWO);
    assert!(
        plan.prompt_assembly()
            .sections()
            .iter()
            .any(|section| { section.source_label() == MANDATORY_PROMPT })
    );
    assert_eq!(plan.skills().len(), 1);
    assert!(plan.skills()[0].selected_body().is_some());
    assert_eq!(plan.memory_access_mode(), ModelMemoryAccessMode::ReadOnly);
    assert_eq!(plan.memory_policy_hash(), MEMORY_POLICY_HASH);
    assert!(plan.memory_use_enabled());
    assert!(!plan.memory_generation_enabled());
    assert_eq!(plan.memory_plan_hash().len(), HASH_HEX_LENGTH);
    assert_eq!(plan.tools().len(), 1);
    assert_eq!(plan.tools()[0].definition().name(), TOOL_NAME);
    assert_eq!(plan.max_tool_calls(), TOOL_CALL_LIMIT);
    assert_eq!(plan.max_parallel_tool_calls(), TOOL_PARALLEL_LIMIT);
    assert!(plan.diagnostics().is_empty());

    let profiles = resolver_profile_set();
    let alternate_resolution = resolve_model_runtime_profile(
        &profiles,
        ModelRuntimeTargetDraft {
            provider_id: ALTERNATE_PROVIDER_ID,
            provider_transport_id: ALTERNATE_PROVIDER_TRANSPORT_ID,
            model_id: ALTERNATE_MODEL_ID,
            model_dialect_id: MODEL_DIALECT_ID,
            router_role: None,
            capability_identity: CAPABILITY_HASH,
        },
    )
    .expect("alternate explicit-default profile");
    let ModelRuntimeProfileResolution::Resolved(alternate_resolved) = alternate_resolution else {
        panic!("required alternate profile must resolve");
    };
    let mut alternate_capabilities = capability_matrix(false);
    alternate_capabilities.provider_id = ALTERNATE_PROVIDER_ID.to_string();
    alternate_capabilities.model_id = ALTERNATE_MODEL_ID.to_string();
    let mut alternate_decision = capability_decision();
    alternate_decision.provider_id = ALTERNATE_PROVIDER_ID.to_string();
    alternate_decision.model_id = ALTERNATE_MODEL_ID.to_string();
    let prompts = prompt_sources();
    let prompt_policy = prompt_policy();
    let catalog = tool_catalog();
    let executors = executor_bindings();
    let alternate_targets = [ALTERNATE_TARGET_LABEL];
    let ceilings = [ModelResourceCeiling {
        model_targets: &alternate_targets,
        ..ceiling()
    }];
    let alternate_plan = compose_model_invocation_plan(ModelInvocationCompositionInput {
        resolved: &alternate_resolved,
        capabilities: &alternate_capabilities,
        capability_decision: &alternate_decision,
        prompt_sources: &prompts,
        prompt_policy: &prompt_policy,
        skills: &[],
        memory_policy: None,
        tool_catalog: &catalog,
        executor_bindings: &executors,
        tool_catalog_identity: CATALOG_HASH,
        authority: ModelInvocationAuthority {
            ceilings: &ceilings,
            approved_tools: &[],
            policy_hash: POLICY_HASH,
        },
    })
    .expect("alternate minimal invocation plan");

    assert_ne!(plan.profile_hash(), alternate_plan.profile_hash());
    assert_eq!(alternate_plan.prompt_assembly().sections().len(), 1);
    assert!(alternate_plan.skills().is_empty());
    assert_eq!(
        alternate_plan.memory_access_mode(),
        ModelMemoryAccessMode::Disabled
    );
    assert!(!alternate_plan.memory_use_enabled());
    assert!(alternate_plan.tools().is_empty());

    assert_contribution_binding(&plan, &alternate_plan);
}

#[test]
fn disabled_tool_mode_is_explicit_and_emits_no_declarations() {
    let (_resolved, disabled) = composed_plan(
        NativeToolCallingMode::Disabled,
        ModelSkillResourcePosture::MetadataOnly,
        false,
    )
    .expect("disabled tool plan");

    assert!(disabled.tools().is_empty());
    assert!(disabled.diagnostics().is_empty());
}

#[test]
fn same_target_retry_reuses_exact_plan_and_fallback_recomposes() {
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.retry]
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.fallback]
    let (_resolved, plan) = composed_plan(
        NativeToolCallingMode::Optional,
        ModelSkillResourcePosture::Selectable,
        true,
    )
    .expect("admitted invocation plan");
    let snapshot = ModelInvocationIdentitySnapshot::from(&plan);
    let retry = evaluate_model_invocation_transition(ModelInvocationTransitionRequest {
        kind: ModelInvocationTransitionKind::SameTargetRetry,
        current_plan: &plan,
        next_target_hash: plan.target().target_hash(),
        current_snapshot: snapshot,
    })
    .expect("same-target retry");
    let fallback = evaluate_model_invocation_transition(ModelInvocationTransitionRequest {
        kind: ModelInvocationTransitionKind::RouterFallback,
        current_plan: &plan,
        next_target_hash: ALTERNATE_HASH,
        current_snapshot: snapshot,
    })
    .expect("cross-target fallback");

    assert_eq!(
        retry,
        ModelInvocationTransitionDecision::ReuseExactPlan {
            plan_hash: plan.plan_hash().to_string(),
        }
    );
    assert_eq!(
        fallback,
        ModelInvocationTransitionDecision::RecomposeForTarget {
            target_hash: ALTERNATE_HASH.to_string(),
        }
    );
}

#[test]
fn one_transport_resolves_two_explicit_model_dialects() {
    // r[verify animus.dialect.verification]
    // r[verify animus.dialect.verification.matrix]
    // r[verify animus.dialect.target.independent]
    // r[verify animus.dialect.binding.shared_transport]
    const ALTERNATE_PROFILE_ID: &str = "generic-model-profile";
    let base_provisional =
        composition_profile_draft(NativeToolCallingMode::Disabled, &[], &[], ZERO_HASH);
    let base_hash =
        blake3_model_runtime_profile_identity(&base_provisional).expect("base dialect hash");
    let base_profile = ModelRuntimeProfileDraft {
        profile_hash: base_hash.as_str(),
        ..base_provisional
    };
    let alternate_provisional = ModelRuntimeProfileDraft {
        profile_id: ALTERNATE_PROFILE_ID,
        dialect_id: ALTERNATE_MODEL_DIALECT_ID,
        binding: ModelRuntimeBindingDraft::Exact {
            provider_id: PROVIDER_ID,
            model_id: ALTERNATE_MODEL_ID,
            router_role: None,
            capability_identity: CAPABILITY_HASH,
        },
        ..base_provisional
    };
    let alternate_hash = blake3_model_runtime_profile_identity(&alternate_provisional)
        .expect("alternate dialect hash");
    let alternate_profile = ModelRuntimeProfileDraft {
        profile_hash: alternate_hash.as_str(),
        ..alternate_provisional
    };
    let set = validate_set_result(&[base_profile, alternate_profile], None)
        .expect("shared transport dialect set");
    let first = resolve_model_runtime_profile(&set, target(None)).expect("first dialect");
    let second = resolve_model_runtime_profile(
        &set,
        ModelRuntimeTargetDraft {
            model_id: ALTERNATE_MODEL_ID,
            model_dialect_id: ALTERNATE_MODEL_DIALECT_ID,
            ..target(None)
        },
    )
    .expect("second dialect");
    let ModelRuntimeProfileResolution::Resolved(first) = first else {
        panic!("first profile must resolve");
    };
    let ModelRuntimeProfileResolution::Resolved(second) = second else {
        panic!("second profile must resolve");
    };

    assert_eq!(
        first.target().provider_transport_id(),
        second.target().provider_transport_id()
    );
    assert_ne!(
        first.target().model_dialect_id(),
        second.target().model_dialect_id()
    );
}

#[test]
fn provider_transition_retains_only_an_explicitly_admitted_dialect() {
    // r[verify animus.dialect.transition]
    let set = resolver_profile_set();
    let current = resolve_model_runtime_profile(&set, target(None)).expect("current target");
    let next = resolve_model_runtime_profile(
        &set,
        ModelRuntimeTargetDraft {
            provider_transport_id: ALTERNATE_PROVIDER_TRANSPORT_ID,
            ..target(None)
        },
    )
    .expect("next transport target");
    let ModelRuntimeProfileResolution::Resolved(current) = current else {
        panic!("current profile must resolve");
    };
    let ModelRuntimeProfileResolution::Resolved(next) = next else {
        panic!("next profile must resolve");
    };
    let admitted = [ModelTransportDialectAdmissionDraft {
        provider_transport_id: ALTERNATE_PROVIDER_TRANSPORT_ID,
        model_dialect_id: MODEL_DIALECT_ID,
    }];
    let transition = evaluate_model_dialect_transition(ModelDialectTransitionRequest {
        current_target: current.target(),
        next_target: next.target(),
        admitted_pairs: &admitted,
    })
    .expect("admitted dialect transition");

    assert_eq!(
        transition.provider_transport_id(),
        ALTERNATE_PROVIDER_TRANSPORT_ID
    );
    assert_eq!(transition.model_dialect_id(), MODEL_DIALECT_ID);
    assert_eq!(transition.transition_hash().len(), HASH_HEX_LENGTH);
}

#[test]
fn exact_tool_call_dispatch_and_metadata_receipt_bind_the_plan() {
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.tool_call]
    // r[verify animus.dialect.ownership]
    // r[verify animus.dialect.ownership.provider]
    // r[verify animus.dialect.ownership.dialect]
    // r[verify agent_harness_runtime.model_runtime_profiles.receipts]
    let (resolved, plan) = composed_plan(
        NativeToolCallingMode::Optional,
        ModelSkillResourcePosture::Selectable,
        true,
    )
    .expect("admitted invocation plan");
    let binding = validate_model_tool_call_binding(&plan, tool_call_draft(&plan))
        .expect("advertised exact tool call");
    let first = project_model_invocation_receipt(
        &resolved,
        &plan,
        ModelInvocationReceiptSensitiveDraft::default(),
    )
    .expect("safe receipt");
    let second = project_model_invocation_receipt(
        &resolved,
        &plan,
        ModelInvocationReceiptSensitiveDraft::default(),
    )
    .expect("deterministic safe receipt");

    assert_eq!(binding.tool_name(), TOOL_NAME);
    assert_eq!(binding.invocation_plan_hash(), plan.plan_hash());
    assert_eq!(first, second);
    assert_eq!(first.provider_transport_id(), PROVIDER_TRANSPORT_ID);
    assert_eq!(first.model_dialect_id(), MODEL_DIALECT_ID);
    assert_eq!(first.skills().len(), 1);
    assert_eq!(first.memory_access_mode(), ModelMemoryAccessMode::ReadOnly);
    assert_eq!(first.memory_policy_hash(), MEMORY_POLICY_HASH);
    assert!(first.memory_use_enabled());
    assert!(!first.memory_generation_enabled());
    assert_eq!(first.memory_plan_hash(), plan.memory_plan_hash());
    assert_eq!(first.tools().len(), 1);
    assert_eq!(first.redaction_marker(), "raw-model-context-omitted");
    assert_eq!(first.receipt_hash().len(), HASH_HEX_LENGTH);
}

#[test]
fn admitted_model_dispatch_requires_the_profile_plan_only_in_required_mode() {
    let dispatch = admitted_dispatch_plan();
    let (_resolved, plan) = composed_plan(
        NativeToolCallingMode::Optional,
        ModelSkillResourcePosture::MetadataOnly,
        true,
    )
    .expect("admitted invocation plan");
    let required = bind_model_dispatch_plan(
        &dispatch,
        ModelProfileEnablementPolicy::Required,
        Some(&plan),
    )
    .expect("required profile dispatch binding");
    let compatibility = bind_model_dispatch_plan(
        &dispatch,
        ModelProfileEnablementPolicy::CompatibilityDisabled,
        None,
    )
    .expect("explicit compatibility-disabled dispatch");

    assert_eq!(required.invocation_plan_hash(), Some(plan.plan_hash()));
    assert_eq!(compatibility.invocation_plan_hash(), None);
}

fn assert_resolved(
    resolution: ModelRuntimeProfileResolution,
    expected_profile: &str,
    expected_specificity: ModelProfileSpecificity,
) {
    let ModelRuntimeProfileResolution::Resolved(resolved) = resolution else {
        panic!("required profile resolution must not enter compatibility mode");
    };
    assert_eq!(resolved.profile().profile_id(), expected_profile);
    assert_eq!(resolved.specificity(), expected_specificity);
}
