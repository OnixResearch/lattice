## ADDED Requirements

### Requirement: Agent-runtime evidence completion
r[lattice.repository_validation.agent_runtime_completion] Lattice MUST maintain exact direct evidence for all 14 reviewed agent-runtime rows without promoting selected markers or changing unrelated linkage.

#### Scenario: Partial runtime boundaries become directly observed
r[lattice.repository_validation.agent_runtime_completion.scenario.boundaries]
- GIVEN missing-route, credential, provider-failure, identity, authority-bearing output, incomplete-status, and unsupported-input boundaries
- WHEN maintained positive and negative tests run
- THEN each boundary MUST directly assert its accepted state, effect, redaction, output, or zero-dispatch clause

#### Scenario: Planning remains pure and shells remain bounded
r[lattice.repository_validation.agent_runtime_completion.scenario.core_shell]
- GIVEN explicit program identity inputs and runtime transport/report effects
- WHEN completion evidence exercises them
- THEN deterministic admission MUST remain testable without transport and shell assertions MUST observe exact effect counts and terminal mapping

#### Scenario: Exactly 14 rows are exact
r[lattice.repository_validation.agent_runtime_completion.scenario.exact]
- GIVEN the reviewed 304-item inventory and 14-ID agent-runtime tranche
- WHEN completion validation runs
- THEN all 14 rows MUST be exact, current-byte-bound, marker-free, and leave every unrelated row unchanged

#### Scenario: Agent-runtime completion drift fails closed
r[lattice.repository_validation.agent_runtime_completion.scenario.fail_closed]
- GIVEN classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion
