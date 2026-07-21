## Phase 1: Dependency and pure boundary

- [ ] [serial] Pin the published Animus adapter/runtime/Steel crates, add the additive `animus` target and runner binding, and preserve existing Pi/Clankers route behavior. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_compatibility]
- [ ] [serial] Implement a pure Animus execution plan with checked BLAKE3 program identity, named two-turn ceiling, supported-input admission, final output decoding, and bounded metadata construction. r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_output]
- [ ] [parallel] Add positive and negative pure-core tests for identity stability/drift, supported text/JSON modes, unsupported image/artifact/session inputs, malformed JSON, and bounded metadata. r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_output]

## Phase 2: Imperative leaf runtime

- [ ] [serial] Implement the bounded LoopFabric wrapper and Animus shell behind `ProductionAgentAdapter`, adapting model calls to the existing Lattice OpenAI-compatible provider shell. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_effects]
- [ ] [serial] Keep the tool registry/capability/executor absent, map terminal statuses to node-scoped diagnostics, and ensure provider credentials are resolved and redacted only at the Lattice transport boundary. r[lattice.agent_runtime.animus_effects] r[lattice.agent_runtime.animus_output]
- [ ] [parallel] Add positive text/JSON execution tests and negative tool, turn-exhaustion, provider-failure, credential-leak, missing-route, and no-fallback tests. r[lattice.agent_runtime.animus_effects] r[lattice.agent_runtime.animus_output] r[lattice.agent_runtime.animus_compatibility]

## Phase 3: Authoring, dogfood, and evidence

- [ ] [serial] Add a typed Nickel Animus provider example, an importable agent workflow, operator documentation, and the required Animus reference with explicit authority and readiness non-claims. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_compatibility]
- [ ] [serial] Dogfood the checked example through config loading, workflow validation, and a deterministic fake transport while proving existing direct routes remain compatible. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_compatibility]
- [ ] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn proposal/design/tasks gates and validation, and bounded `nix flake check`; sync the accepted spec and archive only after all checked tasks and evidence are current. r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_effects] r[lattice.agent_runtime.animus_output] r[lattice.agent_runtime.animus_compatibility]
