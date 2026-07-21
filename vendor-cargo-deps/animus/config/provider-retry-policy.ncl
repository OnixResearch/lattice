# Typed provider retry policy source of truth.
# r[impl agent_harness_runtime.tau_provider_retry.policy]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.provider-retry-policy.v1" in
let policy_id_value = "bounded-provider-retry" in
let cancellation_required = "required" in
let deny_after_semantic = "deny_after_semantic" in
let max_attempt_limit = 8 in
let max_delay_limit_ms = 60000 in
let safe_label_limit_chars = 128 in
let rate_limited = "rate_limited" in
let transient_transport = "transient_transport" in
let service_unavailable = "service_unavailable" in
let request_timeout = "request_timeout" in
let conflict = "conflict" in
let authentication = "authentication" in
let authorization = "authorization" in
let entitlement = "entitlement" in
let policy_denied = "policy_denied" in
let capability_denied = "capability_denied" in
let invalid_request = "invalid_request" in
let unsupported_feature = "unsupported_feature" in
let malformed_request = "malformed_request" in
let unsafe_response = "unsafe_response" in
let missing_service = "missing_service" in
let missing_credential = "missing_credential" in
let unknown = "unknown" in
let canonical_classes = [
  rate_limited,
  transient_transport,
  service_unavailable,
  request_timeout,
  conflict,
  authentication,
  authorization,
  entitlement,
  policy_denied,
  capability_denied,
  invalid_request,
  unsupported_feature,
  malformed_request,
  unsafe_response,
  missing_service,
  missing_credential,
  unknown,
] in
let retryable_class_values = [
  conflict,
  rate_limited,
  request_timeout,
  service_unavailable,
  transient_transport,
] in

let _safe_label = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= safe_label_limit_chars
  && std.string.is_match "^[a-z0-9][a-z0-9._:-]*[a-z0-9]$|^[a-z0-9]$" value
in
let Mapping = {
  provider_family | contracts.safe_identifier "PROVIDER_RETRY_PROVIDER_FAMILY",
  mapping_version | contracts.safe_identifier "PROVIDER_RETRY_MAPPING_VERSION",
  native_class | contracts.safe_identifier "PROVIDER_RETRY_NATIVE_CLASS",
  canonical_class | contracts.enum "PROVIDER_RETRY_CANONICAL_CLASS" canonical_classes,
} in
let PolicyShape = {
  schema | contracts.exact "PROVIDER_RETRY_SCHEMA" schema_identity,
  policy_id | contracts.safe_identifier "PROVIDER_RETRY_POLICY_ID",
  max_attempts | contracts.integer_between "PROVIDER_RETRY_ATTEMPT_BOUND" 1 max_attempt_limit,
  max_delay_ms | contracts.integer_between "PROVIDER_RETRY_DELAY_BOUND" 0 max_delay_limit_ms,
  delay_schedule_ms | Array Number,
  cancellation_posture | contracts.enum "PROVIDER_RETRY_CANCELLATION_POSTURE" [cancellation_required],
  stream_commit_policy | contracts.enum "PROVIDER_RETRY_STREAM_POSTURE" [deny_after_semantic],
  retryable_classes | Array (contracts.enum "PROVIDER_RETRY_RETRYABLE_CLASS" retryable_class_values),
  mappings | Array Mapping,
  policy_hash | contracts.lowercase_blake3 "PROVIDER_RETRY_POLICY_HASH",
} in
let _mapping_key = fun mapping =>
  mapping.provider_family ++ ":" ++ mapping.mapping_version ++ ":" ++ mapping.native_class
in
let _delays_are_valid = fun value =>
  std.array.length value.delay_schedule_ms == value.max_attempts - 1
  && std.array.all
    (fun delay =>
      std.is_number delay
      && delay >= 0
      && delay <= value.max_delay_ms
      && delay % 1 == 0)
    value.delay_schedule_ms
in
let _whole_policy_validator = fun value =>
  if !_safe_label value.policy_id then
    'Error { message = "PROVIDER_RETRY_POLICY_ID" }
  else if std.array.length value.retryable_classes == 0 then
    'Error { message = "PROVIDER_RETRY_EMPTY_CLASSES" }
  else if !contracts.unique_by (fun class => class) value.retryable_classes then
    'Error { message = "PROVIDER_RETRY_DUPLICATE_CLASS" }
  else if std.array.length value.mappings == 0 then
    'Error { message = "PROVIDER_RETRY_EMPTY_MAPPINGS" }
  else if !contracts.unique_by _mapping_key value.mappings then
    'Error { message = "PROVIDER_RETRY_DUPLICATE_MAPPING" }
  else if !std.array.all
    (fun mapping =>
      _safe_label mapping.provider_family
      && _safe_label mapping.mapping_version
      && _safe_label mapping.native_class)
    value.mappings
  then
    'Error { message = "PROVIDER_RETRY_MAPPING_LABEL" }
  else if !_delays_are_valid value then
    'Error { message = "PROVIDER_RETRY_DELAY_SCHEDULE" }
  else
    'Ok
in
let ProviderRetryPolicy = contracts.with_validator PolicyShape _whole_policy_validator in

{
  provider_retry_policy_contract = ProviderRetryPolicy,
  provider_retry_policy | ProviderRetryPolicy = {
    schema = schema_identity,
    policy_id = policy_id_value,
    max_attempts = 4,
    max_delay_ms = 30000,
    delay_schedule_ms = [0, 250, 1000],
    cancellation_posture = cancellation_required,
    stream_commit_policy = deny_after_semantic,
    retryable_classes = retryable_class_values,
    mappings = [
      { provider_family = "mock", mapping_version = "v1", native_class = "auth", canonical_class = authentication },
      { provider_family = "mock", mapping_version = "v1", native_class = "conflict", canonical_class = conflict },
      { provider_family = "mock", mapping_version = "v1", native_class = "network-reset", canonical_class = transient_transport },
      { provider_family = "mock", mapping_version = "v1", native_class = "rate-limit", canonical_class = rate_limited },
      { provider_family = "mock", mapping_version = "v1", native_class = "service-down", canonical_class = service_unavailable },
      { provider_family = "mock", mapping_version = "v1", native_class = "timeout", canonical_class = request_timeout },
      { provider_family = "mock", mapping_version = "v1", native_class = "unsupported", canonical_class = unsupported_feature },
    ],
    policy_hash = "c4c5f883019c739cc4dd3c4edd790dfa038d095109f6bbac2e5f40f7483c15a6",
  },
}
