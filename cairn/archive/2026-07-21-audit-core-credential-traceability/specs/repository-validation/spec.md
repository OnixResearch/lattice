## ADDED Requirements

### Requirement: Core credential traceability audit
r[lattice.repository_validation.credential_traceability_audit] Lattice MUST review the selected core credential traceability tranche through explicit typed classifications and current maintained evidence identities without changing credential behavior or satisfying reviewed credential requirements.

#### Scenario: Complete bounded credential partition
r[lattice.repository_validation.credential_traceability_audit.scenario.complete]
- GIVEN the selected 12 missing core credential identifiers and an explicit expectation table
- WHEN the maintained audit is validated
- THEN every selected identifier MUST have exactly one reviewed non-`unassessed` classification and no unselected credential identifier MUST be classified by this audit

#### Scenario: Credential evidence strength remains explicit
r[lattice.repository_validation.credential_traceability_audit.scenario.evidence]
- GIVEN exact and partial credential evidence with different assertion strength
- WHEN evidence bindings are checked
- THEN every reviewed row MUST bind current safe file-byte identities, partial rows MUST remain explicitly partial, and no reviewed credential marker MUST be added

#### Scenario: Credential audit drift fails closed
r[lattice.repository_validation.credential_traceability_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification drift, rationale drift, evidence-path drift, missing-set change, dangling marker, or successful coverage
- WHEN the maintained credential audit validator checks the tranche
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit
