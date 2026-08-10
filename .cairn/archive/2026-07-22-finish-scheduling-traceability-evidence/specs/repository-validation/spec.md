## ADDED Requirements

### Requirement: Exact scheduling evidence completion
r[lattice.repository_validation.scheduling_completion] Lattice MUST maintain direct assertion-bound evidence for every clause in the seven reviewed scheduling requirements before promotion.

#### Scenario: Terminal history fields are directly asserted
r[lattice.repository_validation.scheduling_completion.scenario.history]
- GIVEN an admitted trigger whose workflow reaches a terminal state
- WHEN trigger history is loaded
- THEN maintained assertions MUST inspect trigger identity, workflow, run identity, idempotency key, status, and redaction-safe payload summary

#### Scenario: Schedule compatibility compares complete plans
r[lattice.repository_validation.scheduling_completion.scenario.compatibility]
- GIVEN one recurring schedule and its adapted trigger definition
- WHEN both evaluators run at the same due time
- THEN maintained assertions MUST compare the complete due-run plans for equality

#### Scenario: Listener startup is observed through the real process
r[lattice.repository_validation.scheduling_completion.scenario.listener]
- GIVEN an isolated store with a saved workflow and enabled webhook trigger
- WHEN the operator listener process starts on loopback and receives a bounded valid HTTP request
- THEN maintained assertions MUST observe an accepted response, terminal workflow run, and safe trigger history before owned child teardown

#### Scenario: Complete scheduling tranche becomes exact without promotion
r[lattice.repository_validation.scheduling_completion.scenario.exact]
- GIVEN direct maintained evidence for all seven scheduling rows
- WHEN completion inventory validation runs
- THEN every selected row MUST be exact, bind current safe file bytes, remain marker-free, and preserve all 324 missing IDs
