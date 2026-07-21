# animus

`animus` is a modular Rust agent-harness framework scaffolded with explicit crate barriers, swappable adapters, and a functional-core / imperative-shell split.

## Architecture

- `crates/animus-core` is the `#![no_std]` functional core. It uses `alloc` only and owns deterministic business logic and typed core state.
- `crates/animus-adapters` is the `#![no_std]` component boundary crate. It defines implementation-free adapter contracts and DTOs for swappable harness components such as models and tools.
- `crates/animus-preserves-schema` owns authority-free schema identities and family catalog metadata; `crates/animus-preserves-adapter` is the minimal adapter-only text/binary bridge with no runtime dependency; `crates/animus-preserves` remains the compatibility facade and owns runtime, dataspace, provenance/session, and policy-visible family modules.
- `crates/animus-steel` is the canonical Steel Scheme loop fabric. It lets embedders script the agent turn loop while Rust revalidates every selected model/tool/response action at adapter boundaries.
- `crates/animus-runtime` is the reusable agent-loop runtime. It drives typed Steel/fabric decisions through injected model and tool adapters, emits ordered runtime events/reports, and provides exact per-model prompt/skill/memory/tool composition, capability-bound semantic role composition and separation, a coherent metadata-only tool catalog, pure admission, injected executor, bounded output, and safe receipt kits while keeping provider, filesystem, process, network, UI, and persistence authority outside the runtime core.
- `crates/animus-std` is the standard-library host compatibility shell. It owns environment/config parsing, feature-gated generated model-profile and semantic-role loading, process-facing orchestration, a pure authority catalog, and facade re-exports for authority-scoped host blocks.
- `crates/animus-std-observability` is the first direct authority-scoped std block. It supplies deterministic no-op and in-memory observability sinks without pulling unrelated context, provider, ingest, transport, credential, persistence, or terminal behavior.
- `crates/animus-tui` is the optional terminal UI host shell. It keeps Rats/Subwayrat/Ratatui-facing dependencies outside reusable core, adapter, Steel, and std runtime contracts; see [docs/ratatui-tool-host-surface.md](docs/ratatui-tool-host-surface.md) for the host bridge and pure render reducer boundary.
- The root `animus` package is a thin compatibility layer that re-exports the workspace crates.

Embedders should depend on the smallest crate set they need: `animus-core` for pure state, `animus-adapters` for port contracts, `animus-preserves-schema` for schema metadata, `animus-preserves-adapter` for adapter-only Preserves text/binary payloads, `animus-preserves` for the broad compatibility facade and runtime-adjacent Preserves families, `animus-steel` for the main scriptable agent-loop fabric, `animus-runtime` for reusable loop orchestration over injected adapters, `animus-std` for the default compatibility host shell, `animus-std-observability` for direct deterministic std sinks, `animus-tui` for an optional Subwayrat-backed terminal host shell, or root `animus` for the combined compatibility surface. The root `animus` crate enables the `std` feature by default for compatibility; disable default features to select the no-std facade over `animus-core` and `animus-adapters`. Runtime configuration is parsed once at the shell boundary and stored in typed structs before being passed into the core. Component calls cross typed adapter contracts so model, tool, memory, and host integrations can be swapped without changing core logic.

## Development safety rails

The workspace denies missing docs, `clippy::all`, `clippy::pedantic`, and undocumented unsafe blocks at crate roots. Unsafe code is currently forbidden by workspace lint policy. `config/repo.ncl` is the Nickel source of truth for repo-owned tool config, and the generated `dylint.toml` sets the repo-adopted Octet lint baseline to `deny` for the pinned Octet input.

Every test suite should include positive and negative tests. Run baseline tests before core changes and repeat them after the change. Avoid process-global state in tests unless explicit isolation is in place.

Agents should use [AGENTS.md](AGENTS.md) for concise Ralph/backpressure operating guidance. This README remains the detailed command and reference index.

See [docs/safety-contract.md](docs/safety-contract.md) for the durable repo-owned safety contract, [docs/contracts-index.md](docs/contracts-index.md) for maintained adapter families and conformance rails, [docs/component-catalog.md](docs/component-catalog.md) for the Nickel-generated Lego map of component ids, authority posture, dependencies, feature gates, and focused checks, [docs/adapter-family-crates.md](docs/adapter-family-crates.md) for focused safe-receipt and observability family crates with `animus-adapters` compatibility re-exports, [docs/ast-grep-agent-boundary-audits.md](docs/ast-grep-agent-boundary-audits.md) for inventory-only structural audit profiles, BLAKE3 rule-bundle identity, bounded finding receipts, and non-claims, [docs/runtime-composition-preludes.md](docs/runtime-composition-preludes.md) for focused runtime domain preludes, neutral fabrics, capability registries, and family feature guidance, [docs/ralph-loop.md](docs/ralph-loop.md) for the bounded Ralph-style Steel profile, repeated fresh-task prompt, completion promise, and host-owned effect boundaries, [docs/runtime-dag-contract-kit.md](docs/runtime-dag-contract-kit.md) for the reusable host-effect-free DAG validation, ordering, blocked-node, and receipt kit, [docs/batch-trajectory-runner.md](docs/batch-trajectory-runner.md) for the local-only batch trajectory runner contract, [docs/background-work-queues.md](docs/background-work-queues.md) for host-owned long-running queue service contracts, [docs/host-control-protocol.md](docs/host-control-protocol.md) for UI-neutral host topology, bounded read, layout, event, wait, and transport-envelope contracts, [docs/openclaw-gateway-control-plane.md](docs/openclaw-gateway-control-plane.md) for optional bounded OpenClaw-inspired gateway DTO validation, [docs/openclaw-node-capability-hosts.md](docs/openclaw-node-capability-hosts.md) for host-owned OpenClaw-inspired node capability DTO validation, [docs/openclaw-session-isolation-tools.md](docs/openclaw-session-isolation-tools.md) for host-store-neutral session key, visibility, maintenance, and provenance planning, [docs/openclaw-security-plugin-guardrails.md](docs/openclaw-security-plugin-guardrails.md) for host-supplied security audit and plugin guardrail validation, [docs/session-restore-contract.md](docs/session-restore-contract.md) for explicit live-process, snapshot, history replay, native resume, handoff, and fallback restore claims, [docs/process-job-profiles.md](docs/process-job-profiles.md) for pure backend-neutral process-job profile resolution, [docs/provenance-ledger.md](docs/provenance-ledger.md) for Animus-owned provenance, ledger replay, safe receipt, artifact, branch, and Toolpath projection boundaries, [docs/controller-continuation-policy.md](docs/controller-continuation-policy.md) for post-turn continuation state, priority, acknowledgement, and receipt boundaries, [docs/turn-lifecycle-hooks.md](docs/turn-lifecycle-hooks.md) for startup, pre-turn, post-turn hook boundaries and pure FSM constraints, [docs/confirmation-broker.md](docs/confirmation-broker.md) for host-owned confirmation request, binding, and receipt boundaries, [docs/checkpoint-rollback.md](docs/checkpoint-rollback.md) for VCS-neutral checkpoint rollback DTOs, confirmation, freshness, and safe receipt boundaries, [docs/context-references.md](docs/context-references.md) for typed context-reference parsing, resolver, budget, adapter-family, std-shell context resolver, and safe provenance boundaries, [docs/observability-adapters.md](docs/observability-adapters.md) for safe metrics/log DTOs and runtime projection, [docs/std-host-adapter-crates.md](docs/std-host-adapter-crates.md) for authority-scoped std blocks and direct observability sinks, [docs/orchestration-adapter-family.md](docs/orchestration-adapter-family.md) for public host-owned board, task, handoff, scheduling, cancellation, lifecycle-evidence, and runtime bridge contracts, [docs/zerostack-context-file-layering.md](docs/zerostack-context-file-layering.md) for host-owned context-file discovery, admission, precedence, and safe layering receipts, [docs/prompt-mode-profiles.md](docs/prompt-mode-profiles.md) for typed prompt profile descriptors, directive parsing, mode binding, chain plans, and safe receipts, [docs/multimodal-image-payloads.md](docs/multimodal-image-payloads.md) for provider-neutral image model input descriptors, capability gates, provider-native fixtures, and safe receipts, [docs/preserves-boundary-schemas.md](docs/preserves-boundary-schemas.md) for family ownership, dependency isolation, schema versions, fixtures, migrations, and stable-versus-review-only claims, [docs/preserves-dataspace-projections.md](docs/preserves-dataspace-projections.md) for optional Preserves dataspace DTO records, [docs/syndicate-host-dataspace-adapter.md](docs/syndicate-host-dataspace-adapter.md) for the feature-gated host-owned Syndicate dataspace seam, [docs/syndicate-turn-protocol-adapter.md](docs/syndicate-turn-protocol-adapter.md) for the bounded capability-free Syndicate turn wire adapter over caller-owned transports, [docs/iroh-agent-comms-adapter.md](docs/iroh-agent-comms-adapter.md) for the feature-gated host-owned Iroh agent communication seam, [docs/iroh-syndicate-composition.md](docs/iroh-syndicate-composition.md) for the explicit one-turn-packet-per-effect composition over caller-owned admitted Iroh sessions, [docs/claude-code-permission-modes.md](docs/claude-code-permission-modes.md) for bounded local permission-mode caveats, [docs/zerostack-permission-kernel.md](docs/zerostack-permission-kernel.md) for the advisory local permission kernel, [docs/claude-code-headless-surfaces.md](docs/claude-code-headless-surfaces.md) for noninteractive headless automation contracts, [docs/claude-code-lifecycle-hooks.md](docs/claude-code-lifecycle-hooks.md) for bounded typed lifecycle hook extension contracts, [docs/claude-code-mcp-bridge.md](docs/claude-code-mcp-bridge.md) for optional MCP adapter-layer interoperability contracts, [docs/pi-tool-contracts.md](docs/pi-tool-contracts.md) for Pi-derived tool metadata, argument, output, mutation, and failure contracts, [docs/tool-executor-kit.md](docs/tool-executor-kit.md) for coherent host-injected executor admission and safe receipts, [docs/hashed-edit-protocol.md](docs/hashed-edit-protocol.md) for BLAKE3 snapshot/tagged-span edit validation and safe receipts, [docs/pi-resource-discovery.md](docs/pi-resource-discovery.md) for Pi-style resource descriptor and progressive-disclosure boundaries, [docs/zerostack-provider-registry-hardening.md](docs/zerostack-provider-registry-hardening.md) for provider API-style resolution, redacted headers, scoped extra body fields, model filtering, and safe selection receipts, [docs/pi-headless-rpc-protocol.md](docs/pi-headless-rpc-protocol.md) for the Pi-style headless command/event protocol boundary, [docs/request-lifecycle.md](docs/request-lifecycle.md) for the maintained request-lifecycle ownership map, [docs/pi-session-tree-ledger.md](docs/pi-session-tree-ledger.md) for the neutral active-leaf replay and label metadata contract, [docs/pi-compaction-branch-summaries.md](docs/pi-compaction-branch-summaries.md) for typed compaction policy, safe cut points, branch handoff summaries, injected summarizer decisions, and redacted summary metadata, [docs/zerostack-midturn-compaction.md](docs/zerostack-midturn-compaction.md) for calibrated prompt pressure, explicit stop/compact/resume planning, ledger-summary validation, and bounded tool-output artifacts, [docs/remote-admission-receipts.md](docs/remote-admission-receipts.md) for transport-neutral admission gates and safe receipts, [docs/status-authority.md](docs/status-authority.md) for semantic agent status precedence, [docs/branchfs-speculative-workspaces.md](docs/branchfs-speculative-workspaces.md) for the bounded speculative-workspace reference boundary, [docs/agent-definition-trust.md](docs/agent-definition-trust.md) for typed agent-definition trust boundaries, [docs/sandlock-sandbox-enforcement.md](docs/sandlock-sandbox-enforcement.md) for portable Sandlock-inspired sandbox descriptor and receipt contracts, [docs/openai-codex-execpolicy-rules.md](docs/openai-codex-execpolicy-rules.md) for sandbox escalation exec-policy contracts, [docs/openai-codex-subagent-threads.md](docs/openai-codex-subagent-threads.md) for explicit subagent delegation and bounded parent handoff contracts, [docs/zerostack-fresh-context-tasks.md](docs/zerostack-fresh-context-tasks.md) for fresh-context task descriptors, read-only side questions, child isolation, attributed results, and cancellation receipts, [docs/openai-codex-jsonl-exec-sdk.md](docs/openai-codex-jsonl-exec-sdk.md) for non-interactive JSONL exec and SDK control contracts, [docs/openai-codex-worktree-automations.md](docs/openai-codex-worktree-automations.md) for VCS-neutral isolated workspace automation contracts, [docs/model-streaming.md](docs/model-streaming.md) for token/thought streaming boundaries, [docs/provider-streaming-adapters.md](docs/provider-streaming-adapters.md) for optional fixture-backed provider stream parsers, [docs/model-router-execution.md](docs/model-router-execution.md) for host-owned router execution service contracts, [docs/model-runtime-profiles.md](docs/model-runtime-profiles.md) for exact per-model prompt/skill/memory/tool composition, fallback isolation, and metadata-only receipts, [docs/agent-role-profiles.md](docs/agent-role-profiles.md) for typed capability-bound implementer/reviewer/verifier/judge roles, exact authority intersections, separation of duty, transition binding, and metadata-only outputs, [docs/retrieval-budgets.md](docs/retrieval-budgets.md) for bounded retrieval policy metadata, [docs/hermes-skill-manifests.md](docs/hermes-skill-manifests.md) for host-provided skill manifest descriptors, [docs/hermes-plugin-tool-manifests.md](docs/hermes-plugin-tool-manifests.md) for host-injected plugin/tool manifest boundaries, [docs/hermes-kanban-orchestration.md](docs/hermes-kanban-orchestration.md) for optional Kanban task graph, workspace, tenant, and handoff orchestration contracts, [docs/hermes-researchspec-lifecycle.md](docs/hermes-researchspec-lifecycle.md) for optional ResearchSpec-inspired study, baseline, hypothesis, run, keep/promotion, and archive gates, [docs/ornith-scaffold-profiles.md](docs/ornith-scaffold-profiles.md) for route-neutral reviewed scaffold strategy profiles, [docs/ornith-trust-boundary-monitor.md](docs/ornith-trust-boundary-monitor.md) for immutable outer trust-boundary descriptors, deterministic monitor denials, judge-veto binding, and redacted violation receipts, [docs/ornith-eval-feedback-loop.md](docs/ornith-eval-feedback-loop.md) for local reviewed scaffold evaluation feedback plans, receipts, review outcomes, and freshness checks, [docs/goose-code-mode-tool-planning.md](docs/goose-code-mode-tool-planning.md) for typed Code Mode-style progressive tool discovery, declarative batch planning, admission, intermediate-data, and safe-receipt boundaries, [docs/goose-recipe-workflows.md](docs/goose-recipe-workflows.md) for reusable recipe-workflow contracts, [docs/goose-recipe-backed-subagents.md](docs/goose-recipe-backed-subagents.md) for recipe-backed child-run boundaries, [docs/claude-code-agent-teams.md](docs/claude-code-agent-teams.md) for bounded worker-task delegation contracts, [docs/goose-acp-mcp-interop.md](docs/goose-acp-mcp-interop.md) for ACP/MCP bridge descriptors, [docs/goose-scheduled-recipes.md](docs/goose-scheduled-recipes.md) for scheduled workflow jobs, [docs/hermes-review-report-contracts.md](docs/hermes-review-report-contracts.md) for structured review report parsing and metric-record boundaries, [docs/qualifier-quality-records.md](docs/qualifier-quality-records.md) for Qualifier-inspired safe quality envelopes, sidecar boundaries, lifecycle transitions, span freshness, and compact agents guidance, [docs/runtime-facade-inventory.md](docs/runtime-facade-inventory.md) for public export classes, [docs/subagent-runtime-facade.md](docs/subagent-runtime-facade.md) for unified subagent facade, context, coordinator, fan-in, lineage, UX, and external conformance boundaries, [docs/focused-verification-rails.md](docs/focused-verification-rails.md) for focused rail conventions, [docs/composition-conformance-rail.md](docs/composition-conformance-rail.md) for cross-module conformance coverage, [docs/clankers-tool-gateway-delivery.md](docs/clankers-tool-gateway-delivery.md) for policy-checked artifact delivery and safe outbox receipts, [docs/dogfoodable-host-readiness-roadmap.md](docs/dogfoodable-host-readiness-roadmap.md) for the non-default dogfoodability gap inventory, readiness matrix, evidence ladder, and claim-hygiene rail, and [docs/release-evidence-index.md](docs/release-evidence-index.md) for the bounded future release-evidence index evaluation.

