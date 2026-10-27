# Typed configuration for the non-default concrete MCP standard-input host.
# r[impl agent_harness_runtime.mcp_client_host.configuration]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.mcp-client-host.v1" in
let mcp_protocol_version = "2025-06-18" in
let mcp_cohort_revision = "b25c0874bf0ba699a58e21ef06f659d839659de3" in
let mcp_cohort_schema_blake3 = "528797f6970a8f24175daf1f994c742f2fe24f0b8230b9456885452a08132a90" in
let mcp_cohort_license_blake3 = "fd12707e19e104da035c74958a676b0567b7dc2df01875ae07f058b79e59422f" in
let required_non_claims = [
  "server_trust",
  "server_correctness",
  "tool_safety",
  "sandboxing",
  "network_isolation",
  "exactly_once_effects",
  "mcp_parity",
  "production_readiness",
  "release_eligibility",
] in
let max_byte_limit = 16777216 in
let max_count_limit = 65536 in
let max_timeout_ms = 300000 in
let hash = contracts.stable_validator "MCP_BLAKE3_IDENTITY" (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let server_posture_contract = contracts.enum "MCP_SERVER_POSTURE" ["required", "optional"] in
let stdio_transport_contract = contracts.exact "MCP_TRANSPORT" "stdio" in
let ToolPolicy = {
  name | contracts.safe_identifier "MCP_TOOL_NAME",
  capability_labels | Array (contracts.safe_identifier "MCP_TOOL_CAPABILITY"),
  resource_policy_refs | Array (contracts.safe_identifier "MCP_RESOURCE_POLICY"),
  ability_policy_refs | Array (contracts.safe_identifier "MCP_ABILITY_POLICY"),
  result_bounds | contracts.safe_identifier "MCP_RESULT_BOUNDS",
} in
let Limits = {
  max_frame_bytes | contracts.positive_integer "MCP_FRAME_BOUND" max_byte_limit,
  max_tools | contracts.positive_integer "MCP_TOOL_COUNT_BOUND" max_count_limit,
  max_tool_name_bytes | contracts.positive_integer "MCP_TOOL_NAME_BOUND" max_byte_limit,
  max_description_bytes | contracts.positive_integer "MCP_DESCRIPTION_BOUND" max_byte_limit,
  max_schema_bytes_each | contracts.positive_integer "MCP_SCHEMA_ITEM_BOUND" max_byte_limit,
  max_schema_bytes_total | contracts.positive_integer "MCP_SCHEMA_TOTAL_BOUND" max_byte_limit,
  max_resource_metadata_bytes | contracts.positive_integer "MCP_RESOURCE_BOUND" max_byte_limit,
  max_diagnostic_bytes | contracts.positive_integer "MCP_DIAGNOSTIC_BOUND" max_byte_limit,
  max_result_bytes | contracts.positive_integer "MCP_RESULT_BOUND" max_byte_limit,
  max_instruction_bytes | contracts.positive_integer "MCP_INSTRUCTION_BOUND" max_byte_limit,
  max_arguments | contracts.positive_integer "MCP_ARGUMENT_COUNT_BOUND" max_count_limit,
  max_environment_keys | contracts.positive_integer "MCP_ENVIRONMENT_COUNT_BOUND" max_count_limit,
  max_calls_per_generation | contracts.positive_integer "MCP_CALL_COUNT_BOUND" max_count_limit,
  initialize_timeout_ms | contracts.positive_integer "MCP_INITIALIZE_TIMEOUT" max_timeout_ms,
  discovery_timeout_ms | contracts.positive_integer "MCP_DISCOVERY_TIMEOUT" max_timeout_ms,
  call_timeout_ms | contracts.positive_integer "MCP_CALL_TIMEOUT" max_timeout_ms,
  cancellation_poll_ms | contracts.positive_integer "MCP_CANCEL_POLL" max_timeout_ms,
  shutdown_timeout_ms | contracts.positive_integer "MCP_SHUTDOWN_TIMEOUT" max_timeout_ms,
  teardown_timeout_ms | contracts.positive_integer "MCP_TEARDOWN_TIMEOUT" max_timeout_ms,
} in
let Server = {
  id | contracts.safe_identifier "MCP_SERVER_ID",
  posture | server_posture_contract,
  transport | stdio_transport_contract,
  executable_label | contracts.safe_identifier "MCP_EXECUTABLE_LABEL",
  executable_identity | hash,
  arguments | Array (contracts.safe_identifier "MCP_ARGUMENT_LABEL"),
  environment_keys | Array (contracts.safe_identifier "MCP_ENVIRONMENT_KEY"),
  credential_refs | Array (contracts.safe_identifier "MCP_CREDENTIAL_REF"),
  working_root_capability | contracts.safe_identifier "MCP_WORKING_ROOT_CAPABILITY",
  working_root_identity | hash,
  protocol_version | contracts.exact "MCP_PROTOCOL_VERSION" mcp_protocol_version,
  policy_identity | hash,
  limits | Limits,
  tool_policies | Array ToolPolicy,
  auto_discovery | contracts.exact "MCP_AUTO_DISCOVERY" false,
  shell_fallback | contracts.exact "MCP_SHELL_FALLBACK" false,
  inherit_environment | contracts.exact "MCP_INHERIT_ENVIRONMENT" false,
} in
let Profile = {
  schema | contracts.exact "MCP_CONFIG_SCHEMA" schema_identity,
  cohort_revision | contracts.stable_validator "MCP_COHORT_REVISION" (fun value =>
    std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
  ),
  cohort_schema_blake3 | hash,
  cohort_license_blake3 | hash,
  server | Server,
  non_claims | Array (contracts.enum "MCP_NON_CLAIM" required_non_claims),
} in
let validate = fun value =>
  if std.array.length value.server.tool_policies == 0 then
    'Error { message = "MCP_EMPTY_TOOL_POLICIES" }
  else if !contracts.all_unique (std.array.map (fun item => item.name) value.server.tool_policies) then
    'Error { message = "MCP_DUPLICATE_TOOL_POLICY" }
  else if !contracts.all_unique value.server.environment_keys then
    'Error { message = "MCP_DUPLICATE_ENVIRONMENT_KEY" }
  else if !contracts.all_unique value.server.credential_refs then
    'Error { message = "MCP_DUPLICATE_CREDENTIAL_REF" }
  else if !std.array.all (fun item => std.array.elem item value.non_claims) required_non_claims then
    'Error { message = "MCP_MISSING_NON_CLAIM" }
  else
    'Ok
in
let McpClientHostContract = contracts.with_validator Profile validate in
{
  mcp_client_host_contract = McpClientHostContract,
  mcp_client_host | McpClientHostContract = {
    schema = schema_identity,
    cohort_revision = mcp_cohort_revision,
    cohort_schema_blake3 = mcp_cohort_schema_blake3,
    cohort_license_blake3 = mcp_cohort_license_blake3,
    server = {
      id = "fixture-stdio",
      posture = "required",
      transport = "stdio",
      executable_label = "animus-mcp-fixture-server",
      executable_identity = "8cfdeb80bd4ac427573f73b74eee2c73dfb2794ac08d502415f75b49985e4ceb",
      arguments = ["serve"],
      environment_keys = [],
      credential_refs = [],
      working_root_capability = "mcp-fixture-root",
      working_root_identity = "c01a43e5e96acf375dd9a314822bb1218421977dd49954c80cd826a93ac0f7d7",
      protocol_version = mcp_protocol_version,
      policy_identity = "92bb8f7d26d1cda1220f6481e6cb2c7ac1e1de8e975d9f9d82351ceff6d40dc6",
      limits = {
        max_frame_bytes = 16384,
        max_tools = 8,
        max_tool_name_bytes = 64,
        max_description_bytes = 512,
        max_schema_bytes_each = 4096,
        max_schema_bytes_total = 8192,
        max_resource_metadata_bytes = 1024,
        max_diagnostic_bytes = 512,
        max_result_bytes = 8192,
        max_instruction_bytes = 1024,
        max_arguments = 16,
        max_environment_keys = 8,
        max_calls_per_generation = 4,
        initialize_timeout_ms = 2000,
        discovery_timeout_ms = 2000,
        call_timeout_ms = 2000,
        cancellation_poll_ms = 10,
        shutdown_timeout_ms = 250,
        teardown_timeout_ms = 2000,
      },
      tool_policies = [{
        name = "read_text",
        capability_labels = ["workspace-read"],
        resource_policy_refs = ["workspace-read-policy"],
        ability_policy_refs = ["mcp-tool-policy"],
        result_bounds = "bounded-json-result",
      }],
      auto_discovery = false,
      shell_fallback = false,
      inherit_environment = false,
    },
    non_claims = required_non_claims,
  },
}
