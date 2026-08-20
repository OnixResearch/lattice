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

### Requirement: Recover dispatch by original request

r[lattice.workflow_interchange.transport_dispatch_recovery] Lattice MUST support a closed recovery command that carries the original admitted capability profile and workflow request without redispatch authority.

#### Scenario: Dispatch response is lost
r[lattice.workflow_interchange.transport_dispatch_recovery.scenario.lost]
- GIVEN a client completed one dispatch write but received no complete valid response
- WHEN it submits the same admitted workflow request for recovery
- THEN the handler MAY look up the exact request without dispatching another workflow.

#### Scenario: Recovery request changed
r[lattice.workflow_interchange.transport_dispatch_recovery.scenario.changed]
- GIVEN a recovery request differs from the original identity-bound request
- WHEN recovery admission runs
- THEN the transport MUST reject it before handler invocation.

### Requirement: Recovery output remains exactly linked

r[lattice.workflow_interchange.transport_dispatch_recovery_result] Dispatch recovery MUST return only the exact linked acceptance or a closed rejection.

#### Scenario: Acceptance is found
r[lattice.workflow_interchange.transport_dispatch_recovery_result.scenario.accepted]
- GIVEN durable state contains the exact admitted request and acceptance
- WHEN recovery runs
- THEN the handler MAY return that linked acceptance.

#### Scenario: Observation lacks the acceptance
r[lattice.workflow_interchange.transport_dispatch_recovery_result.scenario.observed]
- GIVEN a handler returns an execution observation without the exact acceptance
- WHEN recovery output admission runs
- THEN the transport MUST reject the observation and require exact acceptance lookup.

#### Scenario: Request is not found
r[lattice.workflow_interchange.transport_dispatch_recovery_result.scenario.not-found]
- GIVEN the handler cannot find the exact request
- WHEN recovery runs
- THEN the transport MUST return a closed not-found rejection without claiming non-acceptance.

### Requirement: Typed exact handler profile

r[lattice.workflow_interchange.application_profile] Lattice MUST load a typed Nickel handler profile that binds each admitted contract profile and graph identity to one exact saved workflow revision, ordered node mapping, finite active limit, transport profile, and explicit effect-authority mode.

#### Scenario: Exact route is admitted

- GIVEN the profile, graph, saved workflow revision, node identities, and dependencies match
- WHEN handler admission runs
- THEN Lattice MAY create one durable execution acceptance.

#### Scenario: Saved workflow changed

- GIVEN the saved workflow revision differs from the selected route
- WHEN dispatch admission runs
- THEN Lattice MUST reject before workflow effects.

### Requirement: Durable idempotent acceptance

r[lattice.workflow_interchange.application_admission] Lattice MUST store the complete bounded request and acceptance before execution. An exact duplicate MUST return the stored acceptance. A reused idempotency key with different request material MUST fail.

#### Scenario: Dispatch reply is lost

- GIVEN durable admission committed but the reply was lost
- WHEN the consumer sends the original dispatch recovery request
- THEN Lattice MUST return the exact stored acceptance without redispatch.

#### Scenario: Idempotency key conflicts

- GIVEN one idempotency key already names another request
- WHEN dispatch admission runs
- THEN Lattice MUST reject before reserving or executing another run.

### Requirement: Saved workflow runtime binding

r[lattice.workflow_interchange.application_execution] Lattice MUST execute only the exact route-bound saved workflow revision through its existing headless runtime. It MUST persist running state before effects and MUST bind route-exact authority to the admitted request, run, and revision.

#### Scenario: Bound workflow completes

- GIVEN route admission and current authority pass
- WHEN the worker executes the saved workflow
- THEN terminal node and execution observations MUST preserve the consumer request linkage.

#### Scenario: Route effect authority is unavailable

- GIVEN the route does not grant exact workflow effect authority
- WHEN an effectful node reaches dispatch
- THEN the existing effect gate MUST deny it without a fallback executor.

### Requirement: Honest lifecycle and controls

r[lattice.workflow_interchange.application_lifecycle] Observe, reconcile, and cancel MUST bind the exact stored request, acceptance, execution, and current sequence. Pause and resume MUST fail closed until the runtime can pause active external effects safely.

#### Scenario: Current cancellation is requested

- GIVEN one live non-terminal execution and the exact current sequence
- WHEN cancellation runs
- THEN Lattice MUST persist the request and signal the bounded executor cancellation flag.

#### Scenario: Control sequence is stale

- GIVEN the stored sequence differs from the control sequence
- WHEN control admission runs
- THEN Lattice MUST reject without changing execution state.

### Requirement: Process loss remains unknown

r[lattice.workflow_interchange.application_recovery] A durable non-terminal acceptance without a live worker in the current service process MUST become an `Unknown` non-terminal observation. Recovery MUST NOT infer success, infer failure, or redispatch.

#### Scenario: Service restarts during execution

- GIVEN an accepted execution has no live worker after service restart
- WHEN observe or reconcile runs
- THEN Lattice MUST return `Unknown` and preserve the exact acceptance identities.

### Requirement: Bounded safe logs

r[lattice.workflow_interchange.application_logs] Lattice MUST expose only complete linked retained log chunks within the request and transport limits. Application logs MUST omit workflow outputs, commands, credentials, environment values, and provider payloads.

#### Scenario: Linked diagnostic chunk is read

- GIVEN a terminal observation contains one retained diagnostic reference
- WHEN the consumer requests its exact cursor and bound
- THEN Lattice MUST return the complete linked bytes.

#### Scenario: Log chunk is absent

- GIVEN no exact retained reference exists
- WHEN log read runs
- THEN Lattice MUST return a closed not-found result.

### Requirement: Supported Unix service command

r[lattice.workflow_interchange.application_command] On Linux, Lattice MUST provide `workflow-exchange serve` to compose the validated handler, normal store, normal runtime configuration, and published bounded Unix transport.

#### Scenario: Valid command profile serves

- GIVEN a valid app config and handler profile
- WHEN the command starts
- THEN it MUST bind the configured socket and serve no more than the finite connection limit.

#### Scenario: Invalid profile is supplied

- GIVEN the Nickel profile fails evaluation or handler admission
- WHEN the command starts
- THEN it MUST exit before binding the socket or executing a workflow.

### Requirement: Positive and negative application evidence

r[lattice.workflow_interchange.application_verification] Focused checks MUST cover profile admission, dispatch, duplicate handling, acceptance recovery, execution, observation, cancellation, logs, reconciliation, restart uncertainty, drift, stale controls, missing records, and malformed input.

#### Scenario: Focused checks pass

- GIVEN all positive and negative fixtures are present
- WHEN repository validation runs
- THEN claims MUST remain limited to the checked local application boundary and its explicit non-claims.
