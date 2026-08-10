## ADDED Requirements

### Requirement: Workflow-example traceability audit
r[lattice.repository_validation.workflow_example_audit] Lattice MUST maintain a bounded explicit review of both missing workflow-example scenarios without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete workflow-example partition
r[lattice.repository_validation.workflow_example_audit.scenario.complete]
- GIVEN the two missing workflow-example scenario identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Workflow-example evidence strength remains explicit
r[lattice.repository_validation.workflow_example_audit.scenario.evidence]
- GIVEN direct end-to-end evidence for the selfie example and no maintained invalid-example fixture
- WHEN evidence bindings are checked
- THEN the selfie row MUST bind current safe file-byte identities, the invalid row MUST remain evidence-free, no selected marker may be added, and coverage MUST remain unchanged at 327 missing IDs

#### Scenario: Workflow-example audit drift fails closed
r[lattice.repository_validation.workflow_example_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, a dangling marker, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit
