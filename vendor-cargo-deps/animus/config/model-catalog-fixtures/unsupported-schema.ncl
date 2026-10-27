let schema = import "../model-catalog.ncl" in
{
  model_catalog | schema.model_catalog_contract = {
    schema = "animus.model-catalog.v0",
    connections = [{ id = "mock", class = "built_in", credential_env = "" }],
    models = [
      { id = "mock-text", connection_id = "mock", provider_id = "mock", model_id = "mock-text" },
    ],
  },
}
