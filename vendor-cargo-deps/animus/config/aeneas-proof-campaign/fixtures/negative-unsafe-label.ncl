let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
{
  expected_issue = "AENEAS_CAMPAIGN_LABEL",
  fixture | schema.campaign_profile_contract = base & {
    receipt_fields | force = ["RAW_PROMPT"],
  },
}
