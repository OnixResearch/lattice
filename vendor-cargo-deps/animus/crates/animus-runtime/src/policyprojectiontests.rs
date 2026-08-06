#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::non_trait_imports))]

use serde::Deserialize;

const GENERATED_PROCESS_POLICY_JSON: &str =
    include_str!("../../../policy/process-jobs/generated/process-job-profile-policy.json");
const GENERATED_PROCESS_MANIFEST_JSON: &str =
    include_str!("../../../policy/process-jobs/generated/sample-profile-manifest.json");
const GENERATED_TOOL_SCHEMA_JSON: &str =
    include_str!("../../../policy/tool-catalog/generated/tool-catalog-schema.json");
const GENERATED_TOOL_CATALOG_JSON: &str = include_str!("../../../policy/tool-catalog/generated/tool-catalog.json");
const PROCESS_PROFILE_NAME: &str = "quick-check";
const PROCESS_OWNER_LABEL: &str = "repo";
const POLICY_SOURCE_LABEL: &str = "nickel-generated";
const TOOL_PROVENANCE: &str = "nickel-policy";
const TOOL_ARGUMENT_NAME: &str = "context";
const TOOL_VISIBLE_CHARACTER_LIMIT: u32 = 1_024;
const TOOL_OUTPUT_BYTE_LIMIT: u64 = 4_096;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedProcessPolicy {
    schema: String,
    profile_schema_version: u16,
    default_backend: String,
    allowed_backends: Vec<String>,
    allowed_env_prefixes: Vec<String>,
    allowed_cwd_prefixes: Vec<String>,
    allowed_writable_path_prefixes: Vec<String>,
    max_resource_limits: GeneratedResourceLimits,
    notification_defaults: GeneratedNotification,
    identity_metadata_keys: Vec<String>,
    identity_hash_algorithm: String,
    identity_redaction: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedProcessManifest {
    schema_version: u16,
    profiles: Vec<GeneratedProcessProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedProcessProfile {
    name: String,
    backend: String,
    command: String,
    program: String,
    args: Vec<String>,
    cwd: String,
    writable_paths: Vec<String>,
    env: Vec<GeneratedKeyValue>,
    resource_limits: GeneratedResourceLimits,
    notification: GeneratedNotification,
    metadata: Vec<GeneratedKeyValue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedKeyValue {
    key: String,
    value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedResourceLimits {
    timeout_seconds: u32,
    memory_bytes: u64,
    cpu_millis: u32,
    log_bytes: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedNotification {
    notify_on_complete: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedToolSchema {
    schema: String,
    runtime_kinds: Vec<String>,
    capabilities: Vec<String>,
    side_effects: Vec<String>,
    approvals: Vec<String>,
    redactions: Vec<String>,
    schema_reference_prefix: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedToolCatalog {
    schema: String,
    descriptors: Vec<GeneratedToolDescriptor>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct GeneratedToolDescriptor {
    name: String,
    description: String,
    runtime_kind: String,
    capabilities: Vec<String>,
    side_effect: String,
    approval: String,
    input_schema: String,
    output_schema: String,
    redaction: String,
}

#[test]
fn generated_process_policy_and_manifest_match_runtime_dto_meaning() {
    // r[verify agent_harness_runtime.nickel_policy_contracts.projection_freshness]
    let generated_policy = generated_process_policy();
    let generated_manifest = generated_process_manifest();
    let policy = process_policy_from_generated(&generated_policy);
    let manifest = process_manifest_from_generated(&generated_manifest);
    let resolved = manifest
        .resolve(PROCESS_PROFILE_NAME, PROCESS_OWNER_LABEL, &policy)
        .expect("generated process profile should resolve");

    assert_eq!(generated_policy.schema, "animus.process-job-profiles.policy.v1");
    assert_eq!(generated_policy.profile_schema_version, generated_manifest.schema_version);
    assert_eq!(generated_policy.identity_hash_algorithm, "BLAKE3");
    assert_eq!(generated_policy.identity_redaction, "hash-only");
    assert!(generated_policy.notification_defaults.notify_on_complete);
    assert_eq!(generated_policy.identity_metadata_keys, vec![String::from("identity.purpose")]);
    assert_eq!(resolved.request.backend, crate::ProcessJobBackend::Pueue);
    assert_eq!(resolved.request.program.as_deref(), Some("cargo"));
    assert_eq!(resolved.request.args, vec![String::from("check"), String::from("--tests")]);
}

#[test]
fn generated_process_profile_outside_policy_fails_closed() {
    // r[verify agent_harness_runtime.nickel_policy_contracts.fixtures]
    let generated_policy = generated_process_policy();
    let mut generated_manifest = generated_process_manifest();
    generated_manifest.profiles[0].backend = String::from("systemd");
    let policy = process_policy_from_generated(&generated_policy);
    let manifest = process_manifest_from_generated(&generated_manifest);
    let error = manifest
        .resolve(PROCESS_PROFILE_NAME, PROCESS_OWNER_LABEL, &policy)
        .expect_err("disallowed generated backend should fail");

    assert_eq!(error.kind(), crate::ProcessJobProfileErrorKind::DisallowedBackend);
}

#[test]
fn generated_tool_catalog_matches_runtime_dto_meaning() {
    // r[verify agent_harness_runtime.nickel_policy_contracts.projection_freshness]
    let schema = generated_tool_schema();
    let generated = generated_tool_catalog();
    let catalog = tool_catalog_from_generated(&generated);
    let descriptor = &generated.descriptors[0];
    let definition = &catalog.definitions()[0];

    assert_eq!(schema.schema, "animus.tool-catalog.schema.v1");
    assert_eq!(generated.schema, "animus.tool-catalog.v1");
    assert!(schema.runtime_kinds.contains(&descriptor.runtime_kind));
    assert!(schema.side_effects.contains(&descriptor.side_effect));
    assert!(schema.approvals.contains(&descriptor.approval));
    assert!(schema.redactions.contains(&descriptor.redaction));
    assert!(descriptor.capabilities.iter().all(|capability| schema.capabilities.contains(capability)));
    assert!(descriptor.input_schema.starts_with(schema.schema_reference_prefix.as_str()));
    assert_eq!(definition.metadata().name(), descriptor.name);
    assert_eq!(definition.metadata().description(), descriptor.description);
    assert_eq!(definition.metadata().capability_labels(), descriptor.capabilities);
}

#[test]
fn duplicate_generated_tool_descriptors_fail_closed() {
    // r[verify agent_harness_runtime.nickel_policy_contracts.fixtures]
    let mut generated = generated_tool_catalog();
    generated.descriptors.push(generated.descriptors[0].clone());
    let definitions = generated.descriptors.iter().map(tool_definition_from_generated).collect::<Vec<_>>();
    let error =
        crate::tool_executor_kit::ToolCatalog::new(definitions).expect_err("duplicate generated tool should fail");

    assert_eq!(error, crate::tool_executor_kit::ToolExecutorKitError::DuplicateTool);
}

fn generated_process_policy() -> GeneratedProcessPolicy {
    serde_json::from_str(GENERATED_PROCESS_POLICY_JSON).expect("generated process policy JSON should deserialize")
}

fn generated_process_manifest() -> GeneratedProcessManifest {
    serde_json::from_str(GENERATED_PROCESS_MANIFEST_JSON).expect("generated process manifest JSON should deserialize")
}

fn process_policy_from_generated(generated: &GeneratedProcessPolicy) -> crate::ProcessJobProfilePolicy {
    crate::ProcessJobProfilePolicy {
        default_backend: process_backend(generated.default_backend.as_str()),
        allowed_backends: generated.allowed_backends.iter().map(|backend| process_backend(backend.as_str())).collect(),
        allowed_env_prefixes: generated.allowed_env_prefixes.clone(),
        allowed_cwd_prefixes: generated.allowed_cwd_prefixes.clone(),
        allowed_writable_path_prefixes: generated.allowed_writable_path_prefixes.clone(),
        max_timeout_seconds: Some(generated.max_resource_limits.timeout_seconds),
        max_memory_bytes: Some(generated.max_resource_limits.memory_bytes),
        max_cpu_millis: Some(generated.max_resource_limits.cpu_millis),
        max_log_bytes: Some(generated.max_resource_limits.log_bytes),
        policy_source: String::from(POLICY_SOURCE_LABEL),
        policy_hash: String::new(),
    }
    .with_computed_hash()
}

fn process_manifest_from_generated(generated: &GeneratedProcessManifest) -> crate::ProcessJobProfileManifest {
    crate::ProcessJobProfileManifest {
        schema_version: generated.schema_version,
        profiles: generated
            .profiles
            .iter()
            .map(|profile| crate::background::NamedProcessJobProfile {
                name: profile.name.clone(),
                profile: process_profile_from_generated(profile),
            })
            .collect(),
    }
}

fn process_profile_from_generated(generated: &GeneratedProcessProfile) -> crate::ProcessJobProfile {
    crate::ProcessJobProfile {
        backend: Some(process_backend(generated.backend.as_str())),
        command: non_empty_option(generated.command.as_str()),
        program: non_empty_option(generated.program.as_str()),
        args: generated.args.clone(),
        cwd: Some(generated.cwd.clone()),
        writable_paths: generated.writable_paths.clone(),
        env: generated
            .env
            .iter()
            .map(|binding| crate::ProcessJobEnvironmentVariable {
                key: binding.key.clone(),
                value: binding.value.clone(),
            })
            .collect(),
        resources: crate::ProcessJobResourceLimits {
            timeout_seconds: Some(generated.resource_limits.timeout_seconds),
            memory_bytes: Some(generated.resource_limits.memory_bytes),
            cpu_millis: Some(generated.resource_limits.cpu_millis),
            log_bytes: Some(generated.resource_limits.log_bytes),
        },
        notification: crate::ProcessJobNotificationPolicy {
            notify_on_complete: generated.notification.notify_on_complete,
        },
        metadata: generated
            .metadata
            .iter()
            .map(|field| crate::ProcessJobMetadataField {
                key: field.key.clone(),
                value: field.value.clone(),
            })
            .collect(),
    }
}

fn process_backend(label: &str) -> crate::ProcessJobBackend {
    match label {
        "native" => crate::ProcessJobBackend::Native,
        "pueue" => crate::ProcessJobBackend::Pueue,
        "systemd" => crate::ProcessJobBackend::Systemd,
        other => panic!("unknown generated process backend: {other}"),
    }
}

fn non_empty_option(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(String::from(value))
    }
}

fn generated_tool_schema() -> GeneratedToolSchema {
    serde_json::from_str(GENERATED_TOOL_SCHEMA_JSON).expect("generated tool schema JSON should deserialize")
}

fn generated_tool_catalog() -> GeneratedToolCatalog {
    serde_json::from_str(GENERATED_TOOL_CATALOG_JSON).expect("generated tool catalog JSON should deserialize")
}

fn tool_catalog_from_generated(generated: &GeneratedToolCatalog) -> crate::tool_executor_kit::ToolCatalog {
    let definitions = generated.descriptors.iter().map(tool_definition_from_generated).collect::<Vec<_>>();
    crate::tool_executor_kit::ToolCatalog::new(definitions).expect("generated tool catalog should validate")
}

fn tool_definition_from_generated(descriptor: &GeneratedToolDescriptor) -> crate::tool_executor_kit::ToolDefinition {
    assert!(!descriptor.name.is_empty());
    assert!(!descriptor.capabilities.is_empty());
    let capability_labels = descriptor.capabilities.iter().map(String::as_str).collect::<Vec<_>>();
    let prompt_guidelines = [descriptor.output_schema.as_str()];
    let metadata = animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
        name: Some(descriptor.name.as_str()),
        label: Some(descriptor.name.as_str()),
        description: Some(descriptor.description.as_str()),
        parameter_schema_identity: Some(descriptor.input_schema.as_str()),
        prompt_snippet: Some(descriptor.description.as_str()),
        prompt_guidelines: prompt_guidelines.as_slice(),
        source_identity: Some(descriptor.runtime_kind.as_str()),
        provenance: Some(TOOL_PROVENANCE),
        capability_labels: capability_labels.as_slice(),
        result_metadata_shape: Some(descriptor.output_schema.as_str()),
        status: animus_adapters::tool::DefinitionStatus::Active,
    })
    .expect("generated tool metadata should validate");
    crate::tool_executor_kit::ToolDefinition::from_draft(crate::tool_executor_kit::ToolDefinitionDraft {
        metadata,
        executor_binding: descriptor.name.as_str(),
        argument_names: &[TOOL_ARGUMENT_NAME],
        mutation_posture: tool_mutation_posture(descriptor.side_effect.as_str()),
        output_policy: crate::tool_executor_kit::ToolExecutorOutputPolicy::new(
            TOOL_VISIBLE_CHARACTER_LIMIT,
            TOOL_OUTPUT_BYTE_LIMIT,
            descriptor.redaction != "none",
        )
        .expect("bounded generated tool output policy"),
        approval_posture: tool_approval_posture(descriptor.approval.as_str()),
    })
    .expect("generated executor-bound tool definition should validate")
}

fn tool_mutation_posture(side_effect: &str) -> crate::tool_executor_kit::ToolMutationPosture {
    if side_effect == "read_only" {
        crate::tool_executor_kit::ToolMutationPosture::ReadOnly
    } else {
        crate::tool_executor_kit::ToolMutationPosture::DeclaredTargets
    }
}

fn tool_approval_posture(approval: &str) -> animus_adapters::tool::ApprovalPosture {
    match approval {
        "allow" => animus_adapters::tool::ApprovalPosture::PreApproved,
        "prompt" => animus_adapters::tool::ApprovalPosture::RequiresApproval,
        "deny" => animus_adapters::tool::ApprovalPosture::Denied,
        other => panic!("unknown generated tool approval posture: {other}"),
    }
}
