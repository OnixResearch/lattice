let schema = import "../../agent-role-profiles.ncl" in
let base = schema.agent_role_catalog in
let mutated_profiles = std.array.map
  (fun profile =>
    if profile.role_id == "observer" then
      profile & {
        source_scope | force = "project",
        project_profile_admitted | force = false,
      }
    else profile)
  base.profiles
in
{
  expected_issue = "AGENT_ROLE_TRUST_INVARIANT",
  fixture | schema.agent_role_catalog_contract = base & { profiles | force = mutated_profiles },
}
