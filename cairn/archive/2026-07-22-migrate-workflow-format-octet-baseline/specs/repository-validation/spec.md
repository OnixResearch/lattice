# Repository Validation Delta

### Requirement: Bounded Octet baseline migration
r[lattice.repository_validation.octet_baseline_migration] Lattice MUST migrate accepted Octet identities only when the bounded semantic finding policy accepts the complete transition.

#### Scenario: Dynamic counts retain semantic identity
r[lattice.repository_validation.octet_baseline_migration.scenario.dynamic]
- GIVEN assertion-density or function-length messages differ only in dynamic line counts
- WHEN migration compares findings
- THEN it MUST preserve the function-scoped semantic identity

#### Scenario: Exact resolved finding is allowed
r[lattice.repository_validation.octet_baseline_migration.scenario.resolved]
- GIVEN the sole old-only finding is the reviewed assertion-density finding for `workflow_diagnostic_from_runner`
- WHEN migration runs
- THEN it MAY remove that accepted finding

#### Scenario: Semantic drift fails closed
r[lattice.repository_validation.octet_baseline_migration.scenario.fail_closed]
- GIVEN a candidate-only finding or any unapproved old-only finding
- WHEN migration runs
- THEN it MUST reject without writing the baseline
