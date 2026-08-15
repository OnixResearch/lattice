## Phase 1: Core decisions

- [ ] [serial] Implement the canonical key projection over node identity, operation-contract revision, normalized inputs, and optional capability context, backed by BLAKE3. r[lattice.node_result_cache.projection]
- [ ] [serial] Implement the pure retention decision core from supplied values. r[lattice.node_result_cache.decision] r[lattice.node_result_cache.deferral]
- [ ] [serial] Require an explicit capacity with no implicit default bound. r[lattice.node_result_cache.bound]

## Phase 2: Bounded node-result store

- [ ] [depends:lattice.node_result_cache.bound] Implement the bounded node-result store using core decisions. r[lattice.node_result_cache.bound]
- [ ] [depends:lattice.node_result_cache.deferral] Execute deferred releases strictly after the guard releases. r[lattice.node_result_cache.deferral]
- [ ] [serial] Route cache misses to the existing executor unchanged. r[lattice.node_result_cache.boundary]

## Phase 3: Fixtures and evidence

- [ ] [parallel] Add positive fixtures for projection stability, bounded retention, eviction order, promotion at and below threshold, and deferred-release ordering. r[lattice.node_result_cache.verification]
- [ ] [parallel] Add negative and boundary fixtures for missing bounds, reversed watermarks, zero capacity, single slot, threshold 0 and 100. r[lattice.node_result_cache.verification]
- [ ] [serial] Document the claim boundary and the recorded cross-repo retention contract. r[lattice.node_result_cache.boundary]
- [ ] [serial] Run workspace, Clippy, formatting, Cairn, and Nix checks, then record evidence. r[lattice.node_result_cache.verification]
