## Phase 1: Implementation

- [x] [serial] 1.1 Hold one per-work `fencing::Guard` (`File::lock` on `works/<blake3(work_id)>`) from the fence check and record through the binding, the store and `Guard::create_intent`, which rechecks the fence before `write_once` of the intent; refuse `stale_fence` or `crossed_attempt` before any intent or worker (`src/implementation_attempt/cli/session/fencing.rs`, `cli/execution.rs`). Tests: `src/implementation_attempt/cli/session/fencing/tests.rs`. Evidence: `evidence/verification.json`. r[lattice.implementation_attempt.consumer] r[lattice.implementation_attempt.consumer.scenario.fence_lock]
- [x] [serial] 1.2 Compute `dispatched` of `stale_fence` and `crossed_attempt` under the lock from the refused attempt's intent store (`cli/refusal.rs`, `cli/session/intent.rs`, `cli/recovery.rs`). Tests: `src/implementation_attempt/cli/session/fencing/tests.rs`, `src/implementation_attempt/cli/tests/coordinator.rs`, `tests/test_implementation_attempt_refusals.rs`. Evidence: `evidence/verification.json`. r[lattice.implementation_attempt.consumer.scenario.refusal]
- [x] [serial] 1.3 Recheck the fence under the lock before each repair step intent and end the loop without spawning as the retained post-dispatch disposition `fence_superseded`, never a refusal or `dispatched:false`; take the lock for recovery's crossed check and closure marker (`cli/repair/{mod,steps}.rs`, `src/implementation_attempt/repair/state.rs`, `cli/execution.rs`, `cli/recovery.rs`). Tests: `src/implementation_attempt/cli/tests/repair.rs`. Evidence: `evidence/verification.json`. r[lattice.implementation_attempt.consumer.scenario.fence_lock] r[lattice.implementation_attempt.consumer.scenario.recovery]

## Phase 2: Documentation and verification

- [x] [serial] 2.1 State the per-state-root guarantee, the lock and the `dispatched` semantics in `docs/candidate-worker.md` and the changelog. Evidence: `evidence/verification.json`. r[lattice.implementation_attempt.consumer.scenario.refusal]
- [x] [serial] 2.2 Run the focused tests, clippy, fmt, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks, and record the results in `evidence/verification.json`. r[lattice.implementation_attempt.consumer.scenario.fence_lock]

Only checked tasks are complete.
