# Choregraph effect-authority baseline and blocker

## Goal

Lattice must authorize an exact Choregraph `EffectIntent` before any product effect. Lattice must persist the bound attempt before dispatch and must not dispatch during replay.

## Completion evidence

Completion requires all of these facts:

- production Choregraph transitions emit the exact intents that Lattice authorizes;
- Basalt evaluates the exact request with reviewed policy and caller-supplied UCAN facts;
- Lattice persists one bound attempt before the selected product adapter runs;
- denied and replayed intents produce no product effect;
- file, process, and network adapters retain their existing bounded policies.

An isolated Basalt unit test, an adapter over an external allow value, or a task checkbox is not completion evidence.

## Current Lattice baseline

The current executor dispatches product effects directly from `NodeConfig`:

- `src/engine/executor.rs` routes shell commands to the bounded process adapter;
- `src/engine/executor.rs` routes HTTP requests through `HttpClient`;
- `src/engine/executor.rs` routes file plans through `FileSystem`;
- `src/engine/executor.rs` routes images, agents, and subworkflows through their selected adapters;
- `src/capability/mod.rs` opens operator-configured roots and performs capability-relative reads;
- `src/file_node/local.rs` performs capability-relative file operations and retains the existing no-follow behavior.

Scheduler occurrences, webhook claims, and orchestration dispatches persist selected admission identities before their outer effects. Ordinary node effects have no per-effect durable Choregraph intent or attempt record.

The Choregraph replay adapter returns state only and discards dispatch authority. Existing replay tests pass.

## Missing production intent path

The published Choregraph revision is `81b260da2256147ec82cb5dd44c0a0c1dcb7500d`.

The current Lattice projection cannot produce a product effect intent:

1. `src/workflow_format/choregraph.rs` lowers `lattice-workflow-v1`, which has node operations but no effect declarations.
2. `load_workflow_nickel_file` discards the admitted `LoweringOutput` after admission.
3. `choregraph-seaglass` constructs a machine with no effects.
4. `choregraph-seaglass::step_machine` accepts only an empty intent set and returns `AuthorityBoundaryViolation` for a non-empty set.
5. `src/engine/executor.rs` dispatches resolved product plans without a Choregraph transition or `EffectIntent` input.

As a result, adding Basalt before the executor without a production intent producer would authorize a Lattice-synthesized request. It would not authorize an intent emitted by Choregraph.

## Basalt and UCAN source baseline

The reviewed Basalt authority core is at `f10e3fa7e821cb69665caaf1c26624128a5cf0fe`. Basalt pins UCAN revision `2aad993027d48ff148028c537cdaf91f6e5285ca` through `ssh://git@github.com/OnixResearch/ucan.git`.

Basalt declares that anonymous builds are unsupported. Its sandbox-safe Nix rail defers Cargo-dependent checks. An anonymous HTTPS probe requested credentials for both Basalt and UCAN. A direct Lattice dependency would therefore make release builds depend on private source authority.

An exact reviewed vendor snapshot can remove the fetch dependency. Vendoring does not solve the missing production intent path.

## Mechanism portfolio

| Family | Mechanism | State | Exact gap |
|---|---|---|---|
| Direct Basalt dependency | Link Basalt and retain its authenticated UCAN dependency. | Falsified | Anonymous and sandboxed release builds cannot reproduce the dependency closure. |
| Reviewed Basalt and UCAN vendor | Reproduce exact source snapshots and link the real Basalt authority core. | Blocked | Lattice still has no production Choregraph intent to authorize. |
| External decision adapter | Accept a caller-provided Basalt allow decision without linking Basalt. | Falsified | The adapter cannot prove that the current exact request passed the reviewed Basalt core. |
| Request-scoped Choregraph machine | Admit exact resolved effect declarations through `choregraph-core`, emit intents on `Ready -> Start`, then call Basalt. | User decision required | This changes the machine identity model and needs an accepted contract for dynamic request and payload references. |

The portfolio used four mechanism families, targeted source review, one advisory model review, and deterministic repository checks. The terminal state is blocked.

## Adversarial audit

A request-scoped Choregraph machine can emit exact runtime intents with the current published core. It also changes machine identity for each resolved request. The current accepted Lattice adoption binds persisted state and replay to a workflow machine identity.

Using request-scoped machines without a reviewed identity split can create two incompatible meanings for "machine identity." Treating a static template reference as the exact runtime payload can also overclaim payload binding.

The next accepted design must select one of these models:

- one immutable workflow machine with a reviewed parameterized effect-reference contract; or
- a workflow machine identity plus a separate request-scoped effect-machine identity.

The current change does not select either model.

## Baseline checks

The following pre-change checks passed:

```text
nix develop -c cargo test --lib capability::
nix develop -c cargo test --test test_executor --test test_file_nodes --test test_local_process --test test_http_node --test test_replay --test test_post_effect_crash_profile --test test_choregraph_migration
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal authorize-choregraph-effect-intents --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design authorize-choregraph-effect-intents --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks authorize-choregraph-effect-intents --root .
```

The first combined Cargo invocation applied the `capability::` filter to all named test binaries. A second invocation ran the integration tests without that filter.

## Blocker

Implementation cannot continue honestly until an accepted design defines the production Choregraph intent producer and its machine-identity relationship.

No Basalt, UCAN, `cap-root`, executor, store, or policy code changed. The change remains active. The Basalt source limitation remains a release blocker after the intent-model decision.
