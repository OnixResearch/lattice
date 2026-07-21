let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let profile = std.array.first base.profiles in
let bad_script = profile.script & { hash | force = "ABCDEF" } in
{
  expected_issue = "STEEL_RUNTIME_SCRIPT_HASH",
  fixture | schema.steel_runtime_policy_contract =
    base & { profiles | force = [profile & { script | force = bad_script }] },
}
