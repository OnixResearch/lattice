let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
{
  expected_issue = "AENEAS_CAMPAIGN_SKILL_HASH",
  fixture | schema.campaign_profile_contract = base & {
    skill | force = base.skill & { source_blake3 | force = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
  },
}
