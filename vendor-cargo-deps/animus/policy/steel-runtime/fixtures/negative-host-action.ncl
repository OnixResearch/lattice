let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let profile = std.array.first base.profiles in
{
  expected_issue = "STEEL_RUNTIME_HOST_ACTION",
  fixture | schema.steel_runtime_policy_contract =
    base & { profiles | force = [profile & { allowed_host_actions | force = ["filesystem"] }] },
}
