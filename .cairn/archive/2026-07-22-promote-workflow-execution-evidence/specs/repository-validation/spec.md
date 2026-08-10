# Repository Validation Delta

## ADDED Requirements

### Requirement: Exact workflow-execution evidence promotion
r[lattice.repository_validation.workflow_execution_promotion] Lattice MUST promote exactly the 83 completed workflow-execution requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact workflow-execution delta
r[lattice.repository_validation.workflow_execution_promotion.scenario.delta]
- GIVEN the completion baseline with 181 missing requirements
- WHEN workflow-execution evidence is promoted
- THEN current coverage MUST contain 98 missing requirements, remove exactly the 83 allowlisted IDs, add no ID, retain zero dangling markers, and remain `valid: false`

#### Scenario: Workflow-execution markers remain assertion-local
r[lattice.repository_validation.workflow_execution_promotion.scenario.bindings]
- GIVEN the reviewed workflow-execution evidence files and exact binding table
- WHEN promoted marker bindings are validated
- THEN each identifier MUST occur immediately before its named supporting test and MUST be absent from reviewed debt policy

#### Scenario: Workflow-execution promotion drift fails closed
r[lattice.repository_validation.workflow_execution_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted missing sets, duplicate allowlists, missing bindings, stale policy rows, unrelated removals, additions, dangling markers, or successful coverage
- WHEN deterministic promotion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the promotion
