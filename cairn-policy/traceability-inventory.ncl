let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 329,
  expected_missing_hash = "bc3aee4eda70b495ed76feaca0c3a981d97d385d2e733965cc3558282836ca36",
  reviewed = [
    {
      requirement_id = "n8nclone.credentials.webhook_auth.redaction",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/store/mod.rs", blake3 = "6f88954fc5b1028133524c5b49585e47f027e0aa3e2c93ad810dd11d0acf526f" },
        { path = "src/triggers.rs", blake3 = "644c9eed53394e4cc9ae0d7e4c21e8f5400ff833073ce790333c1c65273d8103" },
        { path = "src/webhooks/controls.rs", blake3 = "942f7286a689511ff37359bff729a8d21c2166235f7ab59a3f6d282d7ee7da76" },
        { path = "src/webhooks/flow.rs", blake3 = "a6c4b933b7c15c4e1adbb2d3a246b3d056bddb51f20aecd5faa0ce1a6ae36922" },
        { path = "tests/test_webhook_credential_boundaries.rs", blake3 = "373f5cb927029e468495609e6728c9d2f543ecc28a364d11820fc82a8e0243d9" },
      ],
      rationale_code = "assertions.fingerprinted-context-render-and-raw-store",
    },
    {
      requirement_id = "n8nclone.credentials.webhook_auth.valid",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/headless.rs", blake3 = "90f5deefe9ca359f845a4743b56ddc8b7ea36a3d9bd0a950193d88031782a34e" },
        { path = "src/store/mod.rs", blake3 = "6f88954fc5b1028133524c5b49585e47f027e0aa3e2c93ad810dd11d0acf526f" },
        { path = "src/triggers.rs", blake3 = "644c9eed53394e4cc9ae0d7e4c21e8f5400ff833073ce790333c1c65273d8103" },
        { path = "src/webhooks/flow.rs", blake3 = "a6c4b933b7c15c4e1adbb2d3a246b3d056bddb51f20aecd5faa0ce1a6ae36922" },
        { path = "tests/test_webhook_credential_boundaries.rs", blake3 = "373f5cb927029e468495609e6728c9d2f543ecc28a364d11820fc82a8e0243d9" },
      ],
      rationale_code = "assertions.valid-admission-history-and-raw-store",
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
