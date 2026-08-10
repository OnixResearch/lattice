## Phase 1: Trigger model and admission core

- [x] [serial] r[n8nclone.scheduling.triggers.model] Add persisted trigger definitions, trigger kinds, payload schemas, auth policies, overlap policy, and idempotency policy data.
- [x] [parallel] r[n8nclone.workflow_execution.triggers.payload_binding] Add pure payload-to-workflow-input binding with positive and negative tests for JSON, query, header, and raw body sources.
- [x] [parallel] r[n8nclone.scheduling.triggers.admission] Add pure trigger admission for enabled state, method/content type, overlap, idempotency, and schedule compatibility.

## Phase 2: Webhook and scheduler shells

- [x] [serial] r[n8nclone.credentials.webhook_auth] Resolve webhook auth credentials through credential providers and reject missing or malformed secret references before workflow execution.
- [x] [parallel] r[n8nclone.scheduling.triggers.webhook] Add a webhook adapter that validates requests, binds payloads, and starts headless runner execution through trigger plans.
- [x] [parallel] r[n8nclone.scheduling.triggers.scheduler_compat] Adapt existing schedules into schedule trigger definitions without changing visible daemon behavior.

## Phase 3: Format and UX

- [x] [serial] r[n8nclone.workflow_format.triggers.nickel] Import/export typed trigger definitions in Nickel while preserving workflows without triggers.
- [x] [parallel] r[n8nclone.tui_ux.triggers.manage] Add minimal TUI or CLI inspection for trigger definitions, status, and last error.
- [x] [parallel] r[n8nclone.scheduling.triggers.history] Persist trigger run outcomes with trigger id, workflow name, idempotency key, and safe payload summary.

## Phase 4: Gates

- [x] [serial] r[n8nclone.scheduling.triggers.model] Run focused trigger/webhook tests, `./scripts/check.rs`, and archive or report blockers.
