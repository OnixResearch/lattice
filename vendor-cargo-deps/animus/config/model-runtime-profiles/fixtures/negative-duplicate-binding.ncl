let schema = import "../../model-runtime-profiles.ncl" in
let base = schema.model_runtime_profile_set in
let first = std.array.first base.profiles in
let duplicate = first & {
  profile_id | force = "other-profile",
  profile_hash | force = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
} in
{
  expected_issue = "MODEL_PROFILE_DUPLICATE_BINDING",
  fixture | schema.model_runtime_profile_set_contract =
    base & { profiles | force = std.array.concat base.profiles [duplicate] },
}
