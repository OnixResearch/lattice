# Bounded Ready Frontier Delta

## Requirements

### Requirement: Serial execution has a recorded baseline

r[lattice.workflow_execution.ready_frontier.baseline] The change MUST record current serial results, cache behavior, effect preparation, recovery behavior, result order, and benchmark identities before executor changes.

#### Scenario: Diamond workflow runs serially

- GIVEN a valid diamond workflow with independent middle nodes
- WHEN the baseline executor runs
- THEN evidence MUST record serial start order, completion order, result order, and critical-path duration

### Requirement: Choregraph remains readiness authority

r[lattice.workflow_execution.ready_frontier.choregraph] Lattice MUST consume one immutable Choregraph source with current readiness-completeness evidence. No Lattice adapter or scheduler MUST implement an independent dependency-readiness algorithm.

#### Scenario: Scheduler proposes a non-ready node

- GIVEN Choregraph does not report a node executable
- WHEN the Lattice planner evaluates the frontier
- THEN the node MUST NOT appear in the dispatch selection

### Requirement: Executor integrations have one owner

r[lattice.workflow_execution.ready_frontier.integration] Before executor refactoring, Lattice MUST record the selected backend, memoization, effect-attempt, authority, persistence, and recovery owners. Ambiguous or duplicate ownership MUST block implementation.

#### Scenario: Two attempt cores remain active

- GIVEN two production paths can reserve the same node effect attempt
- WHEN integration validation runs
- THEN the ready-frontier change MUST remain blocked

### Requirement: Concurrency policy is typed and bounded

r[lattice.workflow_execution.ready_frontier.capacity] Typed Nickel policy MUST define a positive total in-flight bound and positive named resource-lane bounds. Omitted policy MUST retain named serial compatibility behavior.

#### Scenario: Valid bounded profile is admitted

- GIVEN total and lane bounds are positive and every selected operation maps to supported lanes
- WHEN configuration validation runs
- THEN Lattice MUST produce one canonical capacity-policy identity

#### Scenario: Invalid capacity fails closed

- GIVEN a bound is zero, excessive, missing, contradictory, or assigned to an unknown lane
- WHEN configuration validation runs
- THEN Lattice MUST reject the profile before workflow dispatch

### Requirement: Frontier planning is pure and deterministic

r[lattice.workflow_execution.ready_frontier.planner] The pure planner MUST select a deterministic bounded subset of the ordered Choregraph frontier. Every ready but unselected node MUST have an exact product blocker.

#### Scenario: Capacity fits every ready node

- GIVEN all ready nodes fit total and lane capacity and have no product blockers
- WHEN the planner runs
- THEN it MUST select every ready node in stable admitted order

#### Scenario: Capacity is smaller than the frontier

- GIVEN more unblocked nodes are ready than current capacity permits
- WHEN the planner runs
- THEN it MUST select the stable bounded prefix and classify remaining nodes as capacity-blocked

### Requirement: Effect conflicts fail closed

r[lattice.workflow_execution.ready_frontier.conflicts] Lattice MUST overlap effectful nodes only when admitted resource keys and lane policy report no conflict. Unresolved aliases MUST remain exclusive or fail admission.

#### Scenario: Two nodes target one exclusive resource

- GIVEN two ready effectful nodes resolve to the same exclusive resource key
- WHEN frontier planning runs
- THEN both nodes MUST NOT be selected together

#### Scenario: Resource alias is unresolved

- GIVEN an effectful node lacks a complete conflict key
- WHEN product admission runs
- THEN Lattice MUST block permissive concurrent dispatch

### Requirement: Dispatch preparation precedes effects

r[lattice.workflow_execution.ready_frontier.dispatch] Before each effectful node starts, Lattice MUST validate current backend, credentials, authority, resource facts, and attempt identity. Required durable facts MUST be committed first.

#### Scenario: One prepared node lacks authority

- GIVEN several ready nodes fit capacity but one effectful node lacks current authority
- WHEN dispatch preparation runs
- THEN Lattice MUST start only admitted nodes and MUST record the denied node blocker

#### Scenario: Commit fails before adapter call

- GIVEN effect preparation passes but required attempt persistence fails
- WHEN dispatch runs
- THEN Lattice MUST NOT invoke the effect adapter

### Requirement: Completion releases only real dependents

r[lattice.workflow_execution.ready_frontier.completion] After each terminal observation, Lattice MUST update state and request the next Choregraph frontier. It MUST NOT wait for unrelated nodes in the same dependency level.

#### Scenario: Fast branch releases a dependent

