## ADDED Requirements

### Requirement: Agent-runtime traceability audit
r[lattice.repository_validation.agent_runtime_audit] Lattice MUST maintain a bounded explicit review of all 14 remaining agent-runtime debt rows without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete agent-runtime partition
r[lattice.repository_validation.agent_runtime_audit.scenario.complete]
- GIVEN the 14 missing agent-runtime identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Agent-runtime evidence strength remains explicit
r[lattice.repository_validation.agent_runtime_audit.scenario.evidence]
- GIVEN direct compatibility, effect, fabric, authority, output, and reasoning evidence
- WHEN evidence bindings are checked
- THEN exact rows MUST cover every clause, partial rows MUST name their direct gap, evidence MUST bind current safe bytes, and no selected marker may be added

#### Scenario: Agent-runtime audit drift fails closed
r[lattice.repository_validation.agent_runtime_audit.scenario.fail_closed]
- GIVEN row, classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit
