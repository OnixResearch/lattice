# Typed policy for the non-default local daemon supervisor and worker shell.
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_frame_bytes = 1048576 in
let maximum_events = 4096 in
let maximum_attachments = 64 in
let maximum_workers = 256 in
let maximum_deadline_ms = 300000 in
let Config = {
  schema | contracts.exact "LOCAL_DAEMON_SCHEMA" "animus.local-daemon.v1",
  protocol_version | contracts.positive_integer "LOCAL_DAEMON_PROTOCOL_VERSION" 1,
  transport | contracts.exact "LOCAL_DAEMON_TRANSPORT" "unix-stream",
  endpoint_authority | contracts.exact "LOCAL_DAEMON_ENDPOINT_AUTHORITY" "explicit-capability-root",
  endpoint_name | contracts.safe_identifier "LOCAL_DAEMON_ENDPOINT_NAME",
  registry_name | contracts.safe_identifier "LOCAL_DAEMON_REGISTRY_NAME",
  worker_binary | contracts.safe_identifier "LOCAL_DAEMON_WORKER_BINARY",
  worker_feature | contracts.exact "LOCAL_DAEMON_WORKER_FEATURE" "live-worker",
  max_frame_bytes | contracts.positive_integer "LOCAL_DAEMON_MAX_FRAME_BYTES" maximum_frame_bytes,
  max_events | contracts.positive_integer "LOCAL_DAEMON_MAX_EVENTS" maximum_events,
  max_attachments | contracts.positive_integer "LOCAL_DAEMON_MAX_ATTACHMENTS" maximum_attachments,
  max_workers | contracts.positive_integer "LOCAL_DAEMON_MAX_WORKERS" maximum_workers,
  operation_deadline_ms | contracts.positive_integer "LOCAL_DAEMON_OPERATION_DEADLINE" maximum_deadline_ms,
  cancel_poll_ms | contracts.positive_integer "LOCAL_DAEMON_CANCEL_POLL" maximum_deadline_ms,
  non_claims | Array String,
} in
let ConfigWithRules = contracts.with_validator Config (fun value =>
  if value.cancel_poll_ms >= value.operation_deadline_ms then
    'Error { message = "LOCAL_DAEMON_CANCEL_POLL_ORDER" }
  else if std.array.length value.non_claims != 7 then
    'Error { message = "LOCAL_DAEMON_NON_CLAIMS" }
  else
    'Ok
) in
{
  local_daemon_config_contract = ConfigWithRules,
  local_daemon_config | ConfigWithRules = {
    schema = "animus.local-daemon.v1",
    protocol_version = 1,
    transport = "unix-stream",
    endpoint_authority = "explicit-capability-root",
    endpoint_name = "animus-local-daemon.sock",
    registry_name = "animus-local-daemon-registry.json",
    worker_binary = "animus-local-daemon-worker",
    worker_feature = "live-worker",
    max_frame_bytes = 1048576,
    max_events = 4096,
    max_attachments = 32,
    max_workers = 128,
    operation_deadline_ms = 30000,
    cancel_poll_ms = 25,
    non_claims = [
      "not-remote-security",
      "not-security-sandbox",
      "not-multi-machine-supervision",
      "not-high-availability",
      "not-service-manager-integration",
      "not-production-readiness",
      "not-release-eligibility",
    ],
  },
}
