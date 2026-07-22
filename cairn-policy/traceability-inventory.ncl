let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 336,
  expected_missing_hash = "9013bbe478784e6cec07f744410b35c27dad79cdf6c0451b496fbdc1a3e3ce7d",
  reviewed = [
    {
      requirement_id = "n8nclone.credentials.provider_cache_identity.no_plaintext",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.provider-key-field-and-value-independence",
    },
    {
      requirement_id = "n8nclone.credentials.provider_diagnostics.redaction.error",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "98fe0cfb08938d8b503c8852b8d4f4a770008cdbcf9da642ee963dfbde9f7fb1" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.fixed-category-safe-identity-render",
    },
    {
      requirement_id = "n8nclone.credentials.provider_diagnostics.redaction.test",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "98fe0cfb08938d8b503c8852b8d4f4a770008cdbcf9da642ee963dfbde9f7fb1" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.serialized-success-and-failure",
    },
    {
      requirement_id = "n8nclone.credentials.providers.command.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "98fe0cfb08938d8b503c8852b8d4f4a770008cdbcf9da642ee963dfbde9f7fb1" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.executor-before-side-effect-and-output-omission",
    },
    {
      requirement_id = "n8nclone.credentials.providers.command.timeout",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "98fe0cfb08938d8b503c8852b8d4f4a770008cdbcf9da642ee963dfbde9f7fb1" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.executor-timeout-blocks-side-effect",
    },
    {
      requirement_id = "n8nclone.credentials.providers.local_vault.resolve",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "98fe0cfb08938d8b503c8852b8d4f4a770008cdbcf9da642ee963dfbde9f7fb1" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.workflow-file-executor-boundary",
    },
    {
      requirement_id = "n8nclone.credentials.providers.local_vault.tamper",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/credentials.rs", blake3 = "98fe0cfb08938d8b503c8852b8d4f4a770008cdbcf9da642ee963dfbde9f7fb1" },
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_provider_credential_boundaries.rs", blake3 = "a388a72dca1eeb437fbe7b3d4576120bf9f681e7ac442b4549045bd1fc5aafe7" },
      ],
      rationale_code = "assertions.rendered-tamper-blocks-side-effect",
    },
    {
      requirement_id = "n8nclone.credentials.webhook_auth.redaction",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/triggers.rs", blake3 = "329343ff4685dbac1c6ace1872c22ad280d23958111d68d9a17f4fa06195e467" },
        { path = "src/webhooks/flow.rs", blake3 = "a6c4b933b7c15c4e1adbb2d3a246b3d056bddb51f20aecd5faa0ce1a6ae36922" },
        { path = "tests/test_triggers.rs", blake3 = "1c3a189b10748ebc1d0d5e6884ee96f9b5cb98107b65798a279752251898848a" },
        { path = "tests/webhooks.rs", blake3 = "8e918ea1c6b729049703077d7cadecd23acf4cee326ad48c9e0271017e5db424" },
      ],
      rationale_code = "assertions.safe-reference-context-missing",
    },
    {
      requirement_id = "n8nclone.credentials.webhook_auth.valid",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/triggers.rs", blake3 = "329343ff4685dbac1c6ace1872c22ad280d23958111d68d9a17f4fa06195e467" },
        { path = "src/webhooks/flow.rs", blake3 = "a6c4b933b7c15c4e1adbb2d3a246b3d056bddb51f20aecd5faa0ce1a6ae36922" },
        { path = "tests/test_triggers.rs", blake3 = "1c3a189b10748ebc1d0d5e6884ee96f9b5cb98107b65798a279752251898848a" },
        { path = "tests/webhooks.rs", blake3 = "8e918ea1c6b729049703077d7cadecd23acf4cee326ad48c9e0271017e5db424" },
      ],
      rationale_code = "assertions.persistence-observation-missing",
    },
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "partial evidence is not a near-complete correctness claim",
    "no exact evidence identified is bounded to the reviewed maintained roots and is not proof of global absence",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
