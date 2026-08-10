# Design: Type orchestration identities

## Context

The orchestration core is an identity-rich state machine. It compares exact definition and workflow revisions, instance and dispatch ownership, budget reservations, proposals, facts, trigger idempotency keys, and hash-linked events.

Several of these values share `String`. Field names document the role, but function signatures and collections do not prevent category exchange.

## Decisions

### Decision: Inventory semantic strings first

**Choice:** Classify raw orchestration strings as stable identity, revision, digest, idempotency key, external wire value, display text, or deferred.

Only stable decision inputs enter the migration.

**Rationale:** Lattice also carries prompts, summaries, labels, and redacted output. Those values do not need nominal identity wrappers.

### Decision: Add exact orchestration types

**Choice:** Add private types for `InstanceId`, `DispatchId`, `ProposalId`, `ReservationId`, `FactId`, and `TriggerIdempotencyKey`.

Add distinct types for definition and workflow revisions, authority-policy identity, event identity, and ledger-head identity. Reuse one private BLAKE3 parser through marker roles where the grammar matches.

**Rationale:** State transitions can require the exact role without repeated runtime expected-role arguments.

### Decision: Preserve `RunId`

**Choice:** Keep the existing numeric `RunId` type and reservation behavior. Replace string owner fields around it with exact orchestration identities.

**Rationale:** Run allocation already has a useful nominal boundary and durable compatibility surface.

### Decision: Separate boundary DTOs

**Choice:** Keep Nickel, CLI, scheduler, webhook, JSON, and redb compatibility structures in their current external shapes. Convert them through checked adapters before pure core decisions.

Use explicit wire projections when storing or reporting typed values.

**Rationale:** The migration must not invalidate existing orchestrations or operator scripts.

### Decision: Keep maps and events typed

**Choice:** Use typed keys in pure maps, sets, event variants, transition requests, budget state, and recovery correlation. Convert to text only for diagnostics, serialization, and external adapter calls.

**Rationale:** Conversion back to strings inside the core discards most of the compiler guarantee.

### Decision: Check crash and replay behavior

**Choice:** Test old persisted fixtures, pending dispatch recovery, cancellation, abandonment, observed-failure retry, trigger deduplication, event-chain replay, and tamper rejection before and after migration.

**Rationale:** These paths depend most strongly on exact identity correlation.

### Decision: Compile-test category errors

**Choice:** Add compile-fail fixtures for instance/dispatch, proposal/fact, reservation/dispatch, definition/workflow revision, policy/event digest, event/head digest, and idempotency/instance substitutions.

**Rationale:** These fixtures prove the new API rejects exchangeable values.

## Functional core and shell boundary

Construction, transition decisions, budget math, event reduction, recovery correlation, and replay remain pure.

Redb I/O, scheduler processes, webhook sockets, provider calls, credentials, terminal behavior, and command execution remain in shells.

## Test design

Positive tests cover each constructor, valid same-role transitions, persistence round trips, duplicate-fact idempotence, owned run recovery, and stable replay.

Negative tests cover malformed hashes, empty or oversized IDs, wrong owners, stale revisions, stale approvals, foreign run records, conflicting idempotency keys, event tampering, and compile-time substitutions.

## Risks and trade-offs

- Persistence adapters add code. Frozen fixtures guard compatibility.
- Generic marker diagnostics can be verbose. Public aliases keep domain names visible.
- Existing internal tests can construct raw structs directly. Test builders must use checked helpers.
- Typed local identities do not solve remote uncertainty. Existing non-claims remain.

## Claim boundary

The change prevents selected local category errors and centralizes identity parsing. It does not prove provider behavior, external effect deduplication, history completeness, or production readiness.
