let schema = import "../model-catalog.ncl" in
{
  model_catalog | schema.model_catalog_contract = {
    schema = "animus.model-catalog.v1",
    connections = [{
      id = "custom",
      class = "custom",
      credential_env = "CUSTOM_API_KEY",
      credential_value = "forbidden-secret",
    }],
    models = [
      { id = "custom", connection_id = "custom", provider_id = "custom", model_id = "custom-model" },
    ],
  },
}
