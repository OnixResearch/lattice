let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_ID",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { profile_id | force = "Capacity Aware" },
}
