let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 327,
  expected_missing_hash = "1c8d9ebc7d34a24365f332e8d386cc1d905601d6541faaa254e3f6d2cd4b8aa4",
  reviewed = [
    {
      requirement_id = "n8nclone.node_templates.parameters.prompt",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/tui/app.rs", blake3 = "edc890f88bd0e8d5f76e66fc6ae2459b0330c94eb0116d0d85682a0a027c3984" },
      ],
      rationale_code = "assertions.parameter-value-collected",
    },
    {
      requirement_id = "n8nclone.node_templates.parameters.prompt.cancel",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/tui/app.rs", blake3 = "edc890f88bd0e8d5f76e66fc6ae2459b0330c94eb0116d0d85682a0a027c3984" },
      ],
      rationale_code = "assertions.cancel-preserves-workflow",
    },
    {
      requirement_id = "n8nclone.node_templates.parameters.prompt.required",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/tui/app.rs", blake3 = "edc890f88bd0e8d5f76e66fc6ae2459b0330c94eb0116d0d85682a0a027c3984" },
        { path = "src/tui/panels.rs", blake3 = "1a66d29221790dbd366e0f517f9b0cb119b2061765567c0a290b65e7f2e83c09" },
        { path = "tests/test_template_parameter_prompt_rendering.rs", blake3 = "b2305992b525b941818c5cf26e68f29327a916a68880a3d6be69cce2721d5039" },
      ],
      rationale_code = "assertions.required-prompt-rendered-before-creation",
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