- GIVEN one branch completes while an unrelated branch remains running
- WHEN the completion reducer updates state
- THEN a newly executable dependent MUST be eligible for immediate planning under current capacity

#### Scenario: Dependency remains incomplete

- GIVEN a consumer still has one incomplete dependency
- WHEN another unrelated node completes
- THEN the consumer MUST remain outside the Choregraph frontier

### Requirement: Concurrent results remain deterministic

r[lattice.workflow_execution.ready_frontier.determinism] Public step results MUST use canonical admitted order. Observed completion order MUST be recorded separately when the execution profile retains it.

#### Scenario: Independent nodes finish in reverse order

- GIVEN two independent nodes complete in the reverse of admitted order
- WHEN Lattice returns canonical step results
- THEN result order MUST remain admitted and observed completion order MUST remain distinct

### Requirement: Positive frontier behavior is covered

r[lattice.workflow_execution.ready_frontier.positive] The focused rail MUST cover diamond, fork, join, expression, cache, capacity, and non-conflicting effect behavior.

#### Scenario: Independent nodes overlap

- GIVEN two ready non-conflicting nodes and available capacity for both
- WHEN the concurrent shell runs
- THEN both nodes MUST start without an unrelated dependency barrier

### Requirement: Negative frontier behavior is covered

r[lattice.workflow_execution.ready_frontier.negative] The focused rail MUST cover premature selection, capacity excess, lane excess, conflicts, missing preparation, failures, cancellation, suspension, stale plans, and result-order drift.

#### Scenario: Planner exceeds one lane

- GIVEN selected nodes require more units than one lane permits
- WHEN the negative rail runs
- THEN the candidate planner MUST fail its lane-bound obligation

### Requirement: Recovery preserves each attempt boundary

r[lattice.workflow_execution.ready_frontier.recovery] Recovery MUST classify every prepared or started effect attempt by exact identity. It MUST NOT invent terminal success or automatically repeat an uncertain effect.

#### Scenario: Several attempts are uncertain after a crash

- GIVEN a crash leaves several started attempts without terminal observations
- WHEN recovery runs
- THEN each attempt MUST remain independently uncertain until an admitted observation or operator action resolves it

### Requirement: Planner selection refines readiness

r[lattice.workflow_execution.ready_frontier.refinement] A production-linked proof rail MUST establish that every selected node is Choregraph-executable and within total and lane capacity. It MUST also establish selection when readiness, capacity, conflict, and product admission all permit dispatch.

#### Scenario: Proof fixture adds a non-ready node

- GIVEN a proof fixture selects one node outside the Choregraph frontier
- WHEN the expected-failure proof runs
- THEN proof MUST fail for readiness refinement

### Requirement: Execution modes have bounded comparisons

r[lattice.workflow_execution.ready_frontier.benchmark] Maintained fixtures MUST compare serial, dependency-level, and ready-frontier modes. Evidence MUST record workload identity, limits, outputs, latency, idle time, dispatch overhead, and maximum in-flight work.

#### Scenario: Paper performance is copied as a target

- GIVEN benchmark policy uses accelerator percentages without local measurements
- WHEN evidence validation runs
- THEN the benchmark claim MUST be rejected

### Requirement: Frontier evidence states non-claims

r[lattice.workflow_execution.ready_frontier.evidence] Documentation and evidence MUST distinguish semantic readiness, product dispatchability, planned work, committed attempts, and observed completions.

#### Scenario: Ready node is presented as executed

- GIVEN Choregraph reports a node ready but no Lattice attempt or completion exists
- WHEN evidence is rendered
- THEN it MUST NOT claim that dispatch or execution occurred

### Requirement: Ready-frontier validation is maintained

r[lattice.workflow_execution.ready_frontier.validation] Closeout MUST run focused tests, proof fixtures, crash rails, compatibility checks, Octet, Cairn gates, and relevant Nix checks.

#### Scenario: Serial compatibility is untested

- GIVEN a candidate lacks current capacity-one compatibility evidence
- WHEN lifecycle validation runs
- THEN the candidate MUST fail closeout

### Requirement: Semantic barriers remain intact

r[lattice.workflow_execution.ready_frontier.boundary] Ready-frontier execution MUST NOT bypass workflow approval, orchestration-round, revision, budget, authority, retry, cancellation, or recovery boundaries.

#### Scenario: Next orchestration round lacks approval

- GIVEN all nodes in the current workflow round complete but the successor revision lacks required approval
- WHEN frontier planning runs
- THEN Lattice MUST NOT dispatch the next round
