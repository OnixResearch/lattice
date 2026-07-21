let Contracts = import "traceability-inventory-contracts.ncl" in
{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 361,
  expected_missing_hash = "df059bfa0b70e75996de6dc41990d0d744197b90bee9195bf1b873c505abaf3c",
  reviewed = [],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
