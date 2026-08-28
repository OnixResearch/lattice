# Incremental DAG Recomputation Pilot Delta

## ADDED Requirements

### Requirement: The incremental candidate remains test-only

r[lattice.incremental_pilot.boundary] The pilot MUST NOT change production scheduling, execution, persistence, retry, authority, cache, or workflow semantics.

#### Scenario: The pilot runs

- GIVEN a validated fixture workflow and explicit pilot policy
- WHEN the candidate and full oracle run
- THEN they MUST run through test-only paths with no production dispatch

#### Scenario: Pilot success requests production use

- GIVEN the pilot meets its selected policy
- WHEN production adoption is proposed
- THEN a separate Cairn MUST define production state, effects, migration, rollback, and evidence

### Requirement: The baseline binds current behavior

r[lattice.incremental_pilot.baseline] The pilot MUST bind current full recomputation and node-result-cache behavior to exact implementation, graph, operation, input, authority, and corpus identities.

#### Scenario: Baseline inputs are complete

- GIVEN every baseline identity and observation is present
- WHEN baseline admission runs
- THEN it MUST admit one comparable baseline cohort

#### Scenario: A baseline identity is stale

- GIVEN the baseline cites a different graph, operation, policy, or implementation identity
- WHEN comparison begins
- THEN the pilot MUST reject the baseline

### Requirement: The pilot projection uses explicit dependencies

r[lattice.incremental_pilot.projection] The pure projection MUST contain exact nodes, edges, operation contracts, inputs, capability context, policy, and declared changed facts.

#### Scenario: A valid graph is projected

- GIVEN one acyclic bounded workflow with complete declared dependencies
- WHEN projection validation runs
- THEN it MUST produce one deterministic canonical projection

#### Scenario: Dependencies are incomplete

- GIVEN the graph has a missing node, cycle, duplicate edge, or hidden dependency claim
- WHEN projection validation runs
- THEN it MUST reject the graph before invalidation

### Requirement: Invalidation is deterministic and bounded

r[lattice.incremental_pilot.candidate] The candidate MUST follow admitted dependency edges from changed identities and MUST stop at explicit traversal and result bounds.

#### Scenario: One source changes

- GIVEN one admitted source identity changes in an acyclic graph
- WHEN invalidation runs
- THEN it MUST return exactly the bounded reachable affected nodes in canonical order

#### Scenario: A bound is exceeded

- GIVEN the affected frontier exceeds a declared node or edge bound
- WHEN invalidation runs
- THEN it MUST return resource exhaustion without a partial adopt decision

### Requirement: Full recomputation is an independent oracle

r[lattice.incremental_pilot.oracle] Every candidate case MUST compare with complete recomputation that does not use the candidate invalidation result.

#### Scenario: Candidate and oracle agree

- GIVEN both paths receive equal explicit workflow facts
- WHEN outputs are compared
- THEN every declared result identity and value MUST agree

#### Scenario: Candidate and oracle diverge

- GIVEN any result, terminal class, or output identity differs
- WHEN comparison runs
- THEN the pilot MUST reject adoption for that cohort

### Requirement: The corpus covers representative and invalid behavior

r[lattice.incremental_pilot.corpus] The corpus MUST include chain, diamond, fan-out, fan-in, deep, sparse, multi-source, no-change, stale, cyclic, missing, overflow, and crossed-authority cases.

#### Scenario: The complete corpus runs

- GIVEN all required positive and negative fixtures are present
- WHEN corpus validation runs
- THEN every fixture MUST produce its declared candidate, oracle, or rejection result

### Requirement: Typed policy owns measurements and thresholds

r[lattice.incremental_pilot.measurement] Typed Nickel policy MUST own corpus requirements, resource bounds, benefit thresholds, overhead limits, and divergence rejection.

#### Scenario: Structural benefit is measured

- GIVEN candidate and baseline runs bind the same corpus and identities
- WHEN measurement runs
- THEN it MUST record bounded visited, planned, reused, divergent, duration, and resource facts

#### Scenario: A threshold is absent

- GIVEN the policy omits a required acceptance value
- WHEN decision admission runs
- THEN it MUST return insufficient evidence

### Requirement: The pilot returns a closed decision

r[lattice.incremental_pilot.decision] The pilot MUST return adopt-candidate, reject-candidate, or insufficient-evidence with exact policy and evidence identities.

#### Scenario: All policy conditions pass

- GIVEN parity, corpus, benefit, overhead, bound, and evidence conditions pass
- WHEN the decision runs
- THEN it MAY return adopt-candidate without changing production behavior

#### Scenario: Evidence is incomplete

- GIVEN any required corpus, measurement, identity, or negative case is missing
- WHEN the decision runs
- THEN it MUST return insufficient-evidence

### Requirement: Verification preserves claim limits

r[lattice.incremental_pilot.verification] The focused rail MUST cover all pilot decisions and MUST state that corpus parity is not general correctness.

#### Scenario: Focused pilot validation runs

- GIVEN the policy, candidate, oracle, corpus, measurements, and decisions are present
- WHEN focused validation runs
- THEN all positive and negative cases MUST pass with bounded non-claims