See [docs/goose-message-queue-interrupts.md](docs/goose-message-queue-interrupts.md) for the pure queued-message reducer, interrupt classifier, fail-closed controls, and safe receipt boundary, and [docs/goose-remote-server-admission.md](docs/goose-remote-server-admission.md) for Goose-style remote backend descriptors, host-supplied handshake summaries, fail-closed admission, and safe receipts.

See [docs/portfolio-search.md](docs/portfolio-search.md) for the pure bounded portfolio state machine, mechanism registry, delayed cross-pollination, gap-strength gates, adversarial audit, honest terminal outcomes, and safe receipt boundary.

See [docs/interrupted-tool-call-repair.md](docs/interrupted-tool-call-repair.md) for exact active-branch tool binding, evidence-bound synthetic failures, append-only repair planning, durable provider gating, idempotence, and metadata-only receipts.

See [docs/provider-retry-execution.md](docs/provider-retry-execution.md) for typed provider-failure classification, bounded same-plan retry, semantic-stream replay denial, injected cancellable timers, provider-neutral progress, and metadata-only receipts.

See [docs/redacted-session-tree-export.md](docs/redacted-session-tree-export.md) for validated-metadata topology projection, redaction-first Nickel policy, deterministic script-free artifacts, explicit destination-capability writes, and exact metadata-only receipts.

See [docs/nickel-policy-contracts.md](docs/nickel-policy-contracts.md) for shared authoring-time contracts, checked policy projections, Rust DTO parity, authority ownership, and projection non-claims.

## Development shell

Enter the pinned Nix development shell:

```sh
nix develop
```

The shell provides Rust, Clippy, rustfmt, rust-analyzer, Nickel, pre-commit, and Octet/Tigerstyle tooling.

## Configuration

Repo-owned tool config lives in `config/repo.ncl` with typed Nickel contracts. The tool-native files `.cargo/config.toml`, `.pre-commit-config.yaml`, `clippy.toml`, and `dylint.toml` are generated artifacts for consumers that require TOML or YAML.

After editing `config/repo.ncl`, regenerate native config files:

```sh
scripts/config/export-nickel-configs
```

Check for generated-config drift:

```sh
check-nickel-configs
# or
nix run .#check-nickel-configs
```

## Wasm compile surface

The supported Wasm compile boundary targets `wasm32-unknown-unknown` for deterministic reusable contracts. The initial supported surface is:

```sh
cargo check -p animus-core --target wasm32-unknown-unknown --locked
cargo check -p animus-adapter-safe --target wasm32-unknown-unknown --locked
cargo check -p animus-adapter-observability --target wasm32-unknown-unknown --locked
cargo check -p animus-adapters --target wasm32-unknown-unknown --locked
cargo check -p animus-router --target wasm32-unknown-unknown --locked
cargo check -p animus --no-default-features --target wasm32-unknown-unknown --locked
```

