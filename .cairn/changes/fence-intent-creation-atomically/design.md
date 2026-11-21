# Design: Fence check, record and intent creation as one critical section

## Context

Baseline: Lattice `647bc59d` (published `db64cd65` plus the bounded same-attempt repair slice).

- `fencing::admit` took `File::lock` on `works/<blake3(work_id)>`, checked and recorded the fence, then dropped the lock. `Context::prepare` then persisted the binding and store, and `Context::run` wrote the intent with `write_once`. Nothing rechecked the fence in between.
- Refusals printed `dispatched:false` for `stale_fence` and `crossed_attempt` unconditionally.
- Recovery called a read-only `fencing::check` and then created its closure marker with `create_new`, outside any lock.
- The repair loop wrote `repair/step-N.intent` before each worker step with no fence check.

## Decisions

### Decision: One held per-work guard

**Choice:** `fencing::Guard` owns the locked handle to `works/<blake3(work_id)>`. `fencing::admit` returns the guard after it checks and records the fence. `Context::prepare` keeps it while it persists the binding and the candidate store. `Context::run` calls `Guard::create_intent`, which rechecks the fence and then calls `write_once` on the intent. The guard is dropped before revalidation, the revision permit and any worker effect. The lock is the existing std `File::lock` (`flock(2)` on Linux) on the pinned `nightly-2026-07-17` toolchain. No dependency is added. Waiters block. There is no timeout and no staleness rule. The kernel releases the lock when the last descriptor closes, including on process exit. The descriptor is opened close-on-exec, so a spawned worker never inherits it.

**Rationale:** One section that the lock covers continuously makes the check, the record and the intent atomic against every other writer of that work in the state root. Before this change, the fence directory was already the lock target, and a directory inside the state root keeps the lock scoped exactly to it. A timeout would let a slow but live holder be overtaken, which is the race this change removes.

### Decision: Recheck before every intent

**Choice:** `create_intent` proceeds only when `decide` returns `Replay`, which means the record at the request's fence names its attempt and no higher fence is recorded. `StaleFence` and `CrossedAttempt` become typed refusals. A missing record is a state error. Each repair step calls `Guard::is_current` under a fresh guard before `records.intend`. If the fence is not current, the step returns `Next::Stop(Disposition::FenceSuperseded)`, so its worker port is never called.

**Rationale:** On the main path the held guard makes the recheck redundant, but the recheck keeps the invariant local to the write: an intent is never created unless the fence is current. That holds even for a code path that took the lock afresh, which the repair steps do. A distinct disposition keeps a fence supersession apart from `kiln_owned`, which means that the worker proposed the continuation. The attempt's intent already exists when the loop runs, and earlier steps may have had effects. `fence_superseded` is therefore retained and printed as a post-dispatch disposition, like every other repair stop. It is never a refusal, and a later `stale_fence` refusal of that attempt reports `dispatched:true`.

### Decision: `dispatched` is read under the lock

**Choice:** `Guard::refused` reads `intent::is_execution_retained(attempt_directory)` while the guard is held. That check is true for any intent bytes other than a `closed-before-execution:` marker, including another request's bytes. `refusal::refuse_fence` prints the record with that value. `refusal::refuse` asserts in debug builds that it is never used for the two fence reasons.

**Rationale:** Every intent and closure marker of the work is created under the same lock. A higher fence or another attempt is already recorded and records are never removed, so no intent of the refused attempt can appear after the read. `dispatched:false` is therefore final for that state root.

### Decision: Recovery under the lock

**Choice:** `recover_existing` acquires the guard. It checks for a crossed attempt, computing `dispatched` under the lock. It runs `reconcile_intent`, which may create the closure marker, and drops the guard before it reads results. Recovery still writes no fence record. It may create the empty `works/<hash>` lock directory.

**Rationale:** Closure and intent creation contend for the same path. Taking the same lock orders them with fence records.

