# Typed source for the non-default fake-backed application-host binding pilot.
# r[impl agent_harness_runtime.application_host_binding_pilot.kamacite_contract]
# r[impl agent_harness_runtime.application_host_binding_pilot.projection]
# r[impl agent_harness_runtime.application_host_binding_pilot.effects]
# r[impl agent_harness_runtime.application_host_binding_pilot.host_selection]
# r[impl agent_harness_runtime.application_host_binding_pilot.boundary]
let contracts = import "../../policy/contracts/primitives.ncl" in

let contract_schema = "kamacite.application-host-binding.v1" in
let required_non_claims = [
  "not effect completeness",
  "not application correctness",
  "not host capability or authorization",
  "not host behavior",
  "not sandbox admission",
  "not provider correctness",
  "not tool safety",
  "not runtime success",
  "not runtime correctness",
  "not resource enforcement",
  "not release eligibility",
] in
let identity_domains = [
  "target-artifact",
  "adapter",
  "algebraic-effect",
  "effect-operation",
  "evaluator",
  "evidence-chain",
] in
let Identity = {
  algorithm | contracts.exact "APPLICATION_BINDING_HASH_ALGORITHM" "blake3",
  domain | contracts.enum "APPLICATION_BINDING_IDENTITY_DOMAIN" identity_domains,
  hex | contracts.lowercase_blake3 "APPLICATION_BINDING_IDENTITY",
} in
let CohortContract = {
  binding_schema | contracts.exact "APPLICATION_BINDING_UPSTREAM_SCHEMA" contract_schema,
  receipt_schema | contracts.exact "APPLICATION_BINDING_UPSTREAM_RECEIPT_SCHEMA" "kamacite.application-host-binding-receipt.v1",
  producer_revision | contracts.exact "APPLICATION_BINDING_UPSTREAM_REVISION" "b21732a7458b660367f7281c717a5acc3f8c8190",
  schema_blake3 | contracts.exact "APPLICATION_BINDING_UPSTREAM_SCHEMA_HASH" "34fdebab3c8a274729f364af8da58691dba38809c67d2643c1f0d0354cb0c80b",
  receipt_schema_blake3 | contracts.exact "APPLICATION_BINDING_UPSTREAM_RECEIPT_SCHEMA_HASH" "3b48bf894cf40990a6d857542e9c3b26df2f6b50510999698323a7cb5cbca39e",
  validation_receipt_blake3 | contracts.exact "APPLICATION_BINDING_UPSTREAM_RECEIPT_HASH" "75ac779da7b5fde841188ba326e81c253ba5b78dbf0cc632bc90f8b58a9fb303",
  core_blake3 | contracts.exact "APPLICATION_BINDING_UPSTREAM_CORE_HASH" "4912fa0d0b81c5ac526d6fd762f4fe83d6d7dbba21cb661f3724bb499af5a1db",
} in
let Metadata = {
  aliases | Array String,
  display_name,
  documentation,
  path,
  source_span,
} in
let BindingShape = {
  kind | contracts.exact "APPLICATION_BINDING_KIND" "application-host-binding",
  schema_version | contracts.exact "APPLICATION_BINDING_SCHEMA" contract_schema,
  application_identity | Identity,
  entrypoint_identity | Identity,
  input_schema_identity | Identity,
  output_schema_identity | Identity,
  effect_summary_identity | Identity,
  effect_summary_closed | contracts.exact "APPLICATION_BINDING_EFFECT_CLOSURE" true,
  effect_operation_identities | Array Identity,
  host_profile_identity | Identity,
  alternative_host_profile_identities | Array Identity,
  fallback_host_profile_identity,
  adapter_cohort_identity | Identity,
  resource_profile_identity | Identity,
  policy_identity | Identity,
  compatibility_identity | Identity,
  expected_adapter_cohort_identity | Identity,
  expected_resource_profile_identity | Identity,
  expected_host_profile_identity | Identity,
  metadata | Metadata,
  semantic_claims | Array String,
  non_claims | Array String,
} in
let _binding_validator = fun value =>
  if value.application_identity.domain != "target-artifact"
    || value.entrypoint_identity.domain != "target-artifact"
  then 'Error { message = "APPLICATION_BINDING_WRONG_ARTIFACT_DOMAIN" }
  else if value.input_schema_identity.domain != "adapter"
    || value.output_schema_identity.domain != "adapter"
    || value.adapter_cohort_identity.domain != "adapter"
    || value.resource_profile_identity.domain != "adapter"
    || value.policy_identity.domain != "adapter"
  then 'Error { message = "APPLICATION_BINDING_WRONG_ADAPTER_DOMAIN" }
  else if value.effect_summary_identity.domain != "algebraic-effect"
    || !std.array.all
      (fun identity => identity.domain == "effect-operation")
      value.effect_operation_identities
  then 'Error { message = "APPLICATION_BINDING_WRONG_EFFECT_DOMAIN" }
  else if value.host_profile_identity.domain != "evaluator"
    || value.compatibility_identity.domain != "evidence-chain"
  then 'Error { message = "APPLICATION_BINDING_WRONG_HOST_DOMAIN" }
  else if std.array.length value.effect_operation_identities != 2
    || !contracts.unique_by (fun identity => identity.hex) value.effect_operation_identities
  then 'Error { message = "APPLICATION_BINDING_EFFECT_SET" }
  else if std.array.length value.alternative_host_profile_identities != 0
    || value.fallback_host_profile_identity != null
  then 'Error { message = "APPLICATION_BINDING_AMBIGUOUS_HOST" }
  else if value.host_profile_identity != value.expected_host_profile_identity
    || value.adapter_cohort_identity != value.expected_adapter_cohort_identity
    || value.resource_profile_identity != value.expected_resource_profile_identity
  then 'Error { message = "APPLICATION_BINDING_COHORT_MISMATCH" }
  else if std.array.length value.semantic_claims != 0
    || value.non_claims != required_non_claims
  then 'Error { message = "APPLICATION_BINDING_CLAIM_BOUNDARY" }
  else if value.metadata != {
    aliases = [],
    display_name = null,
    documentation = null,
    path = null,
    source_span = null,
  }
  then 'Error { message = "APPLICATION_BINDING_UNSAFE_METADATA" }
  else 'Ok
