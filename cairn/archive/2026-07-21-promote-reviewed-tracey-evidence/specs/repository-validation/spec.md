## ADDED Requirements

### Requirement: Reviewed Tracey evidence promotion
r[lattice.repository_validation.reviewed_tracey_promotion] Lattice MUST promote reviewed traceability candidates only through markers attached to exact maintained assertions and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact allowlisted missing-set delta
r[lattice.repository_validation.reviewed_tracey_promotion.scenario.exact_delta]
- GIVEN a baseline failing coverage receipt and an explicit reviewed promotion allowlist
- WHEN the maintained promotion check compares the current failing coverage receipt
- THEN the removed identifiers MUST equal the allowlist, no identifier MUST be added, dangling MUST remain empty, and every unrelated missing identifier MUST remain unchanged

#### Scenario: Markers remain locally bound
r[lattice.repository_validation.reviewed_tracey_promotion.scenario.local_binding]
- GIVEN a reviewed candidate has positive or negative assertions needed for its complete accepted scenario
- WHEN the candidate is promoted
- THEN every marker MUST be attached to an exact supporting test and MUST NOT rely on file-level presence, lexical similarity, generated inventory rows, or archived evidence

#### Scenario: Unexpected promotion fails closed
r[lattice.repository_validation.reviewed_tracey_promotion.scenario.fail_closed]
- GIVEN a duplicate baseline identifier, an unexpected removed identifier, an added identifier, a dangling marker, or successful coverage
- WHEN the maintained promotion check validates the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion
