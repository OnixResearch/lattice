# Fence check, record and intent creation as one critical section

Tracking: [Lattice #6](https://github.com/OnixResearch/lattice/issues/6) and [Lattice #7](https://github.com/OnixResearch/lattice/issues/7). The defect was found by an independent review of Kiln #45's consumer of `lattice implementation-attempt`. Owner-approved round 2026-11-21.

## Why

A `run` checked and recorded its fence under a lock in `cli/session/fencing.rs` and released it. It then wrote its execution intent in `cli/execution.rs` without rechecking. Consider a fence-7 invocation that passes its check and pauses, while another invocation records fence 8. The fence-7 run still writes its intent and starts its worker, so two workers for the same work can run. Meanwhile a further fence-7 invocation receives `stale_fence` with `dispatched:false`, even though the earlier fence-7 run may still start. Kiln settles on that record, so the refusal must be true of every later run in that state root.

## What Changes

- Within one state root, one exclusive `flock(2)` lock per work item (`std::fs::File::lock` on `works/<blake3(work_id)>`) covers three steps of a run: the fence check, the fence record and the creation of the execution intent. The binding and candidate store are persisted inside the same critical section. The OS releases the lock when its holder exits, and it never expires. r[lattice.implementation_attempt.consumer] r[lattice.implementation_attempt.consumer.scenario.fence_lock]
- Before it writes the intent, the run rechecks under the lock that the record at its fence still names its attempt and that no higher fence is recorded. Otherwise it is refused `stale_fence` or `crossed_attempt` before any intent or worker. r[lattice.implementation_attempt.consumer.scenario.fence_lock]
- `stale_fence` and `crossed_attempt` refusals compute `dispatched` under the lock from the refused attempt's directory. It is `true` exactly when an execution intent other than recovery's closure marker exists. r[lattice.implementation_attempt.consumer.scenario.refusal]
- Each repair step rechecks the fence under the same lock before it writes `repair/step-N.intent`. When the fence is no longer current, the step is not spawned and the loop ends as the new `fence_superseded` disposition. That disposition is a retained post-dispatch outcome, never a refusal and never `dispatched:false`, because the attempt's intent exists and earlier steps may have had effects. Recovery's crossed check and closure marker take the same lock. r[lattice.implementation_attempt.consumer.scenario.fence_lock]
- `docs/candidate-worker.md` and the changelog state this guarantee exactly, scoped to one state root.

## Impact

- **Files**: `src/implementation_attempt/cli/session/fencing.rs` (`Guard`, `Refused`, `Error`, the recheck) and `cli/session/fencing/tests.rs`; `cli/execution.rs` (the guard is held from the fence record through the intent, and recovery runs under the lock); `cli/refusal.rs` (`refuse_fence`); `cli/session/intent.rs` (`is_execution_retained`); `cli/recovery.rs` (`is_closed_marker`, and `reconcile_intent`, moved from `cli/execution.rs`); `cli/repair/{mod,steps}.rs` (the step recheck); `src/implementation_attempt/repair/state.rs` (`Disposition::FenceSuperseded`); `cli/tests/{coordinator,repair}.rs`; `docs/candidate-worker.md`; `CHANGELOG.md`.
- **Compatibility**: The request and intent bytes are unchanged, and so are the fence-record layout and the refusal schema. A `stale_fence` or `crossed_attempt` refusal now reports `dispatched:true` when the refused attempt holds an execution intent, where it was always `false` before. The repair observation gains one disposition kind.
- **Testing**: `cargo test -p lattice --lib -- implementation_attempt`, `cargo test -p lattice --test test_implementation_attempt_refusals`, clippy with `-D warnings`, `cargo fmt -p lattice --check`, `octet-precommit-check`, `cairn validate` and this change's gates.

## Dependencies

- Local: `execute-admitted-implementation-attempt` (fencing, intent and refusals), `bound-same-attempt-repair` (the repair loop and its step intents).

## Out of Scope

- Fencing across state roots. Kiln remains the fence authority, and two state roots share no records or lock.
- Changes to Kiln's consumer.
- Further repair-slice review fixes: revocation rechecks, check-snapshot integrity, deadline enforcement and budget totals. They follow separately.
