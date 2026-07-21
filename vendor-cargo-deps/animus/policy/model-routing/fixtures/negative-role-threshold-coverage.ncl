let schema = import "../policy.ncl" in
let base = schema.model_routing_policy in
let changed_roles = std.array.map
  (fun role =>
    if role.role_label == "deep-reasoning" then
      role & { minimum_score | force = base.thresholds.medium_max_score }
    else
      role
  )
  base.roles
in
{
  expected_issue = "MODEL_ROUTING_ROLE_THRESHOLDS",
  fixture | schema.model_routing_policy_contract = base & { roles | force = changed_roles },
}
