## ADDED Requirements

### Requirement: Remaining credential traceability audit
r[lattice.repository_validation.remaining_credential_audit] Lattice MUST maintain a bounded review of exactly the remaining 14 credential debt rows without changing credential behavior or promoting reviewed identifiers.

#### Scenario: Complete remaining credential tranche
r[lattice.repository_validation.remaining_credential_audit.scenario.complete]
- GIVEN the 341-item missing-set baseline after the core credential promotion
- WHEN the remaining credential audit runs
- THEN it MUST classify exactly the 14 allowlisted credential rows once each and MUST leave every other row unassessed

#### Scenario: Evidence strength remains byte-bound
r[lattice.repository_validation.remaining_credential_audit.scenario.evidence]
- GIVEN a selected credential row has exact, partial, or no-exact maintained evidence
- WHEN the audit records its decision
- THEN the classification, bounded rationale, safe evidence paths, and current file-byte BLAKE3 hashes MUST match the reviewed expectation without treating parent markers, archives, inventory rows, or implementation shape as stronger evidence

#### Scenario: Remaining credential audit fails closed
r[lattice.repository_validation.remaining_credential_audit.scenario.fail_closed]
- GIVEN a selected row is missing, duplicated, unexpected, reordered, reclassified, rebound, promoted, or accompanied by missing-set or dangling-marker drift
- WHEN maintained audit validation runs
- THEN it MUST reject the audit with deterministic issue codes while global coverage remains unsuccessful
