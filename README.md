# Lattice

## Development

```bash
nix develop          # enter devshell
pre-commit install   # install the Octet pre-commit hook
scripts/check.rs        # run operator E2E, fmt, tests, and clippy with repo-local target artifacts
scripts/operator-e2e.rs # exercise TUI PTY, webhook TCP, scheduler process, and orchestration recovery boundaries
octet-precommit-check  # run the fail-closed Octet stable-ID gate
cargo octet check    # inspect raw warning-level Octet output
cargo check          # quick compile check
nix flake check -L --option secret-key-files '' # full CI; check outputs do not require a host signing secret
```

<!-- r[verify lattice.repository_validation.baselined_octet_gate.scenario.boundary] -->
The Octet hook runs the lib target profile with every registered lint explicitly observed at `warn`, no disabled or allowed lint entries, and the checked-in `octet-baseline.json` as the admission set. The hook requires a successful Cargo child, complete agreement with all 892 reviewed stable identities, and zero new or stale identities. An isolated Nix fixture injects a new `todo!()` finding and proves that the same hook rejects it. Direct `cargo octet check` output is review evidence only; it does not grant repository-check or release admission.

## Choregraph machine source

Lattice selects `choregraph-core` and the conversion-only `choregraph-seaglass` adapter from `rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF` at revision `fc47c5f4ecbb7b4341af690fa42199f25d57f54c`. Cargo, Nix, and Crane use the same exact source. The former vendored execution-graph implementation is removed.

Choregraph owns graph admission, ordering, readiness, lifecycle, resume, analytics, and bounded replay decisions. Lattice retains Nickel evaluation, product decoding, runtime selection, effects, persistence, retries, triggers, diagnostics, UI, and evidence.

Direct `choregraph-nickel` integration remains blocked because the immutable candidate pins `blake3` `1.8.2`, while the selected Animus packages require `1.8.5`. Lattice does not weaken or replace either reviewed source.

## Durable scheduler occurrences

Before workflow effects, the scheduler now commits a domain-separated BLAKE3 identity for the exact `(schedule_id, due_epoch_ms)` occurrence. One redb transaction verifies the due slot, reserves the globally unique run ID, records the workflow revision and pending attempt, and advances the schedule. Ordinary daemon restart therefore cannot automatically admit that occurrence again.

Inspect and close occurrence uncertainty explicitly:

```bash
lattice --config ./demo/config.ncl scheduler-occurrences list
lattice --config ./demo/config.ncl scheduler-occurrences show <occurrence-b3>
lattice --config ./demo/config.ncl scheduler-occurrences recover <occurrence-b3>
lattice --config ./demo/config.ncl scheduler-occurrences retry <occurrence-b3> --acknowledge-duplicate-risk
lattice --config ./demo/config.ncl scheduler-occurrences abandon <occurrence-b3> --acknowledge-unknown-outcome
```

Recovery settles only an exact terminal run owned by the active occurrence attempt. Missing run evidence remains `PendingUnobserved`. One explicit retry is available only while the saved workflow revision still matches; it reserves a fresh owned run and can duplicate an effect whose original outcome was not persisted. Abandonment records `AbandonedUnknown` without executing again or inventing success, failure, or cancellation.

## Operator crash-boundary evidence

`scripts/operator-e2e.rs` exercises selected post-effect/pre-persistence death windows through real scheduler and webhook processes on supported Unix/Linux hosts. Each fixture appends an identified local effect attempt before killing its Lattice parent, inspects a copied crash store without first repairing the original, and then restarts or redelivers against that original state.

The current scheduler profile records one pending occurrence with an advanced schedule, one local effect attempt, no terminal run, no automatic restart attempt, and explicit unknown-outcome abandonment. The archived scheduler baseline retains the former at-least-once duplicate profile. The archived initial webhook profile likewise retains its former duplicate-risk boundary. The current webhook profile commits a BLAKE3 trigger-scoped claim, reserved run identity, and safe started-history fact before effects; post-effect listener death leaves that claim locally accepted, and redelivery receives a conflict without a second local execution.

Scheduler and webhook claims establish local at-most-one automatic admission only under the single active redb store owner. A pending occurrence or started webhook claim without terminal run evidence is intentionally uncertain and may suppress work even when no external effect is observable. These checks do not prove remote commit identity, provider deduplication, distributed multi-writer safety, response delivery, external cancellation, arbitrary kill safety, equivalent behavior on unsupported hosts, exactly-once external execution, or production readiness.

## Operator TUI recovery evidence

On supported Unix/Linux hosts, the continuous portable-PTY matrix drives the real TUI through successful execution, deterministic failure inspection, an explicit fresh rerun, fatal validation rejection without effects, active cancellation after child readiness, constrained/restored terminal resize, history inspection, keyboard exit, and non-terminal rejection. Durable assertions preserve independent failed and rerun records, keep invalid workflows from creating runs, and require bounded child cancellation to prevent its pending local effect.

The PTY checks assert semantic surfaces rather than byte-identical ANSI frames. They do not prove rendering equivalence across terminal emulators, unsupported-platform behavior, arbitrary child or whole-host cleanup, workflow semantic correctness, external cancellation, or production readiness.

## Traceability debt inventory

The maintained inventory partitions Cairn's exact missing-requirement set without turning inventory rows into Tracey evidence. Generate the failing coverage receipt first, then regenerate or check the BLAKE3-bound report:

```bash
nix run path:../cairn#cairn -- traceability coverage --root . \
  --policy cairn-policy/generated/cairn-policy.json \
  > target/traceability-coverage.json # expected to exit nonzero while debt remains
./scripts/traceability-inventory.rs generate target/traceability-coverage.json
./scripts/traceability-inventory.rs check target/traceability-coverage.json
```

The typed review policy lives at `cairn-policy/traceability-inventory.ncl`; the generated report lives at `cairn-policy/generated/traceability-debt-inventory.json`. The current repository-policy receipt contains exactly 12 missing Radicle-cutover identifiers and no dangling markers. Reviewed candidates bind safe relative evidence paths and current file-byte BLAKE3 hashes. Rows without an explicit decision remain `unassessed`, which does not claim that behavior or evidence is absent, and inventory refresh does not satisfy or promote those requirements.

## Examples

Complex templates and runnable Nickel workflow examples live under `examples/`, including daily digest, CSV cleanup, log triage, webhook trigger cookbook, and the Markdown-backed [`llm-operating-graph`](examples/workflows/llm-operating-graph.md). The LLM example models roles and sequences inside an admitted DAG; feedback and reorganization become explicit inputs to a later acyclic run rather than hidden runtime cycles.

```bash
lattice --config ./demo/config.ncl --init
lattice --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
lattice --config ./demo/config.ncl run daily-digest --output json
```

`--init` is create-only: it exits unsuccessfully if the requested config path already exists and does not rewrite that config or its templates. Edit an existing Nickel config directly, or choose a fresh path for another initialized environment.

## Native OpenAI-compatible agents

[`examples/config/openai-compatible-agent.ncl`](examples/config/openai-compatible-agent.ncl) routes the existing adapter-neutral Pi and Clankers target labels directly to a bounded chat-completions endpoint. Its checked defaults target the local `VibeThinker-3B` service on port 13305 with a reasoning-capable token and time budget. Set `agent_runtime.runtimes = {}` when using those native routes so transport selection remains unambiguous; replace the endpoint, model, and bounds together for another provider.

The native adapter supports strict text and JSON outputs, disables redirects, bounds request/response bytes and time, and never falls back from missing content to hidden reasoning. If a compatible endpoint embeds one exact leading `<think>...</think>` block in content, Lattice discards that block before output typing; unterminated, nested, repeated, or non-leading reasoning markers fail closed. A provider `finish_reason` of `length` is rejected with fixed guidance to increase `max_tokens`, so partial reasoning or final content cannot become typed output. Local HTTP endpoints are supported only without bearer credentials. For an authenticated HTTPS endpoint, set `api_key_env` to a binding name and provide that binding through each agent node's `secret_env` credential reference; do not put the key in Nickel.

```bash
lattice --config examples/config/openai-compatible-agent.ncl import examples/workflows/llm-operating-graph.ncl
lattice --config examples/config/openai-compatible-agent.ncl run llm-operating-graph --output json
```

## Bounded Animus leaf agents

[`examples/config/animus-agent.ncl`](examples/config/animus-agent.ncl) routes the additive `animus` target through a fixed BLAKE3-identified two-turn Steel loop. Animus decides only the inner model/response sequence for one generic agent node. Lattice retains provider credentials, transport limits, Choregraph adapter boundaries, effects, budgets, events, persistence, triggers, approvals, and durable orchestration.

The first slice performs exactly one OpenAI-compatible model dispatch and supports text or JSON output. It exposes no tool definitions, tool capabilities, arbitrary Steel programs, profile loading, sessions, provider fallback, Animus DAGs, or Animus persistence. Returned metadata contains bounded identities and counts, not prompts, responses, credentials, or a second provenance ledger.

```bash
cargo run --example generate_animus_workflow
lattice --config examples/config/animus-agent.ncl import examples/workflows/animus-bounded-agent.ncl
lattice --config examples/config/animus-agent.ncl run animus-bounded-agent --output json
```

The checked-in Animus snapshot is reproducibly refreshed from its reviewed revision with `scripts/sync_animus.rs ../animus vendor-cargo-deps/animus`.

## Durable agent orchestration

Lattice can coordinate immutable workflow revisions across explicit rounds with operator authority, budget reservations, crash recovery, and side-effect-free replay. Each round remains an acyclic workflow admitted through the Choregraph-backed runner. Agent output may propose a successor revision, but it cannot mutate graphs, credentials, triggers, runtime abilities, or budgets without an authorized hash-bound approval.

The checked-in [`llm-review-loop`](examples/orchestrations/llm-review-loop.ncl) definition wraps the Markdown-backed LLM operating graph with bounded rounds, agent invocations, tokens, duration, schedule/webhook trigger identities, and `operator:local` approval authority.

```bash
cargo run --example generate_llm_orchestration
lattice --config ./demo/config.ncl orchestration validate examples/orchestrations/llm-review-loop.ncl
lattice --config ./demo/config.ncl orchestration start examples/orchestrations/llm-review-loop.ncl --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration status <instance-id>
lattice --config ./demo/config.ncl orchestration replay <instance-id> --output json
```

A definition may opt into a positive bound for retries of exact locally observed terminal failures. Each authorized retry starts one fresh round with a new run ID, a new BLAKE3 dispatch identity, and a fresh multidimensional budget reservation:

```bash
lattice --config ./demo/config.ncl orchestration retry <instance-id> --actor operator:local --output json
```

Retry is explicit rather than automatic. It is refused before dispatch when policy is disabled or exhausted, authority is missing, fresh budget does not fit, or the failed instance lacks the exact owned terminal `RunRecord`. A shell error without that record, abandonment, and foreign or legacy-unscoped records never create retry authority.

If the original executor has stopped and recovery reports `PendingUnobserved`, close the uncertain local dispatch explicitly rather than retrying it:

```bash
lattice --config ./demo/config.ncl orchestration recover <instance-id> --output json
lattice --config ./demo/config.ncl orchestration cancel <instance-id> --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration abandon <instance-id> --actor operator:local --output json
lattice --config ./demo/config.ncl orchestration replay <instance-id> --output json
```

Run IDs are reserved atomically before effects, and orchestration records are bound to the exact instance and dispatch; foreign or legacy-unscoped records fail closed instead of satisfying recovery. Terminal headless runs persist the same node-detail and replay model used by run history, with sensitive values replaced by a fixed redaction marker before storage. Abandonment requires a prior authorized cancellation request and refuses to override an owned persisted run. An acknowledged dispatch consumes its complete reserved upper bound, clears local pending state, and terminates as failed with an explicit unknown-remote-outcome summary. Neither abandonment nor bounded observed-failure retry proves provider cancellation, provider failure, arbitrary kill safety, workflow success, provider truth, actual billing or token usage, remote effect deduplication, remote exactly-once execution, or production readiness.

A local event-chain head proves local sequence consistency only. It does not prove external anchoring, independently witnessed completeness, model truth, provider billing truth, or remote exactly-once effects.

## Safe run inspection

Inspect one persisted run without opening redb through an unrestricted renderer:

```bash
lattice --config ./demo/config.ncl runs show 1
lattice --config ./demo/config.ncl runs show 1 --output json
```

The report includes the numeric run identity, a domain-separated BLAKE3 workflow-name fingerprint, lifecycle counts, node IDs/statuses/counts, exact orchestration owner and dispatch identities, dispatch/recovery/abandonment state, bounded budget facts, and run/orchestration replay state. It structurally omits workflow names, node and port labels, output and error text, replay summaries, orchestration objectives and summaries, actors, and trigger keys. The fingerprint is an identity aid, not a confidentiality claim for guessable workflow names.

Historical missing details or replay are reported as `not_recorded`; contradictory present evidence renders a report with `valid: false` and exits unsuccessfully. A missing run fails without creating a data directory or database. Pending-unobserved and abandoned dispatches with no `RunRecord` remain on `orchestration status|recover|replay` because synthesizing a run would falsely claim local observation.

