## Phase 1: Inventory contract

- [x] [serial] Add the typed review policy, missing-set identity, and explicit uncertainty states. r[lattice.repository_validation.traceability_debt_inventory]
- [x] [serial] Add a malformed policy fixture that rejects unsafe or contradictory reviewed evidence. r[lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]

## Phase 2: Deterministic tool

- [x] [serial] Implement the pure inventory core with sorted rows, subsystem summaries, invariant diagnostics, and domain-separated BLAKE3 identities. r[lattice.repository_validation.traceability_debt_inventory.scenario.complete]
- [x] [serial] Implement the thin cargo-script shell for Nickel export and bounded generate/check operations. r[lattice.repository_validation.traceability_debt_inventory]
- [x] [serial] Add positive and negative maintained tests for completeness, duplicates, stale decisions, unsafe paths, tampering, and uncertainty preservation. r[lattice.repository_validation.traceability_debt_inventory.scenario.complete] r[lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed] r[lattice.repository_validation.traceability_debt_inventory.scenario.non_claim]

## Phase 3: Audit and evidence

- [x] [serial] Record the six reviewed safe-inspection/recovery candidates without adding their missing Tracey markers. r[lattice.repository_validation.traceability_debt_inventory.scenario.non_claim]
- [x] [serial] Prove the ordered 367-item missing array and empty dangling array are unchanged and coverage remains unsuccessful. r[lattice.repository_validation.traceability_debt_inventory.scenario.complete]
- [x] [serial] Run Nickel, focused, repository, Cairn, and Nix validation; sync and archive the completed change. r[lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]
