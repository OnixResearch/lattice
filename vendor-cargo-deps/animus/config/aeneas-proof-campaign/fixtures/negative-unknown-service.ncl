let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
let mutated_services = std.array.map
  (fun service => if service.service_id == "octet-aeneas-proof" then service & { service_id | force = "ambient-proof-tool" } else service)
  base.services
in
{
  expected_issue = "AENEAS_CAMPAIGN_PROFILE_INVARIANT",
  fixture | schema.campaign_profile_contract = base & { services | force = mutated_services },
}
