## Phase 1: Pure request planning core

- [x] [serial] r[n8nclone.scheduling.webhook_listener.planning] Add a pure webhook request planner that normalizes method, path, query, headers, content type, raw body, and optional JSON body into trigger event facts.
- [x] [parallel] r[n8nclone.scheduling.webhook_listener.planning.bad_body] Add positive and negative planner tests for valid JSON, malformed JSON, non-JSON raw bodies, missing content types, and case-insensitive headers.
- [x] [parallel] r[n8nclone.scheduling.webhook_listener.responses] Add a pure response mapper for accepted and rejected listener outcomes with redaction assertions.

## Phase 2: Listener shell and admission

- [x] [serial] r[n8nclone.scheduling.webhook_listener.process] Add a `webhook-listen` CLI shell that loads persisted webhook triggers and binds an HTTP listener from explicit config or CLI input.
- [x] [parallel] r[n8nclone.scheduling.webhook_listener.admission] Wire listener requests through credential resolution, overlap/idempotency facts, trigger admission, headless trigger execution, and trigger history persistence.
- [x] [parallel] r[n8nclone.scheduling.webhook_listener.admission.redaction] Add shell tests proving auth failures and request diagnostics never expose plaintext secrets, authorization headers, signatures, or raw payloads.

## Phase 3: Operator controls

- [x] [serial] r[n8nclone.tui_ux.webhook_listener.cli] Add CLI status output for bind address, loaded trigger count, safe listener status, and dry-run validation diagnostics.
- [x] [parallel] r[n8nclone.scheduling.webhook_listener.process.missing] Add listener tests for missing path, disabled trigger, bad method, bad content type, duplicate idempotency key, and malformed body rejection without workflow execution.

## Phase 4: Gates

- [x] [serial] r[n8nclone.scheduling.webhook_listener.process] Run focused webhook-listener tests, `./scripts/check.rs`, `pre-commit run --all-files`, Cairn gates, sync accepted specs, and archive or report blockers.
