## ADDED Requirements

### Requirement: Node-template prompt traceability audit
r[lattice.repository_validation.node_template_prompt_audit] Lattice MUST maintain a bounded explicit review of all three missing node-template prompt rows without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete node-template prompt partition
r[lattice.repository_validation.node_template_prompt_audit.scenario.complete]
- GIVEN the three missing node-template prompt identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Node-template prompt evidence strength remains explicit
r[lattice.repository_validation.node_template_prompt_audit.scenario.evidence]
- GIVEN exact collection and cancellation assertions plus unasserted user-visible required-prompt rendering
- WHEN evidence bindings are checked
- THEN exact and partial rows MUST bind current safe file-byte identities, partial evidence MUST remain partial, no selected marker may be added, and coverage MUST remain unchanged at 327 missing IDs

#### Scenario: Node-template prompt audit drift fails closed
r[lattice.repository_validation.node_template_prompt_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, a dangling marker, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit
