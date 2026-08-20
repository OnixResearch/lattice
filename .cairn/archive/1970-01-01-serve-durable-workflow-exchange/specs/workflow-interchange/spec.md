# Durable Workflow Exchange Application Delta

## ADDED Requirements

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
