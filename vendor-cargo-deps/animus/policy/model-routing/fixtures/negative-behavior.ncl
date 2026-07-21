let schema = import "../policy.ncl" in
{
  expected_issue = "MODEL_ROUTING_MISSING_PRICE_BEHAVIOR",
  fixture | schema.model_routing_policy_contract =
    schema.model_routing_policy
    & { missing_price_behavior | force = "ambient" },
}
