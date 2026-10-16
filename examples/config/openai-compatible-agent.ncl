# Native OpenAI-compatible runtime example for the existing Pi/Clankers target labels.
# Keep API-key values out of this file. For HTTPS providers, set api_key_env to
# a binding name supplied by each agent node's secret_env credential reference.

let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let ProcessLimits = {
  stdin_max_bytes | PositiveNumber,
  stdout_max_bytes | PositiveNumber,
  stderr_max_bytes | PositiveNumber,
  command_agent_timeout_secs | PositiveNumber,
  poll_interval_millis | PositiveNumber,
  teardown_grace_millis | PositiveNumber,
} in
let OpenAiRuntime = {
  endpoint | String,
  model | String,
  api_key_env | Dyn,
  timeout_secs | Number,
  max_tokens | Number,
  temperature_milliunits | Number,
  max_request_bytes | Number,
  max_response_bytes | Number,
  enable_thinking | Dyn,
  system_prompt | String,
} in
let Projection = std.contract.from_predicate (fun value =>
  std.is_string value
  && (value == "json_pretty" || value == "json_compact" || value == "toon_v3")
) in
let StructuredInputProjectionPolicy = {
  pi | optional | Projection,
  clankers | optional | Projection,
  animus | optional | Projection,
} in
let AgentRuntime = {
  max_output_bytes | Number,
  runtimes | Dyn,
  structured_input_projection | StructuredInputProjectionPolicy,
  openai_compatible | {
    pi | OpenAiRuntime,
    clankers | OpenAiRuntime,
  },
} in
let AppConfig = {
  data_dir | String,
  default_timeout_secs | Number,
  shell | String,
  effect_authority | optional | std.contract.from_predicate (fun value =>
    value == null || std.is_string value
  ),
  max_parallelism | Number,
  agent_runtime | AgentRuntime,
  process_limits | ProcessLimits,
} in
let local_chat_completions_endpoint = "http://127.0.0.1:13305/v1/chat/completions" in
let local_model_name = "VibeThinker-3B" in
let agent_timeout_secs = 600 in
let agent_max_tokens = 8192 in
let one_mebibyte = 1048576 in
let global_output_budget_bytes = 67108864 in
let shell_timeout_secs = 60 in
let workflow_parallelism = 4 in
let process_stdin_max_bytes = global_output_budget_bytes in
let process_stdout_max_bytes = global_output_budget_bytes in
let process_stderr_max_bytes = global_output_budget_bytes in
let process_poll_interval_millis = 10 in
let process_teardown_grace_millis = 1000 in
let local_runtime = {
  endpoint = local_chat_completions_endpoint,
  model = local_model_name,
  api_key_env = null,
  timeout_secs = agent_timeout_secs,
  max_tokens = agent_max_tokens,
  temperature_milliunits = 0,
  max_request_bytes = one_mebibyte,
  max_response_bytes = one_mebibyte,
  enable_thinking = false,
  system_prompt = "Return only the output requested by the role prompt.",
} in
(
{
  data_dir = "~/.lattice",
  default_timeout_secs = shell_timeout_secs,
  shell = "sh",
  max_parallelism = workflow_parallelism,
  process_limits = {
    stdin_max_bytes = process_stdin_max_bytes,
    stdout_max_bytes = process_stdout_max_bytes,
    stderr_max_bytes = process_stderr_max_bytes,
    command_agent_timeout_secs = agent_timeout_secs,
    poll_interval_millis = process_poll_interval_millis,
    teardown_grace_millis = process_teardown_grace_millis,
  },
  agent_runtime = {
    max_output_bytes = global_output_budget_bytes,
    # Explicitly empty: each target must select exactly one transport.
    runtimes = {},
    # Experimental and target-scoped. Omit a target to retain pretty JSON.
    structured_input_projection = {
      pi = "toon_v3",
      clankers = "json_compact",
    },
    openai_compatible = {
      pi = local_runtime,
      clankers = local_runtime,
    },
  },
}
) | AppConfig
