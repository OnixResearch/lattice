# Repository Validation Delta

### Requirement: Bounded Octet baseline migration

r[lattice.repository_validation.octet_baseline_migration] Lattice MUST migrate accepted Octet identities only when the bounded semantic finding policy accepts the complete transition.

#### Scenario: Reviewed wording retains semantic identity

r[lattice.repository_validation.octet_baseline_migration.scenario.wording]
- GIVEN an `ambiguous_params` or `numeric_units` message changes only through a reviewed Octet wording transition
- WHEN migration compares findings
- THEN it MUST preserve the warning facts and semantic identity

#### Scenario: Semantic occurrence growth fails closed

r[lattice.repository_validation.octet_baseline_migration.scenario.multiplicity]
- GIVEN a normalized semantic group occurs more times in the candidate run
- WHEN migration compares finding multiplicity
- THEN it MUST reject without writing the baseline

#### Scenario: Unreviewed wording fails closed

r[lattice.repository_validation.octet_baseline_migration.scenario.fail_closed]
- GIVEN a message changes a parameter, unit, type, file, or other warning fact
- WHEN migration compares findings
- THEN it MUST reject without writing the baseline
