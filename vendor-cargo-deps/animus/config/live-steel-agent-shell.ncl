# Typed composition for the non-default live Steel agent shell.

let NonEmptyString = std.contract.from_predicate (fun value => value != "") in
let Positive = std.contract.from_predicate (fun value => value > 0) in
let Blake3 = std.contract.from_predicate (fun value => std.string.length value == 64) in
let ComponentSet = {
  provider | NonEmptyString,
  durable_store | NonEmptyString,
  coding_session | NonEmptyString,
  steel_kernel | NonEmptyString,
  resumable_bridge | NonEmptyString,
  rlm_executor | NonEmptyString,
  child_service | NonEmptyString,
  skill_loader | NonEmptyString,
  control | NonEmptyString,
  event_sink | NonEmptyString,
  receipt_sink | NonEmptyString,
} in
let Provider = {
  provider_id | NonEmptyString,
  model_id | NonEmptyString,
  endpoint_identity | NonEmptyString,
  endpoint_url | NonEmptyString,
  credential_reference | NonEmptyString,
  reasoning_effort | NonEmptyString,
  tool_name | NonEmptyString,
  tool_schema | NonEmptyString,
} in
let Store = {
  capability_label | NonEmptyString,
  session_journal | NonEmptyString,
  session_lock | NonEmptyString,
  kernel_journal | NonEmptyString,
  kernel_lock | NonEmptyString,
  durability | NonEmptyString,
  locking | NonEmptyString,
  privacy | NonEmptyString,
} in
let Services = {
  child | NonEmptyString,
  context | NonEmptyString,
  control | NonEmptyString,
  events | NonEmptyString,
  receipts | NonEmptyString,
} in
let Bounds = {
  provider_timeout_ms | Positive,
  max_provider_response_bytes | Positive,
  max_output_tokens | Positive,
  max_tool_result_chars | Positive,
  max_queue_items | Positive,
  max_file_read_bytes | Positive,
  max_context_bytes | Positive,
  max_context_lines | Positive,
  max_child_status_checks | Positive,
  max_source_chars | Positive,
  max_environment_chars | Positive,
  max_bindings | Positive,
  max_definitions | Positive,
  max_evaluation_fuel | Positive,
  max_recursion_depth | Positive,
  max_allocation_bytes | Positive,
  max_host_request_chars | Positive,
  max_child_depth | Positive,
  max_child_fan_out | Positive,
  max_child_tokens | Positive,
  max_child_wall_time_ms | Positive,
  max_child_result_chars | Positive,
  max_host_calls_per_cell | Positive,
  max_host_result_chars | Positive,
  max_cumulative_host_result_chars | Positive,
  max_bridge_nesting | Positive,
  max_bridge_evaluation_fuel | Positive,
  max_bridge_wall_time_ms | Positive,
} in
let Authority = {
  component_identity | Blake3,
  authorization_identity | Blake3,
  route_binding_identity | Blake3,
  route_scope_identity | Blake3,
  operation_identity | Blake3,
  resource_identity | Blake3,
  generation | Positive,
} in
let Skill = {
  name | NonEmptyString,
  version | NonEmptyString,
  manifest_identity | Blake3,
  frozen_source_identity | Blake3,
  source_identity | Blake3,
  exports | Array NonEmptyString,
  capabilities | Array NonEmptyString,
  max_source_chars | Positive,
  max_evaluation_fuel | Positive,
} in
let Evidence = {
  policy | NonEmptyString,
  redaction_marker | NonEmptyString,
  non_claims | Array NonEmptyString,
} in
let LiveSteelAgentShell = {
  schema | NonEmptyString,
  application_id | NonEmptyString,
  session_identity | Blake3,
  branch_id | NonEmptyString,
  components | ComponentSet,
  provider | Provider,
  store | Store,
  services | Services,
  bounds | Bounds,
  authority | Authority,
  admitted_operations | Array NonEmptyString,
  admitted_child_profiles | Array NonEmptyString,
  command_profiles | Array NonEmptyString,
  tool_bindings | Array NonEmptyString,
  selected_skills | Array Skill,
  evidence | Evidence,
} in

let hash_a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let hash_b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" in
let hash_c = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" in
let hash_d = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd" in
let hash_e = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" in
let skill_source_identity = "6ef8723a0b0594885ef0a0f49d8a58827e58b0166746b370f1505c8a5afe31cc" in
let configured_provider_timeout_ms = 30000 in
let configured_max_provider_response_bytes = 65536 in
let configured_max_output_tokens = 64 in
let configured_max_tool_result_chars = 4096 in
let configured_max_queue_items = 8 in
let configured_max_file_read_bytes = 4096 in
let configured_max_context_bytes = 4096 in
let configured_max_context_lines = 64 in
let configured_max_child_status_checks = 8 in
let configured_max_source_chars = 4096 in
let configured_max_environment_chars = 65536 in
let configured_max_bindings = 64 in
let configured_max_definitions = 128 in
let configured_max_evaluation_fuel = 100000 in
let configured_max_recursion_depth = 64 in
let configured_max_allocation_bytes = 4194304 in
let configured_max_host_request_chars = 16384 in
let configured_max_child_depth = 2 in
let configured_max_child_fan_out = 4 in
let configured_max_child_tokens = 32000 in
let configured_max_child_wall_time_ms = 300000 in
let configured_max_child_result_chars = 16384 in
let configured_max_host_calls_per_cell = 16 in
let configured_max_host_result_chars = 32768 in
let configured_max_cumulative_host_result_chars = 131072 in
let configured_max_bridge_nesting = 4 in
let configured_max_bridge_evaluation_fuel = 1000000 in
let configured_max_bridge_wall_time_ms = 300000 in
{
  live_steel_agent_shell_contract = LiveSteelAgentShell,
  live_steel_agent_shell | LiveSteelAgentShell = {
    schema = "animus.live-steel-agent-shell.v1",
    application_id = "animus-live-steel-agent",
    session_identity = hash_c,
    branch_id = "branch.main",
    components = {
      provider = "animus-std-openai:reqwest-tool-stream",
      durable_store = "animus-std:durable-session-store",
      coding_session = "animus-std:CodingSessionHost",
      steel_kernel = "animus-steel:SteelRlmKernel",
      resumable_bridge = "animus-runtime:execute_bridged_cell",
      rlm_executor = "animus-std-rlm-host:RlmHostExecutor",
      child_service = "animus-std:ConcreteChildSessionService",
      skill_loader = "animus-steel:load_skill_modules",
      control = "animus-live-steel-agent:ExplicitRunControl",
      event_sink = "animus-live-steel-agent:MetadataEventSink",
      receipt_sink = "animus-live-steel-agent:RunReceipt",
    },
    provider = {
      provider_id = "openai",
      model_id = "gpt-5-mini",
      endpoint_identity = "openai-responses",
      endpoint_url = "https://api.openai.com/v1/responses",
      credential_reference = "brokered:injected-openai-api-key",
      reasoning_effort = "low",
      tool_name = "steel_repl",
      tool_schema = "animus.steel-repl.v1",
    },
    store = {
      capability_label = "capability.live-steel-agent-store-root",
      session_journal = "live-session.log",
      session_lock = "live-session.lock",
      kernel_journal = "live-kernel.log",
      kernel_lock = "live-kernel.lock",
      durability = "sync_all_each_commit",
      locking = "exclusive_create_new",
      privacy = "replay_private",
    },
    services = {
      child = "child-service.live-steel-agent",
      context = "context-service.explicit-map",
      control = "control-service.explicit-cancellation",
      events = "event-service.metadata-only",
      receipts = "receipt-service.metadata-only",
    },
    bounds = {
      provider_timeout_ms = configured_provider_timeout_ms,
      max_provider_response_bytes = configured_max_provider_response_bytes,
      max_output_tokens = configured_max_output_tokens,
      max_tool_result_chars = configured_max_tool_result_chars,
      max_queue_items = configured_max_queue_items,
      max_file_read_bytes = configured_max_file_read_bytes,
      max_context_bytes = configured_max_context_bytes,
      max_context_lines = configured_max_context_lines,
      max_child_status_checks = configured_max_child_status_checks,
      max_source_chars = configured_max_source_chars,
      max_environment_chars = configured_max_environment_chars,
      max_bindings = configured_max_bindings,
      max_definitions = configured_max_definitions,
      max_evaluation_fuel = configured_max_evaluation_fuel,
      max_recursion_depth = configured_max_recursion_depth,
      max_allocation_bytes = configured_max_allocation_bytes,
      max_host_request_chars = configured_max_host_request_chars,
      max_child_depth = configured_max_child_depth,
      max_child_fan_out = configured_max_child_fan_out,
      max_child_tokens = configured_max_child_tokens,
      max_child_wall_time_ms = configured_max_child_wall_time_ms,
      max_child_result_chars = configured_max_child_result_chars,
      max_host_calls_per_cell = configured_max_host_calls_per_cell,
      max_host_result_chars = configured_max_host_result_chars,
      max_cumulative_host_result_chars = configured_max_cumulative_host_result_chars,
      max_bridge_nesting = configured_max_bridge_nesting,
      max_bridge_evaluation_fuel = configured_max_bridge_evaluation_fuel,
      max_bridge_wall_time_ms = configured_max_bridge_wall_time_ms,
    },
    authority = {
      component_identity = hash_a,
      authorization_identity = hash_b,
      route_binding_identity = hash_d,
      route_scope_identity = hash_e,
      operation_identity = hash_a,
      resource_identity = hash_c,
      generation = 1,
    },
    admitted_operations = ["context_read", "subagent"],
    admitted_child_profiles = ["focused-child"],
    command_profiles = [],
    tool_bindings = ["steel_repl:animus.steel-repl.v1"],
    selected_skills = [
      {
        name = "live-shell-arithmetic",
        version = "v1",
        manifest_identity = hash_a,
        frozen_source_identity = hash_b,
        source_identity = skill_source_identity,
        exports = ["live-shell-add-two"],
        capabilities = [],
        max_source_chars = 128,
        max_evaluation_fuel = 1024,
      },
    ],
    evidence = {
      policy = "metadata-only",
      redaction_marker = "prompt-provider-tool-steel-host-child-credential-and-path-content-omitted",
      non_claims = [
        "provider_correctness",
        "tool_correctness",
        "task_success",
        "sandboxing",
        "daemon_readiness",
        "terminal_readiness",
        "production_readiness",
        "release_eligibility",
      ],
    },
  },
}
