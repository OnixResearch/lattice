# Workflow Dispatch Recovery Delta Specification

## ADDED Requirements

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
