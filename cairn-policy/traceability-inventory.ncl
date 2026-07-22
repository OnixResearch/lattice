let Contracts = import "traceability-inventory-contracts.ncl" in
{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 361,
  expected_missing_hash = "df059bfa0b70e75996de6dc41990d0d744197b90bee9195bf1b873c505abaf3c",
  reviewed = [
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_core",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "src/main.rs",
        blake3 = "e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d",
      }],
      rationale_code = "source.pure-deterministic-evaluate",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_core.fatal",
      classification = "partial_evidence_only",
      evidence = [
        {
          path = "src/main.rs",
          blake3 = "e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d",
        },
        {
          path = "tests/test_doctor.rs",
          blake3 = "12b75ea05dae3a639f4f024dce632ee853a345e2ddc9c13753ccdde41eb68086",
        },
      ],
      rationale_code = "assertions.enumerated-failures-incomplete",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_core.healthy",
      classification = "partial_evidence_only",
      evidence = [
        {
          path = "src/main.rs",
          blake3 = "e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d",
        },
        {
          path = "tests/test_doctor.rs",
          blake3 = "12b75ea05dae3a639f4f024dce632ee853a345e2ddc9c13753ccdde41eb68086",
        },
      ],
      rationale_code = "assertions.healthy-findings-indirect",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_output",
      classification = "exact_maintained_evidence",
      evidence = [{
        path = "src/main.rs",
        blake3 = "e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d",
      }],
      rationale_code = "source.deterministic-dual-renderers",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_output.json",
      classification = "partial_evidence_only",
      evidence = [
        {
          path = "src/main.rs",
          blake3 = "e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d",
        },
        {
          path = "tests/test_doctor.rs",
          blake3 = "12b75ea05dae3a639f4f024dce632ee853a345e2ddc9c13753ccdde41eb68086",
        },
      ],
      rationale_code = "assertions.mixed-json-output-missing",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_output.text",
      classification = "partial_evidence_only",
      evidence = [
        {
          path = "src/main.rs",
          blake3 = "e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d",
        },
        {
          path = "tests/test_doctor.rs",
          blake3 = "12b75ea05dae3a639f4f024dce632ee853a345e2ddc9c13753ccdde41eb68086",
        },
      ],
      rationale_code = "assertions.mixed-text-output-missing",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_redaction",
      classification = "no_exact_evidence_identified",
      evidence = [],
      rationale_code = "behavior.redaction-inputs-unwired",
    },
    {
      requirement_id = "n8nclone.operator_diagnostics.doctor_redaction.json",
      classification = "no_exact_evidence_identified",
      evidence = [],
      rationale_code = "assertions.json-redaction-missing",
    },
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
