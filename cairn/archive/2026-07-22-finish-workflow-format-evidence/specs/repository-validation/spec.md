# Repository Validation Delta

### Requirement: Workflow-format evidence completion
r[lattice.repository_validation.workflow_format_completion] Lattice MUST maintain direct exact evidence for all 41 audited workflow-format rows before promotion.

#### Scenario: Complete workflow-format boundaries
r[lattice.repository_validation.workflow_format_completion.scenario.behavior]
- GIVEN the five explicit audit gaps
- WHEN maintained positive and negative tests run
- THEN image import, malformed control flow, opaque credentials, and cookbook drift MUST be directly asserted

#### Scenario: All workflow-format rows exact
r[lattice.repository_validation.workflow_format_completion.scenario.exact]
- GIVEN completed maintained evidence
- WHEN inventory is generated
- THEN all 41 rows MUST be exact, byte-bound, and unmarked while missing coverage remains unchanged

#### Scenario: Workflow-format completion drift fails closed
r[lattice.repository_validation.workflow_format_completion.scenario.fail_closed]
- GIVEN evidence, classification, marker, or coverage drift
- WHEN completion validation runs
- THEN it MUST reject deterministically
