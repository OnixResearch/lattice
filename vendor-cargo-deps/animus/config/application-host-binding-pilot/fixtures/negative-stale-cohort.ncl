let profile = import "../profile.ncl" in
{
  expected_issue = "APPLICATION_BINDING_UPSTREAM_REVISION",
  fixture | profile.cohort_contract = profile.kamacite_cohort & {
    producer_revision | force = "0000000000000000000000000000000000000000",
  },
}
