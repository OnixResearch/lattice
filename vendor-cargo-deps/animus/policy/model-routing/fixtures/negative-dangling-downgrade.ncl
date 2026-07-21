let schema = import "../policy.ncl" in
let base = schema.model_routing_policy in
let changed_roles = std.array.map
  (fun role =>
    if role.role_label == "balanced" then
      role & { downgrade_role_label | force = "missing" }
    else
      role
  )
  base.roles
in
{
  expected_issue = "MODEL_ROUTING_DOWNGRADE_GRAPH",
  fixture | schema.model_routing_policy_contract = base & { roles | force = changed_roles },
}
