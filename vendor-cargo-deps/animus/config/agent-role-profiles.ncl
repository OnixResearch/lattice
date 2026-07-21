# Typed capability-bound semantic agent-role source of truth.
# r[impl agent_harness_runtime.agent_role_profiles.contract]
# r[impl agent_harness_runtime.agent_role_profiles.semantic_boundary]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.agent-role-catalog.v1" in
let host_scope = "host" in
let project_scope = "project" in
let trusted_status = "trusted" in
let untrusted_status = "untrusted" in
let admitted_status = "admitted" in
let denied_status = "denied" in
let implementation_class = "implementation" in
let review_class = "review" in
let verification_class = "verification" in
let judgment_class = "judgment" in
let coordination_class = "coordination" in
let observation_class = "observation" in
let scoped_intent = "scoped_intent" in
let subject_read_only = "subject_read_only" in
let verification_service_only = "verification_service_only" in
let no_mutation = "none" in
let clean_recompose = "clean_recompose" in
let bounded_shared = "bounded_shared" in
let read_only_observation = "read_only_observation" in
let candidate_evidence = "candidate" in
let findings_evidence = "findings" in
let check_evidence = "check_evidence" in
let judgment_evidence = "judgment" in
let coordination_evidence = "coordination" in
let observation_evidence = "observation" in
let implementation_output = "implementation_candidate" in
let review_output = "review_report" in
let verification_output = "verification_report" in
let judgment_output = "judgment_report" in
let coordination_output = "coordination_report" in
let observation_output = "observation_report" in
let maximum_safe_label_chars = 128 in
let maximum_purpose_chars = 1024 in
let maximum_profiles = 64 in
let maximum_references = 128 in
let maximum_policies = 64 in
let maximum_actions = 1024 in
let maximum_outputs = 128 in
let maximum_context_units = 1000000 in
let policy_hash_value = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let memory_policy_hash_value = "9999999999999999999999999999999999999999999999999999999999999999" in
let implementer_hash = "c57907aae8160edf2ce5bc941a05d03f1d6f22dd3babf69a5a9f7dd2a99f5556" in
let reviewer_hash = "45b911ffee514570ac8c86feee116254bb2bf86787a233b3d5f3368ff6137fb2" in
let verifier_hash = "bd537afe4a8edbda1ca5ab5e2d61f2a3a53f02c26a5fe18d2eb04dea49bbd713" in
let judge_hash = "64171d6f89a466eceef3380b9149566c634898de4519476f23ad9be06aee03d0" in
let coordinator_hash = "3c9fd74d6a778558bb470221dbd5069ce4174a8e633154c8668c8e7f58ddea21" in
let observer_hash = "44b5d142714b08795bf8abbab3332913e877097fb01db17304017a2875331e57" in

let _lower_label_predicate = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= maximum_safe_label_chars
  && std.string.is_match "^[a-z0-9][a-z0-9._:-]*[a-z0-9]$|^[a-z0-9]$" value
in
let _lowercase_blake3_predicate = fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
in
let _purpose_predicate = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= maximum_purpose_chars
  && !std.string.is_match "RAW_PROMPT|HIDDEN_REASONING|PROVIDER_PAYLOAD|BEGIN PRIVATE KEY|API_KEY=|/home/" value
