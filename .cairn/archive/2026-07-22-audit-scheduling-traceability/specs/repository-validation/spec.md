## ADDED Requirements

### Requirement: Scheduling traceability audit
r[lattice.repository_validation.scheduling_audit] Lattice MUST maintain a bounded explicit review of all seven remaining scheduling debt rows without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete scheduling partition
r[lattice.repository_validation.scheduling_audit.scenario.complete]
- GIVEN the seven missing scheduling identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Scheduling evidence strength remains explicit
r[lattice.repository_validation.scheduling_audit.scenario.evidence]
- GIVEN direct admission, persistence, compatibility, cookbook, and listener evidence
- WHEN evidence bindings are checked
- THEN exact rows MUST cover every accepted clause, partial rows MUST name their direct gap, all reviewed evidence MUST bind current safe file bytes, and no selected marker may be added

#### Scenario: Scheduling audit drift fails closed
r[lattice.repository_validation.scheduling_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, dangling markers, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit
