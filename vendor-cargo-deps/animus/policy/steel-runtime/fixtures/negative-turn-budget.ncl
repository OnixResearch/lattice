let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let profile = std.array.first base.profiles in
let bad_budget = profile.budget & { max_turns | force = 0 } in
{
  expected_issue = "STEEL_RUNTIME_TURN_BUDGET",
  fixture | schema.steel_runtime_policy_contract =
    base & { profiles | force = [profile & { budget | force = bad_budget }] },
}
