# Bounded ready-frontier dispatch

<!-- r[impl lattice.workflow_execution.ready_frontier.evidence] -->
<!-- r[impl lattice.workflow_execution.ready_frontier.boundary] -->

Lattice dispatches ready workflow nodes without a dependency-level barrier. Choregraph remains the graph admission and readiness authority.

## Source and proof boundary

Lattice pins Choregraph runtime revision `fcbae0aa17bc0eab1345529adb9ff104a46540ef` from its canonical Radicle repository. Its production readiness routes through the verified kernel, and its readiness-completeness evidence is accepted at that same revision.

The runtime and readiness-proof revisions therefore coincide. Lattice still compares the Choregraph runtime frontier with the compatibility runner projection and claims bounded frontier compatibility, not whole-product correctness.

At each product dispatch step, Lattice compares the Choregraph runtime frontier with the compatibility runner projection. A mismatch stops dispatch. Focused fixtures enforce the runtime and readiness-proof identities independently, even though they now name the same revision.

The accepted proof establishes Choregraph readiness completeness under its stated assumptions. It does not prove Lattice effects, persistence, recovery, or scheduling policy.

## Ownership

Choregraph owns:

- graph admission;
- deterministic dependency readiness;
- lifecycle transition admission;
- canonical machine identity.

Lattice owns:

- total execution capacity;
- named resource-lane capacity;
- memoization and cache identity;
- authority and credential preparation;
- backend and attempt preparation;
- resource-alias conflict facts;
- effect dispatch;
- completion observation;
- persistence and recovery policy.

The pure planner is in `crates/lattice-core/src/frontier/planner.rs`. The imperative runtime is in `src/engine/frontier/runtime.rs`.

## Policy

A capacity policy has one positive total slot count and positive named lane counts. No lane can exceed the total count.

The supported lanes are:

- `pure`;
- `process`;
- `network`;
- `filesystem`;
- `image`;
- `agent`;
- `operation`;
- `subworkflow`;
- `approval`.

If the application config omits `max_parallelism` and lane policy, Lattice uses one slot. Any non-serial profile must provide every named lane. The generated Nickel config opts in to four slots and validates all named lanes with contracts.

An invalid, partial, contradictory, zero, excessive, or unknown-lane policy fails closed to serial execution at application composition roots. Direct executor construction rejects invalid policies.

## Dispatch

For each observation:

1. Ask Choregraph for pending nodes whose dependencies are complete.
2. Compare the result with the compatibility projection.
3. Prepare immutable inputs, expression values, policy, and conflict facts.
4. Select a stable bounded subset with the pure planner.
5. Reserve total, lane, and conflict capacity before an effect starts.
6. Start selected work.
7. Observe the first completion.
8. Reduce that completion through Choregraph.
9. Recompute readiness immediately.

A fast branch can release its child while an unrelated slow branch is still running.

## Conflicts and blockers

A resolved resource alias conflicts only with the same alias. An unresolved effect can run, but it cannot overlap another effect. Pure work can overlap an unresolved effect.

`Condition` and `Merge` compute over admitted in-memory values and remain pure.
`File(Read)` and `ImageInput` observe live, mutable host state; like mutations,
they default to unresolved conflict participants. A read-only authority grant
does not establish commutativity. Their filesystem and image resource lanes,
capability roots, and authorization requirements remain unchanged.

An explicitly admitted resource-alias fact can refine conflict scheduling. The
default classification does not infer independence from read-only access, a path
string, or an absence of declared writes. A future immutable, content-bound
snapshot profile would need its own admitted evidence; resolved expressions,
aliases, overlapping roots, and host identity rules prevent a path-only shortcut.

This ordering is scoped to effects within the admitted workflow. It does not
prevent unrelated external processes from changing files and does not turn a
live read into an immutable snapshot.

The planner records these blockers:

- total capacity;
- resource-lane capacity;
- unknown lane;
- unresolved conflict;
- resolved effect conflict;
- missing backend;
- missing authority;
- missing credential;
- missing attempt;
- suspension;
- cancellation;
- terminal state.

## Ordering and evidence

`StepResult` values stay in canonical graph order. `Executor::observed_completion_order` records terminal observations separately.

Durable `RunDetailRecord` values preserve both forms:

- `nodes` is canonical presentation order;
- `observed_completion_order` is actual observation order.

Old records remain readable because the completion-order field has a default.

## Recovery boundary

Operation attempts still reserve durable identity before effect dispatch. Post-effect persistence failures remain `unknown` outcomes and require the existing recovery policy. Ready-frontier scheduling does not convert unknown outcomes into retries.

Approval workflows retain their durable suspension path. Workflows without approval gates use ready-frontier dispatch in the headless and TUI composition roots.

## Evidence

Focused positive and negative tests are in `tests/test_ready_frontier_dispatch.rs` and the pure planner module.

They cover:

- capacity-one compatibility;
- stable canonical result order;
- distinct observed completion order;
- no dependency-level barrier;
- total and lane bounds;
- resolved conflicts;
- unresolved conflicts;
- live file/image reads against mutations, in both candidate and active-reservation orderings;
- barrier-controlled subprocess mutations with real capability-rooted file/image adapters;
- pure `Condition`/`Merge` overlap and cancellation of pending live reads;
- missing preparation;
- invalid policies;
- durable completion-order persistence.

The ignored comparative fixture in `tests/test_ready_frontier_dispatch.rs` compares serial capacity-one execution with two-slot ready-frontier execution on the same independent-effect fixture.

The benchmark is comparative evidence for this fixture only. It is not a general latency claim and does not adopt the hardware-specific claims from arXiv `2608.13757`.