The root `animus --no-default-features` profile intentionally excludes `animus-std`, `animus-steel`, `animus-tui`, `animus-preserves`, terminal UI, provider clients, native persistence, process execution, filesystem authority, credentials, browser APIs, WASI host behavior, plugin loading, and host shell orchestration. The dedicated `animus-router` crate is checked separately as a no-std routing decision core. Optional bridges such as Preserves are not part of the initial Wasm rail; add them only with separate compatibility evidence and adapter DTO revalidation tests.

Run the focused rail:

```sh
scripts/cairn/check-wasm-surface --root .
nix build .#checks.x86_64-linux.wasm-surface --no-link -L
```

## Verification

```sh
cargo test -p animus-core
cargo test -p animus-adapters
cargo test -p animus-preserves-schema
cargo test -p animus-preserves-adapter
cargo test -p animus-preserves
cargo test -p animus-preserves syndicate --locked
scripts/cairn/check-preserves-boundary-schemas --root .
cargo test -p animus-steel
cargo test -p animus-runtime
cargo test -p animus-runtime session_persistence --locked
cargo test -p animus-std
cargo test -p animus-std-observability
cargo check -p animus-std --no-default-features
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p animus --features preserves
cargo check -p animus --features steel
cargo check -p animus --features runtime
cargo check -p animus --features syndicate-host
cargo test -p animus-std --features iroh-syndicate-host irohsyndicate --locked
cargo check -p animus --features iroh-syndicate-host --locked
cargo check -p animus --features model-runtime-profiles
cargo check -p animus --features agent-role-profiles
cargo check -p animus --features provider-retry
cargo check -p animus --features tui
cargo tigerstyle check
check-nickel-configs
scripts/cairn/check-nickel-policy-contracts --root .
scripts/cairn/check-model-runtime-profiles --root .
scripts/cairn/check-agent-role-profiles --root .
scripts/cairn/check-provider-retry --root .
scripts/cairn/check-clankers-product-surfaces --root .
scripts/cairn/check-dogfood-readiness --root .
scripts/cairn/check-packaged-headless-dogfood-smoke --root .
scripts/cairn/check-clash-ucan-authority --root .
scripts/cairn/check-plugin-manifest-runtime-neutral --root .
scripts/cairn/check-request-lifecycle --root .
scripts/cairn/check-steel-runtime-policy --root .
scripts/cairn/check-neutral-ledger --root .
scripts/cairn/check-injected-services --root .
scripts/cairn/check-composition-conformance --root .
scripts/cairn/check-adapter-conformance-index --root .
scripts/cairn/check-component-catalog --root .
scripts/cairn/check-runtime-kernel-split --root .
scripts/cairn/check-adapter-family-crates --root .
scripts/cairn/check-ast-grep-agent-boundary-audits --root .
scripts/cairn/check-adapter-maintainability --root .
scripts/cairn/check-release-evidence-index --root .
scripts/cairn/check-current-head-release-evidence-index --root .
scripts/cairn/check-runtime-facade-inventory --root .
scripts/cairn/check-safe-receipts-artifacts --root .
scripts/cairn/check-tool-catalog --root .
scripts/cairn/check-wasm-surface --root .
nix run .#cargo-tigerstyle -- check
nix run .#check-nickel-configs
nix build .#checks.x86_64-linux.adapter-conformance-index --no-link -L
nix build .#checks.x86_64-linux.component-catalog --no-link -L
nix build .#checks.x86_64-linux.runtime-kernel-split --no-link -L
nix build .#checks.x86_64-linux.adapter-family-crates --no-link -L
nix build .#checks.x86_64-linux.ast-grep-agent-boundary-audits --no-link -L
nix build .#checks.x86_64-linux.dogfood-readiness --no-link -L
nix build .#checks.x86_64-linux.packaged-headless-dogfood-smoke --no-link -L
nix build .#checks.x86_64-linux.clash-ucan-authority --no-link -L
nix build .#checks.x86_64-linux.neutral-ledger --no-link -L
nix build .#checks.x86_64-linux.safe-receipts-artifacts --no-link -L
nix build .#checks.x86_64-linux.current-head-release-evidence-index --no-link -L
nix flake check
```

