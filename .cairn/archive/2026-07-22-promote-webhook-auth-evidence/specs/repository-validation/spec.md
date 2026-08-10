## ADDED Requirements

### Requirement: Webhook authentication evidence promotion
r[lattice.repository_validation.webhook_auth_promotion] Lattice MUST promote only the two reviewed exact webhook authentication rows through markers attached to maintained durable-boundary tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact two-ID webhook delta
r[lattice.repository_validation.webhook_auth_promotion.scenario.exact_delta]
- GIVEN the failing 329-item baseline and explicit two-ID webhook allowlist
- WHEN maintained promotion validation compares current coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 327

#### Scenario: Webhook markers remain locally bound
r[lattice.repository_validation.webhook_auth_promotion.scenario.local_binding]
- GIVEN both rows have exact maintained durable-boundary assertions
- WHEN their markers are promoted
- THEN each marker MUST attach directly to its required test and MUST NOT rely on implementation names, file scope, lifecycle tests, generated inventory, or archived evidence

#### Scenario: Webhook promotion drift fails closed
r[lattice.repository_validation.webhook_auth_promotion.scenario.fail_closed]
- GIVEN duplicate, unsorted, missing-removal, unrelated-removal, addition, dangling, successful-coverage, missing-binding, or stale-policy drift
- WHEN maintained promotion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion
