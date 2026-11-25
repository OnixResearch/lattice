## Phase 1: Producer and consumer integration

- [x] [serial] Admit typed Cairn release selection and exact policy propagation while preserving producer self-host behavior. r[lattice.release_integration.selection]
- [x] [serial] Implement actual native MCP validation observation inputs and maintained canonicalization vectors with failure/stale regressions. r[lattice.release_integration.inputs]
- [x] [serial] Export Lattice-owned release and replay selection and admit an exact public producer cohort (`flake.nix`, `cairn-policy/default.ncl`, `tests/test_release_readiness_integration.rs`, `evidence/closeout-verification.md`). r[lattice.release_integration.selection]

## Phase 2: Final verification

- [x] [serial] Generate real MCP inputs after all source/policy edits, execute readiness and required replay, and retain truthful optional/skipped and gate results (`evidence/closeout-verification.md`). r[lattice.release_integration.inputs]
