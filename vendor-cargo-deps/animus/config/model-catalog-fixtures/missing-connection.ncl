let schema = import "../model-catalog.ncl" in
{
  model_catalog | schema.model_catalog_contract = {
    schema = "animus.model-catalog.v1",
    connections = [{ id = "mock", class = "built_in", credential_env = "" }],
    models = [
      { id = "missing", connection_id = "absent", provider_id = "mock", model_id = "mock-text" },
    ],
  },
}
