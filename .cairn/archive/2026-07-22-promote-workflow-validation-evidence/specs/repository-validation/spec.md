# Repository Validation Delta

### Requirement: Workflow-validation evidence promotion
r[lattice.repository_validation.workflow_validation_promotion] The repository MUST promote the exact reviewed workflow-validation tranche only through assertion-local markers.

#### Scenario: Exact bindings
r[lattice.repository_validation.workflow_validation_promotion.scenario.bindings]
- GIVEN 29 exact reviewed IDs
- WHEN markers are added
- THEN each selected ID is bound directly to an allowlisted maintained test

#### Scenario: Exact delta
r[lattice.repository_validation.workflow_validation_promotion.scenario.delta]
- GIVEN the immutable 290-gap baseline
- WHEN current coverage is checked
- THEN exactly the selected IDs are removed, none are added, zero markers dangle, and coverage remains unsuccessful

#### Scenario: Fail-closed drift
r[lattice.repository_validation.workflow_validation_promotion.scenario.fail_closed]
- GIVEN marker, policy, or coverage drift
- WHEN promotion validation runs
- THEN deterministic validation rejects the drift
