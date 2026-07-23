## Phase 1: Atomic admission model

- [ ] [serial] Add a pure domain-separated BLAKE3 trigger-claim identity and post-transaction decision types with positive and negative tests for inserted, duplicate, malformed, missing-key, cross-trigger, and no-idempotency-policy inputs. r[lattice.scheduling.webhook_atomic_idempotency]
- [ ] [serial] Add a redb trigger-admission table and one transaction that rejects an existing trigger-scoped key or atomically reserves a run identity, inserts the immutable claim, and persists the safe accepted/started history fact. r[lattice.scheduling.webhook_atomic_idempotency]
- [ ] [serial] Add store race tests behind a barrier proving exactly one inserted result, one stable existing owner, no overwritten claim, one reserved run, and explicit behavior for idempotency-disabled triggers. r[lattice.scheduling.webhook_atomic_idempotency]

## Phase 2: Listener and execution shell

- [ ] [serial] Route admitted webhook plans through the atomic claim before headless effects, preserve the claim across terminal failure or interruption, and map losing claims to the existing safe duplicate response without allocating another run. r[lattice.scheduling.webhook_atomic_idempotency]
- [ ] [serial] Add bounded concurrent connection handling with an explicit named in-flight limit, owned task teardown, and no unbounded queue or hidden background process. r[lattice.scheduling.webhook_atomic_idempotency]
- [ ] [serial] Add real simultaneous loopback requests proving one accepted path, deterministic duplicate conflicts, one effect, one run, and immutable safe claim/history state; add negative cases for failed winner execution and idempotency-disabled triggers. r[lattice.scheduling.webhook_atomic_idempotency]
- [ ] [serial] Update the dependent post-effect crash profile so a committed webhook claim survives listener death and redelivery cannot start a second local execution, while retaining the archived duplicate-risk baseline. r[lattice.scheduling.webhook_atomic_idempotency]

## Phase 3: Compatibility and validation

- [ ] [serial] Document local at-most-one-admission semantics, uncertain claimed attempts, operator inspection, single-store-owner scope, and exactly-once external-effect non-claims. r[lattice.scheduling.webhook_atomic_idempotency]
- [ ] [serial] Run focused core/store/listener tests, `scripts/operator-e2e.rs`, `scripts/check.rs`, Octet, Cairn validation and all three change gates, then run bounded Nix flake validation and archive exact evidence. r[lattice.scheduling.webhook_atomic_idempotency]
