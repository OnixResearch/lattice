let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
{
  expected_issue = "AENEAS_CAMPAIGN_PROFILE_INVARIANT",
  fixture | schema.campaign_profile_contract = base & {
    non_claims | force = ["campaign conformance does not prove theorem truth"],
  },
}
