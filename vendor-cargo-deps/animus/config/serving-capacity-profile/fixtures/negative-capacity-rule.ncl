let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_CAPACITY_RULE",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { capacity_rule | force = "free_memory_sufficient" },
}
