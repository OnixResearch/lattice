## Context

Lattice's `AgentAdapter` is the established imperative boundary for generic agent nodes. `ProductionAgentAdapter` currently chooses a command route or a direct OpenAI-compatible route, while the runner, Seaglass graph core, credential provider, orchestration ledger, budgets, triggers, and stores remain Lattice-owned.

Animus `0.1.0` exposes synchronous `ModelAdapter` and `ToolAdapter` contracts, `run_agent_loop`, ordered runtime events, typed terminal status, and a Steel loop fabric whose returned decisions are revalidated by Rust. The published source selected for this change is `github.com/OnixResearch/animus` revision `37256d392d75fab519dcc18cff8b3abc2d162bd4`. Animus's own readiness roadmap makes live providers and concrete tools host-owned non-claims, so Lattice must supply those effects rather than infer them from Animus.

## Decisions

### Decision: Keep Animus beneath the existing agent adapter

**Choice:** Add `AgentTarget::Animus` and route it through `ProductionAgentAdapter`; do not add an Animus workflow runtime, scheduler, store, trigger, orchestration controller, or Seaglass replacement.

**Rationale:** One generic agent node remains one Lattice runner step. Lattice and Seaglass retain their current ownership while Animus decides only the bounded inner model/response loop.

### Decision: Ship one immutable two-turn loop before configurable policy

**Choice:** Compile one Steel program that requests the model on the first turn and returns the last model message on the second turn. Bind it to a checked `b3:` identity and enforce a named two-turn ceiling through a Lattice wrapper around `LoopFabric`. Animus's own default maximum remains a secondary fail-closed guard.

**Rationale:** This proves the integration seam without introducing arbitrary script loading, profile selection, tool policy, or a second mutable configuration language surface. A successor Cairn can add reviewed Nickel-selected policy identities after dogfood evidence exists.

### Decision: Reuse Lattice's OpenAI-compatible provider shell

**Choice:** The Animus model adapter constructs a provider-only Lattice request from the validated Animus model request and calls the existing bounded OpenAI-compatible planner, credential lookup, transport, and decoder.

**Rationale:** Lattice already enforces endpoint schemes, HTTPS credentials, request/response byte limits, timeouts, redirect denial, output typing, and redaction. Animus receives neither bearer material nor ambient provider discovery.

### Decision: Disable tool effects in the first slice

**Choice:** Supply no tool definitions, no tool capability, and no tool adapter. A tool decision is admitted only far enough to produce a typed denied/absent runtime failure; it cannot reach a filesystem, process, network, credential, or mutation implementation.

**Rationale:** Tool execution requires a separately reviewed mapping from Animus admission to Lattice effects, approvals, budgets, and receipts. Model-only integration establishes value without weakening those boundaries.

### Decision: Separate pure admission from runtime execution

**Choice:** Put program identity, turn-limit validation, request admission, final output decoding, terminal-status classification inputs, and safe metadata construction in a deterministic core. Keep Steel evaluation, Animus loop execution, OpenAI transport, credential lookup, and adapter orchestration in a thin shell.

**Rationale:** Core behavior remains testable with plain assertions and no provider, network, filesystem, process, clock, database, or Steel runtime.

### Decision: Preserve only bounded evidence

**Choice:** Return program identity, configured turn ceiling, ordered event count, explicit tool-disabled status, and the existing safe provider metadata. Do not persist raw Animus events, prompts, model responses, credentials, or a second provenance ledger.

**Rationale:** Lattice's runner and orchestration event models remain authoritative. Animus evidence describes only one bounded node invocation and does not prove provider truth, objective truth, effect uniqueness, or release eligibility.

## Risks / Trade-offs

- Animus is a broad pre-1.0 dependency; pinning an immutable revision prevents drift but requires explicit successor changes for upgrades.
- `animus-runtime` and Steel increase compile closure and may expose transitive fetch constraints. Cargo and Nix checks must prove the selected published revision is reproducible before archive.
- The first slice performs one model call, not a general agent/tool loop. This is intentional evidence scoping rather than parity with Animus's wider contract surface.
- A synchronous inner loop occupies the existing synchronous agent-adapter call. Cancellation between inner turns is not claimed until the adapter boundary gains an explicit cancellation port.
- Lattice's orchestration budget reserves one generic agent-node invocation. This slice performs exactly one provider dispatch per node; configurable repeated model dispatch remains blocked on explicit budget accounting.
