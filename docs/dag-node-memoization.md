# Bounded DAG Node Memoization

Lattice provides an advisory cache for immutable DAG node results. The cache
never owns workflow execution policy or node correctness.

## Key projection

Each key is a BLAKE3 digest over a tagged, length-framed projection. The
projection contains these fields:

- node identity
- operation-contract revision
- normalized inputs
- optional capability context

The projection excludes clocks, process state, scheduler state, and other
ambient values. Callers must include every supplied value that can change the
node result.

## Explicit bounds

`Config` has no default capacity. Admission rejects these configurations:

- missing capacity
- zero capacity
- a promotion threshold above 100
- a low watermark above the high watermark
- a high watermark above capacity

The store cannot retain more entries than its admitted capacity. If the active
count exceeds the high watermark, the store trims to the low watermark.

## Retention decisions

The pure core makes all retention decisions from supplied values.

- Threshold `0` promotes every hit and provides strict LRU behavior.
- Threshold `100` never promotes a hit and provides FIFO behavior.
- A middle threshold promotes after that many accesses since the prior
  promotion.

Eviction ties use insertion generation and then the cache key. The decision is
stable and does not read a clock.

## Imperative store

`Memoizer` owns the mutex, generation counters, bounded map, and value release. It executes pure eviction, promotion, slot, and watermark decisions.

The store removes values while it holds the guard. It drops removed values only
after the guard is released. This prevents user destructors from running in the
synchronization boundary.

`get_or_execute` returns a cached value on a hit. On a miss, it calls the
supplied executor closure without changing the executor result or error. A
concurrent miss can still execute more than once because the cache is advisory.

## Contract status

The design follows the planned `bounded-cache` retention contract. No published
`bounded-cache` repository revision is available, so this is a policy reference
only. Lattice can adopt a published revision after a separate compatibility
review.

The implementation also reviewed `adanil-code/LRUHashTable` at revision
`f0e436326d36b6909e67c93de387edf19c14f5c1` as an Apache-2.0 design reference.
No source was copied.

## Non-claims

<!-- r[impl lattice.node_result_cache.boundary.scenario.overclaim] -->
A cache hit does not prove node correctness, input freshness, workflow
correctness, exact-once execution, scheduler safety, or release eligibility.
The executor retains all workflow and effect authority.
