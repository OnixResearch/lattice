let schema = import "../policy.ncl" in
let base = schema.model_routing_policy in
{
  expected_issue = "MODEL_ROUTING_THRESHOLD_ORDER",
  fixture | schema.model_routing_policy_contract =
    base
    & {
      thresholds | force = base.thresholds & { simple_max_score | force = base.thresholds.medium_max_score },
    },
}
