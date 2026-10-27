# Typed model-facing Steel RLM kernel policy.
# r[impl agent_harness_runtime.rlm_kernel.profile]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.rlm-kernel-policy.v1" in
let reviewed_revision = "c5991bc853d27754aed345c13ff4d2e05c40f1f4" in
let operation_classes = [
  "file_read",
  "file_edit",
  "shell",
  "tool",
  "subagent",
  "context_read",
  "kernel_clear",
] in
let required_non_claims = [
  "security_sandboxing",
  "model_correctness",
  "task_completion",
  "production_readiness",
  "release_eligibility",
] in
let positive_u32_max = 4294967295 in
let positive_u64_safe_max = 9007199254740991 in
let git_revision = contracts.stable_validator "RLM_KERNEL_COHORT_REVISION" (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Bounds = {
  max_source_chars | contracts.positive_integer "RLM_KERNEL_SOURCE_BOUND" positive_u32_max,
  max_environment_chars | contracts.positive_integer "RLM_KERNEL_ENVIRONMENT_BOUND" positive_u32_max,
  max_bindings | contracts.positive_integer "RLM_KERNEL_BINDING_BOUND" positive_u32_max,
  max_definitions | contracts.positive_integer "RLM_KERNEL_DEFINITION_BOUND" positive_u32_max,
  max_evaluation_fuel | contracts.positive_integer "RLM_KERNEL_FUEL_BOUND" positive_u64_safe_max,
  max_recursion_depth | contracts.positive_integer "RLM_KERNEL_DEPTH_BOUND" positive_u32_max,
  max_allocation_bytes | contracts.positive_integer "RLM_KERNEL_ALLOCATION_BOUND" positive_u64_safe_max,
  max_host_request_chars | contracts.positive_integer "RLM_KERNEL_HOST_REQUEST_BOUND" positive_u32_max,
  max_child_depth | contracts.positive_integer "RLM_KERNEL_CHILD_DEPTH_BOUND" positive_u32_max,
  max_child_fan_out | contracts.positive_integer "RLM_KERNEL_CHILD_FAN_OUT_BOUND" positive_u32_max,
  max_child_tokens | contracts.positive_integer "RLM_KERNEL_CHILD_TOKEN_BOUND" positive_u64_safe_max,
  max_child_wall_time_ms | contracts.positive_integer "RLM_KERNEL_CHILD_WALL_TIME_BOUND" positive_u64_safe_max,
  max_child_result_chars | contracts.positive_integer "RLM_KERNEL_CHILD_RESULT_BOUND" positive_u32_max,
  max_host_calls_per_cell | contracts.positive_integer "RLM_BRIDGE_HOST_CALL_BOUND" positive_u32_max,
  max_host_result_chars | contracts.positive_integer "RLM_BRIDGE_HOST_RESULT_BOUND" positive_u32_max,
  max_cumulative_host_result_chars | contracts.positive_integer "RLM_BRIDGE_CUMULATIVE_RESULT_BOUND" positive_u32_max,
  max_bridge_nesting | contracts.positive_integer "RLM_BRIDGE_NESTING_BOUND" positive_u32_max,
  max_bridge_evaluation_fuel | contracts.positive_integer "RLM_BRIDGE_FUEL_BOUND" positive_u64_safe_max,
  max_bridge_wall_time_ms | contracts.positive_integer "RLM_BRIDGE_WALL_TIME_BOUND" positive_u64_safe_max,
} in
let ProfileShape = {
  schema | contracts.exact "RLM_KERNEL_SCHEMA" schema_identity,
  cohort | contracts.safe_identifier "RLM_KERNEL_COHORT",
  cohort_revision | git_revision,
  profile | contracts.safe_identifier "RLM_KERNEL_PROFILE",
  bounds | Bounds,
  admitted_operations | Array (contracts.enum "RLM_KERNEL_OPERATION_CLASS" operation_classes),
  admitted_child_profiles | Array (contracts.safe_identifier "RLM_KERNEL_CHILD_PROFILE"),
  non_claims | Array (contracts.enum "RLM_KERNEL_NON_CLAIM" required_non_claims),
} in
let profile_validator = fun value =>
  if std.array.length value.admitted_operations == 0 then
    'Error { message = "RLM_KERNEL_EMPTY_OPERATIONS" }
  else if !contracts.all_unique value.admitted_operations then
    'Error { message = "RLM_KERNEL_DUPLICATE_OPERATION" }
  else if std.array.length value.admitted_child_profiles == 0 then
    'Error { message = "RLM_KERNEL_EMPTY_CHILD_PROFILES" }
  else if !contracts.all_unique value.admitted_child_profiles then
    'Error { message = "RLM_KERNEL_DUPLICATE_CHILD_PROFILE" }
  else if !std.array.all (fun item => std.array.elem item value.non_claims) required_non_claims then
    'Error { message = "RLM_KERNEL_MISSING_NON_CLAIM" }
  else
    'Ok
in
let RlmKernelPolicyContract = contracts.with_validator ProfileShape profile_validator in
{
  rlm_kernel_policy_contract = RlmKernelPolicyContract,
  rlm_kernel_policy | RlmKernelPolicyContract = {
    schema = schema_identity,
    cohort = "prime-agent-rlm-2026-08-06",
    cohort_revision = reviewed_revision,
    profile = "model-facing-steel",
    bounds = {
      max_source_chars = 4096,
      max_environment_chars = 65536,
      max_bindings = 64,
      max_definitions = 128,
      max_evaluation_fuel = 100000,
      max_recursion_depth = 64,
      max_allocation_bytes = 4194304,
      max_host_request_chars = 16384,
      max_child_depth = 2,
      max_child_fan_out = 4,
      max_child_tokens = 32000,
      max_child_wall_time_ms = 300000,
      max_child_result_chars = 16384,
      max_host_calls_per_cell = 16,
      max_host_result_chars = 32768,
      max_cumulative_host_result_chars = 131072,
      max_bridge_nesting = 4,
      max_bridge_evaluation_fuel = 1000000,
      max_bridge_wall_time_ms = 300000,
    },
    admitted_operations = operation_classes,
    admitted_child_profiles = ["focused-child"],
    non_claims = required_non_claims,
  },
}
