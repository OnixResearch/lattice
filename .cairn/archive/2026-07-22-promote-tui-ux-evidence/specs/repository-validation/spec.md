# Repository Validation Delta

## ADDED Requirements

### Requirement: Exact final TUI/UX evidence promotion
r[lattice.repository_validation.tui_ux_promotion] Lattice MUST promote exactly the 98 completed TUI/UX requirements through assertion-local markers and an exact final missing-set transition.

#### Scenario: Exact final TUI/UX delta
r[lattice.repository_validation.tui_ux_promotion.scenario.delta]
- GIVEN the completion baseline with 98 missing requirements and unsuccessful coverage
- WHEN TUI/UX evidence is promoted
- THEN current coverage MUST contain zero missing requirements, remove exactly the 98 allowlisted IDs, add no ID, retain zero dangling markers, and be `valid: true`

#### Scenario: TUI/UX markers remain assertion-local
r[lattice.repository_validation.tui_ux_promotion.scenario.bindings]
- GIVEN the reviewed TUI/UX evidence files and exact binding table
- WHEN promoted marker bindings are validated
- THEN each identifier MUST occur immediately before its named supporting test and MUST be absent from reviewed debt policy

#### Scenario: Final TUI/UX promotion drift fails closed
r[lattice.repository_validation.tui_ux_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted missing sets, duplicate allowlists, missing bindings, stale policy rows, incomplete final coverage, unrelated removals, additions, dangling markers, or unsuccessful final coverage
- WHEN deterministic final-promotion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the promotion
