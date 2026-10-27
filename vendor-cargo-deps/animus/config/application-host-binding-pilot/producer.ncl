# Exact producer envelope for Kamacite cross-repository fixture admission.
# r[impl agent_harness_runtime.application_host_binding_pilot.kamacite_contract]
let contracts = import "../../policy/contracts/primitives.ncl" in
let profile = import "profile.ncl" in
let producer_revision_value = "fdd4eff7180c2134a06b99c2bc78335313688e6e" in
let source_identity_value = "75e686e149d9624097ee633562c6ae1529d865c51c7762f810f8d4df61f33bc7" in
let producer_non_claims = [
  "producer fixture does not prove producer correctness",
  "producer fixture does not grant host capability or authorization",
  "producer fixture does not prove runtime success or release eligibility",
] in
let SourceEntry = {
  path | String,
  blake3 | contracts.lowercase_blake3 "APPLICATION_BINDING_PRODUCER_SOURCE_HASH",
} in
let ProducerContract = {
  schema | contracts.exact "APPLICATION_BINDING_PRODUCER_SCHEMA" "animus.application-host-binding-producer.v1",
  producer | contracts.exact "APPLICATION_BINDING_PRODUCER_NAME" "animus",
  producer_revision | contracts.exact "APPLICATION_BINDING_PRODUCER_REVISION" producer_revision_value,
  source_identity | contracts.exact "APPLICATION_BINDING_PRODUCER_SOURCE_IDENTITY" source_identity_value,
  source_identity_algorithm | contracts.exact "APPLICATION_BINDING_PRODUCER_SOURCE_ALGORITHM" "blake3-of-ordered-b3sum-lines-v1",
  source_manifest | Array SourceEntry,
  kamacite_revision | contracts.exact "APPLICATION_BINDING_PRODUCER_KAMACITE_REVISION" "b21732a7458b660367f7281c717a5acc3f8c8190",
  binding | profile.binding_contract,
  non_claims | Array String,
} in
let _producer_validator = fun value =>
  if std.array.length value.source_manifest != 5 then
    'Error { message = "APPLICATION_BINDING_PRODUCER_SOURCE_COUNT" }
  else if !contracts.unique_by (fun entry => entry.path) value.source_manifest then
    'Error { message = "APPLICATION_BINDING_PRODUCER_DUPLICATE_SOURCE" }
  else if value.non_claims != producer_non_claims then
    'Error { message = "APPLICATION_BINDING_PRODUCER_NON_CLAIMS" }
  else
    'Ok
in
let CheckedProducerContract = contracts.with_validator ProducerContract _producer_validator in
{
  producer_contract = CheckedProducerContract,
  producer | CheckedProducerContract = {
    schema = "animus.application-host-binding-producer.v1",
    producer = "animus",
    producer_revision = "fdd4eff7180c2134a06b99c2bc78335313688e6e",
    source_identity = "75e686e149d9624097ee633562c6ae1529d865c51c7762f810f8d4df61f33bc7",
    source_identity_algorithm = "blake3-of-ordered-b3sum-lines-v1",
    source_manifest = [
      { path = "crates/animus-runtime/src/applicationbinding/core.rs", blake3 = "a0c9bd74591b42bfada9a3c1e1a63e436f2f355f62f13d5134de2206323ba49a" },
      { path = "crates/animus-runtime/src/applicationbinding/data.rs", blake3 = "437bf8d31fabccca54b503b1939cc9e56486ac097f93cdfa94edf216cadd5027" },
      { path = "crates/animus-runtime/src/applicationbinding/mod.rs", blake3 = "cb9aa1cc0f3d53f6ac16f770a5ea3a27e8f7d4351811f79b619a406b3c11cf4f" },
      { path = "config/application-host-binding-pilot/profile.ncl", blake3 = "339baed1d5854d3840c3b26415a02a1cca33e46833a1fb4a3115d9629611172e" },
      { path = "config/application-host-binding-pilot/generated/animus-binding.json", blake3 = "52ca72193a1f2e9668e0ceae3c85c16b34817dcab3d0b90532115e78d52d8e35" },
    ],
    kamacite_revision = "b21732a7458b660367f7281c717a5acc3f8c8190",
    binding = profile.binding,
    non_claims = producer_non_claims,
  },
}
