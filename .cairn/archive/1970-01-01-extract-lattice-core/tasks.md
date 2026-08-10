## Phase 1: Boundary

- [x] [serial] Add `lattice-core` with admitted deterministic dependencies. r[lattice.architecture.core_crate]
- [x] [serial] Move value, expression, control-flow, and content-identity decisions into the core crate. r[lattice.architecture.core_crate.pure]
- [x] [serial] Preserve existing root module paths through compatibility re-exports. r[lattice.architecture.core_crate.compatibility]

## Phase 2: Enforcement

- [x] [parallel] Add positive and negative manifest dependency fixtures. r[lattice.architecture.core_crate.dependencies]
- [x] [parallel] Add `crates/` to direct traceability discovery and refresh bound policy evidence. r[lattice.architecture.core_crate.traceability]

## Phase 3: Validation

- [x] [parallel] Repeat focused baseline tests through the compatibility facade. r[lattice.architecture.core_crate.compatibility]
- [x] [parallel] Run core, repository, Octet, Cairn, Clippy, and Nix checks. r[lattice.architecture.core_crate.validation]
- [x] [serial] Sync the accepted requirement, record bounded evidence, and archive the change. r[lattice.architecture.core_crate]
