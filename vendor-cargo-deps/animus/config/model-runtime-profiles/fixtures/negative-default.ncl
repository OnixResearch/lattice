let schema = import "../../model-runtime-profiles.ncl" in
let base = schema.model_runtime_profile_set in
{
  expected_issue = "MODEL_PROFILE_DEFAULT_INVARIANT",
  fixture | schema.model_runtime_profile_set_contract =
    base & { explicit_default_profile_id | force = "mock-standard" },
}
