## Phase 1: Migration

- [x] [serial] Rebase the Lattice policy on the pinned Cairn policy and retain a narrow project overlay. r[lattice.repository_validation.native_lifecycle]
- [x] [serial] Preview and execute the hash-bound lifecycle migration. r[lattice.repository_validation.native_lifecycle.scenario.parity]
- [x] [serial] Update active repository references and remove the legacy lifecycle tree. r[lattice.repository_validation.native_lifecycle.scenario.active_paths]

## Phase 2: Verification

- [x] [parallel] Verify deterministic policy export, unsafe-root rejection, and legacy-only lifecycle rejection. r[lattice.repository_validation.native_lifecycle.scenario.legacy_rejected]
- [x] [parallel] Run focused repository, traceability, Octet, Cairn, and Nix checks. r[lattice.repository_validation.native_lifecycle.scenario.active_paths]
- [x] [serial] Sync the accepted requirement, record evidence, and archive the change. r[lattice.repository_validation.native_lifecycle]
