let schema = import "../policy.ncl" in
{
  expected_issue = "MODEL_ROUTING_POLICY_HASH",
  fixture | schema.model_routing_policy_contract =
    schema.model_routing_policy
    & { policy_hash | force = "ABCDEF" },
}
