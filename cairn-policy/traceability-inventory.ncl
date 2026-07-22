let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 304,
  expected_missing_hash = "dcff4383f0a161102646ca402e01deb4dd684e68e21821c4131638eaa3180fc7",
  reviewed = [
    {
      requirement_id = "lattice.agent_runtime.animus_compatibility.scenario.missing",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.missing-route-request-unchanged-zero-dispatch",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_compatibility.scenario.routes",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.existing-route-bypasses-animus",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_effects.scenario.credential",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell.rs", blake3 = "39244cec69e3768b230760b27111577a908c2cdf2b770636053d9c1a27cdf693" },
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.transport-only-credential-omission",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_effects.scenario.provider-failure",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell.rs", blake3 = "39244cec69e3768b230760b27111577a908c2cdf2b770636053d9c1a27cdf693" },
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.redacted-single-provider-failure",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_effects.scenario.tool-denied",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell/tests.rs", blake3 = "b76e5e38ec26d2938e8f75b2df666062d6ea0227bd2562e63b6ed3485639f299" },
      ],
      rationale_code = "assertions.absent-tool-adapter-fails-without-model-effect",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_fabric.scenario.complete",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.fixed-loop-dispatches-once-and-returns",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_fabric.scenario.exhaustion",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell/tests.rs", blake3 = "b76e5e38ec26d2938e8f75b2df666062d6ea0227bd2562e63b6ed3485639f299" },
      ],
      rationale_code = "assertions.turn-ceiling-blocks-additional-dispatch",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_fabric.scenario.identity",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/core.rs", blake3 = "818987c343e82b1543f1c5099563316ca4d9e368a51cfd56f3b586bb837f5bb8" },
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.identity-admission-rejects-before-shell",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_leaf.scenario.authority",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/core.rs", blake3 = "818987c343e82b1543f1c5099563316ca4d9e368a51cfd56f3b586bb837f5bb8" },
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.authority-content-remains-ordinary-output",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_leaf.scenario.execute",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.runner-executes-one-typed-animus-node",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_output.scenario.non-authoritative",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/shell.rs", blake3 = "39244cec69e3768b230760b27111577a908c2cdf2b770636053d9c1a27cdf693" },
        { path = "src/animus_agent/shell/tests.rs", blake3 = "b76e5e38ec26d2938e8f75b2df666062d6ea0227bd2562e63b6ed3485639f299" },
      ],
      rationale_code = "assertions.all-nonfinal-statuses-map-to-errors",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_output.scenario.typed",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.text-json-values-and-safe-metadata",
    },
    {
      requirement_id = "lattice.agent_runtime.animus_output.scenario.unsupported",
      classification = "exact_maintained_evidence",
      evidence = [
        { path = "src/animus_agent/core.rs", blake3 = "818987c343e82b1543f1c5099563316ca4d9e368a51cfd56f3b586bb837f5bb8" },
        { path = "tests/test_animus_agent.rs", blake3 = "58ffe36017a6b8fd27f357f338c3d2f2f8d861afeb9668a2a9f3f910a1b4eafd" },
      ],
      rationale_code = "assertions.unsupported-preflight-zero-dispatch",
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
