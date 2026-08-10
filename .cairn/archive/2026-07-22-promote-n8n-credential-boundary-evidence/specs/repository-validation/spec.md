## ADDED Requirements

### Requirement: n8n credential boundary evidence promotion
r[lattice.repository_validation.n8n_credential_promotion] Lattice MUST promote only the four reviewed exact n8n credential boundary rows through markers attached to their maintained tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact four-ID n8n credential delta
r[lattice.repository_validation.n8n_credential_promotion.scenario.exact_delta]
- GIVEN the failing 340-item baseline receipt and the explicit four-ID n8n credential allowlist
- WHEN the maintained promotion check compares the current failing receipt
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 336

#### Scenario: n8n credential markers remain locally bound
r[lattice.repository_validation.n8n_credential_promotion.scenario.local_binding]
- GIVEN each reviewed n8n credential row has exact maintained boundary assertions
- WHEN its marker is promoted
- THEN each marker MUST attach directly to every required supporting test and MUST NOT rely on implementation names, file scope, lifecycle classification tests, generated inventory rows, or archived evidence

#### Scenario: n8n credential promotion drift fails closed
r[lattice.repository_validation.n8n_credential_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing expected removal, unrelated removal, added ID, dangling marker, missing local binding, stale reviewed policy row, or successful coverage
- WHEN the maintained promotion validator checks the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion
