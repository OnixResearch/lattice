# Implementation Attempt Consumer

## ADDED Requirements

### Requirement: Explicit one-attempt execution consumer

r[lattice.implementation_attempt.consumer] An explicitly selected CLI MUST bind the admitted immutable envelope, source snapshot, workspace, worker profile and provider before invoking the existing bounded isolated OMP worker. It MUST persist execution intent before effects. Re-entry after possible dispatch MUST reconcile retained results instead of redispatching. Worker text and process exit MUST NOT establish acceptance, integration or project completion.

#### Scenario: Crash after execution intent

r[lattice.implementation_attempt.consumer.scenario.recovery]
- GIVEN an intent persisted before worker execution and a controller crash
- WHEN the same request is submitted again
- THEN it MUST recover the existing candidate or report incomplete/unknown
- AND it MUST NOT blindly execute another worker
