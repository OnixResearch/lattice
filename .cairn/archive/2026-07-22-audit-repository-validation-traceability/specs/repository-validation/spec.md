## ADDED Requirements

### Requirement: Repository-check traceability audit
r[lattice.repository_validation.repository_checks_audit] Lattice MUST maintain a bounded explicit review of all 11 remaining repository-validation debt rows without changing check behavior or promoting reviewed identifiers.

#### Scenario: Complete repository-check partition
r[lattice.repository_validation.repository_checks_audit.scenario.complete]
- GIVEN the 11 missing repository-validation identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Repository-check evidence strength remains explicit
r[lattice.repository_validation.repository_checks_audit.scenario.evidence]
- GIVEN direct and partial signing, source-closure, formatter, and lint evidence
- WHEN evidence bindings are checked
- THEN exact rows MUST cover every accepted clause, partial rows MUST name their direct observation gap, all reviewed evidence MUST bind current safe file bytes, and no selected marker may be added

#### Scenario: Repository-check audit drift fails closed
r[lattice.repository_validation.repository_checks_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, dangling markers, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit
