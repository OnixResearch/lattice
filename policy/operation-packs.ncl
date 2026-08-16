# Typed authoring contracts for closed operation-pack manifests, locks, and registries.
# Rust remains authoritative for BLAKE3 recomputation and cross-field admission.
let IdentityCharacterLength = 67 in
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == IdentityCharacterLength
) in
let ContentItem = {
  id | String,
  role | String,
  path | String,
  content_identity | Identity,
  byte_length | Number,
  media_class | String,
} in
let OperationBinding = {
  type_id | String,
  type_revision | Identity,
  catalog_content_id | String,
  runtime_id | String,
  launch_label | String,
  methods | Array String,
} in
let RuntimeArtifact = {
  id | String,
  content_id | String,
  kind | String,
  launch_labels | Array String,
  platforms | Array String,
  capabilities | Array String,
  timeout_seconds | Number,
  input_bytes | Number,
  output_bytes | Number,
  clear_environment | Bool,
} in
let Analyzer = {
  id | String,
  scope | String,
  data_view | Array String,
  required | Bool,
  runtime_id | String,
  launch_label | String,
  input_bytes | Number,
  timeout_seconds | Number,
  output_bytes | Number,
  output_schema_identity | Identity,
  markdown_allowed | Bool,
  maximum_markdown_bytes | Number,
  retention_identity | Identity,
  sensitivity | String,
  required_non_claims | Array String,
} in
let Skill = {
  name | String,
  description | String,
  body_content_id | String,
  reference_content_ids | Array String,
  hosts | Array String,
  trust | String,
  positive_routes | Array String,
  negative_routes | Array String,
} in
let Dependency = {
  pack_id | String,
  pack_identity | Identity,
} in
let Admission = {
  identity | Identity,
  required_evidence | Array String,
  allowed_licenses | Array String,
  allowed_capabilities | Array String,
} in
let Release = {
  display_version | String,
  channel | String,
  deprecated | Bool,
  superseded_by | optional | Identity,
  publication_epoch_seconds | optional | Number,
} in
let Probe = {
  runtime_id | String,
  launch_label | String,
  effectful | Bool,
  uses_production_credentials | Bool,
} in
let ManifestContract = {
  schema = "lattice.operation-pack-manifest.v1",
  pack_id | String,
  description | String,
  contents | Array ContentItem,
  operations | Array OperationBinding,
  runtimes | Array RuntimeArtifact,
  analyzers | Array Analyzer,
  skills | Array Skill,
  dependencies | Array Dependency,
  platforms | Array String,
  admission | Admission,
  release | Release,
  licenses | Array String,
  probe | optional | Probe,
  required_non_claims | Array String,
} in
let LockContract = {
  schema = "lattice.operation-pack-lock.v1",
  identity | Identity,
  pack_id | String,
  pack_identity | Identity,
  manifest_identity | Identity,
  dependency_identities | { _ | Identity },
  platform | String,
  runtime_identities | { _ | Identity },
  analyzer_identities | { _ | Identity },
  skill_identities | { _ | Identity },
  trust_profile_identity | Identity,
  source_locator | Identity,
} in
let RegistryEntry = {
  pack_id | String,
  pack_identity | Identity,
  description | String,
  display_version | String,
  channel | String,
  platforms | Array String,
  deprecated | Bool,
  superseded_by | optional | Identity,
  content_locator | String,
} in
let RegistryContract = {
  schema = "lattice.operation-pack-registry.v1",
  generated_epoch_seconds | Number,
  entries | Array RegistryEntry,
} in
{
  Manifest = ManifestContract,
  Lock = LockContract,
  Registry = RegistryContract,
}
