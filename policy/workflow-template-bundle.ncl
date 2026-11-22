# Canonical authoring contract for local Lattice workflow-template manifests (schema 2).
#
# Schema 1 manifests, which have neither `mode` nor `params`, still load as `mutating`
# bundles without parameters; see docs/workflow-template-bundles.md.
let digest_bytes = 32 in
let hex_characters_per_byte = 2 in
let digest_hex_characters = digest_bytes * hex_characters_per_byte in
let bundle_schema_version = 2 in
let LowerIdentifier = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let ContentIdentity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == digest_hex_characters
) in
let SchemaVersion = std.contract.from_predicate (fun value => value == bundle_schema_version) in
let Mode = std.contract.from_predicate (fun value => value == "read_only" || value == "mutating") in
let ParamName = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[a-z][a-z0-9_]{0,63}$" value
) in
let ParamEnvKey = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^LATTICE_PARAM_[A-Z0-9_]{1,50}$" value
) in
# Each closed parameter type names exactly its bound fields; the loader checks their values.
let param_type_fields = {
  bool = ["kind"],
  integer = ["kind", "max", "min"],
  enum = ["kind", "values"],
  relative_path = ["kind", "max_bytes"],
} in
let ParamType = std.contract.from_predicate (fun value =>
  std.is_record value
  && std.record.has_field "kind" value
  && std.is_string value.kind
  && std.record.has_field value.kind param_type_fields
  && std.record.fields value == std.record.get value.kind param_type_fields
) in
let ParamValue = std.contract.from_predicate (fun value =>
  std.is_bool value || std.is_number value || std.is_string value
) in
let ParamTarget = {
  node | String,
  shell_env | ParamEnvKey,
} in
let ParamDeclaration = {
  name | ParamName,
  type | ParamType,
  required | Bool,
  default | optional | ParamValue,
  bindings | Array ParamTarget,
} in
let FileBinding = {
  path | String,
  blake3 | ContentIdentity,
} in
let AssetBinding = {
  path | String,
  media_kind | String,
  blake3 | ContentIdentity,
  required | Bool,
} in
let Compatibility = {
  workflow_schema | Number,
  orchestration_schema | optional | Number,
  node_kinds | Array String,
  runtime_kinds | Array String,
  feature_ids | Array String,
} in
let WorkflowTemplateManifest = {
  schema_version | SchemaVersion,
  id | LowerIdentifier,
  display_name | String,
  description | String,
  category | LowerIdentifier,
  mode | Mode,
  params | Array ParamDeclaration,
  workflow | FileBinding,
  orchestration | optional | FileBinding,
  assets | Array AssetBinding,
  compatibility | Compatibility,
  operator_notes | Array String,
  non_claims | Array String,
} in
WorkflowTemplateManifest
