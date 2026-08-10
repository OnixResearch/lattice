## Context

`src/orchestration/core.rs` owns budget policy, dimensions, limits, charges, plans, reservations, consumed totals, breach state, reserve, settle, remaining-budget, and arithmetic helpers. The same module also owns proposal, dispatch, retry, event, replay, and orchestration transitions.

The ledger is pure but coupled to a large product core. Seaglass currently enforces direct resource limits and does not provide a proven durable reservation consumer.

## Success Contract

Completion requires a separately testable local core, unchanged orchestration compatibility, positive and negative accounting tests, thin product adapters, and explicit promotion gates. Moving types without decisions or publishing a repository with only one consumer is false completion.

## Decisions

### Start local

The first destination is a Lattice workspace package or strict internal core named `budget-ledger-core`. It has no independent remote and no cross-repository compatibility promise. This lets the API change while tests expose the real reusable boundary.

### Keep fixed dimensions initially

The local core retains current named dimensions. It does not introduce dynamic string-keyed dimensions, trait-object arithmetic, or a universal unit system. A future second consumer can prove whether generic dimensions are useful.

### Keep policy inputs explicit

The core receives policy, planned charge, reservation identity, and observed charge. It validates required bounds, duplicate reservations, total committed plus reserved charge, checked arithmetic, settlement ownership, and breach state.

The core does not read provider usage, clocks, pricing, or persistence. Lattice shells decide planned charges and convert run observations into charges.

### Keep unknown-outcome charging in the shell

Current orchestration policy charges an acknowledged abandoned dispatch conservatively and an unacknowledged pending dispatch differently. The shell selects the observed charge supplied to settlement. The generic ledger does not infer remote work from dispatch state.

### Preserve persistence and replay

Serialized orchestration types and events remain compatible in the first extraction. Adapters convert between existing product records and local core values if package boundaries require it. Event hashing and replay remain Lattice-owned.

### Require evidence before promotion

A future independent repository requires:

- one second consumer with durable reserve and settle semantics;
- two disposable adapters that remove material duplicate logic;
- explicit source licensing and provenance;
- a `no_std` plus `alloc` feasibility review;
- one canonical Radicle RID and exact immutable revisions;
- no GitHub, sibling, or floating fallback.

Similar scalar limit checks do not satisfy this gate.

## Validation

Positive tests cover admission, multiple independent reservations, settlement below and at reservation, unenforced dimensions, and remaining balances. Negative tests cover missing enforced bounds, duplicate IDs, missing settlement, total limit excess, arithmetic overflow, observed overage, and terminal integration misuse.

## Claim Boundary

The ledger classifies supplied integer accounting facts. It does not prove provider usage, billing, time, resource availability, fairness, work completion, budget policy correctness, or release readiness.

## Risks

- Package extraction can add conversion noise without reuse. Keep types close to current shapes.
- Fixed dimensions can look product-specific. That is intentional during incubation.
- Persisted serde and event hashes can change accidentally. Frozen vectors must guard them.
- Promotion pressure can mistake Seaglass direct limits for durable ledger parity.
