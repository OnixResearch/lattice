## Context

Choregraph owns graph admission and node readiness. Lattice owns runtime scheduling, resources, authorization, attempts, persistence, retries, recovery, and effects.

The current executor validates each node against shared graph decisions, but it still executes one admitted topological list serially. This is stronger synchronization than the graph requires.

The design adapts the minimum-synchronization pattern from *A Barrier-Free Synchronization Algorithm for Multi-Engine AI Accelerators*. It uses durable node state instead of hardware counters.

## Success Contract

Completion requires complete workflow dependencies, proved Choregraph readiness, pure bounded frontier planning, safe effect preparation, deterministic result projection, negative and crash evidence, and comparative benchmarks.

These results are false completion:

- concurrent execution before expression dependencies are explicit;
- level-by-level batches that retain an unrelated barrier;
- starting effects before current authority and attempt persistence;
- claiming no over-synchronization without listing capacity and product blockers;
- allowing completion order to change canonical results;
- treating task-count success as scheduler proof;
- removing semantic approval or recovery boundaries.

## Ownership Flow

```text
admitted workflow and persisted state
  -> Choregraph readiness decision
  -> Lattice pure frontier planner
  -> Lattice product admission and effect preparation
  -> committed dispatch or attempt facts
  -> Lattice concurrent shell
  -> observed completion facts
  -> Lattice state reducer
  -> Choregraph transition and next readiness decision
```

Choregraph does not learn about worker pools, providers, credentials, files, processes, retries, or storage.

## Pure Frontier Planner

The planner receives explicit values:

- the ordered Choregraph ready frontier;
- current node and attempt state;
- a positive total capacity;
- named resource-lane capacities;
- admitted operation and backend facts;
- prepared effect conflict keys;
- suspension, cancellation, and terminal facts;
- stable node-order facts.

It returns:

```text
FrontierPlan {
  selected_nodes,
  blocked_nodes,
}

BlockedNode {
  node_id,
  reason,
}
```

Reasons are typed and bounded. They distinguish total capacity, resource capacity, unresolved conflict, missing backend, missing authority, missing credential, missing attempt, suspension, cancellation, and terminal state.

The planner performs no I/O and does not read time. It does not create attempts or execute effects.

## Minimum Dispatch Condition

A node is selectable only when Choregraph reports it executable. Lattice then applies product blockers.

```text
dispatchable(node)
  = choregraph_executable(node)
  && capacity_available(node)
  && resource_lane_available(node)
  && no_effect_conflict(node)
  && current_product_admission(node)
  && dispatch_preparation_complete(node)
```

If every condition holds, the planner selects the node without waiting for unrelated work. If capacity is smaller than the ready frontier, stable admitted order selects the bounded prefix.

This is not a fairness theorem. It is one deterministic selection rule over explicit current facts.

## Capacity and Resource Lanes

Typed Nickel configuration defines a positive total in-flight limit and positive named lane limits. Code uses named constants for compatibility values and every non-trivial bound.

Operation contracts select required lanes. Examples can include bounded processes, provider calls, filesystem effects, and local pure work.

An unknown effectful operation cannot enter a permissive default lane. It fails admission or uses an explicit conservative exclusive lane.

## Effect Conflict Boundary

Two effectful nodes can overlap only when their admitted operation facts provide non-conflicting resource keys and lane policy permits overlap.

Exact local file targets can use capability-root and logical-target identities after expression preparation. Dynamic or unresolved resource aliases remain exclusive.

The system does not infer commutativity from different node identifiers or different adapter names.

## Imperative Shell

The shell owns concurrent task handles, cancellation tokens, adapter calls, and completion observation.

For each selected effectful node, the shell completes current backend, credential, authority, conflict, and attempt preparation. It commits required durable facts before it invokes the adapter.

Each task receives immutable inputs and an exact expression context. Tasks do not share mutable workflow output maps.

On completion, the shell persists or records the typed observation. The reducer updates state. Then Lattice asks Choregraph for the next ready frontier.

The shell does not wait for every node in the current dependency level.

## Determinism and Result Order

External completion order is nondeterministic. Lattice records observed completion sequence through existing event or attempt ordering.

Public step results remain ordered by admitted topological order and stable node identity. Cache hits use the same terminal reduction path as executed nodes.

A concurrent schedule must not change canonical workflow outputs where the complete dependency graph and operation contracts admit overlap.

## Failure, Cancellation, and Recovery

A failed, cancelled, or suspended node follows current Choregraph and Lattice policy. Unrelated admitted branches can continue only when current product policy permits them.

A crash can leave several effect attempts uncertain. Recovery treats each attempt by exact identity. It does not collapse them into one run-wide success or automatically repeat unknown effects.

Cancellation targets each owned in-flight task. Provider cancellation remains an observation, not proof of external reversal.

## Proof and State-Space Evidence

Add a production-linked proof for the pure planner:

```text
selected(node) => choregraph_executable(node)
selected_count <= available_total_capacity
selected lane use <= available lane capacity
blocked by no unrelated dependency barrier
```

The final property means an eligible node with available capacity and no product blocker appears in the selected set.

Expected-failure fixtures add a non-ready node, exceed capacity, ignore a lane, omit an unblocked node, or launch after denial.

The proof does not cover Tokio, adapters, providers, storage, or the operating system. Shell tests and crash fixtures cover those boundaries.

## Benchmark and Rollout

Compare three modes:

1. serial topological execution;
2. dependency-level barrier execution;
3. bounded ready-frontier execution.

Fixtures include diamond graphs, long and short forks, joins, expression dependencies, cache hits, capacity pressure, resource conflicts, failures, cancellation, suspension, and crash windows.

Record critical-path latency, idle time, dispatch overhead, maximum in-flight work, and result identities. Do not set the paper's Trainium percentages as acceptance targets.

Configuration omission retains serial compatibility during rollout. Explicit reviewed profiles enable larger capacity after correctness and recovery evidence passes.

## Claim Boundary

This change can establish bounded scheduler selection and observed runtime facts. It does not prove external effect correctness, provider fairness, distributed progress, exactly-once execution, or universal performance improvement.