## Cairn drain helpers

Generate a Steel Scheme drain plan for selected Animus changes by asking Pi's Steel tool to run:

```text
file: scripts/ralph-drain-cairns.scm
args: ["--change", "introduce-steel-runtime-loop"]
```

The single-repository helper defaults to the repo-pinned Cairn command (`nix run .#cairn --`) and emits Animus-specific guardrails for roadmap blockers, accepted-spec sync verification, evidence safety, and non-claims.

The temporary Pi-hosted sibling-stack loop is a workspace-local operator tool, not a packaged Animus runtime surface. From the parent OnixResearch workspace root, first inspect its effect-free plan:

```text
file: ralph-drain-stack.scm
args: []
```

Then opt into one fresh `pi -p` round:

```text
file: ralph-drain-stack.scm
args: ["--repo", "cairn", "--run", "--limit", "1"]
```

See `../README.md` for full workspace usage, model overrides, blocker semantics, and non-claims. The canonical helper lives at `../ralph-drain-stack.scm`; Animus intentionally does not retain a second implementation copy.

## Pre-commit hooks

Install the repo hooks once per checkout:

```sh
pre-commit install
```

The `nickel-configs` hook runs `scripts/config/check-nickel-configs` before each commit to reject generated-config drift. The `octet-lints` hook runs `scripts/hooks/octet-lints`; that wrapper invokes the pinned flake app with `nix run .#cargo-tigerstyle -- check` when Nix is available, then falls back to `cargo tigerstyle check`.

## License

Repository-owned Animus crates are `MIT OR Apache-2.0`; complete texts are in [LICENSES](LICENSES). Optional AGPL dependencies and integrations retain their own terms and are not enabled by default. The reusable core/adapters must not acquire accidental AGPL coupling. Third-party code and generated material containing upstream source retain upstream terms.

## References

