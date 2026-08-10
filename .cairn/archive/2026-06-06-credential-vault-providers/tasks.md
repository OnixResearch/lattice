## Phase 1: Provider registry and validation

- [x] [serial] r[n8nclone.credentials.provider_registry] Add provider registry config, provider metadata, capability checks, and fake provider tests.
- [x] [parallel] r[n8nclone.workflow_validation.credentials.providers] Add validation diagnostics for unknown providers, unsupported field access, malformed keys, and missing provider configuration.
- [x] [parallel] r[n8nclone.credentials.provider_diagnostics.redaction] Ensure provider diagnostics and test surfaces never display resolved secret values.

## Phase 2: Concrete providers

- [x] [serial] r[n8nclone.credentials.providers.env] Add environment-variable provider with positive/negative tests for present, missing, empty, and field-access cases.
- [x] [parallel] r[n8nclone.credentials.providers.local_vault] Add encrypted local vault storage, key-source config, integrity validation, and tamper/wrong-key tests.
- [x] [parallel] r[n8nclone.credentials.providers.command] Add external command provider behind `ProcessRunner` with timeout, nonzero, malformed output, and output-limit tests.

## Phase 3: Format and UX

- [x] [serial] r[n8nclone.workflow_format.credentials.providers] Import/export provider configuration references without plaintext secret material.
- [x] [parallel] r[n8nclone.tui_ux.credentials.providers] Add TUI/CLI provider reachability checks and safe credential-reference editing helpers.
- [x] [parallel] r[n8nclone.credentials.provider_cache_identity] Prove cache keys include provider reference identity and exclude resolved secret values across providers.

## Phase 4: Gates

- [x] [serial] r[n8nclone.credentials.provider_registry] Run focused credential-provider tests, `./scripts/check.rs`, and archive or report blockers.
