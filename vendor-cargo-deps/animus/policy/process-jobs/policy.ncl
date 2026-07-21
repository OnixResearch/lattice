# r[impl agent_harness_runtime.nickel_policy_contracts.process_jobs]
let contracts = import "../contracts/primitives.ncl" in

let process_job_schema = "animus.process-job-profile.v1" in
let policy_schema = "animus.process-job-profiles.policy.v1" in
let current_profile_schema_version = 1 in
let maximum_profile_count = 64 in
let maximum_field_count = 64 in
let maximum_timeout_seconds = 600 in
let maximum_memory_bytes = 268435456 in
let maximum_cpu_millis = 1000 in
let maximum_log_bytes = 2097152 in
let supported_backends = ["native", "pueue"] in
let secret_environment_key_markers = [
  "TOKEN",
  "PASSWORD",
  "SECRET",
  "PRIVATE_KEY",
  "API_KEY",
  "ACCESS_KEY",
  "CREDENTIAL",
  "AUTHORIZATION",
] in
let identity_hash_algorithm_value = "BLAKE3" in
let identity_redaction_value = "hash-only" in

let ProfileSchemaVersion = contracts.exact "PROCESS_JOB_PROFILE_SCHEMA" current_profile_schema_version in
let PolicySchema = contracts.exact "PROCESS_JOB_POLICY_SCHEMA" policy_schema in
let ProfileName = contracts.safe_identifier "PROCESS_JOB_PROFILE_NAME" in
let Backend = contracts.enum "PROCESS_JOB_BACKEND" supported_backends in
let SafeAbsolutePath = contracts.safe_absolute_policy_path "PROCESS_JOB_PATH" in
let NonEmptyString = contracts.non_empty_string "PROCESS_JOB_NON_EMPTY_STRING" in
let PositiveTimeout = contracts.positive_integer "PROCESS_JOB_TIMEOUT" maximum_timeout_seconds in
let PositiveMemory = contracts.positive_integer "PROCESS_JOB_MEMORY" maximum_memory_bytes in
let PositiveCpu = contracts.positive_integer "PROCESS_JOB_CPU" maximum_cpu_millis in
let PositiveLog = contracts.positive_integer "PROCESS_JOB_LOG" maximum_log_bytes in
let Blake3Algorithm = contracts.exact "PROCESS_JOB_HASH_ALGORITHM" identity_hash_algorithm_value in
let HashOnly = contracts.exact "PROCESS_JOB_IDENTITY_REDACTION" identity_redaction_value in
let EnvironmentKey = contracts.stable_validator "PROCESS_JOB_ENV_KEY" (fun value =>
  std.is_string value && std.string.is_match "^[A-Z_][A-Z0-9_]*$" value
) in
let MetadataKey = contracts.stable_validator "PROCESS_JOB_METADATA_KEY" (fun value =>
  std.is_string value && std.string.is_match "^[a-z0-9][a-z0-9._-]*$" value
) in

let ResourceLimits = {
  timeout_seconds | PositiveTimeout,
  memory_bytes | PositiveMemory,
  cpu_millis | PositiveCpu,
  log_bytes | PositiveLog,
} in
let NotificationPolicy = {
  notify_on_complete | Bool,
} in
let MetadataField = {
  key | MetadataKey,
  value | NonEmptyString,
} in
let EnvBinding = {
  key | EnvironmentKey,
  value | String,
} in
let ProcessJobProfileShape = {
  name | ProfileName,
  backend | Backend,
  command | String,
  program | String,
  args | Array NonEmptyString,
  cwd | SafeAbsolutePath,
  writable_paths | Array SafeAbsolutePath,
  env | Array EnvBinding,
  resource_limits | ResourceLimits,
  notification | NotificationPolicy,
  metadata | Array MetadataField,
} in
let ProcessJobProfileManifestShape = {
  schema_version | ProfileSchemaVersion,
  profiles | Array ProcessJobProfileShape,
} in
let ProcessJobProfilePolicyShape = {
  schema | PolicySchema,
  profile_schema_version | ProfileSchemaVersion,
  default_backend | Backend,
  allowed_backends | Array Backend,
  allowed_env_prefixes | Array NonEmptyString,
  allowed_cwd_prefixes | Array SafeAbsolutePath,
  allowed_writable_path_prefixes | Array SafeAbsolutePath,
  max_resource_limits | ResourceLimits,
  notification_defaults | NotificationPolicy,
  identity_metadata_keys | Array MetadataKey,
  identity_hash_algorithm | Blake3Algorithm,
  identity_redaction | HashOnly,
} in

let _path_is_under = fun prefixes path =>
  std.array.any
    (fun prefix => path == prefix || contracts.starts_with (prefix ++ "/") path)
    prefixes
in

let _environment_key_is_allowed = fun prefixes key =>
  std.array.any (fun prefix => contracts.starts_with prefix key) prefixes
  && !std.array.any (fun marker => std.string.contains marker key) secret_environment_key_markers
in

let _profile_execution_form_is_valid = fun profile =>
  let has_command = std.string.length profile.command > 0 in
  let has_program = std.string.length profile.program > 0 in
  (has_command && !has_program && std.array.length profile.args == 0)
  || (!has_command && has_program)
in

