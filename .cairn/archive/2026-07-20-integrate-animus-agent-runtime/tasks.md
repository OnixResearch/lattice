## Phase 1: Dependency and pure boundary

- [x] [serial] Pin the published Animus adapter/runtime/Steel crates, add the additive `animus` target and runner binding, and preserve existing Pi/Clankers route behavior. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_compatibility]
- [x] [serial] Implement a pure Animus execution plan with checked BLAKE3 program identity, named two-turn ceiling, supported-input admission, final output decoding, and bounded metadata construction. r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_output]
- [x] [parallel] Add positive and negative pure-core tests for identity stability/drift, supported text/JSON modes, unsupported image/artifact/session inputs, malformed JSON, and bounded metadata. r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_output]

## Phase 2: Imperative leaf runtime

- [x] [serial] Implement the bounded LoopFabric wrapper and Animus shell behind `ProductionAgentAdapter`, adapting model calls to the existing Lattice OpenAI-compatible provider shell. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_effects]
- [x] [serial] Keep the tool registry/capability/executor absent, map terminal statuses to node-scoped diagnostics, and ensure provider credentials are resolved and redacted only at the Lattice transport boundary. r[lattice.agent_runtime.animus_effects] r[lattice.agent_runtime.animus_output]
- [x] [parallel] Add positive text/JSON execution tests and negative tool, turn-exhaustion, provider-failure, credential-leak, missing-route, and no-fallback tests. r[lattice.agent_runtime.animus_effects] r[lattice.agent_runtime.animus_output] r[lattice.agent_runtime.animus_compatibility]

## Phase 3: Authoring, dogfood, and evidence

- [x] [serial] Add a typed Nickel Animus provider example, an importable agent workflow, operator documentation, and the required Animus reference with explicit authority and readiness non-claims. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_compatibility]
- [x] [serial] Dogfood the checked example through config loading, workflow validation, and a deterministic fake transport while proving existing direct routes remain compatible. r[lattice.agent_runtime.animus_leaf] r[lattice.agent_runtime.animus_compatibility]
- [x] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn proposal/design/tasks gates and validation, and bounded `nix flake check`; sync the accepted spec and archive only after all checked tasks and evidence are current. r[lattice.agent_runtime.animus_fabric] r[lattice.agent_runtime.animus_effects] r[lattice.agent_runtime.animus_output] r[lattice.agent_runtime.animus_compatibility]

## Verification evidence

- The pre-change and post-change `scripts/check.rs` rails passed formatting, the full default-member test suite, strict Clippy, and Cairn spec-reference checks.
- Seven focused integration tests passed pure identity/output admission, text/JSON execution, credential confinement, provider failure without fallback, typed example loading, runner dispatch, and existing direct-route compatibility; two unit tests passed turn-exhaustion and absent-tool negative paths.
- The pinned Animus sync script reproduced revision `37256d392d75fab519dcc18cff8b3abc2d162bd4` without snapshot drift, and the Nix source filter retained its compile-time Nickel/JSON/Steel assets.
- `octet-precommit-check` passed with 922 accepted/current findings, zero new findings, zero stale findings, and baseline hash `b3:d97bc567b7450fd4d6a00e8dbf37fd5297acef41efa1ac7a5090708fdefc7764`.
- The self-tested migration preserved the prior 1,172-finding semantic set while updating identities for the changed dependency profile; all 18 initial Animus findings were corrected before baseline migration, so no new source finding was accepted.
- Cairn lifecycle validation and the proposal, design, and tasks gates passed through the canonical sibling Cairn CLI.
- `nix flake check -L --option secret-key-files ''` passed all bounded checks on `x86_64-linux`; incompatible systems were not evaluated.
- Deterministic fake-provider dogfood passed. No live-provider quality, external billing truth, remote exactly-once effect, tool execution, arbitrary Steel, profile/role activation, cancellation-between-turns, or production-readiness claim is made.
