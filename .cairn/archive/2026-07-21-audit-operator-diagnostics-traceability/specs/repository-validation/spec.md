## ADDED Requirements

### Requirement: Bounded traceability tranche review
r[lattice.repository_validation.traceability_tranche_review] Lattice MUST review a selected missing-requirement tranche through explicit typed classifications and maintained evidence identities without changing requirement coverage.

#### Scenario: Complete explicit tranche partition
r[lattice.repository_validation.traceability_tranche_review.scenario.complete]
- GIVEN a bounded subsystem tranche and an explicit expectation table
- WHEN the maintained review is validated
- THEN every selected missing identifier MUST have exactly one reviewed non-`unassessed` classification and no unrelated identifier MUST be classified by that tranche

#### Scenario: Evidence strength remains explicit
r[lattice.repository_validation.traceability_tranche_review.scenario.evidence]
- GIVEN exact, partial, and no-exact review decisions
- WHEN evidence bindings are checked
- THEN exact and partial rows MUST bind current safe file-byte identities, no-exact rows MUST remain evidence-free, and the review MUST NOT satisfy or promote any selected requirement

#### Scenario: Classification or coverage drift fails closed
r[lattice.repository_validation.traceability_tranche_review.scenario.fail_closed]
- GIVEN a missing expected row, unexpected reviewed row, classification drift, rationale drift, evidence-path drift, missing-set change, dangling marker, or successful coverage
- WHEN the maintained review validator checks the tranche
- THEN it MUST return deterministic issue codes and MUST NOT accept the review