let _profile_is_valid = fun policy profile =>
  _profile_execution_form_is_valid profile
  && std.array.elem profile.backend policy.allowed_backends
  && _path_is_under policy.allowed_cwd_prefixes profile.cwd
  && std.array.all (_path_is_under policy.allowed_writable_path_prefixes) profile.writable_paths
  && std.array.all (fun binding => _environment_key_is_allowed policy.allowed_env_prefixes binding.key) profile.env
  && contracts.unique_by (fun binding => binding.key) profile.env
  && contracts.unique_by (fun field => field.key) profile.metadata
  && profile.resource_limits.timeout_seconds <= policy.max_resource_limits.timeout_seconds
  && profile.resource_limits.memory_bytes <= policy.max_resource_limits.memory_bytes
  && profile.resource_limits.cpu_millis <= policy.max_resource_limits.cpu_millis
  && profile.resource_limits.log_bytes <= policy.max_resource_limits.log_bytes
in

let _policy_validator = fun policy =>
  if std.array.length policy.allowed_backends == 0 || !contracts.all_unique policy.allowed_backends then
    'Error { message = "PROCESS_JOB_ALLOWED_BACKENDS" }
  else if !std.array.elem policy.default_backend policy.allowed_backends then
    'Error { message = "PROCESS_JOB_DEFAULT_BACKEND" }
  else if std.array.length policy.allowed_env_prefixes == 0 || !contracts.all_unique policy.allowed_env_prefixes then
    'Error { message = "PROCESS_JOB_ENV_PREFIXES" }
  else if std.array.length policy.allowed_cwd_prefixes == 0 || !contracts.all_unique policy.allowed_cwd_prefixes then
    'Error { message = "PROCESS_JOB_CWD_PREFIXES" }
  else if std.array.length policy.allowed_writable_path_prefixes == 0 || !contracts.all_unique policy.allowed_writable_path_prefixes then
    'Error { message = "PROCESS_JOB_WRITABLE_PREFIXES" }
  else if !contracts.all_unique policy.identity_metadata_keys then
    'Error { message = "PROCESS_JOB_IDENTITY_METADATA_KEYS" }
  else
    'Ok
in
let ProcessJobProfilePolicy = contracts.with_validator ProcessJobProfilePolicyShape _policy_validator in

let _manifest_validator = fun policy manifest =>
  if std.array.length manifest.profiles == 0 || std.array.length manifest.profiles > maximum_profile_count then
    'Error { message = "PROCESS_JOB_PROFILE_COUNT" }
  else if !contracts.unique_by (fun profile => profile.name) manifest.profiles then
    'Error { message = "PROCESS_JOB_DUPLICATE_PROFILE" }
  else if std.array.any
    (fun profile =>
      std.array.length profile.env > maximum_field_count
      || std.array.length profile.metadata > maximum_field_count
      || std.array.length profile.writable_paths > maximum_field_count
    )
    manifest.profiles then
    'Error { message = "PROCESS_JOB_FIELD_COUNT" }
  else if !std.array.all (_profile_is_valid policy) manifest.profiles then
    'Error { message = "PROCESS_JOB_PROFILE_INVARIANT" }
  else
    'Ok
in

let policy_value = {
  schema = policy_schema,
  profile_schema_version = current_profile_schema_version,
  default_backend = "native",
  allowed_backends = supported_backends,
  allowed_env_prefixes = ["APP_"],
  allowed_cwd_prefixes = ["/repo"],
  allowed_writable_path_prefixes = ["/repo/target"],
  max_resource_limits = {
    timeout_seconds = maximum_timeout_seconds,
    memory_bytes = maximum_memory_bytes,
    cpu_millis = maximum_cpu_millis,
    log_bytes = maximum_log_bytes,
  },
  notification_defaults = {
    notify_on_complete = true,
  },
  identity_metadata_keys = ["identity.purpose"],
  identity_hash_algorithm = identity_hash_algorithm_value,
  identity_redaction = identity_redaction_value,
} in
let ProcessJobProfileManifest = contracts.with_validator
  ProcessJobProfileManifestShape
  (_manifest_validator policy_value)
in

let sample_profile_timeout_seconds = 300 in
let sample_profile_memory_bytes = 134217728 in
let sample_profile_cpu_millis = 500 in
let sample_profile_log_bytes = 1048576 in
let sample_manifest_value = {
  schema_version = current_profile_schema_version,
  profiles = [
    {
      name = "quick-check",
      backend = "pueue",
      command = "",
      program = "cargo",
      args = ["check", "--tests"],
      cwd = "/repo",
      writable_paths = ["/repo/target"],
      env = [
        {
          key = "APP_MODE",
          value = "ci",
        },
      ],
      resource_limits = {
        timeout_seconds = sample_profile_timeout_seconds,
        memory_bytes = sample_profile_memory_bytes,
        cpu_millis = sample_profile_cpu_millis,
        log_bytes = sample_profile_log_bytes,
      },
      notification = {
        notify_on_complete = true,
      },
      metadata = [
        {
          key = "identity.purpose",
          value = "developer-smoke",
        },
      ],
    },
  ],
} in
{
  process_job_profile_policy_contract = ProcessJobProfilePolicy,
  process_job_profile_manifest_contract = ProcessJobProfileManifest,
  process_job_profile_policy | ProcessJobProfilePolicy = policy_value,
  sample_profile_manifest | ProcessJobProfileManifest = sample_manifest_value,
  invalid_schema_fixture | ProcessJobProfileManifest = {
    schema_version = "not-a-number",
    profiles = [],
  },
  schema_label = process_job_schema,
}