in
let BindingContract = contracts.with_validator BindingShape _binding_validator in
let id = fun domain_value hex_value => {
  algorithm = "blake3",
  domain = domain_value,
  hex = hex_value,
} in

{
  binding_contract = BindingContract,
  cohort_contract = CohortContract,
  kamacite_cohort | CohortContract = {
    binding_schema = contract_schema,
    receipt_schema = "kamacite.application-host-binding-receipt.v1",
    producer_revision = "b21732a7458b660367f7281c717a5acc3f8c8190",
    schema_blake3 = "34fdebab3c8a274729f364af8da58691dba38809c67d2643c1f0d0354cb0c80b",
    receipt_schema_blake3 = "3b48bf894cf40990a6d857542e9c3b26df2f6b50510999698323a7cb5cbca39e",
    validation_receipt_blake3 = "75ac779da7b5fde841188ba326e81c253ba5b78dbf0cc632bc90f8b58a9fb303",
    core_blake3 = "4912fa0d0b81c5ac526d6fd762f4fe83d6d7dbba21cb661f3724bb499af5a1db",
  },
  source_map = [
    { role = "application", source = "animus-runtime::dogfood_shell", disposition = "existing-identity" },
    { role = "entrypoint", source = "animus-dogfood-smoke", disposition = "existing-identity" },
    { role = "input-schema", source = "DogfoodRunRequest", disposition = "existing-schema-projection" },
    { role = "output-schema", source = "DogfoodReceipt", disposition = "existing-schema-projection" },
    { role = "effect-summary", source = "fake-backed-model-and-tool", disposition = "closed-symbolic-summary" },
    { role = "host-profile", source = "session-authority-profile", disposition = "existing-identity" },
    { role = "adapter-cohort", source = "session-authority-profile.adapter_cohort", disposition = "existing-identity" },
    { role = "resource-profile", source = "process-job-profile-policy", disposition = "existing-identity" },
    { role = "policy", source = "session-authority-profile.policy_authorization", disposition = "existing-identity" },
  ],
  binding | BindingContract = {
    kind = "application-host-binding",
    schema_version = "kamacite.application-host-binding.v1",
    application_identity = id "target-artifact" "5fd77e6bf7e3a618faf48a8a16c398126797a5245011f0d06dd40f45513b9cb1",
    entrypoint_identity = id "target-artifact" "bb661c376f2a83ceedd730d404de471dfcc5abd6f75f9f4a42a95b8afd9a0d42",
    input_schema_identity = id "adapter" "33df3bb18d797392d6421016a76ca9970321dc4d62dda749815a06547419db49",
    output_schema_identity = id "adapter" "f3f6e96b49daecc77fd98bd5434fe34ab23f39d226b506bdfd551ed773b2e81f",
    effect_summary_identity = id "algebraic-effect" "64471a04261cbb5f5d9e9b3df5ba9f2454145535355601760e779d2f6b0a46ac",
    effect_summary_closed = true,
    effect_operation_identities = [
      id "effect-operation" "7fa2709c205ccfa8c4b8773ab97396193a5130a2e42b2f94916802b04e07f5f8",
      id "effect-operation" "c598d03de53f54b01e8bc368840e6394e5cc3334b4e1be118f563ddfb8cfba2a",
    ],
    host_profile_identity = id "evaluator" "53a1955677292245ccd4ccc6c0c3a7f3bcd9431f4ac0d6f1a7f9fb7ca1ac5070",
    alternative_host_profile_identities = [],
    fallback_host_profile_identity = null,
    adapter_cohort_identity = id "adapter" "3333333333333333333333333333333333333333333333333333333333333333",
    resource_profile_identity = id "adapter" "876e8e3aedf280155e40378e5ba1add80ca30a7a0931f9a9c972b898694619f2",
    policy_identity = id "adapter" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    compatibility_identity = id "evidence-chain" "0ec64025422e00a28a2bfdb45f43b8fcd590ec1a4d3325bc07dae71d7ea0dbcb",
    expected_adapter_cohort_identity = id "adapter" "3333333333333333333333333333333333333333333333333333333333333333",
    expected_resource_profile_identity = id "adapter" "876e8e3aedf280155e40378e5ba1add80ca30a7a0931f9a9c972b898694619f2",
    expected_host_profile_identity = id "evaluator" "53a1955677292245ccd4ccc6c0c3a7f3bcd9431f4ac0d6f1a7f9fb7ca1ac5070",
    metadata = {
      aliases = [],
      display_name = null,
      documentation = null,
      path = null,
      source_span = null,
    },
    semantic_claims = [],
    non_claims = required_non_claims,
  },
}
