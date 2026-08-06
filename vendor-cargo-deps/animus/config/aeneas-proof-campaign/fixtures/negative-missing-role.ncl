let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
let remaining_roles = std.array.filter (fun role => role.assignment != "verifier") base.roles in
{
  expected_issue = "AENEAS_CAMPAIGN_PROFILE_INVARIANT",
  fixture | schema.campaign_profile_contract = base & { roles | force = remaining_roles },
}
