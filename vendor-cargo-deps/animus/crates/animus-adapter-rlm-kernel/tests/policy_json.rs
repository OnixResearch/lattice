#![allow(missing_docs)]

use animus_adapter_rlm_kernel::HostOperationClass;
use animus_adapter_rlm_kernel::ResourceBounds;
use animus_adapter_rlm_kernel::RlmKernelPolicy;
use animus_adapter_rlm_kernel::RlmKernelPolicyDraft;

const GENERATED_POLICY: &str = include_str!("../../../config/generated/rlm-kernel-profile.json");

#[test]
fn generated_nickel_policy_is_revalidated_by_rust() {
    // r[verify agent_harness_runtime.rlm_kernel.profile.valid]
    // r[verify agent_harness_runtime.rlm_kernel.verification.positive]
    let value: serde_json::Value = serde_json::from_str(GENERATED_POLICY).expect("generated policy must be JSON");
    let bounds = &value["bounds"];
    let operation_labels = strings(&value["admitted_operations"]);
    let operations = operation_labels.iter().map(|label| operation(label)).collect::<Vec<_>>();
    let child_profiles = strings(&value["admitted_child_profiles"]);
    let child_profile_refs = refs(&child_profiles);
    let non_claims = strings(&value["non_claims"]);
    let non_claim_refs = refs(&non_claims);
    let policy = RlmKernelPolicy::from_draft(&RlmKernelPolicyDraft {
        schema: text(&value["schema"]),
        cohort: text(&value["cohort"]),
        cohort_revision: text(&value["cohort_revision"]),
        profile: text(&value["profile"]),
        bounds: ResourceBounds {
            max_source_chars: number_u32(&bounds["max_source_chars"]),
            max_environment_chars: number_u32(&bounds["max_environment_chars"]),
            max_bindings: number_u32(&bounds["max_bindings"]),
            max_definitions: number_u32(&bounds["max_definitions"]),
            max_evaluation_fuel: number_u64(&bounds["max_evaluation_fuel"]),
            max_recursion_depth: number_u32(&bounds["max_recursion_depth"]),
            max_allocation_bytes: number_u64(&bounds["max_allocation_bytes"]),
            max_host_request_chars: number_u32(&bounds["max_host_request_chars"]),
            max_child_depth: number_u32(&bounds["max_child_depth"]),
            max_child_fan_out: number_u32(&bounds["max_child_fan_out"]),
            max_child_tokens: number_u64(&bounds["max_child_tokens"]),
            max_child_wall_time_ms: number_u64(&bounds["max_child_wall_time_ms"]),
            max_child_result_chars: number_u32(&bounds["max_child_result_chars"]),
        },
        admitted_operations: operations.as_slice(),
        admitted_child_profiles: child_profile_refs.as_slice(),
        non_claims: non_claim_refs.as_slice(),
    })
    .expect("Rust must admit the checked Nickel policy");
    assert_eq!(policy.cohort_revision(), "c5991bc853d27754aed345c13ff4d2e05c40f1f4");
    assert_eq!(policy.identity().len(), 64);
}

fn text(value: &serde_json::Value) -> &str {
    value.as_str().expect("field must be text")
}

fn number_u32(value: &serde_json::Value) -> u32 {
    u32::try_from(number_u64(value)).expect("number must fit u32")
}

fn number_u64(value: &serde_json::Value) -> u64 {
    value.as_u64().expect("field must be a non-negative integer")
}

fn strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .expect("field must be an array")
        .iter()
        .map(|item| String::from(text(item)))
        .collect()
}

fn refs(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

fn operation(label: &str) -> HostOperationClass {
    match label {
        "file_read" => HostOperationClass::FileRead,
        "file_edit" => HostOperationClass::FileEdit,
        "shell" => HostOperationClass::Shell,
        "tool" => HostOperationClass::Tool,
        "subagent" => HostOperationClass::Subagent,
        "context_read" => HostOperationClass::ContextRead,
        "kernel_clear" => HostOperationClass::KernelClear,
        other => panic!("unsupported operation label {other}"),
    }
}
