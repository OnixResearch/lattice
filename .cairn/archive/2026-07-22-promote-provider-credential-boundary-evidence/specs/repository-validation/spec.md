## ADDED Requirements

### Requirement: Provider credential evidence promotion
r[lattice.repository_validation.provider_credential_promotion] Lattice MUST promote only the seven reviewed exact provider credential rows through markers attached to maintained boundary tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact seven-ID provider delta
r[lattice.repository_validation.provider_credential_promotion.scenario.exact_delta]
- GIVEN the failing 336-item baseline receipt and explicit seven-ID provider allowlist
- WHEN maintained promotion validation compares current failing coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 329

#### Scenario: Provider markers remain locally bound
r[lattice.repository_validation.provider_credential_promotion.scenario.local_binding]
- GIVEN every selected provider row has exact maintained boundary assertions
- WHEN its marker is promoted
- THEN each marker MUST attach directly to its required test and MUST NOT rely on implementation names, file scope, lifecycle tests, generated inventory, or archived evidence

#### Scenario: Provider promotion drift fails closed
r[lattice.repository_validation.provider_credential_promotion.scenario.fail_closed]
- GIVEN duplicate, unsorted, missing-removal, unrelated-removal, addition, dangling, successful-coverage, missing-binding, or stale-policy drift
- WHEN the maintained promotion validator runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion
