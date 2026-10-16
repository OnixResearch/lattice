# Storage commit uncertainty

Lattice separates persistence publication from external effect outcomes.

The scheduler claim transaction owns one selected implementation of this boundary. Before publication, Lattice commits an on-store fence containing the exact expected occurrence and reserves its run identity. The publication transaction appends that occurrence and advances its schedule. Only exact persisted read-back can clear the publication fence and admit dispatch.

## Commit outcomes

- A normal successful commit returns the inserted occurrence.
- A commit error returns `StoreError::PublicationUnknown`.
- A lost post-commit observation returns `StoreError::PublicationUnknown`.
- Both uncertain outcomes quarantine selected workflow, approval, effect-attempt, and scheduler mutation paths. The on-store fence survives reopening; a process-local token additionally fails closed if the initial fence commit itself has an unknown result.

An uncertain commit does not become definite non-publication. It does not create `PendingUnobserved` effect state.

## Reconciliation

`Store::reconcile_schedule_occurrence_publication` bypasses ordinary mutation fencing for one exact read-back operation.

- An exact saved successor clears a publication-only fence as `Published`.
- A missing row remains `PublicationUnknown` and keeps its reserved run identity and fence.
- Invalid or conflicting data keeps dispatch blocked without replacing either the expected reservation or observed evidence.
- Read-back of an occurrence alone cannot clear a reopened-history fence.

Occurrence listing, recovery inspection, and run inspection remain available while fenced. An exact terminal run with the active occurrence's run ID and durable owner can settle that occurrence and atomically remove its fence. A retry requires the existing duplicate-risk acknowledgement, current workflow revision, no stored active-run evidence, and a newly reserved run ID. Abandonment requires the existing unknown-outcome acknowledgement and no stored active-run evidence. Neither acknowledgement overrides foreign, nonterminal, or conflicting stored evidence. A closure removes only its own fence; other fenced occurrences continue to block mutations.

## Repair boundary

`runs repair` reports only whether Redb can open the local structure. Before returning, it writes fences for pending persisted occurrences; repair success does not authorize their dispatch. Ordinary writable store opening conservatively fences pending occurrences too, including a clean restart, so death between structural repair and application fencing cannot silently restore authority on the next open. Fences stay in the same Redb database, not in a sidecar file.

Each fence stores one occurrence record, limited to 65,536 encoded bytes, and its publication-versus-reopened-history distinction. Repair streams the existing occurrence table rather than materializing an unbounded recovery list. An existing expected reservation is preserved even if observed rows conflict. The selected mechanism fences scheduler occurrences; it does not invent reconciliation semantics for unrelated application protocols.

The maintained tests terminate a child process from inside the real Redb publication commit's backend `sync_data` call, reopen and repair its store, and require dispatch admission to remain blocked. Separate paired runtime coverage uses current admitted process authority: the shell effect runs normally, is blocked after reopening a pending occurrence, and runs only under an explicit fresh acknowledged reservation.

## Non-claims

This mechanism does not prove Redb durability, filesystem persistence, power-loss behavior, provider deduplication, external exactly-once effects, or production readiness.
