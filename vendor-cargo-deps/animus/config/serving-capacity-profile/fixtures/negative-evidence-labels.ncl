let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "SERVING_PROFILE_EVIDENCE_LABELS",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile & { evidence_labels | force = ["fixture"] },
}
