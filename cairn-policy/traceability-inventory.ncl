let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 340,
  expected_missing_hash = "e0ce72fe7f22dfe1fb007d33d93d6bc0b61b9efbc389339e9a0ed8656bc6f13b",
  reviewed = [
    {
      requirement_id = "n8nclone.credentials.n8n_json.placeholders.export",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.credential-export-observation-missing",
    },
    {
      requirement_id = "n8nclone.credentials.n8n_json.placeholders.plaintext",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/fixtures/n8n/plaintext-credentials.json", blake3 = "2cf81f633764559cf062cb15d188543947e875fff946a4da3050ac8c12cd8cc3" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.plaintext-save-boundary-missing",
    },
    {
      requirement_id = "n8nclone.credentials.n8n_json.placeholders.reference",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.imported-placeholder-resolution-missing",
    },
    {
      requirement_id = "n8nclone.credentials.n8n_parity_secret_safety",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/fixtures/n8n/parity/unsupported/plaintext-node-credential.json", blake3 = "343a47bab676d60d3df80b43c45b5f062c8b004bcf4792705017f8055017f8ff" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "contract.fixture-sentinel-and-output-gaps",
    },
    {
      requirement_id = "n8nclone.credentials.provider_cache_identity.no_plaintext",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.reference-dimensions-incomplete",
    },
    {
      requirement_id = "n8nclone.credentials.provider_diagnostics.redaction.error",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.error-fragment-source-missing",
    },
    {
      requirement_id = "n8nclone.credentials.provider_diagnostics.redaction.test",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.render-persist-observation-missing",
    },
    {
      requirement_id = "n8nclone.credentials.providers.command.resolve",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.executor-boundary-missing",
    },
    {
      requirement_id = "n8nclone.credentials.providers.command.timeout",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.no-runtime-observation-missing",
    },
    {
      requirement_id = "n8nclone.credentials.providers.local_vault.resolve",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.workflow-file-observation-missing",
    },
    {
      requirement_id = "n8nclone.credentials.providers.local_vault.tamper",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/credentials.rs", blake3 = "fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a" },
        { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" },
      ],
      rationale_code = "assertions.rendered-tamper-diagnostic-missing",
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
