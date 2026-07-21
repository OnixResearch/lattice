use alloc::string::String;

use super::*;

#[test]
fn skill_metadata_is_progressive_and_body_free() {
    // r[verify agent_harness_runtime.openai_codex_guidance_skill_budget.skill_metadata.progressive]
    let manifest = manifest(SkillManifestTrustStatus::Trusted);
    let metadata = build_metadata(
        &manifest,
        SkillInvocationPolicy::ExplicitOnly,
        &[SKILL_DEPENDENCY],
    );

    assert_eq!(metadata.name(), SKILL_NAME);
    assert_eq!(metadata.description(), SKILL_DESCRIPTION);
    assert_eq!(metadata.source(), SKILL_SOURCE);
    assert_eq!(
        metadata.dependency_hints(),
        &[String::from(SKILL_DEPENDENCY)]
    );
    assert_eq!(
        metadata.context_budget_cost(),
        nonzero(METADATA_BUDGET_UNITS)
    );
    assert_ne!(metadata.description(), SELECTED_BODY);
}

#[test]
fn deferred_skill_loading_denies_unsafe_requests() {
    // r[verify agent_harness_runtime.openai_codex_guidance_skill_budget.deferred_loading.denied]
    let manifest = manifest(SkillManifestTrustStatus::Trusted);
    let explicit_only = build_metadata(
        &manifest,
        SkillInvocationPolicy::ExplicitOnly,
        &[SKILL_DEPENDENCY],
    );
    let disabled = build_metadata(&manifest, SkillInvocationPolicy::Disabled, &[]);
    let missing_dependency = build_metadata(
        &manifest,
        SkillInvocationPolicy::ImplicitAllowed,
        &[MISSING_DEPENDENCY],
    );

    assert_eq!(
        decide_skill_loading(
            core::slice::from_ref(&explicit_only),
            SkillLoadRequest {
                name: SKILL_NAME,
                mode: SkillInvocationMode::Implicit,
                available_dependencies: &[SKILL_DEPENDENCY],
            },
        ),
        SkillLoadDecision::Deny(SkillLoadDenial::ImplicitInvocationDenied)
    );
    assert_eq!(
        decide_skill_loading(
            &[disabled],
            SkillLoadRequest {
                name: SKILL_NAME,
                mode: SkillInvocationMode::Explicit,
                available_dependencies: &[SKILL_DEPENDENCY],
            },
        ),
        SkillLoadDecision::Deny(SkillLoadDenial::Disabled)
    );
    assert_eq!(
        decide_skill_loading(
            &[missing_dependency],
            SkillLoadRequest {
                name: SKILL_NAME,
                mode: SkillInvocationMode::Explicit,
                available_dependencies: &[SKILL_DEPENDENCY],
            },
        ),
        SkillLoadDecision::Deny(SkillLoadDenial::MissingDependency {
            dependency: String::from(MISSING_DEPENDENCY),
        })
    );
    assert_eq!(
        decide_skill_loading(
            &[explicit_only.clone(), explicit_only],
            SkillLoadRequest {
                name: SKILL_NAME,
                mode: SkillInvocationMode::Explicit,
                available_dependencies: &[SKILL_DEPENDENCY],
            },
        ),
        SkillLoadDecision::Deny(SkillLoadDenial::AmbiguousName)
    );
}

#[test]
fn skill_package_scripts_cannot_execute_through_runtime_core() {
    // r[verify agent_harness_runtime.openai_codex_guidance_skill_budget.skill_package.trust]
    let untrusted_manifest = manifest(SkillManifestTrustStatus::Untrusted);
    let metadata = build_metadata(
        &untrusted_manifest,
        SkillInvocationPolicy::ExplicitOnly,
        &[],
    );
    let untrusted_package = SkillPackageDescriptor::validate(&SkillPackageDraft {
        metadata: &metadata,
        instructions_hash: VALID_HASH,
        optional_scripts: &[SCRIPT_LABEL],
        references: &[REFERENCE_LABEL],
        assets: &[ASSET_LABEL],
        dependencies: &[SKILL_DEPENDENCY],
        trust_status: SkillManifestTrustStatus::Untrusted,
    })
    .expect("untrusted package descriptor should validate as data");

    assert_eq!(
        admit_runtime_core_skill_script(&untrusted_package),
        Err(SkillScriptDenial::UntrustedPackage)
    );

    let trusted_manifest = manifest(SkillManifestTrustStatus::Trusted);
    let trusted_metadata =
        build_metadata(&trusted_manifest, SkillInvocationPolicy::ExplicitOnly, &[]);
    let trusted_package = SkillPackageDescriptor::validate(&SkillPackageDraft {
        metadata: &trusted_metadata,
        instructions_hash: SECOND_HASH,
        optional_scripts: &[SCRIPT_LABEL],
        references: &[],
        assets: &[],
        dependencies: &[],
        trust_status: SkillManifestTrustStatus::Trusted,
    })
    .expect("trusted package descriptor should validate as data");

    assert_eq!(
        admit_runtime_core_skill_script(&trusted_package),
        Err(SkillScriptDenial::RuntimeCoreDoesNotExecuteScripts)
    );
}
