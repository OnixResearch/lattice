# Typed policy for the non-default live terminal agent client.
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_event_batch = 1024 in
let maximum_prompt_bytes = 65536 in
let maximum_poll_ms = 1000 in
let maximum_dimension = 4096 in
let Config = {
  schema | contracts.exact "LIVE_TERMINAL_SCHEMA" "animus.live-terminal-agent-host.v1",
  protocol_version | contracts.positive_integer "LIVE_TERMINAL_PROTOCOL" 1,
  connection_mode | contracts.exact "LIVE_TERMINAL_CONNECTION" "daemon",
  endpoint_authority | contracts.exact "LIVE_TERMINAL_ENDPOINT_AUTHORITY" "explicit-capability-root",
  endpoint_name | contracts.safe_identifier "LIVE_TERMINAL_ENDPOINT_NAME",
  root_id | contracts.safe_identifier "LIVE_TERMINAL_ROOT_ID",
  client_id | contracts.safe_identifier "LIVE_TERMINAL_CLIENT_ID",
  terminal_backend | contracts.exact "LIVE_TERMINAL_BACKEND" "crossterm",
  raw_mode | Bool,
  alternate_screen | Bool,
  hide_cursor | Bool,
  max_event_batch | contracts.positive_integer "LIVE_TERMINAL_MAX_EVENTS" maximum_event_batch,
  max_prompt_bytes | contracts.positive_integer "LIVE_TERMINAL_MAX_PROMPT" maximum_prompt_bytes,
  poll_interval_ms | contracts.positive_integer "LIVE_TERMINAL_POLL_INTERVAL" maximum_poll_ms,
  max_width | contracts.positive_integer "LIVE_TERMINAL_MAX_WIDTH" maximum_dimension,
  max_height | contracts.positive_integer "LIVE_TERMINAL_MAX_HEIGHT" maximum_dimension,
  non_claims | Array String,
} in
let ConfigWithRules = contracts.with_validator Config (fun value =>
  if !value.raw_mode || !value.alternate_screen || !value.hide_cursor then
    'Error { message = "LIVE_TERMINAL_LIFECYCLE_REQUIRED" }
  else if std.array.length value.non_claims != 7 then
    'Error { message = "LIVE_TERMINAL_NON_CLAIMS" }
  else
    'Ok
) in
{
  live_terminal_agent_host_contract = ConfigWithRules,
  live_terminal_agent_host | ConfigWithRules = {
    schema = "animus.live-terminal-agent-host.v1",
    protocol_version = 1,
    connection_mode = "daemon",
    endpoint_authority = "explicit-capability-root",
    endpoint_name = "animus-local-daemon.sock",
    root_id = "root-terminal-one",
    client_id = "terminal-client-one",
    terminal_backend = "crossterm",
    raw_mode = true,
    alternate_screen = true,
    hide_cursor = true,
    max_event_batch = 64,
    max_prompt_bytes = 4096,
    poll_interval_ms = 25,
    max_width = 512,
    max_height = 256,
    non_claims = [
      "not-arbitrary-terminal-compatibility",
      "not-remote-security",
      "not-crash-proof-restoration",
      "not-pty-migration",
      "not-security-sandbox",
      "not-production-readiness",
      "not-release-eligibility",
    ],
  },
}
