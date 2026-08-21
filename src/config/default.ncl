let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let PositiveNatural = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0 && value % 1 == 0
) in
let ProcessLimits = {
  stdin_max_bytes | PositiveNumber,
  stdout_max_bytes | PositiveNumber,
  stderr_max_bytes | PositiveNumber,
  command_agent_timeout_secs | PositiveNumber,
  poll_interval_millis | PositiveNumber,
  teardown_grace_millis | PositiveNumber,
} in
let ConcurrencyPolicy = {
  total_slots | PositiveNatural,
  resource_lanes = {
    pure | PositiveNatural,
    process | PositiveNatural,
    network | PositiveNatural,
    filesystem | PositiveNatural,
    image | PositiveNatural,
    agent | PositiveNatural,
    operation | PositiveNatural,
    subworkflow | PositiveNatural,
    approval | PositiveNatural,
  },
} in
let stream_limit_bytes = 64 * 1024 * 1024 in
let default_command_agent_timeout_secs = 600 in
let default_process_poll_interval_millis = 10 in
let default_process_teardown_grace_millis = 1000 in
let default_frontier_slots = 4 in
let concurrency = {
  total_slots = default_frontier_slots,
  resource_lanes = {
    pure = default_frontier_slots,
    process = default_frontier_slots,
    network = default_frontier_slots,
    filesystem = default_frontier_slots,
    image = default_frontier_slots,
    agent = default_frontier_slots,
    operation = default_frontier_slots,
    subworkflow = default_frontier_slots,
    approval = default_frontier_slots,
  },
} | ConcurrencyPolicy in
let configured_process_limits = {
  stdin_max_bytes = stream_limit_bytes,
  stdout_max_bytes = stream_limit_bytes,
  stderr_max_bytes = stream_limit_bytes,
  command_agent_timeout_secs = default_command_agent_timeout_secs,
  poll_interval_millis = default_process_poll_interval_millis,
  teardown_grace_millis = default_process_teardown_grace_millis,
} | ProcessLimits in
{
  data_dir = "~/.lattice",
  default_timeout_secs = 60,
  shell = "sh",
  max_parallelism = concurrency.total_slots,
  resource_lanes = concurrency.resource_lanes,
  process_limits = configured_process_limits,
  agent_runtime = {
    max_output_bytes = 64 * 1024 * 1024,
    runtimes = {
      pi = {
        command = "pi",
        args = ["--no-session", "-p", "{prompt}"],
      },
      clankers = {
        command = "clankers",
        args = ["-p", "{prompt}"],
      },
    },
    openai_compatible = {},
  },
}
