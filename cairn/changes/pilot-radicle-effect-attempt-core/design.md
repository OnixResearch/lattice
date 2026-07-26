## Context

Scheduler occurrences use `PendingUnobserved`, `Succeeded`, `Failed`, and `AbandonedUnknown`, plus a bounded attempt list and exact active run. Orchestration uses pending and acknowledged dispatch states, run-record recovery, budget reservation/settlement, cancellation authority, retry evidence, and conservative acknowledged abandonment.

These paths have different product shells and persisted data. The external pilot must not force them into one Lattice storage schema.

## Success Contract

Completion is either a production-ready simplifying adapter over an accepted Radicle source or a documented rejection with the exact blocker. Shared enum names, DTO conversion, or unchanged duplicate decisions behind callbacks are false completion.

## Decisions

### Baseline before core changes

Run focused schedule occurrence, orchestration recovery/retry/abandonment, store, inspection, and crash-boundary tests before modifying decision code. Record serialized fixtures and CLI labels that are compatibility surfaces.

### Map facts without changing storage

Each Lattice path converts its current stored model into shared in-memory facts and maps shared decisions back through existing store transactions. The first pilot does not change redb schemas, run IDs, workflow revisions, event kinds, or CLI JSON.

### Preserve three distinct layers

The adapter keeps dispatch phase, supplied observation, and durable closure separate. Missing observations remain pending. Exact terminal observations can settle. Foreign, unscoped, stale, non-terminal, malformed, or conflicting observations fail closed.

### Keep policy and authority local

Lattice decides attempt bounds, duplicate-risk acknowledgement, idempotency policy, cancellation prerequisites, unknown-outcome acknowledgement, actors, workflow-revision checks, retry limits, budget reservation and settlement, and safe summaries. The shared core only classifies supplied admitted facts.

### Measure simplification

Acceptance compares removed decision branches, adapter code, callbacks, negative fixture coverage, and assertion density. If adapters mostly translate product types while retaining all decisions, production adoption is rejected.

### Require exact Radicle source conditionally

No dependency is added unless the producer selected extraction and published explicit licensing, provenance, canonical RID, authority policy, exact object, endpoint, and no-fallback evidence. Cargo and Nix then pin one exact revision. Locks use owning tools; `flake.lock` is not manually edited.

### Retain VCS rollback

Production has one attempt decision path after acceptance. Rollback reverts dependency, generated locks, adapters, and removed local logic together. There is no runtime fallback to local or GitHub source.

## Validation

Positive cases cover reserve, acknowledge, exact success/failure, observed retry, idempotent or acknowledged-risk retry where current policy allows, cancellation prerequisites, and acknowledged abandonment. Negative cases cover identity errors, owner mismatch, stale revisions, non-terminal observations, terminal mutation, retry exhaustion, missing authority, absent acknowledgement, budget mismatch, source drift, and fallback attempts.

## Claim Boundary

Passing checks establish only selected local transition parity over supplied observations. They do not prove remote effects, provider truth, exactly-once behavior, cancellation, rollback, budget truth, crash safety, or release readiness.

## Risks

- Scheduler and orchestration semantics may not share enough decisions.
- Persisted compatibility can be disturbed by innocent type changes.
- Generic retry vocabulary can weaken Lattice acknowledgements. Product policy stays explicit.
- Producer licensing or Radicle publication may remain blocked.
