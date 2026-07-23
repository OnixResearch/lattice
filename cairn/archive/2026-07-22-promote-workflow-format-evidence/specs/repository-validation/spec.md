# Repository Validation Delta

### Requirement: Workflow-format evidence promotion
r[lattice.repository_validation.workflow_format_promotion] Lattice MUST promote exactly the 41 completed workflow-format rows through assertion-local markers.

#### Scenario: Exact workflow-format bindings
r[lattice.repository_validation.workflow_format_promotion.scenario.bindings]
- GIVEN exact maintained tests
- WHEN markers are promoted
- THEN every selected ID MUST bind to an allowlisted test

#### Scenario: Exact workflow-format delta
r[lattice.repository_validation.workflow_format_promotion.scenario.delta]
- GIVEN the immutable 222-gap completion baseline
- WHEN current coverage is compared
- THEN exactly 41 selected IDs MUST be removed, none added, zero dangling remain, and missing coverage MUST equal 181

#### Scenario: Workflow-format promotion drift fails closed
r[lattice.repository_validation.workflow_format_promotion.scenario.fail_closed]
- GIVEN binding, policy, duplicate, or set drift
- WHEN promotion validation runs
- THEN it MUST reject deterministically
