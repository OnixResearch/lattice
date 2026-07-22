let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 340,
  expected_missing_hash = "e0ce72fe7f22dfe1fb007d33d93d6bc0b61b9efbc389339e9a0ed8656bc6f13b",
  reviewed = [
    {
      requirement_id = "n8nclone.credentials.n8n_json.placeholders.export",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/main.rs", blake3 = "e676a71694763d30b16182759eec2cee5eb4fb4dbaa9fd4bee2ba85c16173d5e" },
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/test_n8n_credential_boundaries.rs", blake3 = "d029eec7b4060e71848cb485f158f3275bcdfeb2fa3fb0995976b01568cb715d" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.execution-save-raw-store-cli-export",
    },
    {
      requirement_id = "n8nclone.credentials.n8n_json.placeholders.plaintext",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/main.rs", blake3 = "e676a71694763d30b16182759eec2cee5eb4fb4dbaa9fd4bee2ba85c16173d5e" },
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/fixtures/n8n/parity/unsupported/plaintext-node-credential.json", blake3 = "343a47bab676d60d3df80b43c45b5f062c8b004bcf4792705017f8055017f8ff" },
        { path = "tests/test_n8n_credential_boundaries.rs", blake3 = "d029eec7b4060e71848cb485f158f3275bcdfeb2fa3fb0995976b01568cb715d" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.cli-rejection-precedes-store",
    },
    {
      requirement_id = "n8nclone.credentials.n8n_json.placeholders.reference",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/engine/executor.rs", blake3 = "d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664" },
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/test_n8n_credential_boundaries.rs", blake3 = "d029eec7b4060e71848cb485f158f3275bcdfeb2fa3fb0995976b01568cb715d" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.imported-provider-execution-boundary",
    },
    {
      requirement_id = "n8nclone.credentials.n8n_parity_secret_safety",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/main.rs", blake3 = "e676a71694763d30b16182759eec2cee5eb4fb4dbaa9fd4bee2ba85c16173d5e" },
        { path = "src/n8n_json.rs", blake3 = "668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3" },
        { path = "tests/fixtures/n8n/parity/supported/command-pipeline.json", blake3 = "fa36b003d21abbd60d2164f45c793a08d920085a613fa660fdc7f5d52eefb4cb" },
        { path = "tests/fixtures/n8n/parity/supported/command-pipeline.report.json", blake3 = "65c19b05a3a473a7beafc4936815095a48b37acee066c649602c3b9bd3b5af28" },
        { path = "tests/fixtures/n8n/parity/supported/default-http-method.json", blake3 = "dbc05636f34aef3fb8e6c89110e5e575a1e14d5753bba62dc744b6eb7cceb9bd" },
        { path = "tests/fixtures/n8n/parity/supported/http-credential-chain.json", blake3 = "98b673c47142822d6929559240c31c9c234a370c6768a029d0271c31a82dde5b" },
        { path = "tests/fixtures/n8n/parity/supported/http-credential-chain.report.json", blake3 = "d8130d062520f2c9772a2754be4b628efe1944e88d7fc31254a3cec633aa0c74" },
        { path = "tests/fixtures/n8n/parity/unsupported/expression-url.json", blake3 = "1c3d9326248b3b47e14e42bf921248864fed01be737860acfecce27c07b43fdd" },
        { path = "tests/fixtures/n8n/parity/unsupported/malformed-connection.json", blake3 = "0bc228254029f0d711ae65667289f585920ff8e75e07ba022b599bc68e37ee04" },
        { path = "tests/fixtures/n8n/parity/unsupported/plaintext-node-credential.json", blake3 = "343a47bab676d60d3df80b43c45b5f062c8b004bcf4792705017f8055017f8ff" },
        { path = "tests/fixtures/n8n/parity/unsupported/schedule-trigger.json", blake3 = "fb55a34ab4b89db4434caab084bd035d2695dc992e364a98b0b3f1cd787f8a02" },
        { path = "tests/fixtures/n8n/parity/unsupported/webhook-trigger.json", blake3 = "4f6b45c79fdb49201c2a8277aeeecf77d45662ff67772c469043c18ef2b05992" },
        { path = "tests/fixtures/n8n/parity/unsupported/webhook-trigger.report.json", blake3 = "fa05397b0203671fcd86b8d38149e2339bd58794efc1b12e87f7e6162090f232" },
        { path = "tests/test_n8n_credential_boundaries.rs", blake3 = "d029eec7b4060e71848cb485f158f3275bcdfeb2fa3fb0995976b01568cb715d" },
        { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" },
      ],
      rationale_code = "assertions.bounded-negative-input-output-omission",
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
