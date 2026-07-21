let BoundsPolicy = {
  min_candidates | Number,
  max_candidates | Number,
  min_parallelism | Number,
  max_parallelism | Number,
  min_quorum | Number,
  max_quorum | Number,
  min_synthesis_attempts | Number,
  max_synthesis_attempts | Number,
  min_repair_attempts | Number,
  max_repair_attempts | Number,
} in
let FamilyPolicy = {
  family | String,
  required_roles | Array String,
  compatible_output_contracts | Array String,
  bounds | BoundsPolicy,
  fallback_policies | Array String,
  backend_allowlist | Array String,
} in
let RouterRecipePolicy = {
  schema | String,
  route_families | Array String,
  roles | Array String,
  output_contracts | Array String,
  deadline_classes | Array String,
  fallback_policies | Array String,
  backend_allowlist | Array String,
  family_policies | Array FamilyPolicy,
  policy_hash | String,
} in
let schema_value = "animus.router-recipe-policy.v1" in
let family_single = "single" in
let family_confidence = "confidence" in
let family_ratings = "ratings" in
let family_remom = "remom" in
let family_fusion = "fusion" in
let family_workflow = "workflow" in
let role_primary = "primary" in
let role_escalation = "escalation" in
let role_rater = "rater" in
let role_synthesizer = "synthesizer" in
let role_judge = "judge" in
let role_finalizer = "finalizer" in
let role_worker = "worker" in
let role_repair = "repair" in
let role_fallback = "fallback" in
let output_text = "text" in
let output_json = "json" in
let output_tool_call = "tool_call" in
let output_artifact = "artifact" in
let deadline_short = "short" in
let deadline_standard = "standard" in
let deadline_extended = "extended" in
let fallback_deny = "deny" in
let fallback_best_valid_evidence = "best_valid_evidence" in
let fallback_safe_single = "safe_single" in
let backend_local = "backend:local" in
let backend_remote = "backend:remote" in
let min_candidates_value = 1 in
let max_candidates_value = 8 in
let min_parallelism_value = 1 in
let max_parallelism_value = 4 in
let min_quorum_value = 1 in
let max_quorum_value = 8 in
let min_synthesis_attempts_value = 0 in
let max_synthesis_attempts_value = 3 in
let min_repair_attempts_value = 0 in
let max_repair_attempts_value = 2 in
let policy_hash_value = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" in
let all_outputs = [output_text, output_json, output_tool_call, output_artifact] in
let all_fallbacks = [fallback_deny, fallback_best_valid_evidence, fallback_safe_single] in
let all_backends = [backend_local, backend_remote] in
let standard_bounds = {
  min_candidates = min_candidates_value,
  max_candidates = max_candidates_value,
  min_parallelism = min_parallelism_value,
  max_parallelism = max_parallelism_value,
  min_quorum = min_quorum_value,
  max_quorum = max_quorum_value,
  min_synthesis_attempts = min_synthesis_attempts_value,
  max_synthesis_attempts = max_synthesis_attempts_value,
  min_repair_attempts = min_repair_attempts_value,
  max_repair_attempts = max_repair_attempts_value,
} in
let family_policy = fun family_value required_roles_value => {
  family = family_value,
  required_roles = required_roles_value,
  compatible_output_contracts = all_outputs,
  bounds = standard_bounds,
  fallback_policies = all_fallbacks,
  backend_allowlist = all_backends,
} in
let family_policy_records = [
  family_policy family_single [role_primary],
  family_policy family_confidence [role_primary, role_escalation],
  family_policy family_ratings [role_primary, role_rater],
  family_policy family_remom [role_worker, role_synthesizer],
  family_policy family_fusion [role_judge, role_finalizer],
  family_policy family_workflow [role_worker, role_finalizer],
] in
let route_family_values = [
  family_single,
  family_confidence,
  family_ratings,
  family_remom,
  family_fusion,
  family_workflow,
] in
let role_values = [
  role_primary,
  role_escalation,
  role_rater,
  role_synthesizer,
  role_judge,
  role_finalizer,
  role_worker,
  role_repair,
  role_fallback,
] in
let output_contract_values = all_outputs in
let deadline_class_values = [deadline_short, deadline_standard, deadline_extended] in
let policy_record = {
  schema = schema_value,
  route_families = route_family_values,
  roles = role_values,
  output_contracts = output_contract_values,
  deadline_classes = deadline_class_values,
  fallback_policies = all_fallbacks,
  backend_allowlist = all_backends,
  family_policies = family_policy_records,
  policy_hash = policy_hash_value,
} in
let csv = fun values => std.string.join "," values in
let family_policy_line = fun record =>
  "family|%{record.family}|%{csv record.required_roles}|%{csv record.compatible_output_contracts}|%{record.bounds.min_candidates}|%{record.bounds.max_candidates}|%{record.bounds.min_parallelism}|%{record.bounds.max_parallelism}|%{record.bounds.min_quorum}|%{record.bounds.max_quorum}|%{record.bounds.min_synthesis_attempts}|%{record.bounds.max_synthesis_attempts}|%{record.bounds.min_repair_attempts}|%{record.bounds.max_repair_attempts}|%{csv record.fallback_policies}|%{csv record.backend_allowlist}|%{csv deadline_class_values}"
in
{
  router_recipe_policy | RouterRecipePolicy = policy_record,
  family_policy_lines = std.string.join "\n" (std.array.map family_policy_line family_policy_records),
}
