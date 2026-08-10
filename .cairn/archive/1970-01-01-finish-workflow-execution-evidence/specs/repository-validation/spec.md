# Repository Validation Delta

## ADDED Requirements

### Requirement: Workflow-execution evidence completion
r[lattice.repository_validation.workflow_execution_completion] Lattice MUST maintain direct exact evidence for all 83 audited workflow-execution rows before promotion.

#### Scenario: Pinned Rats source closure
r[lattice.repository_validation.workflow_execution_completion.scenario.rats]
- GIVEN the repository-owned Rats sync boundary
- WHEN source-closure and focused consumer checks run
- THEN the exact pinned revision, required nested sources, generated snapshots, and offline compilation MUST be proved

#### Scenario: All workflow-execution rows exact
r[lattice.repository_validation.workflow_execution_completion.scenario.exact]
- GIVEN completed maintained evidence
- WHEN inventory is generated
- THEN all 83 rows MUST be exact, byte-bound, and unmarked while missing coverage remains unchanged

#### Scenario: Workflow-execution completion drift fails closed
r[lattice.repository_validation.workflow_execution_completion.scenario.fail_closed]
- GIVEN evidence, classification, marker, correlation, or coverage drift
- WHEN completion validation runs
- THEN it MUST reject deterministically
