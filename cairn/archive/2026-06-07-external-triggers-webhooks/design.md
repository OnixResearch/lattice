# Design: External triggers and webhooks

## Architecture

Introduce a pure trigger admission layer and thin adapters for each inbound source.

```text
Manual command / schedule tick / webhook request
        │
        ▼
TriggerAdmissionCore ─ validation, idempotency, overlap, payload binding
        │
        ▼
TriggerShell ─ load workflow, resolve credentials, start runner, persist outcome
```

The core accepts immutable trigger definitions, explicit current time, running trigger identities, idempotency keys, and request facts. It returns either a `TriggerRunPlan` or deterministic diagnostics. It performs no network, filesystem, store, clock, or process I/O.

## Trigger model

A trigger definition contains a stable id, workflow reference, enabled flag, trigger kind, payload schema, input binding, overlap policy, idempotency policy, and optional authentication policy. Schedule triggers reuse recurrence rules. Webhook triggers include method, path token, accepted content types, auth policy, and payload extraction rules.

## Webhook auth and payloads

Webhook shared secrets are credential references. The shell resolves secrets only at request admission time and never serializes plaintext. Initial auth policies should include no-auth for local-only use, static bearer token, and HMAC signature using a configured header and BLAKE3 or protocol-required hash. If an external protocol mandates another algorithm, that algorithm is used for compatibility and documented in the trigger definition.

Payload binding maps JSON body fields, query parameters, headers, and raw body bytes to declared workflow input ports. Binding failures are fatal for the request and do not mutate workflow state.

## Idempotency and overlap

Every trigger kind may derive an idempotency key from request metadata or explicit payload fields. The trigger shell checks and records accepted keys before starting side effects. Overlap policy reuses scheduling semantics: forbid, allow, or queue later once queueing exists.

## Format and compatibility

Typed Nickel import/export gains a `triggers` section. Existing workflows without triggers import unchanged. Current schedule definitions can migrate into schedule trigger definitions while preserving run timing and overlap semantics.

## Testing

Use pure core tests for admission, auth decisions over pre-resolved secret facts, payload binding, overlap, and idempotency. Use fake stores and fake runners for shell tests. Webhook adapter tests cover positive signed requests and negative invalid signature, disabled trigger, bad method, bad content type, malformed JSON, and duplicate idempotency key cases.
