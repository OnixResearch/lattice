## Why

Webhook idempotency currently reads a bounded trigger-history snapshot, makes a pure admission decision, executes the workflow, and persists the accepted trigger record only after execution returns. Sequential listener handling hides the read/execute/write race: parallel deliveries with the same key can both observe no accepted record, and process death after the effect can leave no durable admission fact for redelivery to reject.

The preceding crash-boundary change makes that weakness reproducible. Lattice now needs a durable atomic winner before workflow effects and real concurrent HTTP coverage. The goal is one local admission for a trigger-scoped key, not an exactly-once claim for arbitrary external effects.

## What Changes

- Add a trigger-scoped durable idempotency claim committed atomically with a reserved run identity and safe accepted trigger-history fact before workflow execution.
- Keep pure request planning and policy validation separate from the store transaction; treat the transaction result as the authoritative final admission decision.
- Process a bounded number of webhook connections concurrently so simultaneous loopback requests exercise the real listener boundary instead of a sequential test surrogate.
- Return one accepted execution path and deterministic duplicate conflicts for losing requests without allocating or starting additional workflow runs.
- Preserve claims across execution failure and process restart, update the post-effect crash profile, and expose safe history without raw keys or payloads in responses and logs.
- Document that the claim proves only Lattice-local trigger admission and does not establish exactly-once external execution.

## Impact

- **Files**: `src/store/mod.rs`, `src/triggers.rs`, `src/headless.rs`, `src/webhooks/*`, `src/main.rs`, webhook/store/headless tests, `tests/webhooks.rs`, `scripts/operator-e2e.rs`, `README.md`, and native Cairn lifecycle artifacts.
- **Testing**: pure claim planning, atomic store races, simultaneous TCP requests, duplicate and failure cases, restart/redelivery integration with the crash profile, focused operator E2E, `scripts/check.rs`, Octet, Cairn validation/gates, and bounded Nix flake validation.
