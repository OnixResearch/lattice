## ADDED Requirements

### Requirement: n8n credential boundary evidence completion
r[lattice.repository_validation.n8n_credential_completion] Lattice MUST maintain exact byte-bound evidence for the four reviewed n8n credential rows without promoting their requirement markers or changing unrelated credential classifications.

#### Scenario: Import and export boundaries are observed
r[lattice.repository_validation.n8n_credential_completion.scenario.boundaries]
- GIVEN opaque and plaintext n8n credential imports plus a provider-resolved reference-bearing workflow
- WHEN import execution, rejection, save, raw-store inspection, and CLI export run
- THEN exact assertions MUST observe provider resolution, pre-side-effect failure, store non-creation, opaque exported identity, and plaintext omission

#### Scenario: Parity surfaces preserve the bounded exception
r[lattice.repository_validation.n8n_credential_completion.scenario.parity]
- GIVEN the designated plaintext rejection fixture and every other maintained parity fixture and report
- WHEN import reports, diagnostics, and CLI output are inspected
- THEN only the designated input fixture MAY contain exactly one sentinel and every output surface MUST omit it while retaining reference guidance

#### Scenario: Exactly four rows become exact
r[lattice.repository_validation.n8n_credential_completion.scenario.exact]
- GIVEN the reviewed 340-item inventory and the four-ID n8n credential tranche
- WHEN the completion policy is generated
- THEN exactly those four rows MUST be exact, the other reviewed rows MUST remain unchanged, and none of the four IDs may gain an implementation or verification marker

#### Scenario: n8n completion drift fails closed
r[lattice.repository_validation.n8n_credential_completion.scenario.fail_closed]
- GIVEN a row omission, duplicate, reclassification, rationale or evidence drift, selected marker, missing-set change, dangling marker, or successful coverage
- WHEN the maintained completion validator checks the tranche
- THEN it MUST reject the drift with deterministic issue codes
