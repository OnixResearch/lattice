let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let profile = std.array.first base.profiles in
let bad_script = profile.script & { label | force = "../outside.scm" } in
{
  expected_issue = "STEEL_RUNTIME_SCRIPT_PATH",
  fixture | schema.steel_runtime_policy_contract =
    base & { profiles | force = [profile & { script | force = bad_script }] },
}
