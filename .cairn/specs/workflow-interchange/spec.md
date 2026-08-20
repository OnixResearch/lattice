# Workflow Interchange Specification

## Purpose

Defines the `workflow-interchange` capability.

## Requirements

### Requirement: Versioned workflow consumer contract

r[lattice.workflow_interchange.contract] Lattice MUST publish a versioned workflow request and observation contract that consumers can use without importing Lattice runtime implementations.

#### Scenario: Independent consumer compiles the contract
r[lattice.workflow_interchange.contract.scenario.independent]
- GIVEN a consumer pins the published Lattice revision
- WHEN it builds only `lattice-workflow-contract`
- THEN the contract MUST compile without the Lattice application crate or standard-library effects.

#### Scenario: Runtime type enters the contract
r[lattice.workflow_interchange.contract.scenario.runtime]
- GIVEN a contract change imports Lattice persistence, process, network, UI, or provider implementations
- WHEN repository validation runs
- THEN the contract candidate MUST fail.

### Requirement: Exact request identity linkage

r[lattice.workflow_interchange.request] Each admitted workflow request MUST bind one profile, idempotency key, consumer effect, attempt, candidate, pipeline revision, workflow policy, graph, platform, control set, and observation policy.

#### Scenario: Equal requests are identified
r[lattice.workflow_interchange.request.scenario.equal]
- GIVEN two requests have equal admitted values
- WHEN request identity framing runs
- THEN both requests MUST receive the same BLAKE3 identity.

#### Scenario: Graph changes after identity
r[lattice.workflow_interchange.request.scenario.graph-change]
- GIVEN a node operation identity changes without a new graph identity
- WHEN request admission runs
- THEN admission MUST reject the request before workflow effects.

### Requirement: Bounded graph capability admission

r[lattice.workflow_interchange.graph] Workflow graphs MUST be ordered, acyclic, bounded, and limited to declared node kinds, effect classes, adapter kinds, controls, and platforms.

#### Scenario: Declared graph is accepted
r[lattice.workflow_interchange.graph.scenario.accepted]
- GIVEN every node and edge matches the profile and every identity is current
- WHEN graph admission runs
- THEN Lattice MAY accept the request for application-owned dispatch.

#### Scenario: Graph is cyclic
r[lattice.workflow_interchange.graph.scenario.cycle]
- GIVEN the request graph contains a dependency cycle
- WHEN graph admission runs
- THEN it MUST reject the graph before application effects.

#### Scenario: Adapter kind is undeclared
r[lattice.workflow_interchange.graph.scenario.adapter]
- GIVEN a node requests an adapter kind outside the profile
- WHEN capability admission runs
- THEN it MUST reject the request before application effects.

### Requirement: Exact lifecycle observation admission

r[lattice.workflow_interchange.observation] Workflow observations MUST preserve request, idempotency, workflow, execution, sequence, node, adapter, receipt, and log identities.

#### Scenario: Linked completion is admitted
r[lattice.workflow_interchange.observation.scenario.completed]
- GIVEN every required node has a linked successful terminal result
- WHEN completion admission runs
- THEN Lattice MAY expose the bounded terminal observation to the consumer.

#### Scenario: Paused execution claims success
r[lattice.workflow_interchange.observation.scenario.paused]
- GIVEN a paused execution is marked terminal or successful
- WHEN observation admission runs
- THEN it MUST reject the inconsistent observation.

#### Scenario: Reconciliation uses another execution
r[lattice.workflow_interchange.observation.scenario.stale]
- GIVEN reconciliation returns an execution identity different from the accepted request
- WHEN observation admission runs
- THEN it MUST reject the stale result without changing consumer state.

### Requirement: Bounded controls and log cursors

r[lattice.workflow_interchange.controls_logs] Control and log requests MUST bind the exact accepted execution and stay within declared capabilities and byte limits.

#### Scenario: Current cancellation is admitted
r[lattice.workflow_interchange.controls_logs.scenario.cancel]
- GIVEN cancellation is supported and the request names the current execution and sequence
- WHEN control admission runs
- THEN Lattice MAY pass the request to its application shell.

#### Scenario: Log read exceeds its bound
r[lattice.workflow_interchange.controls_logs.scenario.log-bound]
- GIVEN a log read requests more bytes than the admitted observation policy
- WHEN log admission runs
- THEN it MUST reject the read before transport.

#### Scenario: Log retention has a gap
r[lattice.workflow_interchange.controls_logs.scenario.gap]
- GIVEN a provider reports truncated or missing retained bytes
- WHEN the observation is emitted
- THEN the gap and truncation facts MUST remain explicit.

### Requirement: Producer fixtures and non-claims

r[lattice.workflow_interchange.evidence] Lattice MUST maintain accepted and rejected producer fixtures plus explicit contract non-claims.

#### Scenario: Negative fixture is removed
r[lattice.workflow_interchange.evidence.scenario.negative]
- GIVEN the fixture set lacks malformed identity, cycle, stale execution, false success, or limit cases
- WHEN the focused contract rail runs
- THEN the contract candidate MUST fail.

#### Scenario: Contract checks pass
r[lattice.workflow_interchange.evidence.scenario.nonclaims]
- GIVEN all focused checks pass
- WHEN evidence is recorded
- THEN claims MUST remain limited to bounded shape, identity, linkage, lifecycle, and log facts.

### Requirement: Versioned workflow exchange envelope

r[lattice.workflow_interchange.transport_envelope] Lattice MUST publish closed versioned transport envelopes for dispatch, observe, control, log-read, and reconciliation operations.

