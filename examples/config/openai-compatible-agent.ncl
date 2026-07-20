# Native OpenAI-compatible runtime example for the existing Pi/Clankers target labels.
# Keep API-key values out of this file. For HTTPS providers, set api_key_env to
# a binding name supplied by each agent node's secret_env credential reference.

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
let AgentRuntime = {
  max_output_bytes | Number,
  runtimes | Dyn,
  openai_compatible | {
    pi | OpenAiRuntime,
    clankers | OpenAiRuntime,
  },
} in
let AppConfig = {
  data_dir | String,
  default_timeout_secs | Number,
  shell | String,
  max_parallelism | Number,
  agent_runtime | AgentRuntime,
} in
let local_chat_completions_endpoint = "http://127.0.0.1:13305/v1/chat/completions" in
let agent_timeout_secs = 120 in
let agent_max_tokens = 2048 in
let one_mebibyte = 1048576 in
let global_output_budget_bytes = 67108864 in
let shell_timeout_secs = 60 in
let workflow_parallelism = 4 in
let local_runtime = {
  endpoint = local_chat_completions_endpoint,
  model = "local-model",
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
  agent_runtime = {
    max_output_bytes = global_output_budget_bytes,
    # Explicitly empty: each target must select exactly one transport.
    runtimes = {},
    openai_compatible = {
      pi = local_runtime,
      clankers = local_runtime,
    },
  },
}
) | AppConfig
