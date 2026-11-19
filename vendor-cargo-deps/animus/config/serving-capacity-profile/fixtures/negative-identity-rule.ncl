let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_IDENTITY_RULE",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { identity_rule | force = "model_list_sufficient" },
}
