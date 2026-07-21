let schema = import "../../agent-role-profiles.ncl" in
let base = schema.agent_role_catalog in
let mutated_profiles = std.array.filter (fun profile => profile.role_id != "implementer") base.profiles in
{
  expected_issue = "AGENT_ROLE_BASELINE_INVARIANT",
  fixture | schema.agent_role_catalog_contract = base & { profiles | force = mutated_profiles },
}
