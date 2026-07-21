let schema = import "../policy.ncl" in
let first_role = std.array.at 0 schema.model_routing_policy.roles in
{
  expected_issue = "MODEL_ROUTING_DUPLICATE_ROLE",
  fixture | schema.model_routing_policy_contract =
    schema.model_routing_policy
    & { roles | force = [first_role, first_role] },
}
