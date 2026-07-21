# Typed per-model runtime profile source of truth.
# r[impl agent_harness_runtime.model_runtime_profiles.contract]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.model-runtime-profiles.v1" in
let required_mode = "required" in
let exact_binding = "exact" in
let default_binding = "default" in
let host_scope = "host" in
let project_scope = "project" in
let trusted_status = "trusted" in
let untrusted_status = "untrusted" in
let admitted_status = "admitted" in
let denied_status = "denied" in
let metadata_only_posture = "metadata_only" in
let selectable_posture = "selectable" in
let required_selected_posture = "required_selected" in
let disabled_memory_mode = "disabled" in
let read_only_memory_mode = "read_only" in
let read_write_memory_mode = "read_write" in
let disabled_tool_mode = "disabled" in
let optional_tool_mode = "optional" in
let required_tool_mode = "capability_required" in
let standard_budget_id = "standard-budget" in
let standard_context_units = 4096 in
let standard_tool_calls = 8 in
let standard_parallel_calls = 2 in
let maximum_context_units = 1000000 in
let maximum_tool_calls = 1024 in
let maximum_parallel_calls = 128 in
let maximum_safe_label_chars = 128 in
let policy_hash_value = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let capability_hash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" in
let memory_policy_hash_value = "9999999999999999999999999999999999999999999999999999999999999999" in
let mock_standard_hash = "dba2fff3a916d9b631208f1dae89c7e30591add42e33c0568223432a4fb97428" in
let mock_balanced_hash = "6c55a406c8b9e0855adcf189156bcf1b8e49c6ec03402b2d45d52f46a920c91f" in
let compatibility_default_hash = "afbe344c0f904052774b59c2cf1419de8c5ae445470a04d89a7300acdd48726b" in

let _lower_label_predicate = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= maximum_safe_label_chars
  && std.string.is_match "^[a-z0-9][a-z0-9._:-]*[a-z0-9]$|^[a-z0-9]$" value
in
let _lowercase_blake3_predicate = fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
in
let LowerLabel = contracts.safe_identifier "MODEL_PROFILE_LOWER_LABEL" in
let Binding = {
  kind | contracts.enum "MODEL_PROFILE_BINDING_KIND" [exact_binding, default_binding],
  provider_id | String,
  model_id | String,
  router_role | String,
  capability_identity | String,
} in
let PromptReference = {
  label | LowerLabel,
  required | Bool,
} in
let SkillReference = {
  name | LowerLabel,
  posture | contracts.enum "MODEL_PROFILE_SKILL_POSTURE" [
    metadata_only_posture,
    selectable_posture,
    required_selected_posture,
  ],
} in
let NamedReference = {
  name | LowerLabel,
  required | Bool,
} in
let BudgetPolicy = {
  policy_id | LowerLabel,
  max_context_units | contracts.integer_between "MODEL_PROFILE_CONTEXT_BUDGET" 1 maximum_context_units,
  max_tool_calls | contracts.integer_between "MODEL_PROFILE_TOOL_BUDGET" 1 maximum_tool_calls,
  max_parallel_tool_calls | contracts.integer_between "MODEL_PROFILE_PARALLEL_BUDGET" 1 maximum_parallel_calls,
} in
let Toolset = {
  toolset_id | LowerLabel,
  tool_names | Array LowerLabel,
} in
let Profile = {
  profile_id | LowerLabel,
  binding | Binding,
  source_scope | contracts.enum "MODEL_PROFILE_SOURCE_SCOPE" [host_scope, project_scope],
  trust_status | contracts.enum "MODEL_PROFILE_TRUST_STATUS" [trusted_status, untrusted_status],
  admission_status | contracts.enum "MODEL_PROFILE_ADMISSION_STATUS" [admitted_status, denied_status],
  project_profile_admitted | Bool,
  prompt_mode_ref | LowerLabel,
  prompt_sources | Array PromptReference,
  skills | Array SkillReference,
  memory_access_mode | contracts.enum "MODEL_PROFILE_MEMORY_MODE" [
    disabled_memory_mode,
    read_only_memory_mode,
    read_write_memory_mode,
  ],
  memory_policy_hash | contracts.lowercase_blake3 "MODEL_PROFILE_MEMORY_POLICY_HASH",
  tools | Array NamedReference,
  toolsets | Array NamedReference,
  tool_calling_mode | contracts.enum "MODEL_PROFILE_TOOL_MODE" [
    disabled_tool_mode,
    optional_tool_mode,
    required_tool_mode,
  ],
  context_budget_policy | LowerLabel,
  tool_budget_policy | LowerLabel,
  policy_hash | contracts.lowercase_blake3 "MODEL_PROFILE_POLICY_HASH",
  profile_hash | contracts.lowercase_blake3 "MODEL_PROFILE_CONTENT_HASH",
} in
let ProfileSetShape = {
  schema | contracts.exact "MODEL_PROFILE_SCHEMA" schema_identity,
  enablement | contracts.enum "MODEL_PROFILE_ENABLEMENT" [required_mode, "compatibility_disabled"],
  explicit_default_profile_id | String,
  budget_policies | Array BudgetPolicy,
  toolsets | Array Toolset,
  profiles | Array Profile,
} in

