## Phase 1: Import core

- [x] [serial] r[n8nclone.workflow_format.n8n_json.import] Add pure n8n JSON parser/normalizer, supported-node mapping table, and import report model.
- [x] [parallel] r[n8nclone.workflow_validation.n8n_json.unsupported] Add deterministic diagnostics for unsupported node kinds, malformed connections, unsupported expressions, and behavior changes.
- [x] [parallel] r[n8nclone.credentials.n8n_json.placeholders] Import n8n credential references as opaque placeholders and reject plaintext credential material.

## Phase 2: CLI/format shell

- [x] [serial] r[n8nclone.workflow_format.n8n_json.import_cli] Add import command path that reads n8n JSON, writes mapping reports, and saves valid workflows without changing Nickel behavior.
- [x] [parallel] r[n8nclone.workflow_format.n8n_json.fixtures] Add positive and negative n8n JSON fixtures for shell, HTTP, malformed JSON, unsupported nodes, and credential cases.

## Phase 3: Export support

- [x] [serial] r[n8nclone.workflow_format.n8n_json.export] Export supported n8nclone workflow subsets to deterministic n8n JSON with explicit mapping reports.
- [x] [parallel] r[n8nclone.workflow_validation.n8n_json.export_limits] Reject or report unsupported export features before writing misleading n8n JSON.

## Phase 4: Gates

- [x] [serial] r[n8nclone.workflow_format.n8n_json.import] Run focused n8n interop tests, `./scripts/check.rs`, and archive or report blockers.
