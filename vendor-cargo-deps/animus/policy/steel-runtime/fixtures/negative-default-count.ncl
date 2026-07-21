let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let changed_profiles = std.array.map (fun profile => profile & { default | force = false }) base.profiles in
{
  expected_issue = "STEEL_RUNTIME_DEFAULT_COUNT",
  fixture | schema.steel_runtime_policy_contract = base & { profiles | force = changed_profiles },
}
