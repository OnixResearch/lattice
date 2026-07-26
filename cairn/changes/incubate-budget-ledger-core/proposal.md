## Why

Lattice orchestration contains a useful pure multidimensional reservation and settlement ledger for rounds, agent invocations, tokens, monetary microunits, and duration. Seaglass has direct memory, row, byte, and concurrency limits, but it does not yet have the same durable reserve-and-settle use case.

Creating a shared repository now would generalize from one real consumer. Lattice should first isolate and harden the ledger as a local core. Promotion requires a second consumer with matching reservation semantics, not only similar words such as budget or limit.

## What Changes

- Extract current budget types and pure reserve/settle decisions into a Lattice-local `budget-ledger-core` package or equally strict internal core boundary.
- Keep orchestration dispatch, observed usage collection, abandonment charging, persistence, actors, CLI, and evidence in Lattice shells.
- Add positive and negative tests for enforced dimensions, reservations, concurrent totals, settlement, breach, overflow, missing bounds, duplicates, and unknown outcomes.
- Preserve serialized orchestration compatibility and current safe inspection output.
- Define measurable promotion criteria based on a real second consumer and disposable adapter.
- If later promoted, require an independent repository with explicit licensing and a canonical Radicle RID rather than GitHub.

## Success Criteria

- Ledger logic can be tested without store, clock, process, network, provider, or orchestration setup.
- Checked arithmetic and fail-closed reservation rules preserve current behavior.
- Product shells explicitly decide planned charges, observed usage, and conservative unknown-outcome charging.
- Existing persisted events and inspection output remain compatible.
- No standalone repository is created until a second consumer replaces material duplicate reservation logic.

## Non-Goals

- Generalizing every scalar limit, memory cap, semaphore, quota, rate limit, or billing estimate.
- Moving provider usage truth, budget policy authoring, persistence, or release authority into the core.
- Claiming reservations prove provider billing, elapsed duration, or resource availability.
- Publishing a one-consumer abstraction.

## Impact

- **Source**: orchestration budget types and pure decisions, with thin adapter changes.
- **Testing**: focused baseline and post-change budget/orchestration/replay/inspection tests.
- **Future**: promotion Cairn only after a second consumer and Radicle publication gate exist.
