let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_NON_CLAIMS",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { non_claims | force = ["not-gpu-scheduling"] },
}
