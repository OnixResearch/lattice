let schema = import "../../agent-role-profiles.ncl" in
let base = schema.agent_role_catalog in
let first = std.array.first base.profiles in
{
  expected_issue = "AGENT_ROLE_DUPLICATE_PROFILE",
  fixture | schema.agent_role_catalog_contract =
    base & { profiles | force = std.array.concat base.profiles [first] },
}
