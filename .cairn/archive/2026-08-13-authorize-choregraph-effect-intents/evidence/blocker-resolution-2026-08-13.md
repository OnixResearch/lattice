# Choregraph effect-authority blocker resolution

## Goal

Authorize only an exact Choregraph concrete intent before a Lattice product effect. Persist its attempt before dispatch and deny replay dispatch.

## Completion evidence

Completion requires these observed facts:

- one immutable Choregraph workflow machine emits the concrete intent;
- runtime request and payload references do not change machine identity;
- Basalt evaluates the exact request with caller-supplied verified facts;
- a durable attempt commit occurs before the selected product adapter;
- denial, replay, duplicate, stale, and persistence-error paths execute no product effect;
- file operations use configured root identifiers, `cap-root`, and `cap-std`;
- process and network operations keep their existing bounded adapters.

A Lattice-synthesized intent, an external allow boolean, a test count, or completed task markers are false completion.

## Search budget

The audit used four serial mechanism lenses. The lenses are correlated because one agent performed them.

The audit used local Choregraph, Basalt, Cap Root, Lattice, and Molten source. It used no model worker or unreviewed network source.

## Mechanism registry

| Family | Mechanism | Evidence | State |
|---|---|---|---|
| Static version-1 workflow machine | Infer an effect request from `NodeConfig` after a normal transition. | Pinned revision `81b260da2256147ec82cb5dd44c0a0c1dcb7500d` emits no Lattice effect intent. | Falsified |
| Request-scoped effect machine | Build one machine for each resolved request. | This changes persisted machine meaning and duplicates the workflow machine identity. | Falsified |
| Version-2 workflow machine | Keep static authority fields in one machine and supply exact runtime references through declared slots. | Choregraph revision `85ea5e35caaab23260fcb27bc7c5d995dec546c4` provides `MachineSpecV2`, `step_concrete`, and `step_concrete_machine`. | Validated |
| External allow adapter | Accept an allow value without running the reviewed Basalt core. | The value does not establish exact Basalt evaluation. | Falsified |
| Vendored Basalt boundary | Import one exact Basalt source with its reviewed UCAN snapshot. | Basalt revision `bb8fb5f4a763f532df96b6e65407163ac668834e` uses vendored UCAN revision `277ab768eb532ac0c715a3bbff2a7dc37a702f11`. | Validated |
| Ambient file roots | Let graph data supply host paths. | This gives graph data root-selection authority. | Falsified |
| Configured root registry | Let graph data name an operator-configured root and admit only a relative path. | Cap Root revision `545e77e5a49e4e1d34037aa9a82d924944ac0a1b` supplies pure lexical admission. | Validated |
| Required Molten runtime dependency | Load current collaboration state from a sibling worktree. | The Molten scope change remains active and unpublished. | Blocked |
| Explicit scope-snapshot port | Accept a versioned application value from an external currentness provider and deny when absent. | This keeps production effects disabled without current scope facts and permits bounded fixture coverage. | Validated |

## Selected model

The selected model keeps one immutable version-2 workflow machine. Each effectful node has one declared `Ready -> Start` concrete effect.

The event supplies an exact request reference and a safe payload reference. The safe payload includes credential references but excludes credential values.

The shell obtains current authority and optional scope facts. The pure core validates all exact bindings before Basalt evaluation.

The attempt store reserves one identity before dispatch. A duplicate or persistence error denies the effect.

## Adversarial audit

The model rejects these routes:

- direct dispatch from a `NodeConfig` without a returned concrete intent;
- reuse of a prior Basalt allowance;
- use of `RunOwner` as scope or membership authority;
- graph-selected host paths;
- secret values in canonical payload references;
- a test scope snapshot as a production Molten claim;
- an in-memory attempt record as durable production evidence.

## Baseline validation

These pre-change checks passed on 2026-08-13:

```text
nix develop -c cargo test --lib capability::
nix develop -c cargo test --test test_executor --test test_file_nodes --test test_local_process --test test_http_node --test test_replay --test test_post_effect_crash_profile --test test_choregraph_migration
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal authorize-choregraph-effect-intents --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design authorize-choregraph-effect-intents --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks authorize-choregraph-effect-intents --root .
```

## Implementation validation

The maintained integration path passed these checks on 2026-08-13:

```text
cargo test --test test_choregraph_effect_authority --no-fail-fast
cargo test --test test_workflow_format --no-fail-fast
cargo test --test test_choregraph_adoption --test test_execution_graph_source_admission --no-fail-fast
cargo clippy --package lattice --all-targets --no-deps -- -D warnings
scripts/check.rs
cargo test --workspace --all-targets --no-fail-fast
```

The authority corpus passed 16 tests. The workflow-format suite passed 24 tests, including the concrete V2 handoff. The full workspace check passed after restoring two immutable Trellis HDL golden fixtures omitted from the normalized Basalt vendor tree.

The maintained `scripts/check.rs` run passed after refreshing only the exact historical byte hashes for changed `Cargo.toml`, `README.md`, and `src/store/mod.rs` evidence rows.

The final strict Octet gate passed through `nix develop --builders '' -c octet-precommit-check`. The top-level profile reported `warning-only`, 1,093 raw warning occurrences, 864 accepted/current stable identities, zero new findings, and zero stale findings. Its baseline identity was `b3:4c3fd764a7451dc3b237e839782982480a6dda877215f543ecbc510f3f6005ca`. The separate `lattice-core` profile reported 24 accepted/current stable identities, zero new findings, and zero stale findings.

The Octet baseline migration guard also proved exact semantic equality before installation. The top-level migration reported `old=864 candidate=864 matched=864 unmatched_old=0 unmatched_candidate=0`. The `lattice-core` migration reported `old=24 candidate=24 matched=24 unmatched_old=0 unmatched_candidate=0`. No authority or owner-path finding was admitted as new debt.

The final `nix flake check --builders '' -L --option secret-key-files ''` passed all 16 checks. The Nix-only repair made Crane's vendor tree self-contained for Choregraph's reviewed Kamacite codec. It also included the checked-in Lattice policy JSON, Basalt policy JSON, Basalt source receipt, and normalization note in the filtered source. The focused Nix Clippy, nextest, package, Octet, source-identity, traceability-policy, and format checks all passed.

Final Cairn validation and the proposal, design, and task gates passed after these results. The maintained `scripts/check.rs` also passed after the final source and baseline repairs.

## Non-claims

This resolution does not prove human identity, membership storage, consent policy, key truth, revocation freshness, replay-store durability, UCAN cryptography, operating-system confinement, external effect success, exactly-once behavior, or whole-workflow correctness.
