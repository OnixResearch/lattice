## ADDED Requirements

### Requirement: Provider credential boundary evidence completion
r[lattice.repository_validation.provider_credential_completion] Lattice MUST complete only provider credential rows whose accepted boundaries are directly asserted and BLAKE3-bound, MUST add no selected provider markers, and MUST preserve every missing and dangling linkage state.

#### Scenario: Provider diagnostics are safe before rendering
r[lattice.repository_validation.provider_credential_completion.scenario.diagnostics]
- GIVEN a provider succeeds or returns arbitrary backend text derived from secret material
- WHEN the probe result is constructed and rendered
- THEN output MUST retain only opaque reference identity, reachability, and fixed safe status text and MUST omit backend and secret text

#### Scenario: Provider boundaries are directly observed
r[lattice.repository_validation.provider_credential_completion.scenario.boundaries]
- GIVEN cache, command, local-vault, timeout, tamper, workflow-file, and executor boundaries
- WHEN maintained positive and negative integration tests run
- THEN every reviewed provider clause claimed exact MUST have a direct assertion at its accepted boundary

#### Scenario: Exact provider classifications remain bounded
r[lattice.repository_validation.provider_credential_completion.scenario.exact]
- GIVEN the reviewed provider rows and current maintained evidence bytes
- WHEN completion validation runs
- THEN only exact rows MUST be reclassified, evidence paths MUST bind current BLAKE3 hashes, selected markers MUST remain absent, and the 336-item missing set MUST remain unchanged

#### Scenario: Provider completion drift fails closed
r[lattice.repository_validation.provider_credential_completion.scenario.fail_closed]
- GIVEN classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN the maintained completion validator runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion
