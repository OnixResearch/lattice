## ADDED Requirements

### Requirement: Agent-runtime evidence promotion
r[lattice.repository_validation.agent_runtime_promotion] Lattice MUST promote only the 14 reviewed exact agent-runtime rows through markers attached to maintained assertions and MUST preserve every unrelated linkage state.

#### Scenario: Exact 14-ID agent-runtime delta
r[lattice.repository_validation.agent_runtime_promotion.scenario.delta]
- GIVEN the failing 304-item completion receipt and explicit 14-ID allowlist
- WHEN promotion validation compares current failing coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing MUST equal 290

#### Scenario: Agent-runtime markers remain locally bound
r[lattice.repository_validation.agent_runtime_promotion.scenario.bindings]
- GIVEN every selected row has exact maintained assertions
- WHEN markers are promoted
- THEN each marker MUST attach to its required integration, internal shell, or OpenAI test and MUST NOT rely on file scope or lifecycle evidence

#### Scenario: Agent-runtime promotion drift fails closed
r[lattice.repository_validation.agent_runtime_promotion.scenario.fail_closed]
- GIVEN duplicate, missing-binding, stale-policy, removal, addition, dangling, or successful-coverage drift
- WHEN promotion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion
