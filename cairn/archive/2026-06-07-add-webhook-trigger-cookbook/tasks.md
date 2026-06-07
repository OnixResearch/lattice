## Phase 1: Cookbook workflow and docs

- [x] [serial] r[n8nclone.workflow_format.webhook_cookbook_example] Add a runnable Nickel webhook-triggered workflow example with safe defaults, credential references, and offline-friendly behavior.
- [x] [parallel] r[n8nclone.scheduling.webhook_cookbook_operator_flow] Document import, listener dry-run, listener start, valid request, rejected request, and run-history inspection steps.
- [x] [parallel] r[n8nclone.credentials.webhook_cookbook_secret_safety] Document credential-reference setup without embedding plaintext webhook secrets.

## Phase 2: Smoke tests

- [x] [serial] r[n8nclone.workflow_format.webhook_cookbook_example.import] Add a positive smoke test proving the cookbook workflow imports and validates from Nickel.
- [x] [parallel] r[n8nclone.scheduling.webhook_cookbook_operator_flow.accepted] Add a positive smoke test proving the cookbook valid request starts one workflow run and records safe history.
- [x] [parallel] r[n8nclone.scheduling.webhook_cookbook_operator_flow.rejected] Add negative smoke tests proving invalid path, invalid auth, malformed body, or duplicate idempotency rejects without starting a run.
- [x] [parallel] r[n8nclone.credentials.webhook_cookbook_secret_safety.redaction] Add negative redaction assertions for responses, diagnostics, history, and documentation snippets.

## Phase 3: Gates

- [x] [serial] r[n8nclone.workflow_format.webhook_cookbook_example] Run focused cookbook tests, `./scripts/check.rs`, Cairn validation/gates, sync accepted specs, and archive or report blockers.
