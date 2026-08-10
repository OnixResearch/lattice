## Why

Lattice can author, import, trigger, execute, and replay one typed workflow DAG, and Seaglass supplies its reusable graph admission and lifecycle decisions. It does not yet have a first-class durable construct for coordinating multiple DAG runs toward one objective. Operators must currently place feedback loops, graph revisions, approval checks, budget accounting, and crash recovery in an external controller, which hides authority and can bypass Lattice's event, validation, and evidence boundaries.

## What Changes

- Add a typed orchestration definition and durable instance model around immutable BLAKE3-identified workflow revisions.
- Add a pure orchestration decision core that advances explicit lifecycle, objective, termination, budget, approval, and next-round facts without performing I/O.
- Keep every execution round as a normal acyclic Lattice workflow admitted by Seaglass; represent feedback as an explicit proposal for a later immutable revision rather than a hidden cycle.
- Require agent-produced graph or policy changes to remain inert proposals until an authorized operator or predeclared transition policy admits them.
- Reserve bounded run, invocation, token, monetary, and duration budgets before dispatch; stop new work deterministically when required bounds are absent or exhausted.
- Persist an ordered orchestration event ledger and durable dispatch intents so pause, resume, cancellation, replay, and crash recovery do not depend on ambient process state.
- Add typed Nickel authoring plus headless operator commands for validation, start, status, pause, resume, approval, rejection, cancellation, and replay.
- Integrate existing schedule and webhook triggers without claiming unsupported n8n orchestration parity.

## Success Criteria

- An operator can start a durable orchestration from a validated Nickel definition and observe one BLAKE3-bound instance identity.
- The pure core can be tested from in-memory definitions, state, policy, and facts, and returns deterministic decisions without filesystem, process, network, database, clock, or UI access.
- Each round delegates DAG admission, readiness, ordering, and lifecycle to the existing Seaglass-backed runner while orchestration policy remains owned by Lattice.
- An agent cannot mutate the active graph, widen authority, add credentials, or increase budgets by emitting ordinary workflow output.
- Duplicate triggers, stale approvals, duplicate completion events, budget exhaustion, invalid revisions, terminal-state mutation, and interrupted dispatches fail or recover deterministically.
- Replay reconstructs orchestration state and round lineage without rerunning workflow effects.
- Existing workflows, n8n JSON import behavior, headless runs, schedules, webhooks, and the current LLM operating graph remain compatible.

## Non-Goals

- Full n8n node, expression, credential, or orchestration compatibility.
- Hidden cyclic execution inside a Seaglass-admitted run.
- Ambient agent authority to edit workflows, credentials, triggers, budgets, approvals, or runtime policy.
- Distributed workers, leader election, global fairness, or remote exactly-once side effects in the first implementation.
- Treating model output, provider usage reports, Seaglass decisions, or a passing replay as proof of objective truth, policy correctness, external effect uniqueness, or release eligibility.

## Impact

- **Files**: new orchestration core/shell/store modules, workflow revision and event models, Nickel contracts and format handlers, scheduler/webhook adapters, CLI/TUI status surfaces, examples, tests, and Cairn traceability artifacts.
- **Compatibility**: existing workflows remain leaf execution units; orchestration fields are additive and Lattice-native. Default n8n export rejects orchestration semantics unless an exact mapping is later specified.
- **Testing**: pre-change repository baseline; positive and negative pure-core state tests; budget, approval, duplicate-event, stale-revision, crash-recovery, trigger-idempotency, replay, format, and CLI tests; `scripts/check.rs`; Octet; Cairn gates/validation; bounded `nix flake check`.
