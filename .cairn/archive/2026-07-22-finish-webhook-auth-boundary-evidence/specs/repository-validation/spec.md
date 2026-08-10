## ADDED Requirements

### Requirement: Webhook authentication evidence completion
r[lattice.repository_validation.webhook_auth_completion] Lattice MUST complete only webhook authentication rows whose valid and rejected durable boundaries are directly asserted and BLAKE3-bound, MUST add no selected markers, and MUST preserve unrelated missing and dangling state.

#### Scenario: Safe credential-reference context
r[lattice.repository_validation.webhook_auth_completion.scenario.context]
- GIVEN authentication resolves an opaque credential reference
- WHEN rejection diagnostics and history are rendered or persisted
- THEN they MUST include trigger identity and a domain-separated BLAKE3 reference fingerprint while omitting raw provider, key, token, and secret values

#### Scenario: Webhook persistence boundaries are observed
r[lattice.repository_validation.webhook_auth_completion.scenario.boundaries]
- GIVEN valid and invalid authenticated webhook requests
- WHEN admission, execution, response rendering, history persistence, and raw-store inspection run
- THEN maintained assertions MUST prove valid admission and complete plaintext omission from every inspected durable surface

#### Scenario: Exact webhook classifications remain bounded
r[lattice.repository_validation.webhook_auth_completion.scenario.exact]
- GIVEN the two reviewed webhook rows and current evidence bytes
- WHEN completion validation runs
- THEN only exact rows MUST be reclassified, evidence MUST bind current BLAKE3 hashes, selected markers MUST remain absent, and the 329-item missing set MUST remain unchanged

#### Scenario: Webhook completion drift fails closed
r[lattice.repository_validation.webhook_auth_completion.scenario.fail_closed]
- GIVEN classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion
