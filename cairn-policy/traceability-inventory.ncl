let Contracts = import "traceability-inventory-contracts.ncl" in
{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 367,
  expected_missing_hash = "4b62e23c7c961ef56cff234e25dd31ab1f0ecfc1fb4278537a981f930e48471d",
  reviewed = [
    {
      requirement_id = "lattice.workflow_execution.run_store_recovery.scenario.non-claim",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "tests/test_run_store_recovery.rs",
        blake3 = "ccd64b7cdc5da4346ced7d937413ec4e75b3f71e4fc33798ba36bd6d16b7c549",
      }],
      rationale_code = "assertions.fixed-recovery-output",
    },
    {
      requirement_id = "lattice.workflow_execution.safe_run_inspection.scenario.missing",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "tests/test_run_inspection.rs",
        blake3 = "f726fbd68d1a478dff3da62b65d72604afb1b533452c9191272367876bb09484",
      }],
      rationale_code = "assertions.missing-run",
    },
    {
      requirement_id = "lattice.workflow_execution.safe_run_inspection.scenario.not_recorded",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "tests/test_run_inspection.rs",
        blake3 = "f726fbd68d1a478dff3da62b65d72604afb1b533452c9191272367876bb09484",
      }],
      rationale_code = "assertions.not-recorded",
    },
    {
      requirement_id = "lattice.workflow_execution.safe_run_inspection.scenario.owner_binding",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "tests/test_run_inspection.rs",
        blake3 = "f726fbd68d1a478dff3da62b65d72604afb1b533452c9191272367876bb09484",
      }],
      rationale_code = "assertions.owner-binding",
    },
    {
      requirement_id = "lattice.workflow_execution.safe_run_inspection.scenario.replay",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "tests/test_run_inspection.rs",
        blake3 = "f726fbd68d1a478dff3da62b65d72604afb1b533452c9191272367876bb09484",
      }],
      rationale_code = "assertions.replay-validation",
    },
    {
      requirement_id = "lattice.workflow_execution.safe_run_inspection.scenario.safe_output",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "tests/test_run_inspection.rs",
        blake3 = "f726fbd68d1a478dff3da62b65d72604afb1b533452c9191272367876bb09484",
      }],
      rationale_code = "assertions.safe-output",
    },
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
