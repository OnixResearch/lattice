# Lattice

Lattice is a graph-native workflow and agent runtime. It owns product orchestration, effects, persistence, retries, triggers, diagnostics, and operator interfaces.

Choregraph supplies deterministic graph and machine decisions. Other adapters provide bounded process, agent, file, and provider functions.

## Internal crates

- `lattice-core` owns pure values, expression mapping, control-flow decisions, and schedule identity framing.
- `budget-ledger-core` owns pure orchestration budget reservation and settlement.
- The root `lattice` package owns host effects and preserves the existing public paths through compatibility re-exports.

`lattice-core` has no filesystem, process, network, persistence, clock, provider, or terminal authority.

[Workflow domain boundaries](docs/workflow-domain-boundaries.md) define ownership for Lattice, Choregraph, Subwayrat, persistence, and time.

[Explicit workflow expression dependencies](docs/expression-dependencies.md) define extraction, projection, runtime context, identity, and compatibility boundaries.

[Bounded ready-frontier dispatch](docs/bounded-ready-frontier.md) defines event-driven scheduling, capacities, conflicts, ordering, persistence, and proof boundaries.

[Storage commit uncertainty](docs/storage-commit-uncertainty.md) defines commit outcomes, process-local quarantine, exact read-back reconciliation, and repair non-claims.

[Choregraph effect authority](docs/choregraph-effect-authority.md) defines exact intent binding, Basalt enforcement, durable attempts, scope currentness, and capability routing.

[Durable workflow exchange application](docs/workflow-exchange-application.md) binds the published Unix transport to exact saved workflow revisions, durable acceptance, bounded observations, and explicit uncertainty.

## Development

```bash
nix develop          # enter devshell
pre-commit install   # install the Octet pre-commit hook
scripts/check.rs                              # run operator E2E, guards, fmt, tests, and clippy
scripts/check-workflow-domain-boundaries.rs   # reject core clock reads and non-TUI Subwayrat leakage
scripts/check-orchestration-identity-boundary.rs # reject raw orchestration identities in the typed core
scripts/operator-e2e.rs                       # exercise TUI PTY, webhook TCP, scheduler process, and orchestration recovery boundaries
octet-precommit-check  # run the fail-closed Octet stable-ID gate
cargo octet check    # inspect raw warning-level Octet output
cargo check          # quick compile check
nix flake check -L --option secret-key-files '' # full CI; check outputs do not require a host signing secret
```

The Octet hook uses the library target profile. It treats each configured lint as `deny`.

`octet-baseline.json` records accepted debt. A new finding ID fails the hook until a review accepts it.

## Choregraph machine source

Lattice pins these packages from Choregraph:

- `choregraph-choreography`
- `choregraph-core`
- `choregraph-kamacite`
- `choregraph-nickel`
- `choregraph-seaglass`

The source identity is:

- RID: `rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF`
- Runtime revision: `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`
- Readiness proof revision: `b3e08e19750f53bdbcae970cdf58a47a791ed20b`

Cargo, Nix, and Crane use the same source. The old vendored execution-graph implementation is removed.

Choregraph owns these decisions:

- graph admission and order
- readiness and lifecycle
- resume and analytics
- bounded replay

Lattice retains these functions:

- Nickel evaluation and product decoding
- runtime selection and effects
- persistence and retries
- triggers, diagnostics, UI, and evidence

Lattice also consumes the frozen Choregraph role-projection cohort. It executes exact local protocols with durable uncertainty and current product authority. See [projected role-protocol execution](docs/role-protocol-execution.md).

Nickel workflow import converts decoded values into a bounded in-memory `lattice-workflow-v2` value.

The pure Choregraph lowerer must admit that value before Lattice finishes product decoding. The lowerer performs no I/O or runtime action.

## Durable scheduler occurrences

Before a workflow effect, the scheduler commits a BLAKE3 identity for `(schedule_id, due_epoch_ms)`.

One redb transaction performs these actions:

1. Make sure that the due slot is current.
2. Reserve a globally unique run ID.
3. Record the workflow revision and pending attempt.
4. Advance the schedule.

