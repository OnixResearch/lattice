# Examples

These examples are intentionally more complex than the default starter catalog.
They are useful for demos, import/export validation, and headless regression tests.

## Templates

`examples/templates/` contains advanced Nickel node templates:

- `complex-sort-lines.ncl` — sort piped text with optional sort flags.
- `complex-unique-lines.ncl` — collapse adjacent duplicate lines with optional `uniq` flags.
- `complex-cut-field.ncl` — extract a delimited field with bounded numeric field selection.

## Workflows

`examples/workflows/` contains runnable, workflow-shaped examples:

- `daily-digest.ncl` — filters event records and renders a Markdown digest.
- `csv-cleanup-report.ncl` — filters active CSV records, sorts/deduplicates them, and renders a team report.
- `log-triage.ncl` — filters warning/error log lines and renders an incident summary.
- `complex-text-normalization.ncl` — sorts, deduplicates, filters, replaces, and counts text lines.
- `selfie-to-fine-art.ncl` — routes a selfie plus inspiration images through a generic Clankers agent node.
- `webhook-cookbook.ncl` — imports a credential-reference authenticated webhook trigger for local listener demos.
- `animus-bounded-agent.ncl` — runs one generic agent node through the fixed two-turn, model-only Animus leaf runtime.
- `bounded-build-loop.ncl` — the fixed planner/generator/evaluator DAG used by the bounded local build-and-test host below.

## Construction programs

`examples/construction/` contains captured graph-construction programs. They
import `./choregraph/graph-construction-v1.ncl`, which the native construction
path supplies from the exact selected Choregraph library bytes, so they are not
standalone files for `nickel typecheck` or plain `import`:

- `local-shell-construction.ncl` — one shell `worker` template instantiated twice by name and twice by finite generation, fanned out from a fork and followed by a control-sequenced join.
- `parameterized-shell-construction.ncl` — explicit parameters, independently scoped instances and finite generation.

Compile them through `lattice import --construction` or `lattice authoring draft
... --construction`; the construction authoring tests compile both through that
native path. Construction source is inert until a separate exact effect grant is
selected.

## Orchestrations

`examples/orchestrations/llm-review-loop.ncl` adds a durable multi-round policy to the Markdown-backed LLM operating graph.
`examples/orchestrations/bounded-build-loop.ncl` binds the build loop to two total rounds, four agent invocations, a four-minute total reservation, one observed-failure retry and completion on a successful real evaluator.

The policy defines an immutable workflow identity, operator approval, resource bounds, trigger identities, and replay-safe event history.

The separate `llm-review-loop` uses `Researcher`, `Builder`, `Reviewer`, and `Publisher`. It waits for operator proposals rather than automatically treating model judgement as a successful build.

Regenerate it after changing the referenced workflow topology or portable configuration:

```bash
cargo run --example generate_llm_orchestration
cargo run --example generate_bounded_build_loop
```

Validate and start it after importing `llm-operating-graph.ncl` into the selected store:

```bash
lattice --config ./demo/config.ncl orchestration validate examples/orchestrations/llm-review-loop.ncl
lattice --config ./demo/config.ncl orchestration start examples/orchestrations/llm-review-loop.ncl --actor operator:local
```

Import and run one with a temporary config/store:

```bash
lattice --config ./demo/config.ncl --init
lattice --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
lattice --config ./demo/config.ncl run daily-digest --output json
```

## Bounded planner/generator/evaluator build loop

Run this from the repository root in `nix develop` (Rust and `sh` are required).
Use a **new** directory; preparation refuses to overwrite an existing snapshot:

```sh
cargo run --locked --example run_bounded_build_loop -- prepare /tmp/lattice-loop-demo repair
# Review the three scripts, workflow.ncl, frozen.json and authority.json in that directory.
cargo run --locked --example run_bounded_build_loop -- run /tmp/lattice-loop-demo --approve-local-effects
```

`repair` selects a finite local rule producer: its first candidate fails an actual
Rust unit test; the next uses the retained failure verdict to correct the greeting.
It is not a fake success adapter, model-quality experiment, diffusion repair arm,
or general coding agent. Select `pass` for first-round success, `fail` for repeated
real test failure and exhaustion, or `cancel` for a deliberately slow evaluator
that can be interrupted with Ctrl-C. Each preparation needs a fresh directory.

The planner and generator execute through production command-backed Agent nodes
with `tool_policy = "none"`; the evaluator is a shell role, not a third model. It
runs `rustc --test`, the compiled tests, a normal build, and the built program.
Only an observed owned terminal evaluator failure can authorize the one retry.
Missing verdicts, denied authority and unobserved post-start errors do not enable
automatic retry. A second invocation refuses existing execution state: inspect
the retained store instead of guessing that an interrupted effect failed.

### Construction, execution, and runtime repetition

- **Construction:** the generator emits one finite, acyclic typed Nickel graph.
  Nickel evaluation and constructor expansion do not repeat runtime work.
