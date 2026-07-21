# r[impl agent_harness_runtime.nickel_policy_contracts.steel_runtime]
let contracts = import "../contracts/primitives.ncl" in

let policy_schema = "animus.steel-runtime-policy.v1" in
let maximum_profile_count = 64 in
let maximum_host_action_count = 16 in
let maximum_turn_budget = 1000000 in
let runtime_host_actions = ["model", "tool", "respond", "stop"] in
let fallback_modes = ["block"] in
let receipt_redaction_modes = ["hash-only"] in

let PolicySchema = contracts.exact "STEEL_RUNTIME_SCHEMA" policy_schema in
let ProfileName = contracts.safe_identifier "STEEL_RUNTIME_PROFILE_NAME" in
let ScriptLabel = contracts.stable_validator "STEEL_RUNTIME_SCRIPT_PATH" (fun value =>
  contracts.is_safe_repository_relative_path value
  && std.string.is_match "^[A-Za-z0-9][A-Za-z0-9._/-]*\\.scm$" value
) in
let ScriptHash = contracts.lowercase_blake3 "STEEL_RUNTIME_SCRIPT_HASH" in
let TurnBudget = contracts.positive_integer "STEEL_RUNTIME_TURN_BUDGET" maximum_turn_budget in
let HostAction = contracts.enum "STEEL_RUNTIME_HOST_ACTION" runtime_host_actions in
let FallbackMode = contracts.enum "STEEL_RUNTIME_FALLBACK" fallback_modes in
let ReceiptRedactionMode = contracts.enum "STEEL_RUNTIME_REDACTION" receipt_redaction_modes in

let ScriptBinding = {
  label | ScriptLabel,
  hash | ScriptHash,
} in
let RuntimeBudget = {
  max_turns | TurnBudget,
} in
let ReceiptRedaction = {
  mode | ReceiptRedactionMode,
} in
let SteelRuntimeProfile = {
  name | ProfileName,
  enabled | Bool,
  default | Bool,
  script | ScriptBinding,
  budget | RuntimeBudget,
  allowed_host_actions | Array HostAction,
  fallback_mode | FallbackMode,
  receipt_redaction | ReceiptRedaction,
} in
let SteelRuntimePolicyShape = {
  schema | PolicySchema,
  profiles | Array SteelRuntimeProfile,
} in

let _profile_actions_are_valid = fun profile =>
  std.array.length profile.allowed_host_actions > 0
  && std.array.length profile.allowed_host_actions <= maximum_host_action_count
  && contracts.all_unique profile.allowed_host_actions
in

let _policy_validator = fun policy =>
  let enabled_defaults = std.array.filter
    (fun profile => profile.enabled && profile.default)
    policy.profiles
  in
  if std.array.length policy.profiles == 0 || std.array.length policy.profiles > maximum_profile_count then
    'Error { message = "STEEL_RUNTIME_PROFILE_COUNT" }
  else if !contracts.unique_by (fun profile => profile.name) policy.profiles then
    'Error { message = "STEEL_RUNTIME_DUPLICATE_PROFILE" }
  else if !contracts.unique_by (fun profile => profile.script.label) policy.profiles then
    'Error { message = "STEEL_RUNTIME_DUPLICATE_SCRIPT" }
  else if std.array.length enabled_defaults != 1 then
    'Error { message = "STEEL_RUNTIME_DEFAULT_COUNT" }
  else if std.array.any (fun profile => profile.default && !profile.enabled) policy.profiles then
    'Error { message = "STEEL_RUNTIME_DISABLED_DEFAULT" }
  else if !std.array.all _profile_actions_are_valid policy.profiles then
    'Error { message = "STEEL_RUNTIME_HOST_ACTION_SET" }
  else
    'Ok
in
let SteelRuntimePolicy = contracts.with_validator SteelRuntimePolicyShape _policy_validator in

let default_profile_name = "default-loop" in
let default_script_label = "scripts/default-loop.scm" in
let default_script_hash = "79d887ab1ffd350d191ad7aea6a10fe3de28f108d34271cc99e4a59e537e9d0e" in
let default_max_turns = 2 in
let ralph_profile_name = "ralph-loop" in
let ralph_script_label = "scripts/ralph-loop.scm" in
let ralph_script_hash = "0225dfc88a7cbfe9b1639a199dd961cecee66ecd311a2ff13929867f60b6ca77" in
let ralph_max_turns = 10 in
let fallback_block = "block" in
let receipt_hash_only = "hash-only" in
let policy_value = {
  schema = policy_schema,
  profiles = [
    {
      name = default_profile_name,
      enabled = true,
      default = true,
      script = {
        label = default_script_label,
        hash = default_script_hash,
      },
      budget = {
        max_turns = default_max_turns,
      },
      allowed_host_actions = runtime_host_actions,
      fallback_mode = fallback_block,
      receipt_redaction = {
        mode = receipt_hash_only,
      },
    },
    {
      name = ralph_profile_name,
      enabled = true,
      default = false,
      script = {
        label = ralph_script_label,
        hash = ralph_script_hash,
      },
      budget = {
        max_turns = ralph_max_turns,
      },
      allowed_host_actions = runtime_host_actions,
      fallback_mode = fallback_block,
      receipt_redaction = {
        mode = receipt_hash_only,
      },
    },
  ],
} in
{
  steel_runtime_policy_contract = SteelRuntimePolicy,
  steel_runtime_policy | SteelRuntimePolicy = policy_value,
}
