# Design: Reconcile storage commit uncertainty

## Context

Lattice already models `PendingUnobserved` after uncertain external effects. Storage commit uncertainty occurs earlier and needs a separate state.

The pure core must decide whether dispatch is permitted. The shell owns Redb transactions, process failures, reopen, and effect execution.

## Decisions

### Decision: Persistence and effect uncertainty are distinct

A reservation can be `NotPublished`, `Published`, `PublicationUnknown`, or `Conflicting`. An effect can remain `NotStarted`, `PendingUnobserved`, or terminal.

No transition converts publication uncertainty into effect uncertainty without read-back evidence.

### Decision: Dispatch requires observed reservation

The shell dispatches an effect only after the core receives a `Published` observation for the exact operation and attempt.

A commit error, repair callback, or decode failure quarantines the store and blocks dispatch.

### Decision: Read-back uses exact identities

Reconciliation binds operation, plan, workflow revision, attempt, reservation, and expected predecessor identities. The reopened row is prior, replacement, missing, conflicting, or corrupt.

### Decision: Shared core stays product neutral

Lattice consumes a pinned Transactional Reconciliation Core contract for persistence observation and recovery classification. Lattice retains workflow policy, Redb schema, commands, effects, operator authority, and receipts.

### Decision: Repair does not clear workflow uncertainty

`runs repair` or Redb automatic repair can make the database openable. It cannot restore missing workflow truth or admit effect execution.

After repair, every affected record needs exact reconciliation or explicit fail-closed operator closure.

## Test Design

Positive tests cover durable reservation, reopened replacement, no-effect cancellation before dispatch, and normal terminal publication.

Negative tests cover commit error, process death during commit, repair callback, stale cache, corrupt row, missing row, conflicting revision, foreign attempt, retry before reconciliation, and effect dispatch from quarantine.

## Claim Boundary

The change establishes local Lattice admission under selected Redb and fault cohorts. It does not prove provider deduplication, external exactly-once behavior, or production readiness.
