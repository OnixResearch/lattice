## Phase 1: Core decisions

- [x] [serial] Implement the canonical key projection over policy-source identity, normalized inputs, and capability-context, backed by BLAKE3. r[contract_backends.policy_cache.projection]
- [x] [serial] Implement the pure retention decision core: eviction target, promotion disposition, watermark trim plans, and deferred-release plans from supplied values. r[contract_backends.policy_cache.decision] r[contract_backends.policy_cache.deferral]
- [x] [serial] Require an explicit capacity with no implicit default bound. r[contract_backends.policy_cache.bound]

## Phase 2: Standard store

- [x] [depends:contract_backends.policy_cache.bound] Implement the bounded store that relies on core decisions. r[contract_backends.policy_cache.bound]
- [x] [depends:contract_backends.policy_cache.deferral] Execute deferred releases strictly after the guard releases, with no user destructor inside the synchronization boundary. r[contract_backends.policy_cache.deferral]
- [x] [serial] Keep revocation and policy evaluation freshness consumer-owned. Route cache misses to the existing evaluators unchanged. r[contract_backends.policy_cache.boundary]

## Phase 3: Fixtures and evidence

- [x] [parallel] Add positive fixtures for projection stability, bounded retention, eviction order, promotion at and below threshold, and deferred-release ordering. r[contract_backends.policy_cache.verification]
- [x] [parallel] Add negative and boundary fixtures for missing bounds, reversed watermarks, zero capacity, single slot, threshold 0 and 100, and decision invariance on hit. r[contract_backends.policy_cache.verification]
- [x] [serial] Document the claim boundary and the recorded cross-repo retention contract in repo docs. r[contract_backends.policy_cache.boundary]
- [x] [serial] Run policy freshness, workspace, Clippy, formatting, Cairn, and Nix checks, then record evidence. r[contract_backends.policy_cache.verification]