An ordinary daemon restart cannot automatically admit that occurrence again.

Inspect and close uncertainty explicitly:

```bash
lattice --config ./demo/config.ncl scheduler-occurrences list
lattice --config ./demo/config.ncl scheduler-occurrences show <occurrence-b3>
lattice --config ./demo/config.ncl scheduler-occurrences recover <occurrence-b3>
lattice --config ./demo/config.ncl scheduler-occurrences retry <occurrence-b3> --acknowledge-duplicate-risk
lattice --config ./demo/config.ncl scheduler-occurrences abandon <occurrence-b3> --acknowledge-unknown-outcome
```

Recovery settles only an exact terminal run that the active attempt owns. Missing run evidence remains `PendingUnobserved`.

One explicit retry is available while the saved workflow revision still matches. The retry reserves a new run and can duplicate an unrecorded effect.

Abandonment records `AbandonedUnknown`. It does not run the effect again or invent a terminal result.

## Crash-boundary evidence

`scripts/operator-e2e.rs` exercises selected process-death windows on supported Unix and Linux hosts.

The scheduler profile records these facts after a selected crash:

- one pending occurrence
- an advanced schedule
- one local effect attempt
- no terminal run
- no automatic retry after restart
- explicit unknown-outcome abandonment

The current webhook profile commits a trigger-scoped BLAKE3 claim before effects. A redelivery receives a conflict and does not start a second local run.

These checks establish local at-most-one automatic admission under one active redb owner.

They do not prove provider deduplication, distributed writer safety, response delivery, external cancellation, exactly-once execution, or production readiness.

## TUI recovery evidence

On supported hosts, the PTY matrix exercises these paths:

- successful execution
- deterministic failure inspection
- explicit fresh rerun
- validation rejection without effects
- cancellation after child readiness
- terminal resize and restore
- history inspection and keyboard exit
- non-terminal rejection

The checks compare semantic surfaces, not exact ANSI frames.

They do not prove rendering equivalence, unsupported-host behavior, arbitrary cleanup, external cancellation, workflow correctness, or production readiness.

## Traceability debt

The maintained inventory classifies the exact missing-requirement set. Inventory rows are not traceability evidence.

Generate the expected failing receipt before you update the inventory:

```bash
nix run path:../cairn#cairn -- traceability coverage --root . \
  --policy cairn-policy/generated/cairn-policy.json \
  > target/traceability-coverage.json # expected to exit nonzero while debt remains
./scripts/traceability-inventory.rs generate target/traceability-coverage.json
./scripts/traceability-inventory.rs check target/traceability-coverage.json
```

The review policy is `cairn-policy/traceability-inventory.ncl`. The generated report is `cairn-policy/generated/traceability-debt-inventory.json`.

Each reviewed candidate binds a safe path and current BLAKE3 file hash. An `unassessed` row makes no behavior or evidence claim.

## Workflow examples

Examples in `examples/` include daily digest, CSV cleanup, log triage, webhook triggers, and the [`llm-operating-graph`](examples/workflows/llm-operating-graph.md).

```bash
lattice --config ./demo/config.ncl --init
lattice --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
lattice --config ./demo/config.ncl run daily-digest --output json
```

`--init` is create-only. It fails if the selected config path or another initialization target exists.

It also creates the local `workflow-templates/` catalog. Press `Ctrl-P` and select **Browse Workflow Templates** to open it.

Template selection and preview stay in memory. They do not save, start orchestration, call an agent, or run a node.

See [Local workflow-template bundles](docs/workflow-template-bundles.md) for the bundle contract, preview steps, activation boundary, and non-claims.

The built-in catalog also contains `cairn-agent-drain` and `octet-agent-burndown`. Each bundle runs one visible acyclic round with at most one Pi agent. Strict progress, positive budgets, and separate operator approval control later rounds. The bundles never push changes. See [Guarded agent workflow templates](docs/guarded-agent-templates.md) for commands, recovery steps, and authority limits.

Lattice also provides an identity-bound [agent-evaluation lane](docs/agent-evaluation.md). The lane loads typed Nickel catalog and planning inputs, runs bounded command drivers, keeps objective gates authoritative, and stores metadata-only receipts. Live runs use preview-first commands and require exact operator, target, credential-reference, retention, and cleanup authority.