in
let LowerLabel = contracts.stable_validator "AGENT_ROLE_LOWER_LABEL" _lower_label_predicate in
let LowerHash = contracts.lowercase_blake3 "AGENT_ROLE_LOWERCASE_BLAKE3" in
let RoleClass = contracts.enum "AGENT_ROLE_CLASS" [
  implementation_class,
  review_class,
  verification_class,
  judgment_class,
  coordination_class,
  observation_class,
] in
let MutationPosture = contracts.enum "AGENT_ROLE_MUTATION_POSTURE" [
  scoped_intent,
  subject_read_only,
  verification_service_only,
  no_mutation,
] in
let ContextPosture = contracts.enum "AGENT_ROLE_CONTEXT_POSTURE" [
  clean_recompose,
  bounded_shared,
  read_only_observation,
] in
let EvidencePosture = contracts.enum "AGENT_ROLE_EVIDENCE_POSTURE" [
  candidate_evidence,
  findings_evidence,
  check_evidence,
  judgment_evidence,
  coordination_evidence,
  observation_evidence,
] in
let OutputKind = contracts.enum "AGENT_ROLE_OUTPUT_KIND" [
  implementation_output,
  review_output,
  verification_output,
  judgment_output,
  coordination_output,
  observation_output,
] in
let BudgetPolicy = {
  policy_id | LowerLabel,
  max_actions | contracts.integer_between "AGENT_ROLE_ACTION_BUDGET" 1 maximum_actions,
  max_outputs | contracts.integer_between "AGENT_ROLE_OUTPUT_BUDGET" 1 maximum_outputs,
  max_context_units | contracts.integer_between "AGENT_ROLE_CONTEXT_BUDGET" 1 maximum_context_units,
} in
let Toolset = {
  toolset_id | LowerLabel,
  tool_names | Array LowerLabel,
} in
let SeparationPolicy = {
  policy_id | LowerLabel,
  require_distinct_assignment | Bool,
  require_distinct_actor | Bool,
  require_distinct_run_thread | Bool,
  require_distinct_session | Bool,
  require_distinct_model_profile | Bool,
  require_distinct_workspace | Bool,
  require_distinct_mutation_lineage | Bool,
  require_clean_context | Bool,
  require_subject_immutable | Bool,
  require_verifier_immutable | Bool,
  require_rubric_immutable | Bool,
} in
let Profile = {
  role_id | LowerLabel,
  class | RoleClass,
  purpose | String,
  source_scope | contracts.enum "AGENT_ROLE_SOURCE_SCOPE" [host_scope, project_scope],
  trust_status | contracts.enum "AGENT_ROLE_TRUST_STATUS" [trusted_status, untrusted_status],
  admission_status | contracts.enum "AGENT_ROLE_ADMISSION_STATUS" [admitted_status, denied_status],
  project_profile_admitted | Bool,
  required_capabilities | Array LowerLabel,
  capability_ceiling | Array LowerLabel,
  eligible_model_profiles | Array LowerLabel,
  model_targets | Array LowerLabel,
  prompt_modes | Array LowerLabel,
  prompt_sources | Array LowerLabel,
  skills | Array LowerLabel,
  memory_policy_hashes | Array LowerHash,
  tools | Array LowerLabel,
  toolsets | Array LowerLabel,
  tool_capabilities | Array LowerLabel,
  workspace_scopes | Array LowerLabel,
  mutation_posture | MutationPosture,
  context_posture | ContextPosture,
  evidence_posture | EvidencePosture,
  output_kind | OutputKind,
  input_contract | LowerLabel,
  output_contract | LowerLabel,
  evidence_contract | LowerLabel,
  separation_policy_ref | LowerLabel,
  budget_policy_ref | LowerLabel,
  policy_hash | LowerHash,
  profile_hash | LowerHash,
} in
let CatalogShape = {
  schema | contracts.exact "AGENT_ROLE_SCHEMA" schema_identity,
  known_capabilities | Array LowerLabel,
  input_contracts | Array LowerLabel,
  output_contracts | Array LowerLabel,
  evidence_contracts | Array LowerLabel,
  budget_policies | Array BudgetPolicy,
  toolsets | Array Toolset,
  separation_policies | Array SeparationPolicy,
  profiles | Array Profile,
} in

let _reference_exists = fun values key => std.array.any (fun value => value == key) values in
let _all_unique = fun values => contracts.unique_by (fun value => value) values in
let _bounded_nonempty = fun values maximum =>
  std.array.length values > 0 && std.array.length values <= maximum
in
let _separation_has_dimension = fun policy =>
  policy.require_distinct_assignment
  || policy.require_distinct_actor
  || policy.require_distinct_run_thread
  || policy.require_distinct_session
  || policy.require_distinct_model_profile
  || policy.require_distinct_workspace
  || policy.require_distinct_mutation_lineage
  || policy.require_clean_context
  || policy.require_subject_immutable
  || policy.require_verifier_immutable
  || policy.require_rubric_immutable
in
let _class_posture_is_valid = fun profile =>
  if profile.class == implementation_class then
    profile.mutation_posture == scoped_intent
    && profile.context_posture == clean_recompose
    && profile.evidence_posture == candidate_evidence
    && profile.output_kind == implementation_output
  else if profile.class == review_class then
    profile.mutation_posture == subject_read_only
    && profile.context_posture == clean_recompose
    && profile.evidence_posture == findings_evidence
    && profile.output_kind == review_output
  else if profile.class == verification_class then
    profile.mutation_posture == verification_service_only
    && profile.context_posture == clean_recompose
    && profile.evidence_posture == check_evidence
    && profile.output_kind == verification_output
  else if profile.class == judgment_class then
    profile.mutation_posture == no_mutation
    && profile.context_posture == clean_recompose
    && profile.evidence_posture == judgment_evidence
    && profile.output_kind == judgment_output
  else if profile.class == coordination_class then
    profile.mutation_posture == no_mutation
    && profile.context_posture == bounded_shared
    && profile.evidence_posture == coordination_evidence
    && profile.output_kind == coordination_output
  else
    profile.class == observation_class
    && profile.mutation_posture == no_mutation
    && profile.context_posture == read_only_observation
    && profile.evidence_posture == observation_evidence
    && profile.output_kind == observation_output
