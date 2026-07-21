let schema = import "../../model-runtime-profiles.ncl" in
{
  fixture | schema.model_runtime_profile_set_contract = schema.model_runtime_profile_set,
}
