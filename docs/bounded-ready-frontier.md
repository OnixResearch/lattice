# Bounded ready-frontier dispatch

<!-- r[impl lattice.workflow_execution.ready_frontier.evidence] -->
<!-- r[impl lattice.workflow_execution.ready_frontier.boundary] -->

Lattice dispatches ready workflow nodes without a dependency-level barrier. Choregraph remains the graph admission and readiness authority.

## Source and proof boundary

Lattice pins Choregraph runtime revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07` from its canonical Radicle repository. It separately pins accepted readiness-completeness evidence at revision `b3e08e19750f53bdbcae970cdf58a47a791ed20b`.

The proof revision routes readiness through the verified kernel. The runtime revision uses the preceding direct implementation. Lattice therefore claims bounded frontier compatibility, not direct proof linkage.

At each product dispatch step, Lattice compares the Choregraph runtime frontier with the compatibility runner projection. A mismatch stops dispatch. Focused fixtures also enforce the proof revision and runtime revision as separate immutable identities.

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
- missing preparation;
- invalid policies;
- durable completion-order persistence.

The ignored comparative fixture in `tests/test_ready_frontier_dispatch.rs` compares serial capacity-one execution with two-slot ready-frontier execution on the same independent-effect fixture.

The benchmark is comparative evidence for this fixture only. It is not a general latency claim and does not adopt the hardware-specific claims from arXiv `2608.13757`.