`runs show` remains strictly read-only. It distinguishes a store currently owned by another process from a store that redb cannot open read-only after an unclean writer exit. Retry after an active owner exits. If recovery is required, invoke the explicit mutating command with the same configuration, then retry inspection:

```bash
lattice --config ./demo/config.ncl runs repair
lattice --config ./demo/config.ncl runs show 1 --output json
```

`runs repair` opens only an existing store through redb's write-mode recovery and creates no Lattice application records. It refuses missing, actively owned, unsupported, or unavailable stores and emits fixed diagnostics rather than backend/path details. Recovery may change redb storage metadata; success proves only that redb reopened the file, not semantic correctness, complete history, external anchoring, provider truth, remote exactly-once effects, or production readiness.

## Local file capabilities

Production `LocalFs` read, write, and append effects require at least one explicit `allowed_roots` entry. The runtime selects the most-specific matching root, opens it as a directory capability, and resolves only the relative target through that handle. The same capability core confines production image inputs, Markdown prompt hydration, and generic-agent image/artifact handoffs while preserving their byte, UTF-8, and media checks. Parent traversal and escaping symlinks fail closed, including if an intermediate path is replaced after the root is opened. Configured roots, workflow directories, and agent working directories are trusted authority grants; these readers do not turn untrusted workflow definitions into a host sandbox.

## Bounded local processes

Shell nodes, command-backed agents, and command credential providers share the pinned `bounded-exec` process mechanism. Application Nickel config supplies positive stdin, stdout, stderr, command-agent deadline, polling, and teardown limits. Lattice resolves the configured executable and current directory, while `bounded-exec` owns concurrent stream draining, bounded retention, deadline/cancellation handling, process-group teardown, and reaping. Truncated command-agent or credential output is rejected before parsing.

These bounds report only the observed local process outcome under the supplied policy. They do not claim sandboxing, executable authorization, hermeticity, syscall restriction, remote cancellation, whole-host cleanup, semantic correctness, or release eligibility.

The dependency is pinned at `29dac88ecded94457572db3fdfaaaab95fa91525`
from canonical Radicle project `rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo` through
`https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git`. Cargo, Nix,
and Crane must agree on that endpoint and revision. Acquisition fails rather
than falling back to GitHub for Bounded Exec. This source identity does not
prove dependency correctness, availability of unrelated inputs, consumer
behavior, CI correctness, or release readiness.

## Verification

```bash
verus verus/example_spec.rs
```

## References

- [bounded-exec](https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git): canonical Radicle project `rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo`, with reviewed AGPL bounded child-process mechanics pinned at `29dac88ecded94457572db3fdfaaaab95fa91525`.
- [Pi coding agent](https://pi.dev): SDK, print, RPC, and image prompt surfaces for generic agent-node integration.
- [Clankers](https://github.com/brittonr/clankers): Rust terminal coding agent with one-shot, daemon, MCP/ACP, context image references, and image-generation surfaces for generic agent-node integration.
- [Trellis](../trellis/): verified-logic primitives for graph admission, job lifecycle, durable replay, effect/capability admission, and typed durable references used as design inputs for the generic pipeline runner.
- [Choregraph](https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git): verified graph and machine source pinned at `fc47c5f4ecbb7b4341af690fa42199f25d57f54c`.
- [execution-graph](https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git): independent compatibility consumer and rollback reference.
- [Seaglass](https://github.com/brittonr/seaglass): original compatibility vocabulary and conformance baseline.
- [Callee](https://github.com/baldaworks/callee): Markdown-defined provider-backed roles and bounded `Sequential`/`Loop` composition used as an authoring and orchestration reference.
- [FarmTable](https://github.com/scion-frontiers/farmtable): graph-native ready-task, dependency, critical-path, bottleneck, and lifecycle separation used as an execution-graph architecture reference.
- [cap-std](https://github.com/bytecodealliance/cap-std): portable capability-directory implementation used to keep local file effects descriptor-relative and reject traversal or symlink escapes.
- [Animus](https://github.com/OnixResearch/animus): authority-free adapter contracts, deterministic bounded agent loop, and Rust-revalidated Steel fabric used beneath Lattice's generic agent boundary.
- [Aspen Octet burn-down guard](../aspen/scripts/octet-burndown.scm): strict probe/validate/progress and exact-policy-restoration reference for guarded agent rounds; not a Lattice runtime dependency.
- [mc Ralph Cairn drain loop](../../mc/scripts/ralph-drain-cairns.scm): plan-first bounded work selection and sequential Pi-round reference; not a Lattice runtime dependency.
