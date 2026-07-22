let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 304,
  expected_missing_hash = "dcff4383f0a161102646ca402e01deb4dd684e68e21821c4131638eaa3180fc7",
  reviewed = [
    {
      requirement_id = "lattice.agent_runtime.animus_compatibility.scenario.missing",
      classification = "partial_evidence_only",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "gap.workflow-and-orchestration-state-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_compatibility.scenario.routes",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "assertions.existing-route-bypasses-animus",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_effects.scenario.credential",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/animus_agent/shell.rs", blake3 = "39244cec69e3768b230760b27111577a908c2cdf2b770636053d9c1a27cdf693" },
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "gap.events-and-output-credential-omission-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_effects.scenario.provider-failure",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/animus_agent/shell.rs", blake3 = "39244cec69e3768b230760b27111577a908c2cdf2b770636053d9c1a27cdf693" },
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "gap.redacted-no-fallback-failure-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_effects.scenario.tool-denied",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell/tests.rs", blake3 = "78a2b50b3348ac0048edefa701fcce20bdf078ff33e41bca472a10ea804e0c7b" },
      ],
      rationale_code = "assertions.absent-tool-adapter-fails-without-model-effect",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_fabric.scenario.complete",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "assertions.fixed-loop-dispatches-once-and-returns",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_fabric.scenario.exhaustion",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell/tests.rs", blake3 = "78a2b50b3348ac0048edefa701fcce20bdf078ff33e41bca472a10ea804e0c7b" },
      ],
      rationale_code = "assertions.turn-ceiling-blocks-additional-dispatch",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_fabric.scenario.identity",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/animus_agent/core.rs", blake3 = "e0c21e66da438f45789ab0fa15c5215318d41a8ccd9611c64bfcea8bdd681f4f" },
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "gap.identity-rejection-before-transport-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_leaf.scenario.authority",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/animus_agent/core.rs", blake3 = "e0c21e66da438f45789ab0fa15c5215318d41a8ccd9611c64bfcea8bdd681f4f" },
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "gap.authority-bearing-output-inertness-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_leaf.scenario.execute",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "assertions.runner-executes-one-typed-animus-node",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_output.scenario.non-authoritative",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/animus_agent/shell.rs", blake3 = "39244cec69e3768b230760b27111577a908c2cdf2b770636053d9c1a27cdf693" },
        { path = "src/animus_agent/shell/tests.rs", blake3 = "78a2b50b3348ac0048edefa701fcce20bdf078ff33e41bca472a10ea804e0c7b" },
      ],
      rationale_code = "gap.incomplete-status-mapping-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_output.scenario.typed",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "assertions.text-json-values-and-safe-metadata",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_output.scenario.unsupported",
      classification = "partial_evidence_only",
      evidence = [
        { path = "src/animus_agent/core.rs", blake3 = "e0c21e66da438f45789ab0fa15c5215318d41a8ccd9611c64bfcea8bdd681f4f" },
        { path = "tests/test_animus_agent.rs", blake3 = "a9be7dc27a67ee547d9282b1d5039f77f54e899c62d2fd1648b14fd5af87adde" },
      ],
      rationale_code = "gap.unsupported-zero-dispatch-unasserted",
    },
    {
      requirement_id = "lattice.agent_runtime.inline_reasoning_boundary.scenario.plain",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/openai/execution.rs", blake3 = "04f2e2225e80b0887cc9a553e4c7c2f6c1b8884f9b9f51c41dede25c5146f866" },
      ],
      rationale_code = "assertions.plain-text-and-json-preserved",
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
