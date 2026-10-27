# Typed configuration for the non-default pueue host.
# r[impl agent_harness_runtime.pueue_host.configuration]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.pueue-host.v1" in
let maximum_timeout_ms = 300000 in
let maximum_timeout_seconds = 86400 in
let maximum_byte_bound = 16777216 in
let maximum_count_bound = 65536 in
let command_timeout_value_ms = 30000 in
let daemon_timeout_value_ms = 5000 in
let poll_interval_value_ms = 50 in
let wait_poll_attempt_count = 80 in
let task_kill_after_value_seconds = 5 in
let teardown_timeout_value_ms = 2000 in
let backend_output_bytes = 1048576 in
let visible_log_lines = 1000 in
let visible_log_bytes = 65536 in
let process_timeout_value_seconds = 3600 in
let process_log_byte_limit = 1048576 in
let required_non_claims = [
  "command_correctness",
  "scheduler_reliability",
  "sandboxing",
  "production_readiness",
  "release_eligibility",
] in

let SafeLabel = contracts.safe_identifier "PUEUE_SAFE_LABEL" in
let EnvPrefix = contracts.stable_validator "PUEUE_ENVIRONMENT_PREFIX" (fun value =>
  std.is_string value && std.string.is_match "^[A-Z][A-Z0-9_]*_$" value
) in
let AbsolutePath = contracts.safe_absolute_policy_path "PUEUE_ABSOLUTE_PATH" in
let ConfigLeaf = contracts.safe_repository_relative_path "PUEUE_CONFIG_LEAF" in
let Environment = {
  key | contracts.stable_validator "PUEUE_ENVIRONMENT_KEY" (fun value =>
    std.is_string value && std.string.is_match "^[A-Z][A-Z0-9_]*$" value
  ),
  value | contracts.non_empty_string "PUEUE_ENVIRONMENT_VALUE",
} in
let Metadata = {
  key | SafeLabel,
  value | SafeLabel,
} in
let Profile = {
  name | SafeLabel,
  backend | contracts.exact "PUEUE_PROFILE_BACKEND" "pueue",
  program | AbsolutePath,
  args | Array (contracts.non_empty_string "PUEUE_PROFILE_ARGUMENT"),
  cwd | AbsolutePath,
  env | Array Environment,
  timeout_seconds | contracts.positive_integer "PUEUE_PROFILE_TIMEOUT" maximum_timeout_seconds,
  log_bytes | contracts.positive_integer "PUEUE_PROFILE_LOG_BYTES" maximum_byte_bound,
  notify_on_complete | Bool,
  metadata | Array Metadata,
} in
let Bounds = {
  command_timeout_ms | contracts.positive_integer "PUEUE_COMMAND_TIMEOUT" maximum_timeout_ms,
  daemon_timeout_ms | contracts.positive_integer "PUEUE_DAEMON_TIMEOUT" maximum_timeout_ms,
  poll_interval_ms | contracts.positive_integer "PUEUE_POLL_INTERVAL" maximum_timeout_ms,
  wait_poll_attempts | contracts.positive_integer "PUEUE_WAIT_POLL_ATTEMPTS" maximum_count_bound,
  task_kill_after_seconds | contracts.positive_integer "PUEUE_TASK_KILL_AFTER" maximum_timeout_seconds,
  teardown_timeout_ms | contracts.positive_integer "PUEUE_TEARDOWN_TIMEOUT" maximum_timeout_ms,
  max_backend_stdout_bytes | contracts.positive_integer "PUEUE_STDOUT_BOUND" maximum_byte_bound,
  max_backend_stderr_bytes | contracts.positive_integer "PUEUE_STDERR_BOUND" maximum_byte_bound,
  max_log_lines | contracts.positive_integer "PUEUE_LOG_LINE_BOUND" maximum_count_bound,
  max_log_bytes | contracts.positive_integer "PUEUE_LOG_BYTE_BOUND" maximum_byte_bound,
} in
let Interaction = contracts.enum "PUEUE_INTERACTION" ["wait", "poll"] in
let ProcessPolicy = {
  allowed_env_prefixes | Array EnvPrefix,
  allowed_cwd_prefixes | Array AbsolutePath,
  max_timeout_seconds | contracts.positive_integer "PUEUE_POLICY_TIMEOUT" maximum_timeout_seconds,
  max_log_bytes | contracts.positive_integer "PUEUE_POLICY_LOG_BYTES" maximum_byte_bound,
} in
let HostShape = {
  schema | contracts.exact "PUEUE_CONFIG_SCHEMA" schema_identity,
  namespace | {
    id | SafeLabel,
    root | AbsolutePath,
    config_leaf | ConfigLeaf,
  },
  executables | {
    pueue | AbsolutePath,
    pueued | AbsolutePath,
    timeout | AbsolutePath,
  },
  authority | {
    owner_label | SafeLabel,
    profile_source_label | SafeLabel,
    environment_policy | SafeLabel,
    timeout_policy | SafeLabel,
  },
  capabilities | {
    interactions | Array Interaction,
  },
  bounds | Bounds,
  process_policy | ProcessPolicy,
  profiles | Array Profile,
  non_claims | Array (contracts.enum "PUEUE_NON_CLAIM" required_non_claims),
} in
let env_is_allowed = fun policy entry =>
  std.array.any (fun prefix => contracts.starts_with prefix entry.key) policy.allowed_env_prefixes
