# Repository Validation Delta

### Requirement: Agent-orchestration evidence completion
r[lattice.repository_validation.agent_orchestration_completion] The repository MUST maintain direct exact evidence for all 39 audited agent-orchestration rows before promotion.

#### Scenario: Complete bounded behavior
r[lattice.repository_validation.agent_orchestration_completion.scenario.behavior]
- GIVEN the six explicit audit gaps
- WHEN maintained pure-core tests run
- THEN inert output, overage, status, non-claim, and cycle boundaries are directly asserted

#### Scenario: All rows exact
r[lattice.repository_validation.agent_orchestration_completion.scenario.exact]
- GIVEN completed maintained evidence
- WHEN inventory is generated
- THEN all 39 rows are exact, byte-bound, and unmarked while coverage remains unchanged

#### Scenario: Fail-closed drift
r[lattice.repository_validation.agent_orchestration_completion.scenario.fail_closed]
- GIVEN evidence, classification, marker, or coverage drift
- WHEN completion validation runs
- THEN it rejects deterministically
