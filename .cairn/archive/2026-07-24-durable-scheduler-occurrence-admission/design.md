## Context

`run_scheduler_daemon_once` lists schedules, evaluates due plans, reserves a run ID, executes the workflow, and advances `next_run_epoch_ms` only after the headless shell returns. The checked scheduler crash fixture crosses a local effect checkpoint, kills the Lattice parent before terminal persistence, observes zero runs and an unchanged due schedule, then observes a second effect attempt after restart.

Webhook admission already demonstrates the useful local mechanism: one redb transaction can bind a stable BLAKE3 claim to a reserved run before effects. Scheduled occurrences additionally need exact due-slot validation, schedule advancement in the same transaction, run-owner binding, and an operator path for an unknown outcome that has no redelivery client.

### Completion contract

The change is complete only when one transaction verifies the exact persisted due slot, reserves the global run ID, persists immutable occurrence/slot facts plus a bounded attempt ledger, and advances the schedule before the workflow shell starts. A forced post-effect death followed by ordinary daemon restart must leave exactly one local attempt and one pending occurrence. Recovery may settle only from an exact owned `RunRecord`; otherwise only explicit acknowledged retry or abandonment may close the operator decision.

False completion includes advancing the schedule in a later transaction, keying only by run ID, using a process-local mutex, automatically retrying after restart, treating a missing run record as proof that no effect happened, allowing foreign or unscoped run evidence to settle the occurrence, retrying a changed workflow revision, permitting unbounded retries, or describing local admission as exactly-once external execution.

## Decisions

### Decision: Identify the exact schedule slot

**Choice:** Derive a domain-separated BLAKE3 identity from tagged schedule ID and canonical due-epoch bytes. Store the schedule ID, workflow name, exact workflow revision, due epoch, next epoch, bounded attempt run IDs, active run ID, and status under that identity.

**Rationale:** `(schedule_id, due_epoch_ms)` is the stable semantic occurrence. A global run ID alone cannot prevent a second run from being allocated for the same due slot, while the stored workflow revision prevents an explicit retry from silently executing changed workflow content.

### Decision: Make one store transaction authoritative

**Choice:** In one redb write transaction, verify that the current schedule still matches the planned workflow and due epoch, reject an existing occurrence identity, reserve the global run ID, insert the pending occurrence, and persist the advanced schedule.

**Rationale:** Any read/claim/advance split leaves a crash or contention window. The transaction is the final admission authority; pure planning validates inputs and derives identity before I/O.

### Decision: Bind run history to the occurrence owner

**Choice:** Extend `RunOwner` with a schedule-occurrence identity and execute every initial or retried attempt through the existing owned headless path. A pure recovery classifier accepts only the active run ID with the exact occurrence owner.

**Rationale:** Numeric identity equality without owner binding can let foreign or legacy-unscoped evidence falsely settle an occurrence.

### Decision: Prefer unknown suppression over automatic duplicate risk

**Choice:** A committed occurrence remains `PendingUnobserved` when no exact terminal run exists. Daemon restart does not rerun it because the schedule was already advanced. Inspection exposes the pending identity and active run without inventing an outcome.

**Rationale:** Absence of a terminal record cannot distinguish death before effects from death after effects. Automatic replay chooses at-least-once behavior and reproduces the checked duplicate; durable suppression makes the uncertainty explicit.

### Decision: Bound explicit operator closure

**Choice:** Permit at most one explicit retry after the initial attempt. Retry requires a dedicated duplicate-risk acknowledgement, no persisted active run, exact current workflow revision, and a fresh globally reserved run ID appended to the occurrence ledger. Abandonment requires a separate unknown-outcome acknowledgement and records `AbandonedUnknown` without execution. Both fail closed on terminal, mismatched, exhausted, missing, or foreign state.

**Rationale:** Operators need a deliberate escape hatch, but implicit or unbounded retries would turn unknown outcomes into an uncontrolled effect loop. Separate acknowledgement flags make the chosen risk visible at the command boundary.

## Functional Core / Imperative Shell

The scheduler core validates plans, derives occurrence identity, classifies claim and recovery facts, validates retry/abandon transitions, and renders safe status from explicit inputs. The redb store owns atomic claim/retry/abandon/settlement transactions. The scheduler shell loads workflows, computes revisions, inspects owned run records, invokes the headless executor, and applies core decisions. The CLI only parses acknowledgement/output options and renders results.

## Risks / Trade-offs

- Death after claim but before the first effect can suppress a scheduled run that never produced an external effect; pending status records uncertainty rather than availability.
- An explicit retry can duplicate an effect that occurred before the original process died; the command must require acknowledgement and must not claim safety.
- One active redb owner bounds local admission but does not provide distributed scheduler coordination.
- Workflow revision binding can block retry after an operator edits the saved workflow; abandonment remains available and no migration is inferred.
- Forced-death evidence remains scoped to checked Unix/Linux process semantics and selected local effects.