- **Execution:** each round uses the production immutable snapshot executor,
  ordinary registry/port/resource admission and the exact operator authority
  profile. The example selects a fixed saved workflow, **not** constructor
  inlining or a `SubworkflowNode` call. The dedicated negative fixture also
  exercises the existing saved-child route without child authority.
- **Repetition:** the host commits the existing orchestration `StartRound`,
  `RunFinishedObserved`, `RetryRound`, cancellation and exhaustion transitions to the
  existing hash-linked durable ledger. It has no second retry counter or budget
  engine. A fresh run/dispatch identity never creates a fresh total budget.

Every round freezes `input.json` and retains exactly named handoffs `spec.txt`,
`build.rs`, and `verdict.txt` under `round-N/`, together with build diagnostics,
the actual generated source, and a receipt binding the revision, frozen recipe,
prior verdict and handoff content identities. A cancelled or authority-denied
round records only artifacts actually produced; missing role output is not
invented. `report.json` records pass, fail, exhausted or cancelled plus the
conservative total charges. Duration charges are reservations, not measured CPU
or billing. A monotonic total deadline and per-round watchdog forward cancellation
to the owned executor and await its observation before permitting any next round.
Settlement atomically uses actual owned termination, not the request flag:
completed success or failure wins a late cancellation request; an actual
cancellation consumes its reservation once and stops repetition. A late
cancellation request also prevents retrying an observed failure.

Parameter/source edits cannot replace the in-flight snapshot. Before a later
round, graph admission and exact revision matching run again; an invalid graph,
undeclared runtime, or stale/regenerated revision dispatches no rejected work.
This fixed profile deliberately does not accept generated successor topology.
The unchanged original workflow remains stored and its earlier results remain
bound to that revision. Changing topology requires a separately authorized,
fully admitted immutable successor, not resetting terminal nodes or adding cycles.

### Authority and non-claims

The explicit `--approve-local-effects` flag selects the reviewed, finite-lived
local operator profile. Exact per-node Agent/Process grants are checked before
effects. Planner, generator and evaluator identities are not orchestration
approvers; generated text, tool-policy fields, imports and child references cannot
grant a new effect or approve a revision. Tests remove a role grant, attempt an
extra effectful node and exercise a child with no child authority.

The scripts, profile directory and local host are trusted. Lattice authority is
not an OS filesystem/process sandbox: the commands run as the current user, and
`tool_policy = "none"` is not proof of process isolation. Do not substitute
untrusted scripts or source. Receipts do not prove remote cancellation,
independently witnessed history, general coding capability or model quality.
The optional masked/diffusion repair arm remains **disabled**: no independently
qualified inference-lab producer or base/mask-preserving adapter is selected.

Focused checks (not run merely by preparing this example):

```sh
cargo run --locked --example generate_bounded_build_loop
cargo test --locked --test test_bounded_build_loop
```

## Bounded Animus agent

Regenerate, import, and run the model-only Animus example against an explicitly configured OpenAI-compatible endpoint:

```bash
cargo run --example generate_animus_workflow
lattice --config examples/config/animus-agent.ncl import examples/workflows/animus-bounded-agent.ncl
lattice --config examples/config/animus-agent.ncl run animus-bounded-agent --output json
```

This example is one Lattice agent node, not an Animus workflow orchestrator. Tools, sessions, arbitrary Steel, provider fallback, and Animus persistence remain disabled.

## Webhook cookbook

`webhook-cookbook.ncl` demonstrates the webhook listener path. It does not store plaintext secret material in the workflow.

The workflow references `env:LATTICE_WEBHOOK_TOKEN`. Set this variable from your secret manager before a dry run or listener startup.

```bash
lattice --config ./demo/config.ncl --init
lattice --config ./demo/config.ncl import examples/workflows/webhook-cookbook.ncl
LATTICE_WEBHOOK_TOKEN="$WEBHOOK_TOKEN" lattice --config ./demo/config.ncl webhook-listen --dry-run --bind 127.0.0.1:8787
LATTICE_WEBHOOK_TOKEN="$WEBHOOK_TOKEN" lattice --config ./demo/config.ncl webhook-listen --bind 127.0.0.1:8787
```

From another terminal, send an accepted request with the same environment-backed token:

```bash
curl -sS -X POST http://127.0.0.1:8787/hooks/cookbook-lead \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $WEBHOOK_TOKEN" \
  -H 'Idempotency-Key: cookbook-event-1' \
  --data '{"name":"Ada"}'
```

Rejected requests are deterministic and do not start a workflow run. For example, this missing path returns a safe not-found response:

```bash
curl -sS -X POST http://127.0.0.1:8787/hooks/missing \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $WEBHOOK_TOKEN" \
  -H 'Idempotency-Key: cookbook-event-2' \
  --data '{"name":"Ada"}'
```

Inspect trigger state and safe history after requests:

```bash
lattice --config ./demo/config.ncl triggers list
lattice --config ./demo/config.ncl triggers history
```
