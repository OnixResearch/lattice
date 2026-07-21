let schema = import "../../model-runtime-profiles.ncl" in
let base = schema.model_runtime_profile_set in
{
  expected_issue = "MODEL_PROFILE_SCHEMA",
  fixture | schema.model_runtime_profile_set_contract =
    base & { schema | force = "animus.model-runtime-profiles.v0" },
}
