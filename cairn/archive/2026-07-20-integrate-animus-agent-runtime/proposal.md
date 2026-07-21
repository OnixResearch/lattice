## Why

Lattice can dispatch a generic agent node directly to Pi, Clankers, or one bounded OpenAI-compatible model call, but it does not yet have a reusable bounded inner agent loop. Animus already supplies authority-free model/tool adapter contracts, a deterministic runtime loop, and a Rust-revalidated Steel loop fabric. Integrating that runtime behind Lattice's existing agent-adapter boundary adds multi-turn agent behavior without creating a second workflow orchestrator or transferring credentials, effects, persistence, budgets, triggers, or graph authority out of Lattice.

## What Changes

- Add an `animus` generic-agent target that invokes a fixed, BLAKE3-identified, two-turn Steel program through the existing production agent adapter.
- Adapt Lattice's bounded OpenAI-compatible transport into Animus's model adapter contract while keeping credential lookup at the Lattice provider boundary.
- Keep tools absent in the first slice so any Steel tool request fails closed before a product tool effect.
- Add a pure planning and response-admission core that validates target inputs, turn bounds, program identity, output mode, and redaction-safe metadata independently of network or runtime execution.
- Map Animus terminal status and safe event counts into normal typed Lattice agent outputs and node-scoped diagnostics.
- Preserve existing Pi, Clankers, direct OpenAI-compatible, workflow, runner, orchestration, and n8n interchange behavior.

## Success Criteria

- A headless or runner-dispatched generic agent node targeting `animus` performs one model call and returns its final text or JSON through a two-turn Animus Steel loop.
- The built-in Steel source has a checked BLAKE3 identity and cannot be replaced through workflow input or ambient configuration.
- The Lattice-owned transport resolves bearer credentials only at dispatch and neither Animus input nor returned metadata contains credential material.
- A malformed program identity, unsupported image/artifact/session request, exhausted turn budget, model failure, or tool request fails only the affected node without fallback to another target.
- Returned metadata identifies the bounded Animus runtime, program identity, event count, model provider metadata, and explicit disabled-tool boundary without raw prompts or model responses.
- Positive and negative integration tests, repository checks, Octet, Cairn gates, and bounded Nix validation pass.

## Non-Goals

- Replacing Seaglass, Lattice's runner, or durable orchestration with Animus DAG, Kanban, scheduling, queue, session, provenance, TUI, or headless surfaces.
- Loading arbitrary Steel programs, model profiles, semantic roles, memory policies, skills, or tool definitions in the first slice.
- Providing concrete product tools, ambient tool discovery, provider fallback, remote exactly-once execution, model truth, or objective-completion authority.
- Claiming Animus live-provider readiness independently of the Lattice-owned OpenAI-compatible host adapter.

## Impact

- **Files**: Cargo dependency metadata, generic agent target/routing, a new Animus functional core and shell, runner registry, typed Nickel example, README reference/usage docs, tests, and Cairn traceability artifacts.
- **Compatibility**: `animus` is additive. Existing target labels and their command/OpenAI routing remain unchanged.
- **Testing**: pre-change `scripts/check.rs`; positive text/JSON and typed Nickel integration; negative identity, turn, tool, unsupported-input, credential, provider, and compatibility tests; post-change `scripts/check.rs`; Octet; Cairn gates/validation; bounded `nix flake check`.
