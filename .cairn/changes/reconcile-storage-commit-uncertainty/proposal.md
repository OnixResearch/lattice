# Proposal: Reconcile storage commit uncertainty

## Why

Lattice reserves scheduler occurrences, workflow attempts, approvals, and effect state in Redb before effects. This ordering prevents automatic duplicate admission only when the reservation result is known.

A Redb commit error can leave the reservation outcome uncertain. Treating that result as an ordinary failure can permit a duplicate effect or a false retry decision.

## What Changes

- Separate persistence uncertainty from external effect uncertainty.
- Add typed Redb commit outcomes and store quarantine.
- Block effect dispatch after an uncertain reservation commit.
- Reopen and read back exact operation, attempt, revision, and reservation identities.
- Reconcile through the product-neutral Transactional Reconciliation Core after its publication.
- Treat Redb repair as local structure recovery only.
- Require explicit operator action when persistent state remains unknown or conflicting.
- Add crash and storage-fault evidence for scheduler and workflow paths.

## Impact

- **Core**: persistence observation and reconciliation decisions.
- **Shell**: Redb commit, reopen, repair callback, and effect dispatch order.
- **Adapters**: Redb and pinned transactional-reconciliation component.
- **Operator surfaces**: status, recover, retry, and abandon diagnostics.

## Non-goals

- Do not claim exactly-once external effects.
- Do not treat Redb repair as workflow-history truth.
- Do not retry an effect because a database commit returned an error.
- Do not expose raw database errors as workflow meaning.
