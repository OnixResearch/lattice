let schema = import "../policy.ncl" in
{
  expected_issue = "OK",
  fixture | schema.model_routing_policy_contract = schema.model_routing_policy,
}
