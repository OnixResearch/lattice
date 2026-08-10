# Durable scheduler-occurrence admission validation

Date: 2026-07-24

## Result

Each due scheduler slot now receives a domain-separated BLAKE3 identity over `(schedule_id, due_epoch_ms)`. One redb write transaction verifies the exact persisted slot, reserves its run ID, inserts `PendingUnobserved` occurrence state, and advances the schedule before workflow loading or execution can cross into effects. Concurrent local daemons therefore select one durable occurrence owner and one automatic local attempt.

Daemon restart does not automatically replay a claimed slot. Recovery settles only terminal run evidence whose `RunOwner::ScheduleOccurrence` identity and run ID exactly match the active attempt. Missing, unfinished, foreign, mismatched, or legacy-unscoped evidence fails closed or remains `PendingUnobserved`; it is never promoted to external success. Workflow-revision drift is rejected before retry effects.

Operators can inspect occurrences and explicitly choose one acknowledged retry or acknowledged abandonment. Retry remains bounded to two total attempt IDs and requires the active run record to be absent. Abandonment records `AbandonedUnknown` without claiming success. These controls establish automatic local at-most-one admission only for the checked single active redb store owner. They do not establish external exactly-once execution, provider deduplication, distributed-writer safety, production readiness, or knowledge of an unobserved effect's outcome.

## Baseline

`cairn/archive/2026-07-23-characterize-post-effect-crash-boundaries` demonstrated that post-effect scheduler death left the schedule due, so ordinary restart performed a second identified local attempt. The new process tests retain that death boundary while requiring the claimed occurrence and advanced schedule to suppress restart replay.

## Focused evidence

- `nix develop -c cargo test --test test_scheduler_occurrences`: 14 pure, redb, shell, CLI, exact-owner, revision-drift, acknowledgement, retry-bound, rollback, and contention cases passed.
- `nix develop -c cargo test --test test_scheduler`: four operator cases passed, including pre-effect death plus explicit retry, post-effect death plus abandonment, ordinary restart suppression, and concurrent-daemon one-owner admission.
- `nix develop -c cargo test --test test_traceability_webhook_auth_completion`: two reviewed-hash drift checks passed after the store split.
- Focused store, headless, run-inspection, post-effect profile, Clippy, and traceability tests passed during implementation.
- The real death fixtures use a durable marker effect: pre-effect death records no effect, post-effect death records one effect, ordinary restart records no second effect, explicit retry is the only acknowledged second attempt, and abandonment performs no execution.

## Repository rails

- `scripts/check.rs`: passed after the final implementation and policy refactor, including package formatting, all-target compilation, full tests, operator E2E, Clippy, and Cairn spec checks.
- `nix develop -c octet-precommit-check`: 902 accepted findings, 0 new, and 0 stale. The guarded baseline migrator self-test passed and matched all 1,151 semantic findings while migrating only the pre-existing `src/store/mod.rs` file-length identity; no new Octet debt was accepted.
- Cairn repository validation plus proposal, design, and tasks gates passed.
- Cairn Tracey coverage passed with 926 of 926 accepted requirements referenced after deterministic spec sync.
- `nix flake check -L --option secret-key-files ''`: all x86_64-Linux checks passed; aarch64-darwin, aarch64-linux, and x86_64-darwin were omitted as incompatible systems.
