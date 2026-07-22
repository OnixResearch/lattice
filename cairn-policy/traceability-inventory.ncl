let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 329,
  expected_missing_hash = "bc3aee4eda70b495ed76feaca0c3a981d97d385d2e733965cc3558282836ca36",
  reviewed = [
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
