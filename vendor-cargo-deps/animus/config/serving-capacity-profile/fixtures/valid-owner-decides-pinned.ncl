let schema = import "../../serving-capacity-profile.ncl" in
{
  expected_issue = "OK",
  fixture | schema.serving_capacity_profile_contract =
    schema.serving_capacity_profile
    & {
      profile_id | force = "owner-decides-pinned-v1",
      identity_rule | force = "claimed_accepted",
      capacity_rule | force = "owner_admission_decides",
      replica_rule | force = "pinned",
    },
}
