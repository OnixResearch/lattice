let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_BOUNDS",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { bounds | force = { max_services = 65, max_capacity_observations = 16, max_route_candidates = 32, max_rollout_operations = 8, max_token_bytes = 256, max_snapshot_bytes = 262144 } },
}
