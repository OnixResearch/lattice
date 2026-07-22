let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 317,
  expected_missing_hash = "a4526fac7d7f14c594c1a900a74dd4fd118028c8855ce773b114587aa81bf3bd",
  reviewed = [
    {
      requirement_id = "n8nclone.workflow_examples.image.invalid",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "examples/workflows/invalid-unsupported-image-format.ncl", blake3 = "8cca0881b68a2f3e9b7700c5e3be16e52d907b1ecc0177902c1d33bdda66cb7a" },
        { path = "tests/test_image_workflow_example.rs", blake3 = "690286aaa1bf387d5ef733ff05ff242371e4778e4365871edd71cd9ce19078bd" },
      ],
      rationale_code = "assertions.unsupported-format-without-partial-completion",
    },
    {
      requirement_id = "n8nclone.workflow_examples.image.selfie_to_fine_art",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "examples/workflows/selfie-to-fine-art.ncl", blake3 = "2efd38cfd5d615233f307983ce81baf1e0c61563185526e7b4ad00880854dd0e" },
        { path = "tests/test_image_workflow_example.rs", blake3 = "690286aaa1bf387d5ef733ff05ff242371e4778e4365871edd71cd9ce19078bd" },
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
