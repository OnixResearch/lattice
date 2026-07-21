# Agent Orchestration Delta

## ADDED Requirements

### Requirement: Explicit abandonment of locally unobserved dispatches
r[lattice.agent_orchestration.unobserved_abandon] Lattice MUST allow an authorized operator to abandon a cancellation-requested pending dispatch only when no matching persisted run record is observable, MUST terminate the local orchestration as failed without retrying or claiming a remote outcome, and MUST preserve the abandonment as a replayable event.

#### Scenario: Acknowledged dispatch is abandoned conservatively
r[lattice.agent_orchestration.unobserved_abandon.scenario.acknowledged]
- GIVEN recovery reports that an acknowledged pending dispatch has no matching persisted run record and an authorized operator has requested cancellation
- WHEN that operator abandons the pending dispatch
- THEN Lattice MUST consume the complete reserved upper bound, clear the pending and active run identities, terminate the instance as failed, and record that the remote outcome remains unknown

#### Scenario: Planned dispatch is abandoned before shell invocation
r[lattice.agent_orchestration.unobserved_abandon.scenario.planned]
- GIVEN a pending dispatch remains planned, has no matching persisted run record, and an authorized operator has requested cancellation
- WHEN that operator abandons the pending dispatch
- THEN Lattice MUST release the reservation without consumption, terminate the instance as failed, and MUST NOT invoke the workflow shell

#### Scenario: Observed run must use ordinary recovery
r[lattice.agent_orchestration.unobserved_abandon.scenario.observed]
- GIVEN the store contains a run record matching the pending dispatch
- WHEN an operator requests abandonment
- THEN Lattice MUST reject abandonment without mutation and direct the operator to ordinary recovery

#### Scenario: Abandonment requires cancellation and authority
r[lattice.agent_orchestration.unobserved_abandon.scenario.authority]
- GIVEN a pending dispatch is locally unobserved but cancellation was not requested or the actor is not an authorized approver
- WHEN abandonment is requested
- THEN Lattice MUST reject the request without changing the dispatch, reservation, budget, or event chain

#### Scenario: Repeated or mismatched abandonment fails closed
r[lattice.agent_orchestration.unobserved_abandon.scenario.identity]
- GIVEN an abandonment fact is repeated, names a different dispatch or run, or targets a terminal instance
- WHEN the state machine evaluates it
- THEN Lattice MUST reject it without a second settlement or event

#### Scenario: Replay preserves bounded non-claims
r[lattice.agent_orchestration.unobserved_abandon.scenario.replay]
- GIVEN an unobserved dispatch was abandoned
- WHEN Lattice replays the instance event chain
- THEN replay MUST reconstruct the same terminal state, conservative budget settlement, and fixed safe summary without asserting provider cancellation, provider failure, workflow success, billing truth, or remote exactly-once execution
