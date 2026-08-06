## Phase 1: Baseline and boundary inventory

- [x] [serial] Run focused orchestration budget, transition, replay, serialization, inspection, retry, and abandonment tests before core changes and record current types, vectors, callers, persisted fields, event hashes, and rollback. r[lattice.budget_ledger_incubation.compatibility]
- [x] [serial] Classify pure ledger decisions versus Lattice shell policy for planned charges, observed usage, acknowledged unknown-outcome charging, actors, persistence, events, CLI, and evidence. r[lattice.budget_ledger_incubation.boundary]
- [x] [parallel] Record why Seaglass direct memory/row/byte/concurrency limits do not yet satisfy durable reservation and settlement promotion criteria. r[lattice.budget_ledger_incubation.promotion]

## Phase 2: Local functional core

- [x] [serial] Extract current named dimensions, policy, limits, plans, charges, reservations, consumed totals, breach state, checked arithmetic, reserve, settle, and remaining-balance decisions into a local testable core. r[lattice.budget_ledger_incubation.boundary]
- [x] [serial] Keep orchestration dispatch, usage observation, unknown-outcome charging choice, actor policy, store transactions, events, replay, inspection, and CLI in thin Lattice adapters. r[lattice.budget_ledger_incubation.boundary]
- [x] [parallel] Add positive tests for admitted bounds, independent reservations, total reserved accounting, below/at-bound settlement, unenforced dimensions, and remaining balances. r[lattice.budget_ledger_incubation.compatibility]
- [x] [parallel] Add negative tests for missing enforced bounds, duplicate reservation, missing settlement, limit excess, arithmetic overflow, observed overage, stale identity, and invalid orchestration integration. r[lattice.budget_ledger_incubation.compatibility]

## Phase 3: Compatibility and hardening

- [x] [serial] Preserve serialized orchestration records, fact/event identities, replay, safe budget inspection, retry preflight, and abandonment behavior through frozen vectors and focused integration tests. r[lattice.budget_ledger_incubation.compatibility]
- [x] [parallel] Add purity and dependency guards for store, filesystem, environment, clock, process, network, provider, CLI, event hashing, and product authority. r[lattice.budget_ledger_incubation.boundary]
- [x] [serial] Run focused tests again, `scripts/check.rs`, Octet, Cairn validation and gates, and bounded `nix flake check` before sync and archive. r[lattice.budget_ledger_incubation.compatibility]

## Phase 4: Promotion decision

- [x] [serial] Do not create an independent repository until a second consumer demonstrates equivalent durable reserve/settle semantics through a disposable simplifying adapter. r[lattice.budget_ledger_incubation.promotion]
- [x] [serial] If that gate later passes, create a separate promotion Cairn requiring explicit licensing, provenance, `no_std` review, canonical Radicle RID, exact revisions, no GitHub fallback, two-consumer parity, and rollback. r[lattice.budget_ledger_incubation.promotion]