in
let _profile_collections_are_valid = fun profile =>
  _bounded_nonempty profile.capability_ceiling maximum_references
  && _bounded_nonempty profile.eligible_model_profiles maximum_references
  && _bounded_nonempty profile.model_targets maximum_references
  && _bounded_nonempty profile.prompt_modes maximum_references
  && _bounded_nonempty profile.prompt_sources maximum_references
  && _bounded_nonempty profile.workspace_scopes maximum_references
  && _all_unique profile.required_capabilities
  && _all_unique profile.capability_ceiling
  && _all_unique profile.eligible_model_profiles
  && _all_unique profile.model_targets
  && _all_unique profile.prompt_modes
  && _all_unique profile.prompt_sources
  && _all_unique profile.skills
  && _all_unique profile.tools
  && _all_unique profile.toolsets
  && _all_unique profile.tool_capabilities
  && _all_unique profile.workspace_scopes
  && std.array.all
    (fun required => _reference_exists profile.capability_ceiling required)
    profile.required_capabilities
in
let _profile_references_are_valid = fun profile value =>
  std.array.all
    (fun capability => _reference_exists value.known_capabilities capability)
    profile.capability_ceiling
  && _reference_exists value.input_contracts profile.input_contract
  && _reference_exists value.output_contracts profile.output_contract
  && _reference_exists value.evidence_contracts profile.evidence_contract
  && std.array.any
    (fun policy => policy.policy_id == profile.budget_policy_ref)
    value.budget_policies
  && std.array.any
    (fun policy => policy.policy_id == profile.separation_policy_ref)
    value.separation_policies
  && std.array.all
    (fun reference => std.array.any (fun toolset => toolset.toolset_id == reference) value.toolsets)
    profile.toolsets
in
let _profile_trust_is_valid = fun profile =>
  profile.trust_status == trusted_status
  && profile.admission_status == admitted_status
  && (profile.source_scope != project_scope || profile.project_profile_admitted)
in
let _profile_purpose_is_valid = fun profile => _purpose_predicate profile.purpose in
let _independent_class_separation_is_valid = fun profile value =>
  if profile.class == review_class
    || profile.class == verification_class
    || profile.class == judgment_class
  then
    let policy = std.array.first
      (std.array.filter
        (fun candidate => candidate.policy_id == profile.separation_policy_ref)
        value.separation_policies)
    in
    policy.require_subject_immutable
    && (policy.require_distinct_assignment
      || policy.require_distinct_actor
      || policy.require_distinct_mutation_lineage)
  else true
in
let _baseline_role_is_valid = fun profiles role_id role_class =>
  let matches = std.array.filter (fun profile => profile.role_id == role_id) profiles in
  std.array.length matches == 1 && (std.array.first matches).class == role_class