in
let validate = fun value =>
  if std.array.length value.profiles == 0 then
    'Error { message = "PUEUE_EMPTY_PROFILES" }
  else if !contracts.unique_by (fun profile => profile.name) value.profiles then
    'Error { message = "PUEUE_DUPLICATE_PROFILE" }
  else if value.executables.pueue == value.executables.pueued
    || value.executables.pueue == value.executables.timeout
    || value.executables.pueued == value.executables.timeout
  then
    'Error { message = "PUEUE_EXECUTABLE_COLLISION" }
  else if !std.array.all (fun profile => profile.timeout_seconds <= value.process_policy.max_timeout_seconds) value.profiles then
    'Error { message = "PUEUE_PROFILE_TIMEOUT_POLICY" }
  else if !std.array.all (fun profile => profile.log_bytes <= value.process_policy.max_log_bytes) value.profiles then
    'Error { message = "PUEUE_PROFILE_LOG_POLICY" }
  else if !std.array.all (fun profile => std.array.all (env_is_allowed value.process_policy) profile.env) value.profiles then
    'Error { message = "PUEUE_PROFILE_ENVIRONMENT_POLICY" }
  else if !std.array.all (fun item => std.array.elem item value.non_claims) required_non_claims then
    'Error { message = "PUEUE_MISSING_NON_CLAIM" }
  else
    'Ok
in
let PueueHostContract = contracts.with_validator HostShape validate in
{
  pueue_host_contract = PueueHostContract,
  pueue_host | PueueHostContract = {
    schema = schema_identity,
    namespace = {
      id = "animus-pueue-fixture",
      root = "/tmp/animus-pueue-fixture",
      config_leaf = "pueue.yml",
    },
    executables = {
      pueue = "/run/current-system/sw/bin/pueue",
      pueued = "/run/current-system/sw/bin/pueued",
      timeout = "/run/current-system/sw/bin/timeout",
    },
    authority = {
      owner_label = "animus-pueue-host",
      profile_source_label = "animus-pueue-profiles",
      environment_policy = "animus-pueue-env",
      timeout_policy = "animus-pueue-timeout",
    },
    capabilities = {
      interactions = ["wait", "poll"],
    },
    bounds = {
      command_timeout_ms = command_timeout_value_ms,
      daemon_timeout_ms = daemon_timeout_value_ms,
      poll_interval_ms = poll_interval_value_ms,
      wait_poll_attempts = wait_poll_attempt_count,
      task_kill_after_seconds = task_kill_after_value_seconds,
      teardown_timeout_ms = teardown_timeout_value_ms,
      max_backend_stdout_bytes = backend_output_bytes,
      max_backend_stderr_bytes = backend_output_bytes,
      max_log_lines = visible_log_lines,
      max_log_bytes = visible_log_bytes,
    },
    process_policy = {
      allowed_env_prefixes = ["ANIMUS_"],
      allowed_cwd_prefixes = ["/tmp"],
      max_timeout_seconds = process_timeout_value_seconds,
      max_log_bytes = process_log_byte_limit,
    },
    profiles = [{
      name = "fixture-check",
      backend = "pueue",
      program = "/run/current-system/sw/bin/true",
      args = [],
      cwd = "/tmp",
      env = [{ key = "ANIMUS_TASK", value = "fixture" }],
      timeout_seconds = process_timeout_value_seconds,
      log_bytes = process_log_byte_limit,
      notify_on_complete = false,
      metadata = [{ key = "identity.class", value = "fixture" }],
    }],
    non_claims = required_non_claims,
  },
}
