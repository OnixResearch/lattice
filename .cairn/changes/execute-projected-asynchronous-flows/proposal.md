## Why

Lattice executes finite projected role protocols and preserves uncertain dispatch outcomes. It has no runtime contract for projected asynchronous flows with live inputs, unordered delivery, duplicate observations, or algebraic reducers.

A generic stream loop would erase the exact Choregraph edge contract and Trellis proof scope. Lattice needs a durable executor that consumes immutable flow artifacts, enforces their operator obligations, and retains product authority.

## What Changes

- Consume one immutable Choregraph global flow, complete local-flow set, projection receipt, and Trellis law-evidence cohort.
- Add strict Lattice-owned intake types and reject stale, partial, reordered, or incompatible artifacts.
- Add pure flow-session planning for edge closure, logical ordering, repetition, completeness, operator completion, and evidence requirements.
- Execute closed and live flows with named buffer, event, item, window, retry, and retained-state limits.
- Preserve uncertain dispatch outcomes and forbid automatic retry or stronger transport claims.
- Admit unordered or duplicate-bearing input only when exact law evidence covers the selected reducer implementation and domain.
- Prevent partial monotone results from releasing protected effects without closed-input or prefix-safety evidence.
- Record explicit nondeterminism assumptions and route selected profiles to ChaosControl evidence.
- Reuse existing Lattice persistence, authorization, adapter, recovery, inspection, and replay boundaries.
- Add an unordered duplicate-bearing set-union pilot with positive, negative, crash, and replay fixtures.

## Success Criteria

- A flow session starts only from a complete current Choregraph projection and exact Trellis law cohort.
- An unordered edge cannot reach an order-sensitive operator.
- A duplicate-bearing edge cannot reach a duplicate-sensitive operator without admitted deduplication.
- A live edge cannot reach a completion-dependent operator without a bounded window or checkpoint boundary.
- A partial result cannot release a protected effect from monotonicity alone.
- A crash after possible dispatch remains explicit uncertainty and never causes automatic retry.
- Replay recomputes decisions without transport, persistence mutation, or external effects.

## False Completion

Reading a Choregraph flow artifact and calling an async stream library is not completion. A marker trait without exact proof identity is not law evidence.

A passing transport call is not logical delivery. A monotone partial value is not authority to execute an irreversible effect.

## Impact

- **Core**: pure intake, compatibility, session, edge, operator, window, closure, and effect-release decisions.
- **Shell**: persistence, adapters, transport, clocks, authorization, retries, cancellation, and observations.
- **Storage**: versioned flow sessions, edge positions, item observations, closure markers, windows, outcomes, and assumption records.
- **Testing**: set-union pilot, incompatible reducer denial, live-input denial, duplicates, reordering, loss, crash uncertainty, replay, and claim boundaries.
- **Evidence**: exact producer, law, runtime, adapter, observation, and non-claim identities.

## Dependencies

- Choregraph change `add-asynchronous-flow-profile` must publish immutable global, local-set, projection, schema, and assumption artifacts.
- Trellis change `prove-asynchronous-flow-algebra` must publish the selected law and proof-evidence cohort.
- The existing Lattice projected-role runtime, effect-attempt, persistence, authorization, and recovery boundaries remain authoritative.
- The first slice uses an existing bounded Lattice adapter. A direct Molten transport adapter is a later additive profile.
- OnixOS change `bind-choreography-roles-to-targets` can supply role placement without owning flow semantics.

## Out of Scope

- Defining Choregraph flow semantics or Trellis proof meaning.
- Building a new network stack, broker, or distributed scheduler.
- Automatic resend after uncertain dispatch.
- Transport-level exactly-once claims.
- Releasing protected effects from monotonicity alone.
- Dynamic Molten membership integration in the first pilot.
- Production readiness or release eligibility.

## Affected Specs

- `asynchronous-flow-execution`: intake, admission, durable execution, closure, algebra, effect release, uncertainty, replay, evidence, and non-claims.