in
let _whole_catalog_validator = fun value =>
  if !_bounded_nonempty value.profiles maximum_profiles
    || !_bounded_nonempty value.known_capabilities maximum_references
    || !_bounded_nonempty value.budget_policies maximum_policies
    || !_bounded_nonempty value.separation_policies maximum_policies
  then
    'Error { message = "AGENT_ROLE_EMPTY_OR_OVERBOUND" }
  else if !_all_unique value.known_capabilities
    || !_all_unique value.input_contracts
    || !_all_unique value.output_contracts
    || !_all_unique value.evidence_contracts
  then
    'Error { message = "AGENT_ROLE_DUPLICATE_CATALOG_REFERENCE" }
  else if !contracts.unique_by (fun policy => policy.policy_id) value.budget_policies then
    'Error { message = "AGENT_ROLE_DUPLICATE_BUDGET" }
  else if !contracts.unique_by (fun toolset => toolset.toolset_id) value.toolsets
    || !std.array.all (fun toolset => _bounded_nonempty toolset.tool_names maximum_references && _all_unique toolset.tool_names) value.toolsets
  then
    'Error { message = "AGENT_ROLE_TOOLSET_INVARIANT" }
  else if !contracts.unique_by (fun policy => policy.policy_id) value.separation_policies
    || !std.array.all _separation_has_dimension value.separation_policies
  then
    'Error { message = "AGENT_ROLE_SEPARATION_INVARIANT" }
  else if !contracts.unique_by (fun profile => profile.role_id) value.profiles then
    'Error { message = "AGENT_ROLE_DUPLICATE_PROFILE" }
  else if !std.array.all _profile_purpose_is_valid value.profiles then
    'Error { message = "AGENT_ROLE_UNSAFE_PURPOSE" }
  else if !std.array.all _profile_collections_are_valid value.profiles then
    'Error { message = "AGENT_ROLE_COLLECTION_INVARIANT" }
  else if !std.array.all _class_posture_is_valid value.profiles then
    'Error { message = "AGENT_ROLE_CLASS_INVARIANT" }
  else if !std.array.all _profile_trust_is_valid value.profiles then
    'Error { message = "AGENT_ROLE_TRUST_INVARIANT" }
  else if !std.array.all (fun profile => _profile_references_are_valid profile value) value.profiles then
    'Error { message = "AGENT_ROLE_DANGLING_REFERENCE" }
  else if !std.array.all (fun profile => _independent_class_separation_is_valid profile value) value.profiles then
    'Error { message = "AGENT_ROLE_INDEPENDENCE_INVARIANT" }
  else if !_baseline_role_is_valid value.profiles "implementer" implementation_class
    || !_baseline_role_is_valid value.profiles "reviewer" review_class
    || !_baseline_role_is_valid value.profiles "verifier" verification_class
    || !_baseline_role_is_valid value.profiles "judge" judgment_class
  then
    'Error { message = "AGENT_ROLE_BASELINE_INVARIANT" }
  else
    'Ok
in
let AgentRoleCatalog = contracts.with_validator CatalogShape _whole_catalog_validator in

