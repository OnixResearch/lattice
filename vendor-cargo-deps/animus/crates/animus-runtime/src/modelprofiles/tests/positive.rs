use super::super::*;
use super::support::*;

#[test]
fn profile_contract_and_identity_validate_deterministically() {
    // r[verify agent_harness_runtime.model_runtime_profiles.contract]
    let first = composition_profile_set(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable);
    let second = composition_profile_set(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable);

    assert_eq!(first, second);
    assert_eq!(first.profiles().len(), 1);
    assert_eq!(first.profiles()[0].profile_hash().len(), HASH_HEX_LENGTH);
}

#[test]
fn resolver_uses_role_model_and_explicit_default_specificity() {
    // r[verify agent_harness_runtime.model_runtime_profiles.resolution.role_refinement]
    let set = resolver_profile_set();
    let role = resolve_model_runtime_profile(&set, target(Some(ROUTER_ROLE))).expect("role-refined resolution");
    let model = resolve_model_runtime_profile(&set, target(None)).expect("model resolution");
    let fallback = resolve_model_runtime_profile(&set, ModelRuntimeTargetDraft {
        provider_id: "other",
        model_id: "other-model",
        router_role: None,
        capability_identity: CAPABILITY_HASH,
    })
    .expect("explicit default resolution");

    assert_resolved(role, "mock-role-profile", ModelProfileSpecificity::ExactTargetAndRole);
    assert_resolved(model, "mock-model-profile", ModelProfileSpecificity::ExactTarget);
    assert_resolved(fallback, "explicit-default-profile", ModelProfileSpecificity::ExplicitDefault);
}

#[test]
fn composition_preserves_mandatory_prompt_progressive_skill_and_exact_tool() {
    // r[verify agent_harness_runtime.model_runtime_profiles.composition.ceilings]
    // r[verify agent_harness_runtime.model_runtime_profiles.skills.distinct]
    // r[verify agent_harness_runtime.model_runtime_profiles.memory.existing_policy]
    // r[verify agent_harness_runtime.model_runtime_profiles.prompts.distinct]
    // r[verify agent_harness_runtime.model_runtime_profiles.tools.modes]
    let (_resolved, plan) = composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable, true)
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
    let alternate_resolution = resolve_model_runtime_profile(&profiles, ModelRuntimeTargetDraft {
        provider_id: ALTERNATE_PROVIDER_ID,
        model_id: ALTERNATE_MODEL_ID,
        router_role: None,
        capability_identity: CAPABILITY_HASH,
    })
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
    assert_eq!(alternate_plan.memory_access_mode(), ModelMemoryAccessMode::Disabled);
    assert!(!alternate_plan.memory_use_enabled());
    assert!(alternate_plan.tools().is_empty());
}

#[test]
fn disabled_tool_mode_is_explicit_and_emits_no_declarations() {
    let (_resolved, disabled) =
        composed_plan(NativeToolCallingMode::Disabled, ModelSkillResourcePosture::MetadataOnly, false)
            .expect("disabled tool plan");

    assert!(disabled.tools().is_empty());
    assert!(disabled.diagnostics().is_empty());
}

#[test]
fn same_target_retry_reuses_exact_plan_and_fallback_recomposes() {
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.retry]
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.fallback]
    let (_resolved, plan) = composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable, true)
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

    assert_eq!(retry, ModelInvocationTransitionDecision::ReuseExactPlan {
        plan_hash: plan.plan_hash().to_string(),
    });
    assert_eq!(fallback, ModelInvocationTransitionDecision::RecomposeForTarget {
        target_hash: ALTERNATE_HASH.to_string(),
    });
}

#[test]
fn exact_tool_call_dispatch_and_metadata_receipt_bind_the_plan() {
    // r[verify agent_harness_runtime.model_runtime_profiles.dispatch_binding.tool_call]
    // r[verify agent_harness_runtime.model_runtime_profiles.receipts]
    let (resolved, plan) = composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::Selectable, true)
        .expect("admitted invocation plan");
    let binding = validate_model_tool_call_binding(&plan, tool_call_draft(&plan)).expect("advertised exact tool call");
    let first = project_model_invocation_receipt(&resolved, &plan, ModelInvocationReceiptSensitiveDraft::default())
        .expect("safe receipt");
    let second = project_model_invocation_receipt(&resolved, &plan, ModelInvocationReceiptSensitiveDraft::default())
        .expect("deterministic safe receipt");

    assert_eq!(binding.tool_name(), TOOL_NAME);
    assert_eq!(binding.invocation_plan_hash(), plan.plan_hash());
    assert_eq!(first, second);
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
    let (_resolved, plan) =
        composed_plan(NativeToolCallingMode::Optional, ModelSkillResourcePosture::MetadataOnly, true)
            .expect("admitted invocation plan");
    let required = bind_model_dispatch_plan(&dispatch, ModelProfileEnablementPolicy::Required, Some(&plan))
        .expect("required profile dispatch binding");
    let compatibility = bind_model_dispatch_plan(&dispatch, ModelProfileEnablementPolicy::CompatibilityDisabled, None)
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
