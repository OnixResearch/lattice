let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "OK",
  fixture | schema.serving_capacity_profile_contract = schema.serving_capacity_profile,
}
