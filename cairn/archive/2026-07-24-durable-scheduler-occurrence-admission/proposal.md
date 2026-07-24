## Why

The scheduler currently reserves a run identity, executes the workflow, persists terminal run evidence, and only then advances the schedule. The checked post-effect crash profile proves that process death after a local effect leaves the occurrence due, so daemon restart automatically performs the same scheduled occurrence again.

Lattice needs a durable occurrence winner before workflow effects and an explicit operator decision for the resulting unknown-outcome boundary. The goal is local at-most-one automatic admission for one `(schedule_id, due_epoch_ms)` occurrence, not exactly-once execution or proof of an external effect.

## What Changes

- Derive a domain-separated BLAKE3 occurrence identity from the schedule identity and exact due epoch.
- Atomically verify the due schedule, reserve the run identity, persist durable occurrence state with immutable slot facts, and advance the schedule before workflow effects.
- Bind terminal run evidence to the exact occurrence owner and reconcile pending occurrences only from matching persisted run records.
- Add safe occurrence inspection plus explicit bounded retry and abandonment commands; automatic daemon restart never retries a claimed occurrence.
- Update the real post-effect and pre-effect process-death profiles to prove automatic suppression, pending-unknown reporting, and operator-authorized closure.
- Preserve explicit non-claims for external commitment, deduplication, distributed writers, exactly-once execution, and production readiness.

## Impact

- **Files**: `src/scheduler.rs`, `src/scheduler/shell.rs`, `src/store/mod.rs`, `src/headless.rs`, `src/main.rs`, scheduler/store/process tests, `scripts/operator-e2e.rs`, `README.md`, and native Cairn lifecycle artifacts.
- **Testing**: pure identity/admission/recovery tests, redb transaction and owner-binding tests, real scheduler death/restart/retry/abandon process tests, focused scheduler/store/headless checks, operator E2E, `scripts/check.rs`, Octet, Cairn gates and traceability, and bounded Nix flake validation.
