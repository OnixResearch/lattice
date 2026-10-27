# Typed shipped model catalog source of truth.
# Runtime hosts parse only the checked JSON projection.
# r[impl animus.catalog.version]
# r[impl animus.catalog.secret]
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.model-catalog.v1" in
let built_in_class = "built_in" in
let custom_class = "custom" in
let maximum_connections = 32 in
let maximum_models = 128 in
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let LowerLabel = contracts.safe_identifier "MODEL_CATALOG_LOWER_LABEL" in
let EnvironmentName = std.contract.from_predicate (fun value =>
  std.is_string value
  && (value == "" || std.string.is_match "^[A-Z][A-Z0-9_]*$" value)
) in
let Connection = {
  id | LowerLabel,
  class | contracts.enum "MODEL_CATALOG_CONNECTION_CLASS" [built_in_class, custom_class],
  credential_env | EnvironmentName,
} in
let Model = {
  id | LowerLabel,
  connection_id | LowerLabel,
  provider_id | LowerLabel,
  model_id | NonEmptyString,
} in
let CatalogShape = {
  schema | contracts.exact "MODEL_CATALOG_SCHEMA" schema_identity,
  connections | Array Connection,
  models | Array Model,
} in
let catalog_is_valid = fun catalog =>
  let connection_count = std.array.length catalog.connections in
  let model_count = std.array.length catalog.models in
  let connection_ids = std.array.map (fun connection => connection.id) catalog.connections in
  if connection_count == 0 || connection_count > maximum_connections then
    'Error { message = "MODEL_CATALOG_CONNECTION_BOUND" }
  else if model_count == 0 || model_count > maximum_models then
    'Error { message = "MODEL_CATALOG_MODEL_BOUND" }
  else if !contracts.unique_by (fun connection => connection.id) catalog.connections then
    'Error { message = "MODEL_CATALOG_DUPLICATE_CONNECTION" }
  else if !contracts.unique_by (fun model => model.id) catalog.models then
    'Error { message = "MODEL_CATALOG_DUPLICATE_MODEL" }
  else if !std.array.all
    (fun model => std.array.elem model.connection_id connection_ids)
    catalog.models
  then
    'Error { message = "MODEL_CATALOG_MISSING_CONNECTION" }
  else
    'Ok
in
let ModelCatalog = contracts.with_validator CatalogShape catalog_is_valid in

{
  model_catalog_contract = ModelCatalog,
  model_catalog | ModelCatalog = {
    schema = schema_identity,
    connections = [
      {
        id = "mock",
        class = built_in_class,
        credential_env = "",
      },
    ],
    models = [
      {
        id = "mock-text",
        connection_id = "mock",
        provider_id = "mock",
        model_id = "mock-text",
      },
    ],
  },
}
