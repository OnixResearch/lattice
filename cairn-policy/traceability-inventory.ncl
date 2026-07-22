let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 324,
  expected_missing_hash = "f33a71ef4aa5bb871a48a6d8214500238153898df53779d7706dc4a56433a7ee",
  reviewed = [
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
