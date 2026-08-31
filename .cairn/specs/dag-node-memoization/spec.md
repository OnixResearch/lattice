# Dag Node Memoization Specification

## Purpose

Defines the `dag-node-memoization` capability.

## Requirements

### Requirement: Canonical key projection

r[lattice.node_result_cache.projection] The cache MUST key every entry by the BLAKE3 digest of a canonical projection over node identity, operation-contract revision, normalized inputs, and optional capability context. The projection MUST exclude ambient state that does not change the node result.

#### Scenario: Same node request projects to the same key

r[lattice.node_result_cache.projection.scenario.equal]
- GIVEN two node requests with equal identity, contract revision, inputs, and context
- WHEN the cache projects both keys
- THEN the keys MUST be equal

#### Scenario: Distinct node requests project to distinct keys

r[lattice.node_result_cache.projection.scenario.distinct]
- GIVEN two node requests whose identity, contract revision, or inputs differ
- WHEN the cache projects both keys
- THEN the keys MUST differ

### Requirement: Explicit capacity

r[lattice.node_result_cache.bound] The cache MUST require an explicit capacity. The cache MUST NOT exceed that capacity and MUST NOT assume an implicit default bound.

#### Scenario: Operator configures a capacity

r[lattice.node_result_cache.bound.scenario.configured]
- GIVEN an operator supplies an explicit capacity
- WHEN the cache is built
- THEN the cache MUST retain at most that many entries

#### Scenario: Insertion at capacity

r[lattice.node_result_cache.bound.scenario.full]
- GIVEN the cache is at capacity
- WHEN an insertion requires a slot
- THEN the cache MUST evict by policy
- AND the entry count MUST stay at or below capacity

### Requirement: Deterministic retention decisions

r[lattice.node_result_cache.decision] The decision core MUST decide eviction and promotion from supplied values only. It MUST read no clock, memory, or scheduler state. The promotion decision MUST honor an explicit threshold where 0 means strict LRU and 100 means FIFO.

#### Scenario: Strict LRU retention

r[lattice.node_result_cache.decision.scenario.strict_lru]
- GIVEN a threshold of 0 and an entry now qualifies as most recently used
- WHEN the core evaluates promotion
- THEN the entry MUST promote

#### Scenario: FIFO retention

r[lattice.node_result_cache.decision.scenario.fifo]
- GIVEN a threshold of 100 and any entry age
- WHEN the core evaluates promotion
- THEN the entry MUST NOT promote

### Requirement: Deferred release ordering

r[lattice.node_result_cache.deferral] The core MUST return release plans so the shell drops results after it releases the guard. The shell MUST NOT run user destructors inside the synchronization boundary.

#### Scenario: Result evicted under contention

r[lattice.node_result_cache.deferral.scenario.evicted]
- GIVEN a result is evicted while other threads wait
- WHEN the shell processes the release plan
- THEN the guard MUST be released first
- AND the result MUST drop only after release

### Requirement: Executor authority

r[lattice.node_result_cache.boundary] A cache hit MUST NOT change workflow semantics. The cache MUST NOT claim that a hit proves node correctness or run freshness. The cross-repo retention contract MUST be recorded in repo docs.

#### Scenario: Over-claim of cache authority

r[lattice.node_result_cache.boundary.scenario.overclaim]
- GIVEN a consumer claims that a cache hit proves node correctness
- WHEN the claim boundary is verified
- THEN the claim MUST fail unless the executor owns it

### Requirement: Fixture coverage

r[lattice.node_result_cache.verification] Positive, negative, and boundary fixtures MUST cover projection stability, bounded retention, retention decisions, deferred release, and the claim boundary.

#### Scenario: Complete focused matrix passes

r[lattice.node_result_cache.verification.scenario.matrix]
- GIVEN decisions, fixtures, documentation, and dependency evidence are complete
- WHEN focused workspace, Clippy, Cairn, and Nix verification runs
- THEN in-bounds inputs MUST behave as declared
- AND every missing-bound, reversed-watermark, boundary, or over-claim input MUST fail as declared