[Typed operation contracts](docs/operation-contracts.md) define reusable methods separately from data-only instances. Direct runs and promoted workflow nodes share one identity-bound plan. Execution is bounded, preview-first, authority-checked, and metadata-only in durable evidence.

[Admitted execution backend orchestration](docs/execution-backend-orchestration.md) binds one reviewed Animus planner cohort to product-owned route admission, durable attempts, recovery, and safe inspection.

[Immutable operation data](docs/operation-data.md) preserves selected outputs as exact resource or file versions. A closed query IR resolves dynamic selectors before dispatch. Retention and orphan pruning remain separate.

[Content-addressed operation packs](docs/operation-packs.md) distribute exact catalogs, external runtimes, analyzers, and passive skills. Pure admission binds trust, platform, license, capability, dependency, evidence, and lock facts before any effect.

Edit an existing Nickel file directly, or select a new path.

## Native OpenAI-compatible agents

[`examples/config/openai-compatible-agent.ncl`](examples/config/openai-compatible-agent.ncl) maps Pi and Clankers target labels to a bounded chat-completions endpoint.

The checked defaults use the local `VibeThinker-3B` service on port 13305. They include token and time limits.

Set `agent_runtime.runtimes = {}` for native routes. Change the endpoint, model, and limits together for another provider.

The adapter supports strict text and JSON output. It disables redirects and bounds request bytes, response bytes, and time.

It never uses hidden reasoning as final content. One leading `<think>...</think>` block can be removed from content.

Nested, repeated, nonleading, or incomplete markers fail closed. A `finish_reason` value of `length` also fails.

Local HTTP endpoints must not use bearer credentials. For authenticated HTTPS, use `api_key_env` and a `secret_env` credential reference.

Do not put credentials in Nickel.

```bash
lattice --config examples/config/openai-compatible-agent.ncl import examples/workflows/llm-operating-graph.ncl
lattice --config examples/config/openai-compatible-agent.ncl run llm-operating-graph --output json
```

## Bounded Animus leaf agents

[`examples/config/animus-agent.ncl`](examples/config/animus-agent.ncl) maps the `animus` target to a fixed BLAKE3-identified two-turn Steel loop.

Animus controls only the inner model and response sequence for one generic agent node.

Lattice retains credentials, transport limits, graph boundaries, effects, budgets, events, persistence, triggers, approvals, and orchestration.

The first profile performs one model dispatch. It supports text or JSON output.

It exposes no tools, arbitrary Steel programs, profile loading, sessions, fallback, Animus graphs, or Animus persistence.

Returned metadata contains bounded identities and counts. It excludes prompts, responses, credentials, and a second provenance ledger.

```bash
cargo run --example generate_animus_workflow
lattice --config examples/config/animus-agent.ncl import examples/workflows/animus-bounded-agent.ncl
lattice --config examples/config/animus-agent.ncl run animus-bounded-agent --output json
```

Refresh the reviewed Animus snapshot with:

```bash
scripts/sync_animus.rs ../animus vendor-cargo-deps/animus
```

## Durable workflow approvals

A typed approval node stops a run at a persisted Choregraph suspension point. The initiating process does not wait for an operator.

Approval decisions and workflow resume are separate actions. Resume reloads the exact checkpoint and runs only the remaining ready nodes.

Use `lattice approvals list`, `lattice approvals show`, and `lattice approvals receipt` for safe metadata. Use `lattice approvals decide` to record an approval or rejection.

Use `lattice workflow resume` with the exact request, revision, event head, owner, typed values, and opaque credential references.

See [Durable workflow approvals](docs/workflow-approvals.md) for the node contract, command syntax, authority checks, expiry behavior, recovery limits, and non-claims.

## Durable agent orchestration

Lattice coordinates immutable workflow revisions across explicit rounds. Each round remains an acyclic workflow that Choregraph admits.

Agent output can propose a successor revision. It cannot change graphs, credentials, triggers, abilities, or budgets without a hash-bound approval.

The [`llm-review-loop`](examples/orchestrations/llm-review-loop.ncl) example uses bounded rounds, invocations, tokens, duration, trigger identities, and `operator:local` authority.

```bash
cargo run --example generate_llm_orchestration
lattice --config ./demo/config.ncl orchestration validate examples/orchestrations/llm-review-loop.ncl
lattice --config ./demo/config.ncl orchestration start examples/orchestrations/llm-review-loop.ncl --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration status <instance-id>
lattice --config ./demo/config.ncl orchestration replay <instance-id> --output json
```

An orchestration can allow a positive retry limit for exact local terminal failures.

```bash
lattice --config ./demo/config.ncl orchestration retry <instance-id> --actor operator:local --output json
```

Retry is explicit. Lattice rejects it before dispatch when authority, policy, evidence, or budget is insufficient.

A shell error, abandonment, or foreign run does not grant retry authority.

If recovery returns `PendingUnobserved`, close the uncertain dispatch explicitly:

```bash
lattice --config ./demo/config.ncl orchestration recover <instance-id> --output json
lattice --config ./demo/config.ncl orchestration cancel <instance-id> --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration abandon <instance-id> --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration replay <instance-id> --output json
```

Lattice reserves run IDs before effects. Foreign and legacy records fail closed.

[Orchestration identities](docs/orchestration-identities.md) use distinct checked Rust roles. Nickel, CLI, webhook, scheduler, JSON, and redb adapters preserve their existing text shapes.

Typed identities prove local role separation and checked syntax. They do not prove provider truth, remote cancellation, external exactly-once effects, independent history, or production readiness.

Abandonment requires prior authorized cancellation. It does not prove provider cancellation, provider failure, billing truth, remote deduplication, or production readiness.

A local event-chain head proves local sequence consistency only.

## Safe run inspection

Inspect one persisted run:

```bash
lattice --config ./demo/config.ncl runs show 1
lattice --config ./demo/config.ncl runs show 1 --output json
```

The report includes IDs, statuses, counts, bounded budgets, and replay state. It removes names, labels, text, objectives, actors, and trigger keys.

The workflow-name fingerprint aids identity. It is not a confidentiality claim for guessable names.

Missing historical data is `not_recorded`. Contradictory data produces `valid: false` and a nonzero exit.

`runs show` is read-only. If redb needs explicit recovery, run:

```bash
lattice --config ./demo/config.ncl runs repair
lattice --config ./demo/config.ncl runs show 1 --output json
```

`runs repair` opens an existing store in write-mode recovery. It creates no Lattice records.

Success proves only that redb reopened the file.

## Local file capabilities

Production `LocalFs` effects require at least one `allowed_roots` entry.

The runtime opens the most-specific matching root as a directory capability. It resolves the relative target through that handle.

The same core confines image inputs, Markdown prompts, and agent image or artifact handoffs.

Parent traversal and escaping symlinks fail closed. Configured roots and working directories remain trusted authority grants.

This mechanism does not turn an untrusted workflow into a host sandbox.

## Bounded local processes

Shell nodes, command agents, and command credential providers use the pinned `bounded-exec` mechanism.

Nickel config supplies positive limits for streams, deadlines, polling, and teardown. Lattice selects the executable and working directory.

`bounded-exec` drains streams, retains bounded prefixes, handles deadlines, tears down process groups, and reaps children.

Lattice rejects truncated agent or credential output before parsing.

This result does not prove sandboxing, executable authorization, hermeticity, syscall limits, semantic correctness, or release eligibility.

The dependency source is:

- RID: `rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo`
- Revision: `29dac88ecded94457572db3fdfaaaab95fa91525`
- URL: `https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git`

Cargo, Nix, and Crane must use that source. Acquisition fails instead of using GitHub as a fallback.

## Workflow consumer contract

`crates/lattice-workflow-contract` publishes the versioned request and observation boundary for consumers such as Kiln.

The `no_std + alloc` crate owns bounded wire values, deterministic BLAKE3 identities, and pure admission. It imports no Lattice runtime implementation.

The contract preserves consumer effect, attempt, candidate, pipeline, policy, graph, and idempotency identities. Lattice workflow and execution identities remain native observations.

See [`crates/lattice-workflow-contract/README.md`](crates/lattice-workflow-contract/README.md) for the integration flow and explicit non-claims.

## Verification

```bash
verus verus/example_spec.rs
```

## References

- [bounded-exec](https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git) — bounded child-process mechanics at revision `29dac88ecded94457572db3fdfaaaab95fa91525`.
- [nickel-eval](https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git) — product-neutral Nickel evaluation at revision `bf16cf223ed3dfe75a28f02ac21ed7891a60bac1`.
- [content-identity-core](https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git) — product-neutral identity framing at revision `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`; see [source and role policy](docs/content-identity-source.md).
- [qaml-ai/camelAI](https://github.com/qaml-ai/camelAI) — reviewed manifest and evaluation-lane reference at revision `70dd57a6219d9a19b25840aebbf1105c63c3a5f2`. Lattice does not adopt its Cloudflare coupling, raw transcript defaults, or judge-authoritative rollout decisions.
- [Aspen](https://github.com/OnixResearch/aspen) — `scripts/octet-burndown.scm` is a reviewed design reference at BLAKE3 `791fba3ab3c78ededa9406b57a3d90163984541d453f183f632d8326e3aa5322`.
- [mc](https://github.com/OnixResearch/mc) — `scripts/ralph-drain-cairns.scm` is a reviewed design reference from commit `7f36751aa4aadcbb048442c06dfbcf2addec4aaa`, at BLAKE3 `94ff7d3c3c527d272782c1bbd84b79da557fc6392af9dfbbaf7a21cb03f703b9`.
- [Pi coding agent](https://pi.dev) — generic agent SDK and RPC reference.
- [Clankers](https://github.com/brittonr/clankers) — generic coding-agent integration reference.
- [Trellis](../trellis/) — verified graph, lifecycle, replay, effect, and identity primitives.
- [Choregraph](https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git) — runtime source at revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07` and readiness-completeness evidence at revision `b3e08e19750f53bdbcae970cdf58a47a791ed20b`.
- [Basalt](https://github.com/OnixResearch/basalt) — policy core vendored from revision `bb8fb5f4a763f532df96b6e65407163ac668834e` because anonymous source access is unavailable. Nested Cargo manifests are normalized locally; `vendor-cargo-deps/basalt/NORMALIZATION.md` records that boundary.
- [Cap Root](https://seed.radicle.garden/z3cWBCK7VPbEmbL5MAvKJWKjCE745.git) — lexical path admission at revision `545e77e5a49e4e1d34037aa9a82d924944ac0a1b`.
- [execution-graph](https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git) — compatibility and rollback reference.
- [Seaglass](https://github.com/brittonr/seaglass) — original compatibility vocabulary.
- [Callee](https://github.com/baldaworks/callee) — provider-role and bounded loop reference.
- [FarmTable](https://github.com/scion-frontiers/farmtable) — ready-task and critical-path architecture reference.
- [cap-std](https://github.com/bytecodealliance/cap-std) — capability-directory implementation.
- [Animus](https://github.com/OnixResearch/animus) — adapter contracts and bounded agent loop.
- [Swamp](https://github.com/swamp-club/swamp) — AGPL-3.0 durable-operation and manual-gate design reference at revision `a648c1307fd5cffa029cb8348b2ec20cb727bd66`. It is not a dependency.
- [Aspen Octet burn-down guard](../aspen/scripts/octet-burndown.scm) — guarded-round burn-down reference; not a Lattice runtime dependency.
- [mc Ralph Cairn drain loop](../../mc/scripts/ralph-drain-cairns.scm) — bounded drain-loop reference; not a Lattice runtime dependency.
- [universal-weave](https://github.com/transkatgirl/universal-weave) — Loom-style branching tree/DAG primitives. Design reference only for future branching-run exploration in the TUI. Not a dependency: the run DAG stays with Choregraph, and records stay in redb with BLAKE3 identity.

Lattice does not adopt Swamp dynamic TypeScript loading, YAML/CEL authoring, plaintext secrets, or registry metadata as authority.
