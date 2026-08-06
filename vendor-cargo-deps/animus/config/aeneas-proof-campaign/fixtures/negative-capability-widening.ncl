let schema = import "../../aeneas-proof-campaign.ncl" in
let base = schema.aeneas_proof_campaign_bundle.campaign_profile in
let mutated_roles = std.array.map
  (fun role => if role.assignment == "statement-reviewer" then role & { capability_ceiling | force = ["source.read", "statement.review", "theorem.write"] } else role)
  base.roles
in
{
  expected_issue = "AENEAS_CAMPAIGN_PROFILE_INVARIANT",
  fixture | schema.campaign_profile_contract = base & { roles | force = mutated_roles },
}
