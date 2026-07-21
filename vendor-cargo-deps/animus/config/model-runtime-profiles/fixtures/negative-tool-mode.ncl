let schema = import "../../model-runtime-profiles.ncl" in
let base = schema.model_runtime_profile_set in
let first = std.array.first base.profiles in
let changed = first & { tool_calling_mode | force = "prompt_emulated" } in
let changed_profiles = std.array.map
  (fun profile => if profile.profile_id == first.profile_id then changed else profile)
  base.profiles
in
{
  expected_issue = "MODEL_PROFILE_TOOL_MODE",
  fixture | schema.model_runtime_profile_set_contract =
    base & { profiles | force = changed_profiles },
}
