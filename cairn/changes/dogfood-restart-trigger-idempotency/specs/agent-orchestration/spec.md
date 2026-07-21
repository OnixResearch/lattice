## MODIFIED Requirements

### Requirement: Orchestration trigger admission
r[lattice.agent_orchestration.triggers] Lattice MUST route schedule and webhook requests for orchestration start or signal actions through typed trigger, idempotency, overlap, payload, and authority admission before mutating an instance.

#### Scenario: Valid trigger starts one instance
r[lattice.agent_orchestration.triggers.scenario.start]
- GIVEN an enabled trigger references a valid orchestration definition and supplies compatible typed inputs with a fresh idempotency key
- WHEN trigger admission succeeds
- THEN Lattice MUST start exactly one durable instance for that admitted trigger fact

#### Scenario: Duplicate trigger does not duplicate work
r[lattice.agent_orchestration.triggers.scenario.duplicate]
- GIVEN an orchestration trigger idempotency key was already accepted
- WHEN the same trigger fact is received again
- THEN Lattice MUST return the prior admission outcome and MUST NOT create another instance or budget reservation

#### Scenario: Duplicate admission survives process restart
r[lattice.agent_orchestration.triggers.scenario.process-restart]
- GIVEN one process durably accepted an orchestration trigger fact and released its store ownership
- WHEN a separately started process reopens that store and submits the exact same definition revision, trigger identity, and idempotency key
- THEN Lattice MUST return the original instance and MUST NOT create another admission, instance, run, or budget charge
