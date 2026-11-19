let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_OWNER_ADMISSION",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { owner_admission | force = "optional" },
}