#### Scenario: Independent consumer decodes an exchange
r[lattice.workflow_interchange.transport_envelope.scenario.decode]
- GIVEN a consumer pins the published Lattice revision
- WHEN it encodes and decodes a supported exchange operation
- THEN both sides MUST preserve the exact operation and contract values.

#### Scenario: Unknown operation enters the wire
r[lattice.workflow_interchange.transport_envelope.scenario.unknown]
- GIVEN a wire value names an unsupported operation variant
- WHEN envelope decoding runs
- THEN it MUST reject the value before handler invocation.

### Requirement: Deterministic exchange identity

r[lattice.workflow_interchange.transport_identity] Every request and successful response MUST bind a deterministic tagged BLAKE3 identity to its complete typed content.

#### Scenario: Equal requests are framed
r[lattice.workflow_interchange.transport_identity.scenario.equal]
- GIVEN two equal admitted requests
- WHEN identity framing runs
- THEN both requests MUST receive the same identity.

#### Scenario: Body changes after identity
r[lattice.workflow_interchange.transport_identity.scenario.changed]
- GIVEN a request body changes without a new identity
- WHEN transport admission runs
- THEN it MUST reject the request before handler invocation.

### Requirement: Bounded Unix transport

r[lattice.workflow_interchange.transport_bounds] The Unix transport MUST use explicit positive frame, log, connection, and deadline limits.

#### Scenario: Bounded request arrives
r[lattice.workflow_interchange.transport_bounds.scenario.admitted]
- GIVEN an admitted peer sends one complete frame within every selected limit
- WHEN the server reads the frame
- THEN it MAY pass one admitted command to the handler.

#### Scenario: Frame exceeds its limit
r[lattice.workflow_interchange.transport_bounds.scenario.oversized]
- GIVEN a frame header declares more bytes than the selected limit
- WHEN frame admission runs
- THEN it MUST reject the frame without allocating the declared body.

#### Scenario: Peer stalls
r[lattice.workflow_interchange.transport_bounds.scenario.timeout]
- GIVEN a peer does not complete a frame before the selected read deadline
- WHEN socket I/O runs
- THEN the server MUST close that exchange without handler invocation.

### Requirement: Explicit local peer policy

r[lattice.workflow_interchange.transport_peer] The Unix server MUST compare observed peer credentials with an explicit admitted peer policy before request decoding.

#### Scenario: Peer identity is admitted
r[lattice.workflow_interchange.transport_peer.scenario.admitted]
- GIVEN the operating system reports a peer user identity allowed by policy
- WHEN connection admission runs
- THEN the server MAY read one bounded request.

#### Scenario: Peer identity differs
r[lattice.workflow_interchange.transport_peer.scenario.denied]
- GIVEN the reported peer user identity is not allowed
- WHEN connection admission runs
- THEN the server MUST reject the connection before reading a request body.

### Requirement: Exact handler boundary

r[lattice.workflow_interchange.transport_handler] The transport MUST invoke only an explicit application-owned workflow exchange handler with values that passed contract and transport admission.

#### Scenario: Dispatch request is valid
r[lattice.workflow_interchange.transport_handler.scenario.dispatch]
- GIVEN a valid profile and workflow request
- WHEN dispatch admission succeeds
- THEN the handler MUST receive the exact admitted values.

#### Scenario: Control request is stale
r[lattice.workflow_interchange.transport_handler.scenario.stale]
- GIVEN a control request names a stale sequence or another execution
- WHEN request admission runs
- THEN the transport MUST reject it before handler invocation.

### Requirement: Bounded response and log admission

r[lattice.workflow_interchange.transport_response] The transport MUST validate response identity, contract linkage, frame size, and log-byte linkage before returning data to a client.

#### Scenario: Linked log response arrives
r[lattice.workflow_interchange.transport_response.scenario.log]
- GIVEN a handler returns one admitted observation and the exact bounded retained log bytes
- WHEN response admission runs
- THEN the transport MAY return the response.

#### Scenario: Log bytes exceed the request
r[lattice.workflow_interchange.transport_response.scenario.oversized]
- GIVEN a handler returns more log bytes than requested
- WHEN response admission runs
- THEN the transport MUST replace the result with a safe rejection.

### Requirement: Disconnect ambiguity

r[lattice.workflow_interchange.transport_ambiguity] The client MUST distinguish failure before a complete request write from failure after the request write.

#### Scenario: Connection fails before write completion
r[lattice.workflow_interchange.transport_ambiguity.scenario.not-accepted]
- GIVEN the client cannot finish the request frame
- WHEN failure classification runs
- THEN it MUST report `not_accepted`.

#### Scenario: Connection fails before a complete response
r[lattice.workflow_interchange.transport_ambiguity.scenario.unknown]
- GIVEN the client completed the request write but did not receive a complete valid response
- WHEN failure classification runs
- THEN it MUST report `unknown_after_write` and MUST NOT retry automatically.

### Requirement: Transport evidence and non-claims

r[lattice.workflow_interchange.transport_evidence] Maintained checks MUST include positive and negative producer fixtures, strict lint gates, and explicit non-claims.

#### Scenario: Only happy-path fixtures exist
r[lattice.workflow_interchange.transport_evidence.scenario.negative]
- GIVEN the fixture set lacks malformed, oversized, stale, denied-peer, timeout, or disconnect cases
- WHEN repository validation runs
- THEN the transport candidate MUST fail.

#### Scenario: Transport checks pass
r[lattice.workflow_interchange.transport_evidence.scenario.nonclaims]
- GIVEN all maintained checks pass
- WHEN evidence is recorded
- THEN claims MUST remain limited to the checked local protocol, identity, linkage, peer, and bound facts.
