# Tasks: Type orchestration identities

## Phase 1: Inventory and foundation

- [x] [serial] I1 Inventory orchestration IDs, revisions, digests, idempotency keys, wire values, and display strings across core and persistence boundaries. r[lattice.agent_orchestration.nominal_identities.inventory]
- [x] [serial] V1 Capture baseline Nickel, JSON, redb fixture, event-chain, crash-recovery, and replay outputs. r[lattice.agent_orchestration.nominal_identities.compatibility]
- [x] [serial] I2 Add private role-specific ID, revision, idempotency, and BLAKE3 types with checked constructors and explicit accessors. r[lattice.agent_orchestration.nominal_identities.types]
- [x] [parallel] V2 Add positive and negative construction tests for empty, oversized, malformed, stale, and unsupported values. r[lattice.agent_orchestration.nominal_identities.validation]

## Phase 2: Core migration

- [x] [serial] I3 Migrate definition, instance, proposal, approval, fact, and trigger transition inputs to exact types. r[lattice.agent_orchestration.nominal_identities.transitions]
- [x] [serial] I4 Migrate dispatch, reservation, run ownership, recovery, cancellation, abandonment, and retry correlation. r[lattice.agent_orchestration.nominal_identities.recovery]
- [x] [serial] I5 Migrate event identity, ledger head, authority-policy identity, definition revision, workflow revision, and replay maps. r[lattice.agent_orchestration.nominal_identities.events]
- [x] [parallel] I6 Add explicit Nickel, CLI, scheduler, webhook, JSON, and redb wire adapters. r[lattice.agent_orchestration.nominal_identities.wire_boundary]

## Phase 3: Compatibility and failure evidence

- [x] [parallel] V3 Add compile-pass same-role fixtures and compile-fail cross-role fixtures for every maintained orchestration identity family. r[lattice.agent_orchestration.nominal_identities.compile_time]
- [x] [parallel] V4 Add wrong-owner, stale-revision, stale-approval, foreign-run, duplicate-trigger, conflicting-fact, and event-tamper tests. r[lattice.agent_orchestration.nominal_identities.failure_matrix]
- [x] [parallel] V5 Prove accepted Nickel, JSON, redb, inspection, and event-ledger representations remain compatible. r[lattice.agent_orchestration.nominal_identities.compatibility]
- [x] [parallel] V6 Rerun scheduler and webhook crash-boundary, orchestration recovery, replay, run inspection, and operator E2E checks. r[lattice.agent_orchestration.nominal_identities.recovery]

## Phase 4: Enforcement and closeout

- [x] [serial] I7 Add Lattice orchestration domains to the future Octet nominal-domain policy. r[lattice.agent_orchestration.nominal_identities.octet]
- [x] [serial] I8 Document typed identity ownership, wire adapters, persistence compatibility, and external-effect non-claims. r[lattice.agent_orchestration.nominal_identities.docs]
- [x] [serial] V7 Run focused tests, `scripts/check.rs`, Octet pre-commit checking, Cairn validation, change gates, and relevant Nix checks. r[lattice.agent_orchestration.nominal_identities.final_checks]

## Verification coverage

- `Scenario: Identity inventory excludes display text` -> I1
- `Scenario: Same-role orchestration identity is accepted` -> I2, V2, V3
- `Scenario: Cross-role orchestration identity does not compile` -> I2, V3
- `Scenario: Transitions require exact instance and proposal roles` -> I3, V4
- `Scenario: Recovery accepts only exact dispatch ownership` -> I4, V4, V6
- `Scenario: Event and ledger-head roles remain distinct` -> I5, V3, V4
- `Scenario: Boundary DTOs preserve external formats` -> I6, V5
- `Scenario: Existing persisted orchestration replays` -> V1, V5, V6
- `Scenario: Octet rejects raw-domain regression` -> I7, V7
- `Scenario: Typed local identities do not prove remote effects` -> I8, V7
