# Typed policy for the optional Prime Agent persistent-goal cohort.
let contracts = import "../policy/contracts/primitives.ncl" in
let max_objective_identity_length_value = 128 in
let minimum_budget_limit_value = 1 in
let Profile = {
  schema | contracts.exact "GOAL_SCHEMA" "animus.persistent-goals.v1",
  cohort | contracts.exact "GOAL_COHORT" "prime-agent-persistent-goals",
  license | contracts.exact "GOAL_LICENSE" "MIT",
  source_revision | contracts.safe_identifier "GOAL_SOURCE_REVISION",
  max_objective_identity_length | contracts.positive_integer "GOAL_MAX_OBJECTIVE_ID" max_objective_identity_length_value,
  minimum_budget_limit | contracts.positive_integer "GOAL_MIN_BUDGET" minimum_budget_limit_value,
  statuses | Array String,
  actions | Array String,
  receipt_fields | Array String,
  non_claims | Array String,
  one_active_goal | Bool,
  explicit_completion_only | Bool,
  host_injected_accounting | Bool,
} in
let ProfileWithRules = contracts.with_validator Profile (fun value =>
  if std.array.length value.statuses != 6 then
    'Error { message = "GOAL_STATUS_COUNT" }
  else if std.array.length value.actions != 6 then
    'Error { message = "GOAL_ACTION_COUNT" }
  else if std.array.length value.receipt_fields < 7 then
    'Error { message = "GOAL_RECEIPT_FIELDS" }
  else if std.array.length value.non_claims < 4 then
    'Error { message = "GOAL_NON_CLAIMS" }
  else if !std.array.elem "not-objective-achievement" value.non_claims then
    'Error { message = "GOAL_ACHIEVEMENT_NON_CLAIM" }
  else if !std.array.elem "not-automatic-inference" value.non_claims then
    'Error { message = "GOAL_INFERENCE_NON_CLAIM" }
  else if !value.one_active_goal || !value.explicit_completion_only || !value.host_injected_accounting then
    'Error { message = "GOAL_BOUNDARY_RULES" }
  else
    'Ok
) in
{
  persistent_goals_policy_contract = ProfileWithRules,
  persistent_goals_policy | ProfileWithRules = {
    schema = "animus.persistent-goals.v1",
    cohort = "prime-agent-persistent-goals",
    license = "MIT",
    source_revision = "c5991bc853d27754aed345c13ff4d2e05c40f1f4",
    max_objective_identity_length = max_objective_identity_length_value,
    minimum_budget_limit = minimum_budget_limit_value,
    statuses = ["active", "paused", "completed", "budget_limited", "errored", "cleared"],
    actions = ["pause", "resume", "complete", "clear", "error", "budget_exhausted"],
    receipt_fields = [
      "policy_identity",
      "transition",
      "status",
      "budget_class",
      "token_counter_identity",
      "elapsed_counter_identity",
      "continuation_counter_identity",
    ],
    non_claims = [
      "not-objective-achievement",
      "not-work-quality",
      "not-automatic-inference",
      "not-production-or-release-readiness",
    ],
    one_active_goal = true,
    explicit_completion_only = true,
    host_injected_accounting = true,
  },
}
