# Enforce Lattice core and shell crate boundaries

## Why

Lattice already separates several domains into `core.rs` and `shell.rs` modules. The scheduler, replay model, orchestration reducer, stores, network adapters, process adapters, and operator surfaces still compile inside one package with the same dependency authority. Module layout therefore documents the boundary, but Cargo does not enforce dependency direction.

Workflow metadata also reads the wall clock inside domain constructors. This makes otherwise deterministic workflow changes depend on ambient time.

## What Changes

- Add a host-capability-free `lattice-core` crate for Lattice-owned workflow, scheduler-admission, replay, and orchestration decisions.
- Keep redb, clocks, environment access, HTTP, processes, credentials, Nickel evaluation, CLI behavior, and TUI behavior in the existing Lattice shell package.
- Replace ambient time reads in core constructors and transitions with explicit timestamp facts. Preserve shell-facing compatibility through adapters.
- Add explicit Octet architecture topology for core, shell, adapter, and forbidden dependency directions.
- Add positive and negative fixtures for the crate boundary, deterministic transitions, compatibility, and architecture diagnostics.

## Capability Delta

The `repository-validation` spec domain gains requirements for a compiler-enforced FCIS boundary, explicit temporal facts, compatibility adapters, and architecture enforcement.

## Impact

- **Core source**: `src/engine/workflow.rs`, `src/scheduler.rs`, `src/scheduler/admission.rs`, `src/scheduler/admission/model.rs`, `src/replay.rs`, and selected parts of `src/orchestration/core.rs`.
- **Shell source**: scheduler and orchestration shells, stores, headless execution, credentials, webhooks, CLI, and TUI.
- **Dependencies**: the new core crate can depend only on reviewed deterministic data and graph dependencies. It cannot depend on Tokio, redb, HTTP, CLI, TUI, process, filesystem, environment, or clock providers.
- **Public behavior**: accepted workflow files, persisted records, replay bundles, schedule occurrence identities, commands, and output remain compatible.
- **Active work**: extraction must reconcile ownership with active orchestration and scheduler pilots before moving shared source.

## Non-Goals

- Replacing Choregraph or duplicating Choregraph-owned graph and machine decisions.
- Rewriting persistence, provider, process, HTTP, credential, or TUI implementations.
- Requiring a flag-day `no_std` conversion for every Lattice domain.
- Changing scheduler retry, replay, workflow serialization, or orchestration authority semantics.
- Claiming semantic correctness, exactly-once execution, provider correctness, or release readiness from source-shape checks.

## Verification Expectations

Implementation must record focused baseline results before moves. It must then run synchronous core tests, shell compatibility tests, compile and source-policy negatives, Octet, Cairn validation and gates, `scripts/check.rs`, and the relevant Nix checks.
