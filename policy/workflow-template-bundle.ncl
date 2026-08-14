# Canonical authoring contract for local Lattice workflow-template manifests.
let digest_bytes = 32 in
let hex_characters_per_byte = 2 in
let digest_hex_characters = digest_bytes * hex_characters_per_byte in
let LowerIdentifier = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let ContentIdentity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == digest_hex_characters
) in
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
  schema_version | Number,
  id | LowerIdentifier,
  display_name | String,
  description | String,
  category | LowerIdentifier,
  workflow | FileBinding,
  orchestration | optional | FileBinding,
  assets | Array AssetBinding,
  compatibility | Compatibility,
  operator_notes | Array String,
  non_claims | Array String,
} in
WorkflowTemplateManifest
