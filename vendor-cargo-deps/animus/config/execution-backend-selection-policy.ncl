# Typed weakest-admitted execution backend policy.
let contracts = import "../policy/contracts/primitives.ncl" in
let backend_class_count = 4 in
let non_claim_count = 4 in
let BackendClass = [| 'EmbeddedComponent, 'GuardedLocalProcess, 'IsolatedHost, 'ExternalMachine |] in
let ClassPolicy = {
  class | BackendClass,
  admitted_requirements | Array String,
} in
let Policy = {
  schema | contracts.exact "BACKEND_SELECTION_SCHEMA" "animus.execution-backend-selection.v1",
  policy_id | contracts.lowercase_blake3 "BACKEND_SELECTION_POLICY_ID",
  preference_order | Array BackendClass,
  class_policies | Array ClassPolicy,
  deny_silent_fallback | Bool,
  receipt_redaction | String,
  non_claims | Array String,
} in
let CheckedPolicy = contracts.with_validator Policy (fun value =>
  if std.array.length value.preference_order != backend_class_count then
    'Error { message = "BACKEND_SELECTION_ORDER_COUNT" }
  else if std.array.length value.class_policies != backend_class_count then
    'Error { message = "BACKEND_SELECTION_CLASS_COUNT" }
  else if !value.deny_silent_fallback then
    'Error { message = "BACKEND_SELECTION_FALLBACK" }
  else if std.array.length value.non_claims != non_claim_count then
    'Error { message = "BACKEND_SELECTION_NON_CLAIMS" }
  else
    'Ok
) in
let embedded_requirements = [
  "wasm_component",
  "no_filesystem",
  "read_only_filesystem",
  "network_denied",
  "in_process",
  "no_terminal",
  "no_peripheral",
  "ephemeral",
  "mediated_isolation",
] in
let guarded_process_requirements = [
  "wasm_component",
  "native_binary",
  "script_runtime",
  "no_filesystem",
  "read_only_filesystem",
  "writable_workspace",
  "network_denied",
  "network_allowlisted",
  "single_process",
  "process_tree",
  "no_terminal",
  "pseudoterminal",
  "no_peripheral",
  "ephemeral",
  "durable_workspace",
  "guarded_process_isolation",
] in
let all_requirements = [
  "wasm_component",
  "native_binary",
  "script_runtime",
  "no_filesystem",
  "read_only_filesystem",
  "writable_workspace",
  "network_denied",
  "network_allowlisted",
  "in_process",
  "single_process",
  "process_tree",
  "no_terminal",
  "pseudoterminal",
  "no_peripheral",
  "browser",
  "host_device",
  "ephemeral",
  "durable_workspace",
  "mediated_isolation",
  "guarded_process_isolation",
  "host_isolation",
] in
{
  execution_backend_selection_policy_contract = CheckedPolicy,
  execution_backend_selection_policy | CheckedPolicy = {
    schema = "animus.execution-backend-selection.v1",
    policy_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    preference_order = ['EmbeddedComponent, 'GuardedLocalProcess, 'IsolatedHost, 'ExternalMachine],
    class_policies = [
      { class = 'EmbeddedComponent, admitted_requirements = embedded_requirements },
      { class = 'GuardedLocalProcess, admitted_requirements = guarded_process_requirements },
      { class = 'IsolatedHost, admitted_requirements = all_requirements },
      { class = 'ExternalMachine, admitted_requirements = all_requirements },
    ],
    deny_silent_fallback = true,
    receipt_redaction = "metadata-only-v1",
    non_claims = [
      "not-backend-availability",
      "not-sandbox-enforcement",
      "not-work-completion",
      "not-output-correctness",
    ],
  },
}
