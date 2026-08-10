# Repository Validation Delta

## ADDED Requirements

### Requirement: Remaining TUI/UX traceability audit
r[lattice.repository_validation.tui_ux_traceability_audit] Lattice MUST classify the exact remaining 98 TUI/UX requirements against bounded maintained evidence without changing Tracey linkage.

#### Scenario: Exact TUI/UX audit scope
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.scope]
- GIVEN the failing 98-item Tracey receipt after workflow-execution promotion
- WHEN the TUI/UX audit runs
- THEN every missing ID MUST appear exactly once and no non-missing ID may enter the audit

#### Scenario: Byte-bound TUI/UX evidence
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.evidence]
- GIVEN bounded repository-owned evidence snapshots
- WHEN an audit row cites evidence
- THEN its safe relative path and current file-byte BLAKE3 hash MUST match the reviewed snapshot

#### Scenario: Honest TUI/UX classification
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.classification]
- GIVEN direct assertions and identified behavioral gaps
- WHEN evidence strength is classified
- THEN each row MUST be `exact_maintained_evidence` or `partial_evidence_only` with an explicit bounded rationale and no inventory row may satisfy the requirement

#### Scenario: TUI/UX audit drift fails closed
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.fail_closed]
- GIVEN a missing row, duplicate row, unsafe path, hash drift, invalid classification, changed missing set, dangling marker, successful coverage, or selected marker
- WHEN deterministic audit validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the audit
