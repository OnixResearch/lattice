let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
{
  expected_issue = "AENEAS_CAMPAIGN_POSITIVE_BUDGET",
  fixture | schema.campaign_profile_contract = base & {
    budgets | force = base.budgets & { proof_time_ms | force = 0 },
  },
}
