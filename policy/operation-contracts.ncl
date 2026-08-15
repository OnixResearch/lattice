# Typed authoring contracts for Lattice operation catalogs and instances.
let IdentityCharacterLength = 67 in
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == IdentityCharacterLength
) in
# These wire labels stay strings for JSON export. Rust admits only the closed enum values.
let ValueType = String in
let Capability = String in
let SideEffect = String in
let Preflight = String in
let Bounds = {
  timeout_seconds | Number,
  input_bytes | Number,
  output_bytes | Number,
  artifact_bytes | Number,
} in
let Field = {
  name | String,
  value_type | ValueType,
  required | Bool,
  max_bytes | Number,
} in
let RecordContract = {
  fields | Array Field,
  deny_unknown_fields | Bool,
} in
let Method = {
  name | String,
  description | String,
  arguments | RecordContract,
  output | RecordContract,
  preflight | Array Preflight,
  capabilities | Array Capability,
  side_effect | SideEffect,
} in
let Runtime = {
  kind | String,
  key | String,
  executor_identity | Identity,
} in
let CredentialBinding = {
  name | String,
  required | Bool,
  allow_field | Bool,
} in
let OperationType = {
  id | String,
  schema_version | Number,
  revision | Identity,
  description | String,
  tags | Array String,
  methods | Array Method,
  runtime | Runtime,
  capabilities | Array Capability,
  credential_bindings | Array CredentialBinding,
  bounds | Bounds,
  required_non_claims | Array String,
} in
let SecretRef = {
  provider | String,
  key | String,
  field | optional | String,
} in
let OperationInstance = {
  id | String,
  display_alias | String,
  type_id | String,
  type_revision | Identity,
  target | { _ | Dyn },
  tags | { _ | String },
  method_defaults | { _ | { _ | Dyn } },
  credentials | { _ | SecretRef },
} in
{
  Catalog = { schema | String, types | Array OperationType },
  InstanceFile = { schema | String, instances | Array OperationInstance },
  InvocationPlan = {
    schema | String,
    identity | Identity,
    catalog_identity | Identity,
    type_id | String,
    type_revision | Identity,
    instance_id | String,
    instance_identity | Identity,
    method | String,
    method_identity | Identity,
    arguments | { _ | Dyn },
    arguments_identity | Identity,
    output_contract | RecordContract,
    output_contract_identity | Identity,
    runtime | {
      kind | String,
      key | String,
      executor_identity | Identity,
      executable_path | String,
      available | Bool,
      maximum_bounds | Bounds,
    },
    capabilities | Array Capability,
    target | { _ | Dyn },
    credentials | { _ | SecretRef },
    bounds | Bounds,
    required_non_claims | Array String,
  },
}
