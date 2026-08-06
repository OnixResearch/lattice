let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle in
{
  expected_issue = "source_count",
  fixture | schema.campaign_bundle_contract = base & {
    skill_manifest | force = base.skill_manifest & { source_count | force = 2 },
  },
}
