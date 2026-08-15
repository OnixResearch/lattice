# Typed authoring contracts for immutable operation data and closed selectors.
# Rust admits the closed enum labels and cross-field rules.
let IdentityCharacterLength = 67 in
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == IdentityCharacterLength
) in
let BoundedText = String in
let Producer = {
  operation_type | BoundedText,
  type_revision | Identity,
  instance | BoundedText,
  method | BoundedText,
  invocation | Identity,
  run_id | Number,
  node | BoundedText,
  workflow_revision | Identity,
} in
let Retention = {
  identity | Identity,
  maximum_versions | optional | Number,
} in
let Lifetime = {
  kind | String,
  run_id | optional | Number,
  expires_epoch_seconds | optional | Number,
} in
let Payload = {
  class | String,
  schema_identity | optional | Identity,
  selectable_paths | optional | Array (Array BoundedText),
  value | optional | Dyn,
  media_class | optional | BoundedText,
  bytes | optional | Array Number,
} in
let RecordDraftContract = {
  logical_key | BoundedText,
  sequence | Number,
  producer | Producer,
  output_name | BoundedText,
  output_contract_identity | Identity,
  tags | { _ | BoundedText },
  sensitivity | String,
  lifetime | Lifetime,
  retention | Retention,
  outcome | BoundedText,
  payload | Payload,
  required_non_claims | Array String,
} in
let TagSelector = {
  key | BoundedText,
  value | BoundedText,
} in
let QueryContract = {
  schema = "lattice.operation-data-query.v1",
  version_identity | optional | Identity,
  record_identity | optional | Identity,
  record_class | optional | String,
  operation_type | optional | BoundedText,
  method | optional | BoundedText,
  instance | optional | BoundedText,
  output_name | optional | BoundedText,
  schema_identity | optional | Identity,
  tag | optional | TagSelector,
  producer_run_id | optional | Number,
  workflow_revision | optional | Identity,
  outcome | optional | BoundedText,
  field_path | optional | Array BoundedText,
  order | String,
  maximum_result_count | Number,
} in
{
  RecordDraft = RecordDraftContract,
  Query = QueryContract,
}
