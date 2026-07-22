# Repository Validation Delta

### Requirement: Workflow-validation evidence completion
r[lattice.repository_validation.workflow_validation_completion] The repository MUST maintain direct assertions that close every gap identified by the workflow-validation audit before promotion.

#### Scenario: Behavior completion
r[lattice.repository_validation.workflow_validation_completion.scenario.behavior]
- GIVEN a runner declares an effect unsupported by its selected runtime
- WHEN admission runs
- THEN validation reports a deterministic fatal diagnostic before runtime execution

#### Scenario: Complete evidence
r[lattice.repository_validation.workflow_validation_completion.scenario.evidence]
- GIVEN the ten audited evidence gaps
- WHEN maintained tests run
- THEN positive and negative assertions cover every accepted clause

#### Scenario: Exact classification
r[lattice.repository_validation.workflow_validation_completion.scenario.classification]
- GIVEN completed maintained evidence
- WHEN inventory is checked
- THEN all 29 reviewed workflow-validation rows are exact while coverage remains unchanged and unmarked
