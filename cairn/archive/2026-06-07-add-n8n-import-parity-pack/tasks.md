## Phase 1: Fixture pack

- [x] [serial] r[n8nclone.workflow_format.n8n_parity_fixture_pack] Add supported and unsupported n8n JSON fixture directories with metadata describing each compatibility intent.
- [x] [parallel] r[n8nclone.workflow_format.n8n_parity_fixture_pack.supported] Add supported fixtures for common importable workflow shapes and positive import assertions.
- [x] [parallel] r[n8nclone.workflow_validation.n8n_parity_diagnostics] Add unsupported fixtures for common rejected n8n constructs and negative diagnostic assertions.

## Phase 2: Mapping reports and docs

- [x] [serial] r[n8nclone.workflow_format.n8n_parity_mapping_reports] Add deterministic golden mapping-report assertions for supported and unsupported fixtures.
- [x] [parallel] r[n8nclone.credentials.n8n_parity_secret_safety] Add secret-placeholder redaction assertions for fixtures, reports, diagnostics, and test output.
- [x] [parallel] r[n8nclone.workflow_format.n8n_parity_fixture_pack.docs] Document the tested supported subset, rejected feature classes, and fixture contribution workflow.

## Phase 3: Gates

- [x] [serial] r[n8nclone.workflow_format.n8n_parity_fixture_pack] Run focused n8n fixture tests, `./scripts/check.rs`, Cairn validation/gates, sync accepted specs, and archive or report blockers.
