# Typed metadata-only session runtime authority-profile fixture and parity source.
# r[impl agent_harness_runtime.session_authority_profile.contract]
# r[impl agent_harness_runtime.session_authority_profile.identity]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.session-authority-profile.v1" in
let component_classes = [
  "policy_authorization",
  "permission",
  "sandbox_intent",
  "model_runtime",
  "tool_catalog",
  "capability_ceiling",
  "hook_plugin_set",
  "agent_definition_set",
  "context_retrieval",
  "adapter_cohort",
] in
let required_component_count = 10 in
let redaction_marker_value =
  "raw-policy-prompt-tool-extension-credential-path-and-provider-material-omitted"
in
let Component = {
  class | contracts.enum "SESSION_AUTHORITY_COMPONENT_CLASS" component_classes,
  identity | contracts.lowercase_blake3 "SESSION_AUTHORITY_COMPONENT_IDENTITY",
} in
let ProfileShape = {
  schema | contracts.exact "SESSION_AUTHORITY_PROFILE_SCHEMA" schema_identity,
  components | Array Component,
  expected_profile_identity | contracts.lowercase_blake3 "SESSION_AUTHORITY_PROFILE_IDENTITY",
  redaction_marker | contracts.exact "SESSION_AUTHORITY_REDACTION_MARKER" redaction_marker_value,
} in
let _whole_profile_validator = fun value =>
  if std.array.length value.components != required_component_count then
    'Error { message = "SESSION_AUTHORITY_MISSING_COMPONENT" }
  else if !contracts.unique_by (fun component => component.class) value.components then
    'Error { message = "SESSION_AUTHORITY_DUPLICATE_COMPONENT" }
  else if !std.array.all
    (fun class => std.array.any (fun component => component.class == class) value.components)
    component_classes
  then
    'Error { message = "SESSION_AUTHORITY_MISSING_COMPONENT" }
  else
    'Ok
in
let SessionAuthorityProfileContract = contracts.with_validator
  ProfileShape
  _whole_profile_validator
in
{
  session_authority_profile_contract = SessionAuthorityProfileContract,
  session_authority_profile | SessionAuthorityProfileContract = {
  schema = schema_identity,
  # Input order is deliberately non-canonical; Rust owns canonical ordering and identity.
  components = [
    { class = "tool_catalog", identity = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" },
    { class = "policy_authorization", identity = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
    { class = "adapter_cohort", identity = "3333333333333333333333333333333333333333333333333333333333333333" },
    { class = "permission", identity = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" },
    { class = "context_retrieval", identity = "2222222222222222222222222222222222222222222222222222222222222222" },
    { class = "sandbox_intent", identity = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" },
    { class = "model_runtime", identity = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd" },
    { class = "capability_ceiling", identity = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" },
    { class = "hook_plugin_set", identity = "0000000000000000000000000000000000000000000000000000000000000000" },
    { class = "agent_definition_set", identity = "1111111111111111111111111111111111111111111111111111111111111111" },
  ],
  expected_profile_identity = "53a1955677292245ccd4ccc6c0c3a7f3bcd9431f4ac0d6f1a7f9fb7ca1ac5070",
  redaction_marker = redaction_marker_value,
  },
}
