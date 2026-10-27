# Policy Evaluation Cache Specification

## Purpose

Defines the `policy-evaluation-cache` capability.

## Requirements

### Requirement: Canonical key projection

r[contract_backends.policy_cache.projection] The cache MUST key every entry by the BLAKE3 digest of a canonical projection that binds the policy-source identity, the normalized decision inputs, and the capability-context projection. The projection MUST exclude ambient state that does not change the decision.

#### Scenario: Same decision inputs produce the same key

r[contract_backends.policy_cache.projection.equal]
- GIVEN two evaluations whose policy, normalized inputs, and capability context are equal
- WHEN the cache projects both keys
- THEN the keys MUST be equal

#### Scenario: Distinct decision inputs produce distinct keys

r[contract_backends.policy_cache.projection.distinct]
- GIVEN two evaluations whose policy or decision inputs differ
- WHEN the cache projects both keys
- THEN the keys MUST differ

### Requirement: Explicit capacity

r[contract_backends.policy_cache.bound] The cache MUST require an explicit capacity. The cache MUST NOT exceed that capacity and MUST NOT assume an implicit default bound.

#### Scenario: Operator configures a capacity

r[contract_backends.policy_cache.bound.configured]
- GIVEN an operator supplies an explicit capacity
- WHEN the cache is built
- THEN the cache MUST retain at most that many entries

#### Scenario: Insertion at capacity

r[contract_backends.policy_cache.bound.full]
- GIVEN the cache is at capacity
- WHEN an insertion requires a slot
- THEN the cache MUST evict by policy
- AND the entry count MUST stay at or below capacity

### Requirement: Deterministic retention decisions

r[contract_backends.policy_cache.decision] The decision core MUST decide eviction and promotion from supplied values only. It MUST read no clock, memory, or scheduler state. The promotion decision MUST honor an explicit threshold where 0 means strict LRU and 100 means FIFO.

#### Scenario: Strict LRU retention

r[contract_backends.policy_cache.decision.strict_lru]
- GIVEN a threshold of 0 and a node now qualifies as most recently used
- WHEN the core evaluates promotion
- THEN the node MUST promote

#### Scenario: FIFO retention

r[contract_backends.policy_cache.decision.fifo]
- GIVEN a threshold of 100 and any node age
- WHEN the core evaluates promotion
- THEN the node MUST NOT promote

#### Scenario: Core reads environmental state

r[contract_backends.policy_cache.decision.no_env]
- GIVEN an implementation reads a clock, memory, or scheduler state
- WHEN the core purity check runs
- THEN that implementation MUST fail the check

### Requirement: Deferred release ordering

r[contract_backends.policy_cache.deferral] The core MUST return release plans so the shell drops results after it releases the guard. The shell MUST NOT run user destructors inside the synchronization boundary.

#### Scenario: Result evicted under contention

r[contract_backends.policy_cache.deferral.evicted]
- GIVEN a result is evicted while other threads wait
- WHEN the shell processes the release plan
- THEN the guard MUST be released first
- AND the result MUST drop only after release

### Requirement: Policy and freshness authority

r[contract_backends.policy_cache.boundary] A cache hit MUST NOT change policy semantics. The cache MUST NOT claim that a hit proves policy correctness or revocation freshness. Freshness stays consumer-owned. The cross-repo retention contract MUST be recorded in repo docs.

#### Scenario: Over-claim of cache authority

r[contract_backends.policy_cache.boundary.overclaim]
- GIVEN a consumer claims that a cache hit proves policy correctness or revocation freshness
- WHEN the claim boundary is verified
- THEN the claim MUST fail unless the consumer owns it

### Requirement: Fixture coverage

r[contract_backends.policy_cache.verification] Positive, negative, and boundary fixtures MUST cover projection stability, bounded retention, retention decisions, deferred release, and the claim boundary.

#### Scenario: Complete focused matrix passes

r[contract_backends.policy_cache.verification.matrix]
- GIVEN decisions, fixtures, documentation, and dependency evidence are complete
- WHEN focused policy, workspace, Clippy, Cairn, and Nix verification runs
- THEN in-bounds inputs MUST behave as declared
- AND every missing-bound, reversed-watermark, boundary, or over-claim input MUST fail as declared
