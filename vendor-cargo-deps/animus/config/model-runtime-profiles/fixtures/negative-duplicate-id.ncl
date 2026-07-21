let schema = import "../../model-runtime-profiles.ncl" in
let base = schema.model_runtime_profile_set in
let first = std.array.first base.profiles in
{
  expected_issue = "MODEL_PROFILE_DUPLICATE_ID",
  fixture | schema.model_runtime_profile_set_contract =
    base & { profiles | force = std.array.concat base.profiles [first] },
}
