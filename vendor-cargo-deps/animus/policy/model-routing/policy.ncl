# r[impl agent_harness_runtime.nickel_policy_contracts.model_routing]
let contracts = import "../contracts/primitives.ncl" in

let policy_schema_version = 1 in
let maximum_score = 65535 in
let maximum_weight = 65535 in
let missing_price_behaviors = ["deny", "allow-with-warning", "host-fallback"] in
let budget_behaviors = ["prefer-cheaper-then-deny"] in
let no_downgrade_role = "" in

let SchemaVersion = contracts.exact "MODEL_ROUTING_SCHEMA" policy_schema_version in
let RoleLabel = contracts.safe_identifier "MODEL_ROUTING_ROLE_LABEL" in
let OptionalRoleLabel = contracts.optional_safe_identifier "MODEL_ROUTING_DOWNGRADE_LABEL" in
let Score = contracts.non_negative_integer "MODEL_ROUTING_SCORE" maximum_score in
let Weight = contracts.non_negative_integer "MODEL_ROUTING_WEIGHT" maximum_weight in
let MissingPriceBehavior = contracts.enum "MODEL_ROUTING_MISSING_PRICE_BEHAVIOR" missing_price_behaviors in
let BudgetBehavior = contracts.enum "MODEL_ROUTING_BUDGET_BEHAVIOR" budget_behaviors in
let PolicyHash = contracts.lowercase_blake3 "MODEL_ROUTING_POLICY_HASH" in

let Role = {
  role_label | RoleLabel,
  minimum_score | Score,
  downgrade_role_label | OptionalRoleLabel,
} in
let Thresholds = {
  simple_max_score | Score,
  medium_max_score | Score,
} in
let ComplexityWeights = {
  token_bucket_weight | Weight,
  file_bucket_weight | Weight,
  tool_bucket_weight | Weight,
  risk_bucket_weight | Weight,
} in
let RoutingPolicyShape = {
  schema_version | SchemaVersion,
  roles | Array Role,
  thresholds | Thresholds,
  weights | ComplexityWeights,
  missing_price_behavior | MissingPriceBehavior,
  budget_behavior | BudgetBehavior,
  policy_hash | PolicyHash,
} in

let _ordered_role_minima = fun roles =>
  let state = std.array.fold_left
    (fun previous role => {
      valid = previous.valid && (previous.first || role.minimum_score > previous.score),
      first = false,
      score = role.minimum_score,
    })
    { valid = true, first = true, score = 0 }
    roles
  in
  state.valid
in

let _roles_cover_thresholds = fun policy =>
  (std.array.first policy.roles).minimum_score == 0
  && std.array.any
    (fun role =>
      role.minimum_score > policy.thresholds.simple_max_score
      && role.minimum_score <= policy.thresholds.medium_max_score
    )
    policy.roles
  && std.array.any
    (fun role => role.minimum_score > policy.thresholds.medium_max_score)
    policy.roles
in

let _downgrade_is_valid = fun roles role =>
  if role.downgrade_role_label == no_downgrade_role then
    true
  else
    let matches = std.array.filter
      (fun candidate => candidate.role_label == role.downgrade_role_label)
      roles
    in
    std.array.length matches == 1
    && (std.array.first matches).minimum_score < role.minimum_score
in

let _routing_policy_validator = fun policy =>
  if std.array.length policy.roles == 0 then
    'Error { message = "MODEL_ROUTING_EMPTY_ROLES" }
  else if !contracts.unique_by (fun role => role.role_label) policy.roles then
    'Error { message = "MODEL_ROUTING_DUPLICATE_ROLE" }
  else if policy.thresholds.simple_max_score >= policy.thresholds.medium_max_score then
    'Error { message = "MODEL_ROUTING_THRESHOLD_ORDER" }
  else if !_ordered_role_minima policy.roles then
    'Error { message = "MODEL_ROUTING_ROLE_ORDER" }
  else if !_roles_cover_thresholds policy then
    'Error { message = "MODEL_ROUTING_ROLE_THRESHOLDS" }
  else if !std.array.all (_downgrade_is_valid policy.roles) policy.roles then
    'Error { message = "MODEL_ROUTING_DOWNGRADE_GRAPH" }
  else
    'Ok
in
let RoutingPolicy = contracts.with_validator RoutingPolicyShape _routing_policy_validator in

let simple_max_score_value = 4 in
let medium_max_score_value = 10 in
let economy_minimum_score_value = 0 in
let balanced_minimum_score_value = 5 in
let reasoning_minimum_score_value = 11 in
let token_bucket_weight_value = 2 in
let file_bucket_weight_value = 1 in
let tool_bucket_weight_value = 2 in
let risk_bucket_weight_value = 3 in
let deny_missing_price_value = "deny" in
let prefer_cheaper_then_deny_value = "prefer-cheaper-then-deny" in
let policy_hash_value = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" in
let policy_value = {
  schema_version = policy_schema_version,
  roles = [
    {
      role_label = "economy",
      minimum_score = economy_minimum_score_value,
      downgrade_role_label = no_downgrade_role,
    },
    {
      role_label = "balanced",
      minimum_score = balanced_minimum_score_value,
      downgrade_role_label = "economy",
    },
    {
      role_label = "deep-reasoning",
      minimum_score = reasoning_minimum_score_value,
      downgrade_role_label = "balanced",
    },
  ],
  thresholds = {
    simple_max_score = simple_max_score_value,
    medium_max_score = medium_max_score_value,
  },
  weights = {
    token_bucket_weight = token_bucket_weight_value,
    file_bucket_weight = file_bucket_weight_value,
    tool_bucket_weight = tool_bucket_weight_value,
    risk_bucket_weight = risk_bucket_weight_value,
  },
  missing_price_behavior = deny_missing_price_value,
  budget_behavior = prefer_cheaper_then_deny_value,
  policy_hash = policy_hash_value,
} in
{
  model_routing_policy_contract = RoutingPolicy,
  model_routing_policy | RoutingPolicy = policy_value,
}
