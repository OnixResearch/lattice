#![allow(
    dead_code,
    missing_docs,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use animus_adapter_rlm_kernel::{
    HostOperationClass, REQUIRED_NON_CLAIMS, ResourceBounds, RlmKernelPolicy, RlmKernelPolicyDraft,
};

pub const PRIME_AGENT_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
pub const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub const CHILD_PROFILE: &str = "focused-child";
pub const SOURCE_CHAR_LIMIT: u32 = 4_096;
pub const ENVIRONMENT_CHAR_LIMIT: u32 = 16_384;
pub const BINDING_LIMIT: u32 = 16;
pub const DEFINITION_LIMIT: u32 = 32;
pub const FUEL_LIMIT: u64 = 100_000;
pub const RECURSION_LIMIT: u32 = 64;
pub const ALLOCATION_LIMIT: u64 = 1_048_576;
pub const HOST_REQUEST_CHAR_LIMIT: u32 = 8_192;
pub const CHILD_DEPTH_LIMIT: u32 = 2;
pub const CHILD_FAN_OUT_LIMIT: u32 = 4;
pub const CHILD_TOKEN_LIMIT: u64 = 16_000;
pub const CHILD_WALL_TIME_LIMIT_MS: u64 = 60_000;
pub const CHILD_RESULT_CHAR_LIMIT: u32 = 8_192;
pub const HOST_CALL_LIMIT: u32 = 4;
pub const HOST_RESULT_CHAR_LIMIT: u32 = 32_768;
pub const CUMULATIVE_HOST_RESULT_CHAR_LIMIT: u32 = 65_536;
pub const BRIDGE_NESTING_LIMIT: u32 = 4;
pub const BRIDGE_FUEL_LIMIT: u64 = 500_000;
pub const BRIDGE_WALL_TIME_LIMIT_MS: u64 = 120_000;

pub const OPERATIONS: &[HostOperationClass] = &[
    HostOperationClass::FileRead,
    HostOperationClass::FileEdit,
    HostOperationClass::Shell,
    HostOperationClass::Tool,
    HostOperationClass::Subagent,
    HostOperationClass::ContextRead,
    HostOperationClass::KernelClear,
];

pub fn policy() -> RlmKernelPolicy {
    RlmKernelPolicy::from_draft(&RlmKernelPolicyDraft {
        schema: animus_adapter_rlm_kernel::RLM_KERNEL_POLICY_SCHEMA,
        cohort: "prime-agent-rlm-2026-08-06",
        cohort_revision: PRIME_AGENT_REVISION,
        profile: "model-facing-steel",
        bounds: ResourceBounds {
            max_source_chars: SOURCE_CHAR_LIMIT,
            max_environment_chars: ENVIRONMENT_CHAR_LIMIT,
            max_bindings: BINDING_LIMIT,
            max_definitions: DEFINITION_LIMIT,
            max_evaluation_fuel: FUEL_LIMIT,
            max_recursion_depth: RECURSION_LIMIT,
            max_allocation_bytes: ALLOCATION_LIMIT,
            max_host_request_chars: HOST_REQUEST_CHAR_LIMIT,
            max_child_depth: CHILD_DEPTH_LIMIT,
            max_child_fan_out: CHILD_FAN_OUT_LIMIT,
            max_child_tokens: CHILD_TOKEN_LIMIT,
            max_child_wall_time_ms: CHILD_WALL_TIME_LIMIT_MS,
            max_child_result_chars: CHILD_RESULT_CHAR_LIMIT,
            max_host_calls_per_cell: HOST_CALL_LIMIT,
            max_host_result_chars: HOST_RESULT_CHAR_LIMIT,
            max_cumulative_host_result_chars: CUMULATIVE_HOST_RESULT_CHAR_LIMIT,
            max_bridge_nesting: BRIDGE_NESTING_LIMIT,
            max_bridge_evaluation_fuel: BRIDGE_FUEL_LIMIT,
            max_bridge_wall_time_ms: BRIDGE_WALL_TIME_LIMIT_MS,
        },
        admitted_operations: OPERATIONS,
        admitted_child_profiles: &[CHILD_PROFILE],
        non_claims: REQUIRED_NON_CLAIMS,
    })
    .expect("test policy must be valid")
}