### Decision: Single-flight first runs and durable pre-intent refusals (review of `6384d535`)

**Context:** The review of the workflow-agent wiring found that request fields outside the envelope (`workflow_agent`, `lineage`) let two valid request files share one attempt-fence and session. A run that passed the unlocked no-intent check could pause; another request could then pin, take the lock, create its intent and spawn; the paused run then admitted its session before the lock and was refused `revision_not_admitted` with an unconditional `dispatched:false`, or it pinned a second pair that leaked when its intent creation failed. A request could also create an intent after another request's `dispatched:false` refusal, because pre-write refusals left the state root untouched.

**Choice:** `Context::first_run` takes the per-work guard once and holds it through: reading the intent store (any retained intent or closure marker goes to the existing reconciliation: duplicate bytes recover, other bytes are `substituted_request` with `dispatched:true`, another request's closure marker is the new `attempt_closed` with `dispatched:false`), the lineage and repair-port gates, `Guard::record`, `revision::admit`, the binding and store, and `Guard::create_intent` (`cli/execution/admission.rs`). Gates return `refusal::deferred::Stop`; a typed `deferred::Pending` refusal is settled by `Context::settle`, which, under the guard, reads `intent::is_execution_retained` and, when it is false, exclusively creates this request's `closed-before-execution:` marker before printing the record. `profile_revision_superseded` is settled the same way. A session this run pinned is abandoned on every path that ends before its intent; `revision::abandon` closes only a fresh pin. `seam::effect`, `retained` and `close` compare the pin's revision with the request's subject, so a request of another revision of the same definition never acts on the pinned session.

**Rationale:** With the intent store read first under the same lock that creates intents, at most one request of an attempt-fence passes admission, so the loser never pins. Writing the closure marker before printing makes `dispatched:false` a fact about the attempt-fence in this state root rather than about this request. The marker reuses recovery's closure, so every reader already handles it; `inspect` now reports it as `instance: null`. The cost is that a pre-intent refusal writes the state root, the lock and attempt directories and one marker, replacing the `bound-same-attempt-repair` rule that such refusals leave the state root untouched. The fence record precedes session admission, so a `revision_not_admitted` attempt keeps its fence record. Lock order is per-work lock, then the revision store; no path takes the lock while holding the store.

## Tests

- `cli/session/fencing/tests.rs`: a fence-7 run that passed its check is refused `stale_fence` (`dispatched:false`) by `create_intent` after fence 8 is recorded, and no intent exists. A fence-7 invocation after a fence-7 intent exists gets `stale_fence` with `dispatched:true`. A fence-7 invocation with no intent gets `dispatched:false`, after which neither `create_intent` nor `admit` can create the fence-7 intent. A closure marker reads as not dispatched. Crossed refusals compute `dispatched`. The lock excludes other handles until the guard drops. A child process holds the lock, and it is released when the child exits.
- `cli/tests/repair.rs`: a first step runs, records fence 8 and fails a repairable check. The second step is never spawned and has no `step-2.intent`. The outcome is the retained repair observation with disposition `fence_superseded`, one step and one revision, with no `dispatched` or `reason` field. Recovery reports it, and a later fence-7 refusal of the attempt reports `dispatched:true`.
- `cli/tests/coordinator.rs` and `tests/test_implementation_attempt_refusals.rs`: the existing stale, crossed, substituted and superseded cases still pass.

## Risks / Trade-offs

- Persisting the binding (the compute port's admission) and creating the store happen while the lock is held, so other invocations of the same work in that state root wait for them. That is the cost of one critical section. Other work items do not wait.
- The guarantee is per state root. Two state roots, or a state root on a filesystem without `flock` semantics, are not covered. Kiln remains the fence authority.
- The lock is advisory. A writer that bypasses Lattice's code is not excluded.
- If another thread of an embedding process forks while a guard is held, the child shares the locked descriptor until its `exec` closes it, so the lock can outlive the dropped guard by that window. It is never released early.