- [Basalt](https://github.com/OnixResearch/basalt) — hardened UCAN contract boundaries for Nickel policy artifacts and Steel DTO evaluation backends. Added as the optional Cargo feature `basalt-reference` through the SSH-pinned `basalt` package and as a flake source input.
- [Cairn](https://github.com/OnixResearch/cairn) — Rust lifecycle engine for native repository lifecycle work. Added as the optional Cargo feature `cairn-reference` through the SSH-pinned `cairn-core` package and as a flake source input.
- [Trellis](https://github.com/OnixResearch/trellis) — composable verified-logic building blocks. Added as the optional Cargo feature `trellis-reference` through the SSH-pinned `verified-logic` package and as a flake source input.
- [`../clankers`](../clankers) — terminal coding agent in Rust, inspired by pi and built to be hacked on.
- [Clash](https://github.com/empathic/clash) — command-line agent safety harness used as a bounded reference for capability taxonomy, explain traces, sandbox descriptors, and scenario tests; not adopted wholesale.
- [Pi](https://github.com/earendil-works/pi) — self-extensible coding agent harness with tool calling, terminal UI, and multi-provider LLM runtime; used as a reference codebase.
- [Tau](https://github.com/huggingface/tau) — small Pi-inspired Python coding-agent reference used for interrupted tool-call repair, durable append-only session behavior, a narrow coding-session facade, cancellable provider retry progress, and session-tree export scenarios; Animus adapts bounded behavior behind its own Rust, authority, redaction, BLAKE3, and receipt contracts rather than copying Tau's application monolith or storage defaults.
- [Git AI](https://github.com/git-ai-project/git-ai) — explicit line-level AI provenance, checkpointing, attribution stats, and Git notes interoperability used as a bounded reference for VCS-neutral Animus provenance; Git-specific behavior remains optional adapter scope.
- [Toolpath](https://github.com/empathic/toolpath) — tool-agnostic artifact transformation provenance format used as a bounded reference for optional redacted Graph/Path/Step interchange; Animus-owned provenance remains internal.
- [Qualifier](https://github.com/empathic/qualifier) — continuous annotation and Metabox-style record substrate used as a bounded reference for safe quality records, span freshness, append-only sidecars, and AGENTS-CLI guidance; not adopted as the Animus session ledger or core runtime dependency.
- [Preserves](https://preserves.dev/) — expressive data model and text/binary serialization formats for portable adapter boundary payloads. Added through the optional `animus-preserves` crate and root `preserves` feature.
- [Syndicate](https://syndicate-lang.org/) / [Syndicate Protocol](https://synit.org/book/protocol.html) / [Syndicate/rs](https://git.syndicate-lang.org/syndicate-lang/syndicate-rs) — syndicated actors, dataspaces, assertion-based state replication, facets, conversational concurrency, and Preserves turn packets; adapted through optional host-owned dataspace and capability-free turn seams, not as a core dependency or full relay implementation.
- [Iroh](https://github.com/n0-computer/iroh) — peer-to-peer connectivity and content-addressed transport reference used for the optional host-owned `iroh-host` communication seam; live endpoints, tickets, relays, keys, and network policy stay outside reusable crates.
- [Steel Scheme](https://github.com/mattwparas/steel) — embeddable Scheme implementation used by `animus-steel` as the main scriptable agent-loop fabric.
- [Rats/Subwayrat](https://github.com/brittonr/rats) — Rust terminal UI workspace; `animus-tui` uses the `subwayrat/crates/rat-widgets` and `subwayrat/crates/rat-keymap` packages through SSH-pinned Cargo dependencies.
- [`../hermes`](../hermes) — Hermes Agent workspace used as a bounded reference for skill manifests, plugin/tool manifests, review report contracts, Kanban orchestration, policy profiles, retrieval budgets, background work queues, and ResearchSpec lifecycle patterns.
- [Herdr](https://github.com/ogulcancelik/herdr) — terminal-native agent multiplexer reference for semantic agent status authority, session restore semantics, host-control protocol shape, and plugin/action manifests; adapted only as bounded patterns behind Animus-owned contracts.
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) — coding-agent product reference for layered project memory, permission modes, workflow commands, lifecycle hooks, agent teams, headless automation, and MCP bridge ergonomics; adapted only as bounded patterns behind Animus-owned contracts.
- [Goose](https://goose-docs.ai/) — general-purpose agent reference for reusable recipes, structured automation output, ACP/MCP interoperability, recipe-backed subagents, and layered security review; adapted only as bounded patterns behind Animus-owned contracts.
- [vLLM Semantic Router micro-agent recipes](https://vllm.ai/blog/2026-06-29-micro-agent-frontier-models) — router-side Confidence, Ratings, ReMoM, Fusion, and workflow recipe patterns used as a bounded reference for Animus-owned model-router contracts; no vLLM serving dependency or benchmark claims are adopted.
- [Agent Memory Techniques](https://github.com/NirDiamant/Agent_Memory_Techniques) — taxonomy, routing, retrieval-quality, evaluation, and production-memory pattern reference for future Animus-owned memory contracts; Python notebooks, managed-service assumptions, prompt text, datasets, provider calls, and concrete storage implementations are not adopted.
- [Claude Converter](https://github.com/FredyRivera-dev/claude_converter) — compact session-to-Hugging-Face `messages` conversion and inspection surface used as a bounded reference for Animus-owned dataset export; Animus sessions remain the source format.
- [OpenAI Codex](https://github.com/openai/codex) — coding-agent reference for exec policy rules, hook trust lifecycles, non-interactive JSONL execution and SDK control, worktree automations, subagent threads, guidance/skill budgeting, opt-in memories, and read-only review mode; adapted only as bounded patterns behind Animus-owned contracts.
- [OpenClaw](https://github.com/openclaw/openclaw) — personal AI assistant reference for gateway control-plane handshakes, node capability hosts, session isolation, session tools, security audit diagnostics, and plugin manifest guardrails; adapted only as bounded host-owned patterns behind Animus-owned contracts.
- [Sandlock](https://github.com/multikernel/sandlock) — lightweight Linux process sandbox using Landlock, seccomp-bpf, seccomp user notification, COW filesystem handling, and network/HTTP ACLs; useful as a bounded reference for std-shell sandbox enforcement and descriptor translation, not a core dependency.
- [BranchFS](https://github.com/multikernel/branchfs) — FUSE-based speculative branching filesystem with copy-on-write workspaces, nested branches, atomic commit/abort, and `@branch` paths; useful as a bounded reference for optional workspace-isolation adapters, not a required dependency.
- [`../mc`](../mc) — sibling Cairn-heavy workspace whose `scripts/ralph-drain-cairns.scm` Steel loop was adapted as an Animus-specific drain planner; no Minecraft runtime behavior is adopted.
- [Zerostack](https://github.com/gi-dellav/zerostack) — minimal Rust coding-agent reference for permission modes, prompt profiles, provider registry hardening, hashed edits, mid-turn compaction, fresh-context tasks, and context-file layering; adapted only as clean-room bounded patterns due to GPL licensing.
- [Turnstone](https://github.com/turnstonelabs/turnstone) — self-hosted agent harness reference for output guards, intent validation, headless eval scoring, tool-loop eventing, and local-first orchestration; adapted only as bounded host-shell and reusable-contract patterns, not as product code.
- [Ralph](https://github.com/snarktank/ralph), [Everything is a Ralph loop](https://ghuntley.com/loop/), [The Ralph Playbook](https://github.com/ghuntley/how-to-ralph-wiggum), [Claude Ralph Loop](https://claude.com/plugins/ralph-loop), and [Don't waste your backpressure](https://banay.me/dont-waste-your-backpressure/) — repeated single-task loop and automated-feedback references used for the bounded `ralph-loop` Steel policy profile; Animus does not copy shell/plugin behavior or grant ambient filesystem, VCS, shell, browser, provider, network, or PR authority.
- [joweeba/dextract](https://github.com/joweeba/dextract), [joweeba/dpdf](https://github.com/joweeba/dpdf), and [joweeba/dKNOW](https://github.com/joweeba/dKNOW) — document extraction and knowledge-ingest references for host-owned adapter DTOs, fail-closed format handling, budgets, diagnostics, and safe receipts; Animus does not adopt parser authority in reusable crates.
- [joweeba/gamma-crown](https://github.com/joweeba/gamma-crown), [joweeba/mly](https://github.com/joweeba/mly), and [joweeba/dML](https://github.com/joweeba/dML) — ML evidence and conversion-report references for optional model-evidence adapters; Animus treats such reports as bounded advisory inputs unless future Cairn gates promote a narrower profile.
