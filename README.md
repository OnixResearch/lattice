# Lattice

Lattice is a graph-native workflow and agent runtime written in Rust. It combines
Nickel workflow definitions, a command-line interface and terminal UI, persistent
execution state, and provider adapters.

Each workflow is a directed acyclic graph. Multi-round agent orchestration links
immutable workflow revisions through explicit budgets and approvals; an agent's
output is not permission to change the graph or expand its authority.

**Source:** [Radicle](https://seed.radicle.garden/z2yGPFsQZsUcFNkiyR4Dkj22BuxnC.git)
· [GitHub mirror](https://github.com/OnixResearch/lattice)

## Current status

The standalone CLI and TUI accept an explicit local-operator Nickel profile via
the `effect_authority` configuration field. Without one, shell, HTTP, and file
effects still fail closed before adapter dispatch. Embedders can instead supply
their own `RunContextProvider`.
See [Choregraph effect authority](docs/choregraph-effect-authority.md) for exact
workflow inspection, profile authoring, expiry, and file-root configuration.

The quickstart begins with effect-free import and export. Executing its shell
pipeline requires a reviewed operator profile; authorization does not configure
an HTTP transport, and the stock executor still uses `NoopHttpClient`.

## Implemented components

- **Workflow execution:** import and export typed Nickel workflows, execute
  dependency graphs, and bound parallel work with resource limits.
- **Terminal operation:** edit and run workflows in the TUI, or use headless
  commands with machine-readable run summaries.
- **Agents:** use command-backed Pi and Clankers adapters, OpenAI-compatible
  endpoints, or a bounded Animus leaf-agent profile.
- **Durable control:** persist runs, scheduled occurrences, webhook claims,
  approvals, and multi-round orchestration state, with explicit recovery actions.
- **Reusable operations:** inspect, plan, and run typed operation contracts;
  preserve versioned output data; and admit content-addressed operation packs.
- **Consumer integration:** expose a bounded, identity-bound workflow contract
  and Unix-socket exchange without giving consumers Lattice's runtime internals.

## Getting started

### Build environment

Use the repository's Nix development shell. It supplies the pinned Rust and
Nickel toolchain and the native build dependencies.

```sh
git clone https://github.com/OnixResearch/lattice.git
cd lattice
nix develop
cargo run --locked -- --help
```

The current flake fetches its pinned Cairn source over GitHub SSH. GitHub SSH
access is therefore a build prerequisite even though Lattice's own source is
public. Dependency acquisition also needs network access or an existing local
cache. Exact sources live in [Cargo.toml](Cargo.toml), [flake.nix](flake.nix), and
their lockfiles; do not substitute sibling checkouts or floating revisions.

### Import and export a workflow

The [daily digest](examples/workflows/daily-digest.ncl) example defines a
three-step shell pipeline over embedded sample records. Importing and exporting
it does not execute those commands or require a model, API key, or external
service.

From the repository root, inside the development shell:

```sh
demo="$(mktemp -d)"
cat > "$demo/config.ncl" <<EOF
{
  data_dir = "$demo/state",
  default_timeout_secs = 60,
  shell = "sh",
}
EOF

cargo run --locked -- --config "$demo/config.ncl" import examples/workflows/daily-digest.ncl
cargo run --locked -- --config "$demo/config.ncl" export daily-digest "$demo/daily-digest.ncl"
```

`import` validates the workflow and saves it in the selected store. `export`
writes the saved workflow back to a typed Nickel file. Neither command runs the
workflow.

The configuration uses an absolute `data_dir` so this demo's workflows and run
state do not use your normal store. `--config` selects a configuration file; it
does not relocate state by itself. Keep `$demo` to inspect the run later.

### Execution and run inspection

`run` takes a saved workflow's name, not a file path:

```sh
cargo run --locked -- --config "$demo/config.ncl" run daily-digest --output json
```

Without `effect_authority`, this records a failed run and exits with status 1;
the stored node error is `effect dispatch failed: AuthorityUnavailable`. It does
not run the shell pipeline. Inspect the saved workflow before granting effects:

```sh
cargo run --locked -- --config "$demo/config.ncl" authority inspect daily-digest
```

Inspection prints exact workflow, machine, and policy identities and required
grant tuples. It does not grant authority, change the store, or execute nodes.
Follow the [operator-profile steps](docs/choregraph-effect-authority.md#standalone-operator-profile)
to review those requirements, select a finite-lived profile, and rerun the same
command. The same configured provider is used by TUI, scheduler, orchestration,
and approval-resume hosts.

The JSON run summary reports status, completed steps, failures, and cache hits,
not rendered node output.

Use the `run_id` returned by the run summary in place of `RUN_ID`:

```sh
cargo run --locked -- --config "$demo/config.ncl" runs show RUN_ID --output json
```

`runs show` is a read-only, content-omitting report. It exposes execution and
replay metadata without dumping arbitrary workflow text or node output.

Omit the subcommand to open the TUI in an interactive terminal:

```sh
cargo run --locked -- --config "$demo/config.ncl"
```

For a separate, unused configuration path, `--init` creates the default config,
node templates, and a local `shell-preview` workflow-template bundle. It refuses
to overwrite existing target files. Initialization does not import or run a
workflow. See [workflow-template bundles](docs/workflow-template-bundles.md).

## Workflows and agents

Start with [the workflow examples](examples/README.md) for local shell pipelines,
CSV cleanup, log triage, webhooks, and agent workflows. Inspect each example's
commands, file paths, credentials, and runtime requirements before running it.

Agent setup is explicit:

- [OpenAI-compatible configuration](examples/config/openai-compatible-agent.ncl)
  defines the endpoint, model, output mode, and request limits. The example uses
  a local service; it is not a hosted endpoint supplied by Lattice.
- [Animus configuration](examples/config/animus-agent.ncl) selects a bounded
  leaf-agent loop. Lattice retains workflow effects, credentials, persistence,
  and orchestration authority.
- [Guarded workflow templates](docs/guarded-agent-templates.md) define bounded
  agent rounds with objective checks, progress rules, and operator-controlled
  continuation.
- [Agent evaluation](docs/agent-evaluation.md) provides identity-bound plans,
  bounded drivers, and metadata-only receipts. Live evaluation requires explicit
  target and credential authority.

Do not put plaintext credentials in Nickel. Use credential references and the
selected provider's delivery mechanism. A configured model or agent runtime is
required only for workflows that use it; the local quickstart does not.

## Architecture

Lattice owns orchestration and host effects. Choregraph supplies graph admission,
readiness, machine transitions, and bounded replay decisions. Provider adapters
remain outside the pure decision layers.

| Component | Responsibility |
| --- | --- |
| [`lattice`](src/) | CLI, TUI, execution adapters, persistence, retries, triggers, and host effects |
| [`lattice-core`](crates/lattice-core/) | Pure workflow values, expression mapping, control-flow decisions, and schedule identity |
| [`budget-ledger-core`](crates/budget-ledger-core/) | Pure orchestration budget reservation and settlement |
| [`lattice-workflow-contract`](crates/lattice-workflow-contract/) | Bounded consumer requests, observations, and deterministic identities |
| [`lattice-workflow-transport`](crates/lattice-workflow-contract/transport/) | Bounded workflow exchange over Unix sockets |
| [Choregraph](https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git) | Graph and machine semantics consumed by Lattice |

The pure core does not open files, spawn processes, access the network or clock,
or own terminal state. See [workflow domain boundaries](docs/workflow-domain-boundaries.md)
for the ownership contract and [vendored source notes](vendor-cargo-deps/README.md)
for import and maintenance procedures.

## Safety and recovery

- **Workflows can cause real effects.** Shell commands and agents run with the
  authority of their configured host environment. Bounded execution and local
  file capabilities are not a sandbox for untrusted workflows.
- **File access is explicit.** Production local-file effects require configured
  `allowed_roots`. Those roots and working directories remain trusted grants.
- **Uncertainty is durable.** A crash or lost observation is not proof of failure
  and does not justify an automatic retry. Use the relevant recovery command;
  explicit retries can duplicate an unobserved effect.
- **Approvals are not execution.** Recording a decision and resuming an exact
  suspended workflow are separate, identity-bound actions.
- **Evidence is scoped.** Local checks and replay facts do not establish external
  exactly-once effects, remote cancellation, arbitrary workflow correctness, or
  production readiness.

| Topic | Documentation |
| --- | --- |
| Scheduling and crash boundaries | [Storage commit uncertainty](docs/storage-commit-uncertainty.md) |
| Approvals and resume | [Durable workflow approvals](docs/workflow-approvals.md) |
| Multi-round identity and authority | [Orchestration identities](docs/orchestration-identities.md) |
| Concurrent execution | [Bounded ready-frontier dispatch](docs/bounded-ready-frontier.md) |
| Node result reuse | [Bounded DAG node memoization](docs/dag-node-memoization.md) |
| Reusable methods and outputs | [Operation contracts](docs/operation-contracts.md), [operation data](docs/operation-data.md), [operation packs](docs/operation-packs.md) |
| Consumer integration | [Workflow exchange application](docs/workflow-exchange-application.md), [consumer contract](crates/lattice-workflow-contract/README.md) |
| Projected local protocols | [Role-protocol execution](docs/role-protocol-execution.md) |

## Development

Run checks from the repository root, using the pinned development shell:

```sh
nix develop
scripts/check.rs
octet-precommit-check
nix flake check -L --option secret-key-files ''
```

`scripts/check.rs` runs operator end-to-end checks, boundary guards, formatting,
focused checks, tests, Clippy, and specification validation. The Octet gate
rejects new finding IDs outside the reviewed baseline. The Nix command disables
host signing keys only for that check invocation; it does not change normal
package or release signing policy.

Accepted requirements and active changes live in [`.cairn/`](.cairn/). Keep
implementation, examples, and their evidence aligned; generated inventories and
model output are not completion authority. Imported snapshots must be refreshed
through the repository-owned sync scripts, not edited as ordinary local source.

## License

The main `lattice` package declares **AGPL-3.0-or-later** in
[Cargo.toml](Cargo.toml). Workspace crates and imported dependencies retain their
own declared licenses.
