# Lattice

## Development

```bash
nix develop          # enter devshell
pre-commit install   # install the Octet pre-commit hook
scripts/check.rs     # run fmt, tests, and clippy with repo-local target artifacts
octet-precommit-check # run Octet at deny/error level against octet-baseline.json
cargo octet check    # inspect raw Octet output
cargo check          # quick compile check
nix flake check -L --option secret-key-files '' # full CI; check outputs do not require a host signing secret
```

The Octet hook runs the lib target profile, keeps every configured Octet lint at `deny`, and accepts the current checked-in `octet-baseline.json` debt. New unaccepted Octet finding IDs fail the hook.

## Examples

Complex templates and runnable Nickel workflow examples live under `examples/`, including daily digest, CSV cleanup, log triage, webhook trigger cookbook, and the Markdown-backed [`llm-operating-graph`](examples/workflows/llm-operating-graph.md). The LLM example models roles and sequences inside an admitted DAG; feedback and reorganization become explicit inputs to a later acyclic run rather than hidden runtime cycles.

```bash
lattice --config ./demo/config.ncl --init
lattice --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
lattice --config ./demo/config.ncl run daily-digest --output json
```

## Native OpenAI-compatible agents

[`examples/config/openai-compatible-agent.ncl`](examples/config/openai-compatible-agent.ncl) routes the existing adapter-neutral Pi and Clankers target labels directly to a bounded chat-completions endpoint. Set `agent_runtime.runtimes = {}` when using those native routes so transport selection remains unambiguous, then replace the example endpoint and model.

The native adapter supports strict text and JSON outputs, disables redirects, bounds request/response bytes and time, and never falls back from missing content to hidden reasoning. Local HTTP endpoints are supported only without bearer credentials. For an authenticated HTTPS endpoint, set `api_key_env` to a binding name and provide that binding through each agent node's `secret_env` credential reference; do not put the key in Nickel.

```bash
lattice --config examples/config/openai-compatible-agent.ncl import examples/workflows/llm-operating-graph.ncl
lattice --config examples/config/openai-compatible-agent.ncl run llm-operating-graph --output json
```

## Bounded Animus leaf agents

[`examples/config/animus-agent.ncl`](examples/config/animus-agent.ncl) routes the additive `animus` target through a fixed BLAKE3-identified two-turn Steel loop. Animus decides only the inner model/response sequence for one generic agent node. Lattice retains provider credentials, transport limits, workflow and Seaglass graph semantics, effects, budgets, events, persistence, triggers, approvals, and durable orchestration.

The first slice performs exactly one OpenAI-compatible model dispatch and supports text or JSON output. It exposes no tool definitions, tool capabilities, arbitrary Steel programs, profile loading, sessions, provider fallback, Animus DAGs, or Animus persistence. Returned metadata contains bounded identities and counts, not prompts, responses, credentials, or a second provenance ledger.

```bash
cargo run --example generate_animus_workflow
lattice --config examples/config/animus-agent.ncl import examples/workflows/animus-bounded-agent.ncl
lattice --config examples/config/animus-agent.ncl run animus-bounded-agent --output json
```

The checked-in Animus snapshot is reproducibly refreshed from its reviewed revision with `scripts/sync_animus.rs ../animus vendor-cargo-deps/animus`.

## Durable agent orchestration

Lattice can coordinate immutable workflow revisions across explicit rounds with operator authority, budget reservations, crash recovery, and side-effect-free replay. Each round remains an acyclic workflow admitted through the Seaglass-backed runner. Agent output may propose a successor revision, but it cannot mutate graphs, credentials, triggers, runtime abilities, or budgets without an authorized hash-bound approval.

The checked-in [`llm-review-loop`](examples/orchestrations/llm-review-loop.ncl) definition wraps the Markdown-backed LLM operating graph with bounded rounds, agent invocations, tokens, duration, schedule/webhook trigger identities, and `operator:local` approval authority.

```bash
cargo run --example generate_llm_orchestration
lattice --config ./demo/config.ncl orchestration validate examples/orchestrations/llm-review-loop.ncl
lattice --config ./demo/config.ncl orchestration start examples/orchestrations/llm-review-loop.ncl --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration status <instance-id>
lattice --config ./demo/config.ncl orchestration replay <instance-id> --output json
```

If the original executor has stopped and recovery reports `PendingUnobserved`, close the uncertain local dispatch explicitly rather than retrying it:

```bash
lattice --config ./demo/config.ncl orchestration recover <instance-id> --output json
lattice --config ./demo/config.ncl orchestration cancel <instance-id> --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration abandon <instance-id> --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration replay <instance-id> --output json
```

Abandonment requires a prior authorized cancellation request and refuses to override a persisted run. An acknowledged dispatch consumes its complete reserved upper bound, clears local pending state, and terminates as failed with an explicit unknown-remote-outcome summary. It does not retry the effect or prove provider cancellation, provider failure, workflow success, billing truth, or remote exactly-once execution.

A local event-chain head proves local sequence consistency only. It does not prove external anchoring, independently witnessed completeness, model truth, provider billing truth, or remote exactly-once effects.

## Local file capabilities

Production `LocalFs` read, write, and append effects require at least one explicit `allowed_roots` entry. The runtime selects the most-specific matching root, opens it as a directory capability, and resolves only the relative target through that handle. The same capability core confines production image inputs, Markdown prompt hydration, and generic-agent image/artifact handoffs while preserving their byte, UTF-8, and media checks. Parent traversal and escaping symlinks fail closed, including if an intermediate path is replaced after the root is opened. Configured roots, workflow directories, and agent working directories are trusted authority grants; these readers do not turn untrusted workflow definitions into a host sandbox.

## Verification

```bash
verus verus/example_spec.rs
```

## References

- [Pi coding agent](https://pi.dev): SDK, print, RPC, and image prompt surfaces for generic agent-node integration.
- [Clankers](https://github.com/brittonr/clankers): Rust terminal coding agent with one-shot, daemon, MCP/ACP, context image references, and image-generation surfaces for generic agent-node integration.
- [Trellis](../trellis/): verified-logic primitives for graph admission, job lifecycle, durable replay, effect/capability admission, and typed durable references used as design inputs for the generic pipeline runner.
- [Seaglass](https://github.com/brittonr/seaglass): owner of the vendored dependency-light execution-graph core used for deterministic admission, readiness, lifecycle, critical-path, and bottleneck decisions.
- [Callee](https://github.com/baldaworks/callee): Markdown-defined provider-backed roles and bounded `Sequential`/`Loop` composition used as an authoring and orchestration reference.
- [FarmTable](https://github.com/scion-frontiers/farmtable): graph-native ready-task, dependency, critical-path, bottleneck, and lifecycle separation used as an execution-graph architecture reference.
- [cap-std](https://github.com/bytecodealliance/cap-std): portable capability-directory implementation used to keep local file effects descriptor-relative and reject traversal or symlink escapes.
- [Animus](https://github.com/OnixResearch/animus): authority-free adapter contracts, deterministic bounded agent loop, and Rust-revalidated Steel fabric used beneath Lattice's generic agent boundary.
