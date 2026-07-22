let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 324,
  expected_missing_hash = "f33a71ef4aa5bb871a48a6d8214500238153898df53779d7706dc4a56433a7ee",
  reviewed = [
    {
      requirement_id = "n8nclone.scheduling.triggers.admission.disabled",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/webhooks.rs", blake3 = "696ac9d227f8019ab527743752f6ac186fdd7c445aaf48dd5a7bcfb35e81e95d" },
      ],
      rationale_code = "assertions.disabled-rejected-without-run",
    },
    {
      requirement_id = "n8nclone.scheduling.triggers.history.persist",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/headless.rs", blake3 = "90f5deefe9ca359f845a4743b56ddc8b7ea36a3d9bd0a950193d88031782a34e" },
        { path = "src/triggers.rs", blake3 = "644c9eed53394e4cc9ae0d7e4c21e8f5400ff833073ce790333c1c65273d8103" },
        { path = "tests/test_triggers.rs", blake3 = "70b4de440c9d2da09ab03148553b6b3e029f8bdd21e9c9d6273b4e3fc1ff2c54" },
      ],
      rationale_code = "assertions.terminal-history-complete-and-safe",
    },
    {
      requirement_id = "n8nclone.scheduling.triggers.scheduler_compat.due",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_triggers.rs", blake3 = "70b4de440c9d2da09ab03148553b6b3e029f8bdd21e9c9d6273b4e3fc1ff2c54" },
      ],
      rationale_code = "assertions.full-legacy-adapted-plan-equality",
    },
    {
      requirement_id = "n8nclone.scheduling.triggers.webhook.invalid",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_triggers.rs", blake3 = "70b4de440c9d2da09ab03148553b6b3e029f8bdd21e9c9d6273b4e3fc1ff2c54" },
        { path = "tests/webhooks.rs", blake3 = "696ac9d227f8019ab527743752f6ac186fdd7c445aaf48dd5a7bcfb35e81e95d" },
      ],
      rationale_code = "assertions.invalid-webhooks-reject-without-run",
    },
    {
      requirement_id = "n8nclone.scheduling.triggers.webhook.valid",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_triggers.rs", blake3 = "70b4de440c9d2da09ab03148553b6b3e029f8bdd21e9c9d6273b4e3fc1ff2c54" },
        { path = "tests/webhooks.rs", blake3 = "696ac9d227f8019ab527743752f6ac186fdd7c445aaf48dd5a7bcfb35e81e95d" },
      ],
      rationale_code = "assertions.valid-webhook-binds-and-runs",
    },
    {
      requirement_id = "n8nclone.scheduling.webhook_cookbook_operator_flow",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "examples/README.md", blake3 = "9ba2964ecb63eb81823898b1243d0c391e9b4772533ee0958b98cfdeb91e91bb" },
        { path = "examples/workflows/webhook-cookbook.ncl", blake3 = "e449f943f0af5f6b2f49ce4dca45d9279ca2897410d63b57e3163bff86d03e95" },
        { path = "tests/webhooks.rs", blake3 = "696ac9d227f8019ab527743752f6ac186fdd7c445aaf48dd5a7bcfb35e81e95d" },
      ],
      rationale_code = "assertions.cookbook-import-dry-run-admit-reject-history",
    },
    {
      requirement_id = "n8nclone.scheduling.webhook_listener.process.start",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/main.rs", blake3 = "e676a71694763d30b16182759eec2cee5eb4fb4dbaa9fd4bee2ba85c16173d5e" },
        { path = "tests/webhooks.rs", blake3 = "696ac9d227f8019ab527743752f6ac186fdd7c445aaf48dd5a7bcfb35e81e95d" },
      ],
      rationale_code = "assertions.live-listener-process-accepts-and-persists",
    },
    {
      requirement_id = "n8nclone.workflow_examples.image.invalid",
      classification = "no_exact_evidence_identified",
      evidence = [],
      rationale_code = "fixture.absent-from-maintained-roots",
    },
    {
      requirement_id = "n8nclone.workflow_examples.image.selfie_to_fine_art",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "examples/workflows/selfie-to-fine-art.ncl", blake3 = "2efd38cfd5d615233f307983ce81baf1e0c61563185526e7b4ad00880854dd0e" },
        { path = "tests/test_image_workflow_example.rs", blake3 = "657bb2f91f7f1a2471d8bd7d9556c1cb87fd41916ea255ce645e757227f099c0" },
      ],
      rationale_code = "assertions.import-execute-and-image-output",
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
