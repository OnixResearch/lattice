let schema = import "../policy.ncl" in
let base = schema.model_routing_policy in
{
  expected_issue = "MODEL_ROUTING_WEIGHT",
  fixture | schema.model_routing_policy_contract =
    base
    & {
      weights | force = base.weights & { token_bucket_weight | force = -1 },
    },
}
