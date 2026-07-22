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
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.reference-only-cache-key",
    },
    {
      requirement_id = "n8nclone.credentials.provider_interface.missing",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.pre-runtime-missing-secret",
    },
    {
      requirement_id = "n8nclone.credentials.provider_interface.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.execution-boundary-resolution",
    },
    {
      requirement_id = "n8nclone.credentials.provider_registry.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.registered-provider-resolution",
    },
    {
      requirement_id = "n8nclone.credentials.provider_registry.unknown",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/workflow_validation.rs", blake3 = "49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.pre-runtime-unknown-provider",
    },
    {
      requirement_id = "n8nclone.credentials.providers.env.missing",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.pre-runtime-missing-env",
    },
    {
      requirement_id = "n8nclone.credentials.providers.env.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.environment-secret-resolution",
    },
    {
      requirement_id = "n8nclone.credentials.redaction.output",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.all-maintained-surfaces-redacted",
    },
    {
      requirement_id = "n8nclone.credentials.redaction.validation",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/workflow_validation.rs", blake3 = "49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.validation-text-json-safe",
    },
    {
      requirement_id = "n8nclone.credentials.secret_refs.model.reject_plaintext",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/workflow_format.rs", blake3 = "083eccc368e65dd7f7378b20b7bdd62d6bf3dc11f712d3b5a57c99d48db4a276" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.typed-rejection-before-store",
    },
    {
      requirement_id = "n8nclone.credentials.secret_refs.model.store_ref",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/workflow_format.rs", blake3 = "083eccc368e65dd7f7378b20b7bdd62d6bf3dc11f712d3b5a57c99d48db4a276" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
      ],
      rationale_code = "assertions.post-resolution-save-export-safe",
    },
    {
      requirement_id = "n8nclone.credentials.validation.malformed",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "src/workflow_validation.rs", blake3 = "49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374" },
        { path = "tests/test_credentials.rs", blake3 = "3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e" },
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
