# Design: Webhook listener server

## Architecture

Add a thin HTTP listener shell around the existing trigger core.

```text
HTTP server adapter
        │ raw request, peer/config facts
        ▼
WebhookRequestPlanner ─ pure request normalization, body parsing, trigger match candidate selection
        │ TriggerEvent + safe request diagnostics
        ▼
WebhookAdmissionShell ─ store reads, credential resolution, idempotency/overlap facts
        │ TriggerRunPlan or rejection record
        ▼
Headless trigger adapter ─ workflow run + trigger history persistence
```

The planner is a pure functional core. It accepts immutable request facts, configured limits, and trigger definitions. It returns a request plan, matching diagnostics, or safe response decisions. It performs no socket I/O, store access, credential lookup, clock reads, or workflow execution.

The listener shell owns the imperative work: binding sockets, reading request bodies within configured limits, loading triggers from the store, resolving credential references, collecting running/idempotency facts, calling `admit_trigger_event`, starting headless runs, writing trigger history, and returning HTTP responses.

## Request planning

Request facts include method, path, raw query pairs, headers, optional content type, and body bytes. The planner normalizes header lookup case-insensitively, parses JSON only when content type declares JSON, preserves raw body bytes for signature verification and raw-body bindings, and returns deterministic diagnostics for malformed bodies or ambiguous trigger matches.

Trigger matching first filters enabled webhook triggers by path and then lets the existing admission core enforce method, content type, auth, overlap, idempotency, and binding. This keeps matching behavior predictable while preserving current trigger diagnostics.

## Response decisions

Response mapping is another pure core that converts listener outcomes into stable status codes and JSON/text response bodies. Accepted responses include the trigger id and run id. Rejected responses include a stable rejection code and safe message. They never include plaintext secrets, authorization headers, signature values, raw request bodies, or unredacted JSON payloads.

## CLI shell

Add `webhook-listen` as a headless command using the configured data store. The command accepts a bind address from CLI/config and a dry-run mode. Dry-run loads triggers, validates webhook trigger definitions and credential references, prints safe status, and exits without binding a socket.

## Testing

Pure tests cover request normalization, JSON parse failures, trigger matching, response mapping, and redaction. Shell tests use fake stores, fake credential providers, and an in-process listener/client to cover valid request start, invalid auth, missing path, duplicate idempotency, disabled trigger, and malformed body outcomes.
