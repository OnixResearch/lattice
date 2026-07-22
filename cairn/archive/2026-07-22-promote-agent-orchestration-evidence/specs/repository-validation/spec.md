# Repository Validation Delta

### Requirement: Agent-orchestration evidence promotion
r[lattice.repository_validation.agent_orchestration_promotion] The repository MUST promote exactly the 39 completed agent-orchestration rows through assertion-local markers.

#### Scenario: Exact bindings
r[lattice.repository_validation.agent_orchestration_promotion.scenario.bindings]
- GIVEN exact maintained tests
- WHEN markers are promoted
- THEN every selected ID is bound to an allowlisted test

#### Scenario: Exact delta
r[lattice.repository_validation.agent_orchestration_promotion.scenario.delta]
- GIVEN the immutable 261-gap baseline
- WHEN coverage is compared
- THEN exactly 39 selected IDs are removed, none added, zero dangling remain, and coverage stays unsuccessful

#### Scenario: Fail-closed drift
r[lattice.repository_validation.agent_orchestration_promotion.scenario.fail_closed]
- GIVEN binding, policy, or set drift
- WHEN promotion validation runs
- THEN it rejects deterministically
