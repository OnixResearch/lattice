# Repository Validation Delta

### Requirement: Workflow-validation traceability audit
r[lattice.repository_validation.workflow_validation_audit] The repository MUST maintain a byte-bound classification audit for every selected missing workflow-validation requirement without promoting Tracey coverage.

#### Scenario: Complete classification
r[lattice.repository_validation.workflow_validation_audit.scenario.complete]
- GIVEN the current workflow-validation missing subset
- WHEN the audit is checked
- THEN every selected ID has exactly one explicit exact, partial, or no-exact decision

#### Scenario: Evidence identity
r[lattice.repository_validation.workflow_validation_audit.scenario.evidence]
- GIVEN a reviewed decision cites maintained evidence
- WHEN the audit is checked
- THEN every safe relative evidence path matches its recorded BLAKE3 file hash

#### Scenario: Fail-closed drift
r[lattice.repository_validation.workflow_validation_audit.scenario.fail_closed]
- GIVEN classification, evidence, marker, or coverage drift
- WHEN the audit is checked
- THEN deterministic validation rejects the drift
