## Why

Lattice has two durable attempt-like paths: scheduler occurrences and agent-orchestration dispatches. Both preserve uncertain outcomes, exact run ownership, explicit retry or abandonment authority, and terminal settlement. Seaglass has related sink-attempt and receipt semantics.

The proposed `effect-attempt-core` may remove duplicate Lattice decisions and provide a real second consumer for Seaglass's extraction. It must not weaken Lattice's persisted compatibility, explicit acknowledgements, budget handling, or unknown-outcome claims.

## What Changes

- Baseline scheduler occurrence and orchestration dispatch states, transitions, stores, CLI recovery, tests, and evidence.
- Review the producer pilot and require one licensed immutable Radicle source only if extraction was accepted.
- Build disposable adapters for both Lattice paths over separate dispatch-phase, observation, and closure facts.
- Preserve exact-owner recovery, pending uncertainty, duplicate-risk acknowledgement, unknown-outcome acknowledgement, and terminal immutability.
- Keep run IDs, schedules, workflow revisions, budgets, actor authority, persistence transactions, CLI, and external effects in Lattice.
- Admit or reject production cutover from measured simplification and complete positive and negative parity.

## Success Criteria

- Both adapters pass current positive and negative fixtures without changing persisted schema or external commands.
- The shared core replaces material duplicate decisions and requires fewer product callbacks than retained logic.
- Missing, foreign, unscoped, non-terminal, stale, or conflicting evidence never becomes success or failure.
- Retry and abandonment retain exact current acknowledgements and policy gates.
- Any production dependency uses one exact Radicle revision without GitHub or sibling fallback.

## Non-Goals

- Exactly-once effects, provider deduplication, distributed transactions, rollback completeness, external cancellation, or outbox persistence.
- Generalizing every Lattice run or trigger record.
- Moving persistence, budgets, authority, schedules, or product evidence into the shared core.
- Requiring extraction when the pilot does not simplify Lattice.

## Impact

- **Source**: `src/scheduler/admission*`, `src/orchestration/core.rs`, thin shells/adapters, and focused stores/tests only if admitted.
- **Dependencies**: conditional exact Radicle source and generated locks.
- **Testing**: current crash/recovery, retry, abandonment, owner, persisted compatibility, source, Octet, Cairn, and Nix checks.
