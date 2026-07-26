## Phase 1: Lattice baseline and producer review

- [ ] [serial] Run focused scheduler occurrence and orchestration attempt tests before core changes and record states, transitions, stores, serialized fixtures, CLI labels, crash boundaries, budgets, authority, and rollback. r[lattice.effect_attempt_adoption.compatibility]
- [ ] [serial] Review the producer's lossless alignment, simplifying two-consumer pilot, explicit license, provenance, canonical RID, exact source, endpoint, and no-fallback evidence; reject adoption when extraction was not selected. r[lattice.effect_attempt_adoption.source]
- [ ] [parallel] Add an exact mapping table for both Lattice paths across identity, parent, dispatch, observation, closure, retry, abandonment, and retained product authority. r[lattice.effect_attempt_adoption.boundary]

## Phase 2: Disposable dual adapters

- [ ] [serial] Build in-memory scheduler and orchestration adapters without changing redb schemas, run IDs, workflow revisions, event kinds, safe summaries, or CLI JSON. r[lattice.effect_attempt_adoption.boundary]
- [ ] [parallel] Add parity fixtures for reserve, acknowledge, exact success/failure, recovery, current allowed retries, cancellation prerequisites, and acknowledged unknown abandonment. r[lattice.effect_attempt_adoption.compatibility]
- [ ] [parallel] Add negative malformed/duplicate/parent, foreign/unscoped/stale/non-terminal/conflicting observation, terminal mutation, retry exhaustion, missing authority, missing acknowledgement, budget mismatch, and persisted-schema fixtures. r[lattice.effect_attempt_adoption.compatibility]
- [ ] [parallel] Measure removed branches, adapter size, callbacks, assertions, and retained policy for both paths. r[lattice.effect_attempt_adoption.cutover]

## Phase 3: Adoption decision and conditional source

- [ ] [serial] Adversarially audit exactly-once overclaims, provider deduplication, retry safety, abandonment, rollback, cancellation, persisted compatibility, budget ownership, and adapter complexity. r[lattice.effect_attempt_adoption.cutover]
- [ ] [serial] Select adopt or reject; reject when adapters retain equivalent product logic or any mapping weakens current fail-closed behavior. r[lattice.effect_attempt_adoption.cutover]
- [ ] [serial] If adopted, align Cargo and Nix on the same Radicle adapter and immutable revision and regenerate locks with owning tools without manually editing `flake.lock`. r[lattice.effect_attempt_adoption.source]
- [ ] [serial] Remove duplicate local decisions only after full parity and record one VCS rollback; do not retain runtime, sibling, vendor, or GitHub fallback. r[lattice.effect_attempt_adoption.cutover]

## Phase 4: Closeout

- [ ] [parallel] Update README references, attempt mapping, exact source, persisted compatibility, rollback, and bounded non-claims. r[lattice.effect_attempt_adoption.source] r[lattice.effect_attempt_adoption.boundary]
- [ ] [serial] Run focused tests again, operator crash evidence, `scripts/check.rs`, Octet, Cairn validation and gates, source/schema guards, and bounded `nix flake check` before sync and archive, or archive a truthful rejection with its blocker. r[lattice.effect_attempt_adoption.compatibility] r[lattice.effect_attempt_adoption.cutover]
