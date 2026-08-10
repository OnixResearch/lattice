## ADDED Requirements

### Requirement: Core credential evidence promotion
r[lattice.repository_validation.core_credential_promotion] Lattice MUST promote the reviewed core credential tranche only through markers attached to exact maintained tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact 12-ID credential delta
r[lattice.repository_validation.core_credential_promotion.scenario.exact_delta]
- GIVEN the failing 353-item baseline receipt and the explicit 12-ID credential allowlist
- WHEN the maintained promotion check compares the current failing receipt
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 341

#### Scenario: Credential markers remain locally bound
r[lattice.repository_validation.core_credential_promotion.scenario.local_binding]
- GIVEN each reviewed credential scenario has exact maintained assertions
- WHEN its marker is promoted
- THEN the marker MUST be attached directly to an exact supporting test and MUST NOT rely on parent markers, file scope, lexical similarity, generated inventory rows, or archived evidence

#### Scenario: Credential promotion drift fails closed
r[lattice.repository_validation.core_credential_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing expected removal, unrelated removal, added ID, dangling marker, stale reviewed policy row, or successful coverage
- WHEN the maintained promotion validator checks the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion
