let schema = import "../../agent-role-profiles.ncl" in
let base = schema.agent_role_catalog in
let mutated_profiles = std.array.map
  (fun profile =>
    if profile.role_id == "reviewer" then
      profile & { output_contract | force = "missing-output-contract" }
    else profile)
  base.profiles
in
{
  expected_issue = "AGENT_ROLE_DANGLING_REFERENCE",
  fixture | schema.agent_role_catalog_contract = base & { profiles | force = mutated_profiles },
}
