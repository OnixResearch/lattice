let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
{
  expected_issue = "model_runtime_profile_ref",
  fixture | schema.campaign_profile_contract = base & {
    model_runtime_profile_ref | force = "ambient-model-provider",
  },
}
