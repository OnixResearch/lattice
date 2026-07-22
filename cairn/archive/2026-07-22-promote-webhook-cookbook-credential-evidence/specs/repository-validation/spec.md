## ADDED Requirements

### Requirement: Webhook cookbook credential evidence promotion
r[lattice.repository_validation.cookbook_credential_promotion] Lattice MUST promote only the reviewed webhook cookbook credential row through markers attached to its exact maintained tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact one-ID cookbook delta
r[lattice.repository_validation.cookbook_credential_promotion.scenario.exact_delta]
- GIVEN the failing 341-item baseline receipt and the one-ID cookbook allowlist
- WHEN the maintained promotion check compares the current failing receipt
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 340

#### Scenario: Cookbook marker remains locally bound
r[lattice.repository_validation.cookbook_credential_promotion.scenario.local_binding]
- GIVEN the reviewed cookbook row has exact maintained accepted and rejected smoke assertions
- WHEN its marker is promoted
- THEN the marker MUST attach directly to both exact supporting tests and MUST NOT rely on child markers, file scope, lexical similarity, generated inventory rows, or archived evidence

#### Scenario: Cookbook promotion drift fails closed
r[lattice.repository_validation.cookbook_credential_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing expected removal, unrelated removal, added ID, dangling marker, stale reviewed policy row, or successful coverage
- WHEN the maintained promotion validator checks the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion
