# ACP nominal-domain declarations for current Animus checks and future Octet enforcement.
# r[impl agent_harness_runtime.acp_nominal_identities.octet]
let contracts = import "../policy/contracts/primitives.ncl" in
let schema_identity = "animus.acp-nominal-identities.v1" in
let external_id_byte_limit = 128 in
let required_blake3_hex_chars = 64 in
let domain_names = [
  "acp_correlation_id",
  "acp_session_id",
  "animus_session_id",
  "session_identity",
  "authority_profile_identity",
  "acp_run_id",
  "acp_entry_id",
  "acp_connection_identity",
  "workspace_identity",
  "resource_identity",
  "session_reference",
  "receipt_identity",
] in
let Domain = {
  name | contracts.enum "ACP_NOMINAL_DOMAIN_NAME" domain_names,
  representation | contracts.enum "ACP_NOMINAL_REPRESENTATION" ["external_bounded", "lowercase_blake3"],
  scope | contracts.enum "ACP_NOMINAL_SCOPE" ["wire_admission", "connection_state", "facade", "receipt_projection"],
} in
let PolicyShape = {
  schema | contracts.exact "ACP_NOMINAL_SCHEMA" schema_identity,
  external_id_max_bytes | contracts.integer_between "ACP_NOMINAL_EXTERNAL_BOUND" 1 external_id_byte_limit,
  blake3_hex_chars | contracts.integer_between "ACP_NOMINAL_HASH_LENGTH" required_blake3_hex_chars required_blake3_hex_chars,
  domains | Array Domain,
  raw_identity_posture | contracts.enum "ACP_NOMINAL_RAW_POSTURE" ["deny_in_declared_scopes"],
  authority_claim | contracts.enum "ACP_NOMINAL_AUTHORITY_CLAIM" ["category_only"],
} in
let _validator = fun value =>
  if std.array.length value.domains != std.array.length domain_names then
    'Error { message = "ACP_NOMINAL_DOMAIN_COVERAGE" }
  else if !contracts.unique_by (fun domain => domain.name) value.domains then
    'Error { message = "ACP_NOMINAL_DUPLICATE_DOMAIN" }
  else
    'Ok
in
let Policy = contracts.with_validator PolicyShape _validator in
let external = fun domain_name domain_scope => { name = domain_name, representation = "external_bounded", scope = domain_scope } in
let hash = fun domain_name domain_scope => { name = domain_name, representation = "lowercase_blake3", scope = domain_scope } in
{
  acp_nominal_identity_contract = Policy,
  acp_nominal_identities | Policy = {
    schema = schema_identity,
    external_id_max_bytes = external_id_byte_limit,
    blake3_hex_chars = required_blake3_hex_chars,
    domains = [
      external "acp_correlation_id" "wire_admission",
      external "acp_session_id" "connection_state",
      hash "animus_session_id" "connection_state",
      hash "session_identity" "connection_state",
      hash "authority_profile_identity" "connection_state",
      external "acp_run_id" "facade",
      external "acp_entry_id" "facade",
      hash "acp_connection_identity" "connection_state",
      hash "workspace_identity" "facade",
      hash "resource_identity" "wire_admission",
      external "session_reference" "facade",
      hash "receipt_identity" "receipt_projection",
    ],
    raw_identity_posture = "deny_in_declared_scopes",
    authority_claim = "category_only",
  },
}
