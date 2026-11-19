let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_REDACTION",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { redaction | force = "raw_payloads" },
}
