## ADDED Requirements

### Requirement: Live observations participate in conflict ordering

r[lattice.workflow_execution.ready_frontier.live_reads] Lattice MUST classify live `File(Read)` and `ImageInput` observations as unresolved conflict participants unless an explicitly admitted conflict fact justifies a stronger classification. Read-only authority, path strings, and absence of declared writes MUST NOT establish purity. `Condition` and `Merge` MUST retain pure in-memory computation semantics. The correction MUST preserve resource lanes, admitted aliases and dependencies, total and lane limits, capacity-one behavior, cancellation, authority, capability roots, and canonical-versus-observed completion ordering. Conflict scheduling orders effects within the admitted workflow; it MUST NOT claim to prevent unrelated external writers or establish immutable snapshots.

#### Scenario: Two-slot product frontier contains a live read and mutation
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.candidates]
- GIVEN a ready shell mutation and a ready live file read or image load without an admitted conflict alias
- WHEN candidates are built through the product classifier and planned with two slots and every named lane
- THEN the planner MUST NOT select both nodes, in either candidate ordering.

#### Scenario: An effect already owns a reservation
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.reservations]
- GIVEN a shell mutation or a live file/image observation owns a product reservation
- WHEN the opposite effect is considered with spare total and lane capacity
- THEN the pending effect MUST be blocked by unresolved conflict in both orderings.

#### Scenario: Immutable computation overlaps an unresolved effect
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.pure]
- GIVEN an unresolved effect and an in-memory Condition or Merge
- WHEN planned together or against an existing reservation with sufficient capacity
- THEN the pure computation MUST remain eligible to overlap the effect in both orderings.

#### Scenario: Existing capacity and cancellation blockers remain authoritative
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.bounds]
- GIVEN a live read with exhausted total capacity, exhausted resource-lane capacity, or cancellation
- WHEN product candidates are planned
- THEN each corresponding existing blocker MUST prevent selection without changing the filesystem or image lane.

#### Scenario: A real subprocess mutation holds an explicit barrier
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.executor]
- GIVEN a real asynchronous shell subprocess owns an intermediate state of a temporary capability-rooted file while an independent live file/image read is ready without an admitted alias
- WHEN the subprocess signals barrier entry and all runnable selected work is polled before explicit release
- THEN the read adapter MUST NOT start until mutation completion, and MUST subsequently observe the final bytes through the real local adapter without sleep-based synchronization.

#### Scenario: Admitted aliases and dependencies retain serialization
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.explicit_order]
- GIVEN the real subprocess/read fixture has either an admitted same-resource alias or an explicit output-reference dependency
- WHEN the mutation is held and released by the barrier
- THEN the read MUST remain ordered after mutation and successful results MUST retain canonical presentation and separate observed completion order.

#### Scenario: Pending live observations respect cancellation
r[lattice.workflow_execution.ready_frontier.live_reads.scenario.cancellation]
- GIVEN a subprocess mutation holds the barrier while an independent live file/image read is pending
- WHEN cancellation is requested before barrier release
- THEN the pending read MUST NOT enter its adapter or be reported as completed.
