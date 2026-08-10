## Context

`handle_webhook_connection` currently loads recent trigger history, derives `TriggerAdmissionContext`, runs pure request admission, invokes `run_admitted_trigger_plan`, and saves the accepted `TriggerRunRecord` after the headless workflow completes. `save_trigger_run` is an independent upsert keyed by record fields. The listener awaits each connection before accepting the next one, so sequential process tests cannot expose concurrent admission.

The store already has a useful pattern for orchestration trigger claims: one redb write transaction returns inserted or duplicate. Webhook claims need a separate trigger-scoped contract because they reserve a workflow run identity, persist a safe accepted fact, and have different response and history semantics.

This change depends on `characterize-post-effect-crash-boundaries`; implementation must retain that archived baseline and deliberately update the webhook restart profile after durable claims are introduced.

### Completion contract

The change is complete only when simultaneous real loopback requests with one trigger-scoped idempotency key contend after both connections are established, exactly one store transaction claims the key and reserves the run, exactly one workflow starts, every loser receives the stable duplicate response, and restart/redelivery cannot start a second run after the claim committed. A sequential listener, a process-local mutex, two pure decisions over the same stale context, response counts without durable claim inspection, or a unique run ID without a trigger-scoped claim is not completion evidence.

False completion includes claiming after the effect, relying on bounded history lookback as the uniqueness authority, overwriting a prior claim, deleting a claim after execution failure, leaking the raw idempotency key, allowing `IdempotencyPolicy::None` to create hidden claims, or describing one local winner as exactly-once remote execution.

## Decisions

### Decision: Make the store transaction authoritative

**Choice:** Add a narrow store operation that receives a validated trigger plan and a domain-separated BLAKE3 claim identity derived from the trigger identity and idempotency key. In one redb write transaction it checks uniqueness, reserves the next run identity, inserts the immutable claim, and persists a safe accepted/started trigger-history record. Return a typed inserted-with-run-id or duplicate-with-existing-run-id result.

**Rationale:** A read followed by a later write cannot prevent two contenders. One transaction closes the race and gives the workflow shell a run identity that was durably bound to the trigger/key before effects.

### Decision: Keep policy in a pure admission core

**Choice:** Continue to normalize requests and validate trigger enablement, authentication, payload binding, overlap, and idempotency-key derivation in pure code. Add a pure post-claim decision that maps inserted, duplicate, malformed, and no-idempotency-policy facts to execution or rejection. The imperative shell alone performs the redb transaction.

**Rationale:** Store I/O should not absorb request parsing or policy semantics. The transaction arbitrates uniqueness; the core remains directly testable without a database or network.

### Decision: Retain claims after uncertain or failed execution

**Choice:** Once the claim commits, never free it automatically because workflow execution fails, the listener dies, or the HTTP response is lost. Update safe trigger history with terminal failure when observable, while a crash can leave the pre-effect accepted/started fact. Redelivery receives the duplicate response and does not execute again.

**Rationale:** Releasing a claim after an unknown boundary recreates duplicate-effect risk. Retention may require operator investigation and can suppress an unobserved attempt, but it preserves the stronger local at-most-one-admission statement.

### Decision: Exercise bounded concurrent listener handling

**Choice:** Refactor the accept shell to own a named maximum in-flight bound, spawn connection handlers under that bound, and drain completed handlers without unbounded task growth. Share only thread-safe store, trigger, credential, and limit inputs.

**Rationale:** Atomic store tests are necessary but insufficient for the requested process boundary. Real simultaneous clients must reach concurrent handlers. A reviewed bound prevents an idempotency test from introducing unbounded server concurrency.

### Decision: Preserve no-idempotency behavior explicitly

**Choice:** Triggers configured with `IdempotencyPolicy::None` do not create a claim and may admit separate concurrent events according to overlap policy. Tests make that distinction explicit.

**Rationale:** Silent deduplication would change the declared trigger policy and make unrelated requests share authority accidentally.

## Functional Core / Imperative Shell

The pure core derives scoped claim identity, validates claim inputs, and maps transaction facts to execution or duplicate rejection. The store shell owns run-counter and claim/history mutation in one transaction. The listener shell owns TCP acceptance, bounded task orchestration, clock input, response writes, and logging. Workflow effects remain in the existing headless executor.

## Risks / Trade-offs

- A committed claim followed by death before the workflow effect can suppress later delivery even though no external effect occurred; the durable record must expose local uncertainty rather than claim success.
- The single redb owner prevents multiple listener processes from sharing one store; this change proves bounded in-process contention, not distributed multi-writer admission.
- Concurrent handlers can reorder unrelated responses and history records; stable identity and per-request outcomes must not depend on arrival completion order.
- Existing trigger-history serialization must remain backward readable, and raw idempotency keys must not be added to operator-facing diagnostics.
