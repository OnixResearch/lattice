# Change: Add bounded retries for observed terminal failures

## Why

Lattice can safely reconcile an owned persisted run and can explicitly abandon an uncertain dispatch, but every observed failed round currently terminalizes its orchestration permanently. Operators need one narrow retry path for failures that Lattice has actually observed and persisted. Reusing the same dispatch or retrying a `PendingUnobserved` outcome would cross the existing uncertainty boundary and risk duplicate external effects.

## What Changes

- Add an optional typed retry policy with an explicit maximum count of observed-failure retries.
- Record bounded retry eligibility only when an exact orchestration-owned `RunRecord` terminalizes with one or more failures.
- Add an authorized `orchestration retry` command that atomically starts a fresh round with a fresh run/dispatch identity and fresh budget reservation.
- Reject automatic retry, disabled/exhausted retry policy, foreign ownership, abandoned dispatches, shell failures without exact persisted terminal evidence, and every pending or `PendingUnobserved` dispatch.
- Preserve deterministic replay and schema-v1 identity compatibility by omitting absent/default retry fields from canonical serialization.

## Impact

- Affected spec: `agent-orchestration`
- Affected code: orchestration core, shell, Nickel format, CLI, safe inspection joins, examples, and focused tests
- No provider retry, backoff scheduler, arbitrary workflow retry, retry of uncertain effects, provider cancellation, remote exactly-once, or production-readiness claim is added.
