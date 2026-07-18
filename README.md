# n8nclone

## Development

```bash
nix develop          # enter devshell
pre-commit install   # install the Octet pre-commit hook
scripts/check.rs     # run fmt, tests, and clippy with repo-local target artifacts
octet-precommit-check # run Octet at deny/error level against octet-baseline.json
cargo octet check    # inspect raw Octet output
cargo check          # quick compile check
nix flake check      # full CI (build + test + clippy + fmt + Octet)
```

The Octet hook runs the lib target profile, keeps every configured Octet lint at `deny`, and accepts the current checked-in `octet-baseline.json` debt. New unaccepted Octet finding IDs fail the hook.

## Examples

Complex templates and runnable Nickel workflow examples live under `examples/`, including daily digest, CSV cleanup, log triage, webhook trigger cookbook, and the Markdown-backed [`llm-operating-graph`](examples/workflows/llm-operating-graph.md). The LLM example models roles and sequences inside an admitted DAG; feedback and reorganization become explicit inputs to a later acyclic run rather than hidden runtime cycles.

```bash
n8nclone --config ./demo/config.ncl --init
n8nclone --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
n8nclone --config ./demo/config.ncl run daily-digest --output json
```

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
