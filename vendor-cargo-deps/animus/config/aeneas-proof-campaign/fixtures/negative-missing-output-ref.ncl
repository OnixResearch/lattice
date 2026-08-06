let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
let mutated_stages = std.array.map
  (fun stage => if stage.stage_id == "request-proof" then stage & { output_contract | force = "" } else stage)
  base.stages
in
{
  expected_issue = "output_contract",
  fixture | schema.campaign_profile_contract = base & { stages | force = mutated_stages },
}
