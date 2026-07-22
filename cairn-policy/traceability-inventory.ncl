let Contracts = import "traceability-inventory-contracts.ncl" in
{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 353,
  expected_missing_hash = "0e205e16faa2c958a0ba9f90bb64bbc8d09c81c16f8dd93f9798fd958a4835e1",
  reviewed = [
    {
      requirement_id = "n8nclone.credentials.cache_identity.reference",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.reference-only-cache-key",
    },
    {
      requirement_id = "n8nclone.credentials.provider_interface.missing",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.pre-runtime-missing-secret",
    },
    {
      requirement_id = "n8nclone.credentials.provider_interface.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.execution-boundary-resolution",
    },
    {
      requirement_id = "n8nclone.credentials.provider_registry.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.registered-provider-resolution",
    },
    {
      requirement_id = "n8nclone.credentials.provider_registry.unknown",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/workflow_validation.rs", blake3 = "49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.pre-runtime-unknown-provider",
    },
    {
      requirement_id = "n8nclone.credentials.providers.env.missing",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.pre-runtime-missing-env",
    },
    {
      requirement_id = "n8nclone.credentials.providers.env.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.environment-secret-resolution",
    },
    {
      requirement_id = "n8nclone.credentials.redaction.output",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "partial.output-surfaces-incomplete",
    },
    {
      requirement_id = "n8nclone.credentials.redaction.validation",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/workflow_validation.rs", blake3 = "49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374" },
      ],
      rationale_code = "partial.validation-rendering-unasserted",
    },
    {
      requirement_id = "n8nclone.credentials.secret_refs.model.reject_plaintext",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/workflow_format.rs", blake3 = "d0a33436c867c123db1057560622a5fd9718e97430a785dd13e2db8c3cfc66ac" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "partial.import-diagnostic-no-save-unasserted",
    },
    {
      requirement_id = "n8nclone.credentials.secret_refs.model.store_ref",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/workflow_format.rs", blake3 = "d0a33436c867c123db1057560622a5fd9718e97430a785dd13e2db8c3cfc66ac" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "partial.serialization-not-save-export",
    },
    {
      requirement_id = "n8nclone.credentials.validation.malformed",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0" },
        { path = "src/workflow_validation.rs", blake3 = "49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374" },
        { path = "tests/test_credentials.rs", blake3 = "2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1" },
      ],
      rationale_code = "assertions.fatal-malformed-reference",
    },
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
