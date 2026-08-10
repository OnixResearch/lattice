# Repository Validation Delta

## ADDED Requirements

### Requirement: TUI/UX evidence completion
r[lattice.repository_validation.tui_ux_evidence_completion] Lattice MUST complete every partial row from the bounded TUI/UX audit before any selected requirement is promoted.

#### Scenario: All TUI/UX rows become exact
r[lattice.repository_validation.tui_ux_evidence_completion.scenario.exact]
- GIVEN the immutable audit with 81 exact and 17 partial rows
- WHEN bounded TUI behavior and assertions are completed
- THEN all 98 rows MUST be exact against current safe relative evidence paths and current file-byte BLAKE3 hashes

#### Scenario: Completion preserves Tracey debt
r[lattice.repository_validation.tui_ux_evidence_completion.scenario.coverage]
- GIVEN exactly 98 missing TUI/UX requirements and zero dangling markers
- WHEN completion evidence is recorded
- THEN the missing set MUST remain unchanged, coverage MUST remain unsuccessful, and no selected marker may be attached

#### Scenario: TUI/UX completion drift fails closed
r[lattice.repository_validation.tui_ux_evidence_completion.scenario.fail_closed]
- GIVEN a stale partial classification, missing row, unsafe evidence path, byte-hash drift, selected marker, changed missing set, dangling marker, or successful coverage
- WHEN deterministic completion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept completion