let _binding_is_valid = fun binding =>
  if binding.kind == exact_binding then
    _lower_label_predicate binding.provider_id
    && _lower_label_predicate binding.model_id
    && (binding.router_role == "" || _lower_label_predicate binding.router_role)
    && _lowercase_blake3_predicate binding.capability_identity
  else
    binding.kind == default_binding
    && binding.provider_id == ""
    && binding.model_id == ""
    && binding.router_role == ""
    && binding.capability_identity == ""
in
let _binding_key = fun profile =>
  profile.binding.kind
  ++ ":" ++ profile.binding.provider_id
  ++ ":" ++ profile.binding.model_id
  ++ ":" ++ profile.binding.router_role
in
let _references_are_unique = fun profile =>
  contracts.unique_by (fun reference => reference.label) profile.prompt_sources
  && contracts.unique_by (fun reference => reference.name) profile.skills
  && contracts.unique_by (fun reference => reference.name) profile.tools
  && contracts.unique_by (fun reference => reference.name) profile.toolsets
in
let _budget_is_valid = fun policy =>
  policy.max_parallel_tool_calls <= policy.max_tool_calls
in
let _toolset_is_valid = fun toolset =>
  std.array.length toolset.tool_names > 0
  && contracts.unique_by (fun name => name) toolset.tool_names
in
let _reference_exists = fun values key =>
  std.array.any (fun value => value == key) values
in
let _profile_references_are_valid = fun profile budget_ids toolset_ids =>
  _reference_exists budget_ids profile.context_budget_policy
  && _reference_exists budget_ids profile.tool_budget_policy
  && std.array.all (fun reference => _reference_exists toolset_ids reference.name) profile.toolsets
in
let _profile_posture_is_valid = fun profile =>
  (profile.source_scope != project_scope || profile.project_profile_admitted)
  && (profile.tool_calling_mode != disabled_tool_mode
    || (std.array.length profile.tools == 0 && std.array.length profile.toolsets == 0))
