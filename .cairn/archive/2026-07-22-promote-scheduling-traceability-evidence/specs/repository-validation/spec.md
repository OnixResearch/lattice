## ADDED Requirements

### Requirement: Exact scheduling evidence promotion
r[lattice.repository_validation.scheduling_promotion] Lattice MUST promote exactly the seven completed scheduling requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact scheduling delta
r[lattice.repository_validation.scheduling_promotion.scenario.delta]
- GIVEN the completion baseline with 324 missing requirements
- WHEN scheduling evidence is promoted
- THEN current coverage MUST contain 317 missing requirements, remove exactly the seven allowlisted IDs, add no ID, retain zero dangling markers, and remain `valid: false`

#### Scenario: Scheduling markers remain assertion-local
r[lattice.repository_validation.scheduling_promotion.scenario.bindings]
- GIVEN the direct and composite exact scheduling tests
- WHEN bindings are validated
- THEN every promoted identifier MUST attach only to its required supporting tests and MUST be absent from reviewed debt policy

#### Scenario: Scheduling promotion drift fails closed
r[lattice.repository_validation.scheduling_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted sets, duplicate allowlists, missing bindings, stale policy, unrelated removal, addition, dangling markers, or successful coverage
- WHEN promotion validation runs
- THEN it MUST return explicit issue codes and reject the transition
