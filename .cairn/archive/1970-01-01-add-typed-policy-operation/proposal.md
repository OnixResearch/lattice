# Proposal: Add an optional bounded typed policy operation

Tracking: [Lattice #23](https://github.com/OnixResearch/lattice/issues/23). Producer: [Animus #15](https://github.com/OnixResearch/animus/issues/15) (`crates/animus-frozen-policy`, Animus candidate revision `1c370d5029a2667051c46bdfa1919e78e65e2f4b`).

## Why

Small, repeated workflow decisions are a candidate for a frozen fast policy instead of a deliberating model or a hand-written rule. A parsed model answer does not establish that the answer belongs to the declared action space, that it was produced by the artifact the workflow selected, or that any downstream operation is authorized. Lattice already owns the workflow branch, its admission and its durable run record; Animus #15 now supplies a pure, identity-bound frozen selection leaf with bounded receipts. The missing work is one Lattice consumer that binds those identities, validates the candidate independently, records the decision durably, replays it without inference, and routes only through the continuation the workflow already declared.

## What Changes

- Define the versioned operation profile `lattice.typed-policy-profile.v1` for one selection task: input projection and schema, bounded label set, result schema, input/receipt limits, deterministic and wall-clock deadlines, supported artifact/runtime cohort and declared fallback. Only the `select` shape is implemented. r[lattice.typed_policy.profile]
- Bind an enabled profile to one declared Condition node of one workflow. The policy chooses one of that node's already declared branch labels; it cannot add a node, port, tool, credential, label or effect. Disabled or absent profiles leave execution, cache keys and run records byte-identical. r[lattice.typed_policy.route]
- Invoke the Animus frozen policy through a Lattice Animus adapter (`src/animus_agent/frozen.rs`) with an immutable artifact identity binding model, feature schema, action schema, policy kind and runtime. Inference stays outside the pure Lattice core, which only projects, validates and resolves. r[lattice.typed_policy.invocation]
- Validate the returned receipt independently in Lattice and classify it as accepted, abstained, unavailable, timed-out or invalid, plus cancelled. Confidence is evidence for the producer's abstention rule, never permission. r[lattice.typed_policy.validation]
- Resolve every non-accepted outcome through the profile's declared bounded fallback or an explicit terminal failure; cancellation is always terminal. r[lattice.typed_policy.fallback]
- Persist the invocation, projected-input, candidate-set, artifact and profile identities, the bounded raw receipt, accepted and routed labels, fallback reason, reserved/observed operation units and eventual workflow outcome in the existing run detail record. r[lattice.typed_policy.record]
- Replay a persisted decision from the record and receipt alone, starting no adapter, model or effect work; a profile or artifact change is a compatibility refusal, never a rewrite. r[lattice.typed_policy.replay]
- Stay stateless: a profile requesting session or recurrent state is refused, keeping the Animus session rejection until #12 qualifies state. r[lattice.typed_policy.stateless]
- Measure the log-triage routing task against the deterministic rule and the present reasoning-model path under thresholds declared before evaluation, on held-out families, and retain the actual result, including a no-go. r[lattice.typed_policy.measurement]

## Impact

- **Dependencies:** re-vendor Animus at `1c370d5029a2667051c46bdfa1919e78e65e2f4b` through `scripts/sync_animus.rs`, adding `animus-frozen-policy` (no_std + alloc; `blake3`, `serde`, `serde_json` only) as a path dependency.
- **Files:** `src/workflow/decision.rs` and `src/workflow/decision/` (pure core), `src/animus_agent/frozen.rs` (adapter shell), `src/engine/executor/choice.rs` (Condition hook) and the cache-key fold, `RunNodeDetail` optional `policy_decision`, headless persistence, approval suspension/resume persistence in `src/workflow/gate/shell/{suspension,resume,policy}.rs`, `tests/test_typed_policy.rs`, `tests/test_typed_policy_resume.rs`, `tests/fixtures/typed-policy/`, `examples/typed_policy_measurement.rs`, `docs/typed-policy-operations.md`.
- **Testing:** focused `test_typed_policy` and `test_typed_policy_resume`, existing control-flow/examples/run-history/workflow-approval regressions, Clippy `-D warnings`, fmt, Cairn validation and gates, Octet semantic delta, and the Nix `nextest` check.

## Existing Work and Scope Boundaries

- Animus owns the frozen-policy runtime, its artifact contract and its receipts; RSI Lab owns training, distillation and export. Lattice owns the domain projection, label set, validation, workflow composition and durable records.
- [Lattice #16](https://github.com/OnixResearch/lattice/issues/16) remains declared preference-factor recording with exact-label role-protocol admission; this change does not choose role-protocol labels.
- [Lattice #17](https://github.com/OnixResearch/lattice/issues/17) owns uncertainty after role-protocol dispatch; no policy result settles an effect.
- [Lattice #12](https://github.com/OnixResearch/lattice/issues/12) owns state, checkpoints and any recurrent-policy envelope.
- [Lattice #13](https://github.com/OnixResearch/lattice/issues/13) owns usage accounting; this change records reserved and observed deterministic operation units separately and records no charge.

## Out of Scope

- A second workflow engine, a universal model runtime, a new node kind, a CLI flag or a default-on profile.
- Training, online weight updates, artifact download, device loading or a GPU path.
- Extract, rank, classify-to-value or gate shapes beyond the one `select` shape the fixture needs.
- Choosing role-protocol choice labels, authorizing effects, or replacing receiver/terminal observations.
- Any claim that the fixture policy improves on the rule or on a reasoning model.

## Affected Specs

- `typed-policy-operations` (new): profile, route, invocation, validation, fallback, record, replay, stateless and measurement requirements.
