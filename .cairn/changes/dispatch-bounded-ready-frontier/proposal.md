## Why

Lattice admits DAG workflows through Choregraph, but its main executor still walks one topological list and awaits every node in sequence.

Independent nodes cannot overlap. A fast branch also cannot release its dependents until unrelated earlier work finishes.

After workflow dependencies become complete and Choregraph proves exact readiness, Lattice can replace incidental serialization with bounded dependency-driven dispatch.

## What Changes

- Add a pure ready-frontier planner over Choregraph readiness, product blockers, resource capacity, and stable ordering.
- Add typed Nickel concurrency and resource-lane policy with positive named bounds.
- Dispatch every eligible unblocked node that fits current capacity without waiting for unrelated nodes.
- Recompute affected readiness after each persisted terminal observation instead of waiting for a dependency level.
- Prepare authority, backend, credentials, effect conflict facts, and durable attempts before effectful dispatch.
- Preserve unknown outcomes and existing retry, cancellation, suspension, cache, replay, and recovery boundaries.
- Keep public result order canonical while recording observed completion order separately.
- Add production-linked scheduler refinement evidence, negative fixtures, crash tests, and serial-versus-frontier benchmarks.

## Success Criteria

- No node starts before Choregraph reports it executable.
- An eligible node does not wait for unrelated work when capacity exists and no product blocker applies.
- Effectful nodes have current authorization and a committed attempt before execution.
- Conflicting or unresolved effects do not run concurrently.
- Capacity-one mode preserves current serial semantic fixtures.
- Concurrent completion order does not change canonical result presentation.
- Crash recovery does not invent success or automatically repeat an uncertain effect.

## Dependencies

- Archive and publish Choregraph `prove-readiness-completeness` at an immutable revision.
- Archive Lattice `make-workflow-dependencies-explicit`.
- Complete `orchestrate-admitted-execution-backends`, or record a reviewed non-overlap disposition before dispatcher changes.
- Reconcile current memoization and effect-attempt pilots before final executor integration.

## Non-Goals

- Removing approval, orchestration-round, revision, budget, or recovery boundaries.
- Proving external effect success, provider fairness, distributed scheduling, exactly-once execution, or universal latency improvement.
- Adding cyclic Choregraph machines or Trainium semaphore allocation.
- Moving runtime scheduling, capacity, persistence, or authority into Choregraph.

## Impact

- **Core**: pure frontier selection, blocker classification, resource-lane admission, and canonical result planning.
- **Shell**: concurrent task ownership, completion reduction, cancellation, and durable effect preparation.
- **Configuration**: typed Nickel concurrency profiles and named limits.
- **Evidence**: refinement proof, state-machine fixtures, crash boundaries, and comparative benchmarks.
