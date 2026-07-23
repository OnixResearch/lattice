let Contracts = import "traceability-inventory-contracts.ncl" in

{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 181,
  expected_missing_hash = "87f32950fa7ce692a6a5c98f02d174e050d8cce18d05e785821c84795e81dba0",
  reviewed = [],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "partial evidence is not a near-complete correctness claim",
    "no exact evidence identified is bounded to the reviewed maintained roots and is not proof of global absence",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
