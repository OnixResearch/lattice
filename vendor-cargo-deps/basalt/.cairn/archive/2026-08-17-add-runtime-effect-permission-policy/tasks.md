## Phase 1: Policy and pure decision core

- [x] [serial] Pin the reviewed agentOS permission source revision, record its license and selected concepts, and add the bounded reference to `README.md`. r[contract_backends.runtime_effect_permissions.boundary]
- [x] [serial] Define typed Nickel contracts for effect scopes, operations, normalized subjects, match modes, priorities, defaults, profiles, decisions, and non-claims. r[contract_backends.runtime_effect_permissions.policy]
- [x] [depends:contract_backends.runtime_effect_permissions.policy] Export checked runtime policy JSON and add typed Rust DTO parity checks. r[contract_backends.runtime_effect_permissions.policy]
- [x] [depends:contract_backends.runtime_effect_permissions.policy] Implement pure deterministic matching, ambiguity denial, contract composition, and optional verified-UCAN input composition. r[contract_backends.runtime_effect_permissions.decision]

## Phase 2: Evidence and consumers

- [x] [parallel] Add positive exact and prefix match fixtures for filesystem, network, process, environment, and binding scopes. r[contract_backends.runtime_effect_permissions.verification]
- [x] [parallel] Add negative fixtures for unknown values, malformed subjects, ambiguous conflicts, stale profiles, missing authority, lossy mapping, and enforcement overclaims. r[contract_backends.runtime_effect_permissions.verification]
- [x] [serial] Add metadata-only BLAKE3 receipts and frozen Animus and Molten mapping fixtures without runtime dependencies. r[contract_backends.runtime_effect_permissions.evidence]
- [x] [serial] Run policy freshness, focused tests, downstream compatibility, Cairn validation and gates, and release verification. r[contract_backends.runtime_effect_permissions.verification]
