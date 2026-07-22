## ADDED Requirements

### Requirement: Exact workflow-example evidence promotion
r[lattice.repository_validation.workflow_example_promotion] Lattice MUST promote exactly the two completed workflow-example requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact workflow-example delta
r[lattice.repository_validation.workflow_example_promotion.scenario.delta]
- GIVEN 317 missing requirements
- WHEN both workflow-example rows are promoted
- THEN coverage MUST contain 315 missing requirements, remove only those two IDs, add nothing, retain zero dangling markers, and remain `valid: false`

#### Scenario: Workflow-example bindings remain local
r[lattice.repository_validation.workflow_example_promotion.scenario.bindings]
- GIVEN the complete positive and negative image fixture tests
- WHEN marker bindings are checked
- THEN each ID MUST attach directly to its supporting test and be absent from reviewed debt policy
