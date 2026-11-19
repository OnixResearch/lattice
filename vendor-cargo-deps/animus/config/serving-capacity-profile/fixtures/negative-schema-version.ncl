let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_SCHEMA",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { schema | force = "animus.serving-capacity-profile.v2" },
}