let subject_bound = {
  policy_id = "subject-bound",
  require_distinct_assignment = false,
  require_distinct_actor = false,
  require_distinct_run_thread = false,
  require_distinct_session = false,
  require_distinct_model_profile = false,
  require_distinct_workspace = false,
  require_distinct_mutation_lineage = false,
  require_clean_context = false,
  require_subject_immutable = true,
  require_verifier_immutable = false,
  require_rubric_immutable = false,
} in
let independent_review = {
  policy_id = "independent-review",
  require_distinct_assignment = true,
  require_distinct_actor = true,
  require_distinct_run_thread = true,
  require_distinct_session = true,
  require_distinct_model_profile = true,
  require_distinct_workspace = true,
  require_distinct_mutation_lineage = true,
  require_clean_context = true,
  require_subject_immutable = true,
  require_verifier_immutable = false,
  require_rubric_immutable = false,
} in
let independent_verifier = independent_review & {
  policy_id | force = "independent-verifier",
  require_verifier_immutable | force = true,
} in
let independent_judge = independent_review & {
  policy_id | force = "independent-judge",
  require_rubric_immutable | force = true,
} in
let common_profile = {
  source_scope = host_scope,
  trust_status = trusted_status,
  admission_status = admitted_status,
  project_profile_admitted = false,
  required_capabilities = ["context_read"],
  model_targets = ["mock:mock-text"],
  prompt_modes = ["standard"],
  memory_policy_hashes = [memory_policy_hash_value],
  tools = [],
  workspace_scopes = ["workspace:project"],
  input_contract = "bounded-role-input",
  output_contract = "bounded-role-output",
  evidence_contract = "metadata-only-evidence",
  policy_hash = policy_hash_value,
} in
{
  agent_role_catalog_contract = AgentRoleCatalog,
  agent_role_catalog | AgentRoleCatalog = {
    schema = schema_identity,
    known_capabilities = ["context_read", "mutation_intent", "tool_use"],
    input_contracts = ["bounded-role-input"],
    output_contracts = ["bounded-role-output"],
    evidence_contracts = ["metadata-only-evidence"],
    budget_policies = [
      { policy_id = "implementer-budget", max_actions = 64, max_outputs = 8, max_context_units = 8192 },
      { policy_id = "reviewer-budget", max_actions = 32, max_outputs = 8, max_context_units = 4096 },
      { policy_id = "verifier-budget", max_actions = 32, max_outputs = 8, max_context_units = 4096 },
      { policy_id = "judge-budget", max_actions = 16, max_outputs = 8, max_context_units = 2048 },
      { policy_id = "coordinator-budget", max_actions = 32, max_outputs = 16, max_context_units = 4096 },
      { policy_id = "observer-budget", max_actions = 16, max_outputs = 8, max_context_units = 2048 },
    ],
    toolsets = [
      { toolset_id = "readonly-context", tool_names = ["inspect_context"] },
    ],
    separation_policies = [subject_bound, independent_review, independent_verifier, independent_judge],
    profiles = [
      common_profile & {
        role_id = "implementer",
        class = implementation_class,
        purpose = "Produce bounded implementation candidates without completion authority.",
        capability_ceiling = ["context_read", "mutation_intent", "tool_use"],
        eligible_model_profiles = ["mock-standard", "mock-balanced"],
        prompt_sources = ["host-policy", "model-guidance"],
        skills = ["proof-search"],
        toolsets = ["readonly-context"],
        tool_capabilities = ["observe_read"],
        mutation_posture = scoped_intent,
        context_posture = clean_recompose,
        evidence_posture = candidate_evidence,
        output_kind = implementation_output,
        separation_policy_ref = "subject-bound",
        budget_policy_ref = "implementer-budget",
        profile_hash = implementer_hash,
      },
      common_profile & {
        role_id = "reviewer",
        class = review_class,
        purpose = "Review an immutable subject and emit advisory findings.",
        capability_ceiling = ["context_read", "tool_use"],
        eligible_model_profiles = ["mock-balanced"],
        prompt_sources = ["host-policy", "model-guidance"],
        skills = ["proof-search"],
        toolsets = ["readonly-context"],
        tool_capabilities = ["observe_read"],
        mutation_posture = subject_read_only,
        context_posture = clean_recompose,
        evidence_posture = findings_evidence,
        output_kind = review_output,
        separation_policy_ref = "independent-review",
        budget_policy_ref = "reviewer-budget",
        profile_hash = reviewer_hash,
      },
      common_profile & {
        role_id = "verifier",
        class = verification_class,
        purpose = "Run supplied named checks over an immutable subject and report bounded evidence.",
        capability_ceiling = ["context_read"],
        eligible_model_profiles = ["mock-standard"],
        prompt_sources = ["host-policy"],
        skills = [],
        toolsets = [],
        tool_capabilities = [],
        mutation_posture = verification_service_only,
        context_posture = clean_recompose,
        evidence_posture = check_evidence,
        output_kind = verification_output,
        separation_policy_ref = "independent-verifier",
        budget_policy_ref = "verifier-budget",
        profile_hash = verifier_hash,
      },
      common_profile & {
        role_id = "judge",
        class = judgment_class,
        purpose = "Apply a pinned rubric without granting authority or overriding deterministic denials.",
        capability_ceiling = ["context_read"],
        eligible_model_profiles = ["mock-standard"],
        prompt_sources = ["host-policy"],
        skills = [],
        toolsets = [],
        tool_capabilities = [],
        mutation_posture = no_mutation,
        context_posture = clean_recompose,
        evidence_posture = judgment_evidence,
        output_kind = judgment_output,
        separation_policy_ref = "independent-judge",
        budget_policy_ref = "judge-budget",
        profile_hash = judge_hash,
      },
      common_profile & {
        role_id = "coordinator",
        class = coordination_class,
        purpose = "Coordinate bounded metadata-only work and handoffs without inherited authority.",
        capability_ceiling = ["context_read"],
        eligible_model_profiles = ["mock-standard"],
        prompt_sources = ["host-policy"],
        skills = [],
        toolsets = [],
        tool_capabilities = [],
        mutation_posture = no_mutation,
        context_posture = bounded_shared,
        evidence_posture = coordination_evidence,
        output_kind = coordination_output,
        separation_policy_ref = "subject-bound",
        budget_policy_ref = "coordinator-budget",
        profile_hash = coordinator_hash,
      },
      common_profile & {
        role_id = "observer",
        class = observation_class,
        purpose = "Observe bounded admitted records without mutation or completion authority.",
        capability_ceiling = ["context_read"],
        eligible_model_profiles = ["mock-standard"],
        prompt_sources = ["host-policy"],
        skills = [],
        toolsets = [],
        tool_capabilities = [],
        mutation_posture = no_mutation,
        context_posture = read_only_observation,
        evidence_posture = observation_evidence,
        output_kind = observation_output,
        separation_policy_ref = "subject-bound",
        budget_policy_ref = "observer-budget",
        profile_hash = observer_hash,
      },
    ],
  },
}
