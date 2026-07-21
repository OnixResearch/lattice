let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let profile = std.array.first base.profiles in
{
  expected_issue = "STEEL_RUNTIME_DUPLICATE_PROFILE",
  fixture | schema.steel_runtime_policy_contract = base & { profiles | force = [profile, profile] },
}
