# Repository Validation Delta

### Requirement: Agent-orchestration traceability audit
r[lattice.repository_validation.agent_orchestration_audit] The repository MUST maintain a byte-bound classification audit for all 39 selected missing agent-orchestration requirements without promotion.

#### Scenario: Complete classification
r[lattice.repository_validation.agent_orchestration_audit.scenario.complete]
- GIVEN the selected missing subset
- WHEN audit validation runs
- THEN every ID has exactly one explicit exact or partial decision

#### Scenario: Evidence identity
r[lattice.repository_validation.agent_orchestration_audit.scenario.evidence]
- GIVEN reviewed maintained evidence
- WHEN hashes are checked
- THEN every safe path matches its current BLAKE3 bytes

#### Scenario: Fail-closed drift
r[lattice.repository_validation.agent_orchestration_audit.scenario.fail_closed]
- GIVEN decision, evidence, marker, or coverage drift
- WHEN validation runs
- THEN the audit rejects it deterministically
