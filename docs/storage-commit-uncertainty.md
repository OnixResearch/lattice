# Storage commit uncertainty

Lattice separates persistence publication from external effect outcomes.

The scheduler claim transaction owns one selected implementation of this boundary. A successful commit permits normal dispatch only after the exact saved occurrence maps to a published shared-core reservation.

## Commit outcomes

- A normal successful commit returns the inserted occurrence.
- A commit error returns `StoreError::PublicationUnknown`.
- A lost post-commit observation returns `StoreError::PublicationUnknown`.
- Both uncertain outcomes quarantine all selected workflow, approval, effect-attempt, and scheduler mutation paths on the current store handle.

An uncertain commit does not become definite non-publication. It does not create `PendingUnobserved` effect state.

## Reconciliation

`Store::reconcile_schedule_occurrence_publication` bypasses normal quarantined reads for one exact read-back operation.

- An exact saved successor clears quarantine as `Published`.
- A missing row remains `PublicationUnknown` and keeps quarantine.
- Invalid or conflicting data returns a bounded store error and keeps quarantine.

Retry and abandonment remain blocked while quarantine is active.

## Repair boundary

`runs repair` reports only whether Redb can open the local structure. It does not clear a live Lattice quarantine or prove workflow-history completeness.

A process restart does not retain the process-local quarantine token. Operators must preserve uncertain-operation evidence outside a disposable process when restart recovery is required.

## Non-claims

This mechanism does not prove Redb durability, filesystem persistence, power-loss behavior, provider deduplication, external exactly-once effects, or production readiness.
