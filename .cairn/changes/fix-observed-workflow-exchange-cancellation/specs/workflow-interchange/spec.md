## ADDED Requirements

### Requirement: Observed owned runtime cancellation

r[lattice.workflow_interchange.observed_cancellation] The workflow-exchange application MUST derive canonical cancellation from observed owned runtime termination, not merely durable cancellation intent or an atomic request flag. An observed local cancellation MUST produce durable Cancelled and prevent pending dependent effects. Success and failure that complete without an observed cancellation MUST retain their ordinary outcomes, including cancellation races. Published wire schemas and identity domains MUST remain unchanged.

#### Scenario: Started local work is cancelled
r[lattice.workflow_interchange.observed_cancellation.scenario.active]
- GIVEN an admitted workflow with a started owned local child and a pending dependent effect
- WHEN an exact accepted control request causes owned runtime cancellation
- THEN the child MUST stop, the pending dependent effect MUST NOT execute, and durable observation MUST report terminal Cancelled with successful false.

#### Scenario: Completion wins cancellation
r[lattice.workflow_interchange.observed_cancellation.scenario.race]
- GIVEN successful or failed real local work with no observed cancellation, with or without late cancellation intent
- WHEN completion is classified
- THEN the ordinary completed or failed outcome MUST be retained and cancellation intent alone MUST NOT manufacture Cancelled.
