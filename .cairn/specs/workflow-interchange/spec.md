# Workflow Interchange Specification

## Purpose

Defines the `workflow-interchange` capability.

## Requirements

### Requirement: Versioned workflow consumer contract

r[lattice.workflow_interchange.contract] Lattice MUST publish a versioned workflow request and observation contract that consumers can use without importing Lattice runtime implementations.

#### Scenario: Independent consumer compiles the contract

- GIVEN a consumer pins the published Lattice revision
- WHEN it builds only `lattice-workflow-contract`
- THEN the contract MUST compile without the Lattice application crate or standard-library effects.

#### Scenario: Runtime type enters the contract

- GIVEN a contract change imports Lattice persistence, process, network, UI, or provider implementations
- WHEN repository validation runs
- THEN the contract candidate MUST fail.

### Requirement: Exact request identity linkage

r[lattice.workflow_interchange.request] Each admitted workflow request MUST bind one profile, idempotency key, consumer effect, attempt, candidate, pipeline revision, workflow policy, graph, platform, control set, and observation policy.

#### Scenario: Equal requests are identified

- GIVEN two requests have equal admitted values
- WHEN request identity framing runs
- THEN both requests MUST receive the same BLAKE3 identity.

#### Scenario: Graph changes after identity

- GIVEN a node operation identity changes without a new graph identity
- WHEN request admission runs
- THEN admission MUST reject the request before workflow effects.

### Requirement: Bounded graph capability admission

r[lattice.workflow_interchange.graph] Workflow graphs MUST be ordered, acyclic, bounded, and limited to declared node kinds, effect classes, adapter kinds, controls, and platforms.

#### Scenario: Declared graph is accepted

- GIVEN every node and edge matches the profile and every identity is current
- WHEN graph admission runs
- THEN Lattice MAY accept the request for application-owned dispatch.

#### Scenario: Graph is cyclic

- GIVEN the request graph contains a dependency cycle
- WHEN graph admission runs
- THEN it MUST reject the graph before application effects.

#### Scenario: Adapter kind is undeclared

- GIVEN a node requests an adapter kind outside the profile
- WHEN capability admission runs
- THEN it MUST reject the request before application effects.

### Requirement: Exact lifecycle observation admission

r[lattice.workflow_interchange.observation] Workflow observations MUST preserve request, idempotency, workflow, execution, sequence, node, adapter, receipt, and log identities.

#### Scenario: Linked completion is admitted

- GIVEN every required node has a linked successful terminal result
- WHEN completion admission runs
- THEN Lattice MAY expose the bounded terminal observation to the consumer.

#### Scenario: Paused execution claims success

- GIVEN a paused execution is marked terminal or successful
- WHEN observation admission runs
- THEN it MUST reject the inconsistent observation.

#### Scenario: Reconciliation uses another execution

- GIVEN reconciliation returns an execution identity different from the accepted request
- WHEN observation admission runs
- THEN it MUST reject the stale result without changing consumer state.

### Requirement: Bounded controls and log cursors

r[lattice.workflow_interchange.controls_logs] Control and log requests MUST bind the exact accepted execution and stay within declared capabilities and byte limits.

#### Scenario: Current cancellation is admitted

- GIVEN cancellation is supported and the request names the current execution and sequence
- WHEN control admission runs
- THEN Lattice MAY pass the request to its application shell.

#### Scenario: Log read exceeds its bound

- GIVEN a log read requests more bytes than the admitted observation policy
- WHEN log admission runs
- THEN it MUST reject the read before transport.

#### Scenario: Log retention has a gap

- GIVEN a provider reports truncated or missing retained bytes
- WHEN the observation is emitted
- THEN the gap and truncation facts MUST remain explicit.

### Requirement: Producer fixtures and non-claims

r[lattice.workflow_interchange.evidence] Lattice MUST maintain accepted and rejected producer fixtures plus explicit contract non-claims.

#### Scenario: Negative fixture is removed

- GIVEN the fixture set lacks malformed identity, cycle, stale execution, false success, or limit cases
- WHEN the focused contract rail runs
- THEN the contract candidate MUST fail.

#### Scenario: Contract checks pass

- GIVEN all focused checks pass
- WHEN evidence is recorded
- THEN claims MUST remain limited to bounded shape, identity, linkage, lifecycle, and log facts.
