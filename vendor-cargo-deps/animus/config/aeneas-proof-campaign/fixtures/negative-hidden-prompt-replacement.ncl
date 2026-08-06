let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
{
  expected_issue = "replaces_runtime_policy",
  fixture | schema.campaign_profile_contract = base & {
    skill | force = base.skill & { replaces_runtime_policy | force = true },
  },
}