in
let _whole_profile_set_validator = fun value =>
  let budgets = value.budget_policies in
  let toolsets = value.toolsets in
  let profiles = value.profiles in
  let budget_ids = std.array.map (fun policy => policy.policy_id) budgets in
  let toolset_ids = std.array.map (fun toolset => toolset.toolset_id) toolsets in
  let defaults = std.array.filter (fun profile => profile.binding.kind == default_binding) profiles in
  if std.array.length budgets == 0 || std.array.length profiles == 0 then
    'Error { message = "MODEL_PROFILE_EMPTY_SET" }
  else if !contracts.unique_by (fun policy => policy.policy_id) budgets then
    'Error { message = "MODEL_PROFILE_DUPLICATE_BUDGET" }
  else if !std.array.all _budget_is_valid budgets then
    'Error { message = "MODEL_PROFILE_BUDGET_ORDER" }
  else if !contracts.unique_by (fun toolset => toolset.toolset_id) toolsets then
    'Error { message = "MODEL_PROFILE_DUPLICATE_TOOLSET" }
  else if !std.array.all _toolset_is_valid toolsets then
    'Error { message = "MODEL_PROFILE_TOOLSET_INVARIANT" }
  else if !contracts.unique_by (fun profile => profile.profile_id) profiles then
    'Error { message = "MODEL_PROFILE_DUPLICATE_ID" }
  else if !contracts.unique_by _binding_key profiles then
    'Error { message = "MODEL_PROFILE_DUPLICATE_BINDING" }
  else if !std.array.all (fun profile => _binding_is_valid profile.binding) profiles then
    'Error { message = "MODEL_PROFILE_BINDING_INVARIANT" }
  else if !std.array.all _references_are_unique profiles then
    'Error { message = "MODEL_PROFILE_DUPLICATE_REFERENCE" }
  else if !std.array.all (fun profile => _profile_references_are_valid profile budget_ids toolset_ids) profiles then
    'Error { message = "MODEL_PROFILE_DANGLING_REFERENCE" }
  else if !std.array.all _profile_posture_is_valid profiles then
    'Error { message = "MODEL_PROFILE_POSTURE_INVARIANT" }
  else if std.array.length defaults != 1
    || (std.array.first defaults).profile_id != value.explicit_default_profile_id
  then
    'Error { message = "MODEL_PROFILE_DEFAULT_INVARIANT" }
  else
    'Ok
in
let ModelRuntimeProfileSet = contracts.with_validator
  ProfileSetShape
  _whole_profile_set_validator
in

let host_policy_prompt = { label = "host-policy", required = true } in
let model_guidance_prompt = { label = "model-guidance", required = false } in
let common_profile = {
  source_scope = host_scope,
  trust_status = trusted_status,
  admission_status = admitted_status,
  project_profile_admitted = false,
  prompt_mode_ref = "standard",
  memory_access_mode = disabled_memory_mode,
  memory_policy_hash = memory_policy_hash_value,
  tools = [],
  toolsets = [],
  tool_calling_mode = disabled_tool_mode,
  context_budget_policy = standard_budget_id,
  tool_budget_policy = standard_budget_id,
  policy_hash = policy_hash_value,
} in
{
  model_runtime_profile_set_contract = ModelRuntimeProfileSet,
  model_runtime_profile_set | ModelRuntimeProfileSet = {
    schema = schema_identity,
    enablement = required_mode,
    explicit_default_profile_id = "compat-default",
    budget_policies = [
      {
        policy_id = standard_budget_id,
        max_context_units = standard_context_units,
        max_tool_calls = standard_tool_calls,
        max_parallel_tool_calls = standard_parallel_calls,
      },
    ],
    toolsets = [
      {
        toolset_id = "readonly-context",
        tool_names = ["inspect_context"],
      },
    ],
    profiles = [
      common_profile & {
        profile_id = "mock-standard",
        binding = {
          kind = exact_binding,
          provider_id = "mock",
          model_id = "mock-text",
          router_role = "",
          capability_identity = capability_hash,
        },
        prompt_sources = [host_policy_prompt],
        skills = [],
        profile_hash = mock_standard_hash,
      },
      common_profile & {
        profile_id = "mock-balanced",
        binding = {
          kind = exact_binding,
          provider_id = "mock",
          model_id = "mock-text",
          router_role = "balanced",
          capability_identity = capability_hash,
        },
        prompt_sources = [host_policy_prompt, model_guidance_prompt],
        skills = [{ name = "proof-search", posture = selectable_posture }],
        memory_access_mode | force = read_only_memory_mode,
        profile_hash = mock_balanced_hash,
      },
      common_profile & {
        profile_id = "compat-default",
        binding = {
          kind = default_binding,
          provider_id = "",
          model_id = "",
          router_role = "",
          capability_identity = "",
        },
        prompt_sources = [host_policy_prompt],
        skills = [],
        profile_hash = compatibility_default_hash,
      },
    ],
  },
}
