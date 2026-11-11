# Proposal: Qualify a pure interaction-net worker profile

Tracking: [Lattice #14](https://github.com/OnixResearch/lattice/issues/14). Consumer: [Mneme #5](https://github.com/OnixResearch/mneme/issues/5).

## Why

Mneme may evaluate a bounded pure computation through an interaction-net engine. Lattice already admits exact operation packs, launches reviewed executables through Bounded Exec, and publishes versioned operation data. A computation profile can reuse those paths without introducing another workflow engine or moving effects into graph reduction.

The missing work is a qualified contract for the selected worker: immutable program and input identities, a closed extrinsic vocabulary, finite resource limits, checked results, and honest interruption/recovery semantics. The first shared profile is deliberately small and sequential: two wrapping signed 64-bit integer inputs and one signed 64-bit result, computed by a finite acyclic program of `Input`, `Const`, `Add`, `Sub`, and `Mul`. A net normal form, process exit code, or model-written result does not by itself establish a valid Mneme projection or completed implementation work.

This is a proposed, opt-in profile. No worker execution or production qualification is claimed by this change.

## What Changes

- Package one pinned Rust worker/adapter using the existing `bounded_process` operation-pack class and typed operation route. r[lattice.pure_inet_profile.route]
- Bind every invocation to exact serialized program, input, schema, producer, executable, extrinsic, profile, limit, output-contract, and enclosing attempt identities. r[lattice.pure_inet_profile.identity]
- Admit only the producer's explicitly qualified pure extrinsics and reject ambient I/O, mutable host handles, native callbacks, and undeclared effects. r[lattice.pure_inet_profile.purity]
- Enforce process and evaluator limits and distinguish configured reservations from observed execution counters. r[lattice.pure_inet_profile.bounds]
- Validate a bounded typed evaluation envelope and retain usable results or diagnostic artifacts through existing operation-data ownership. r[lattice.pure_inet_profile.results]
- Preserve authorization, durable dispatch, cancellation, unknown outcomes, and explicit fresh recomputation from immutable inputs. r[lattice.pure_inet_profile.recovery]
- Keep Mneme projection admission and Kiln/Lattice campaign ownership outside the worker. r[lattice.pure_inet_profile.ownership]
- Qualify the real selected executable with independent result checks and failure fixtures before enabling the profile. r[lattice.pure_inet_profile.verification]

## Impact

- **Profile and fixtures:** add one optional content-addressed pack, a closed worker contract, and exact producer/runtime compatibility evidence.
- **Thin adapter:** normalize the worker's versioned protocol into existing typed invocation/output and artifact boundaries. Reuse the current process runner; do not add a runtime kind or `NodeConfig` variant.
- **Consumers:** Mneme supplies an immutable authorized snapshot and validates the resulting projection. The existing campaign owner retains claims, budgets across attempts, acceptance, and publication.
- **Dependencies:** [Noble #37](https://github.com/OnixResearch/noble-lang/issues/37) owns the future exact net-byte, rule, and readback contract. Qualify that producer's serialized pure-program and bounded-evaluation protocol before advertising a runnable profile. No producer worker CLI is assumed to exist, and this change specifies no invented CLI flags.
- **Mneme consumer:** coordinate with [Mneme #5](https://github.com/OnixResearch/mneme/issues/5) and its `add-optional-inet-compute` change; its input/output semantics and independent reference workload remain the common consumer contract.

## Existing Work and Scope Boundaries

- [Lattice #6](https://github.com/OnixResearch/lattice/issues/6) owns one admitted coding attempt; this profile is a pure computation inside an admitted operation.
- [Lattice #7](https://github.com/OnixResearch/lattice/issues/7) owns the Pi/OMP candidate-producing worker and its workspace boundary. This profile neither edits repositories nor creates code candidates.
- [Lattice #8](https://github.com/OnixResearch/lattice/issues/8) owns command-agent cancellation propagation. Reuse existing process cancellation here; do not duplicate that fix or route a net evaluator through an LLM adapter.
- [Lattice #12](https://github.com/OnixResearch/lattice/issues/12) owns portable workload checkpoints. The first net profile supports fresh recomputation from immutable inputs, not continuation of a serialized heap, pointer graph, or VM snapshot.
- [Lattice #13](https://github.com/OnixResearch/lattice/issues/13) owns worker usage accounting. Record net counters with their own units/provenance and never label a reservation or missing measurement as observed usage.
- The active `add-wasm-component-operation-pack-runtime` change remains separate. This proposal adds no Wasm backend, shared runtime, or fallback between execution classes.
- An external IVM at revision prefix `cf0c3258` may be a separately pinned comparison cohort only. It is not the admitted Noble producer or a fallback; resolve its full immutable identity before any comparison evidence is accepted.

## Out of Scope

- A new interaction-net engine, language, scheduler, global claim service, task database, or merge path.
- Replacing Choregraph readiness, Datalog semantics, Prolog search, Mneme's canonical storage, or its evidence/retention authority.
- Net-host filesystem, process, network, clock, random, credential, device, or dynamic-plugin access.
- Multicore or GPU/device reduction; widening the sequential first profile requires a separate change.
- Live mutation of canonical data, effect-intent dispatch, arbitrary net checkpoint/resume, or automatic retry of unknown work.
- Default adoption, automatic runtime acquisition, universal confluence claims, performance guarantees, or production-readiness claims.

## Affected Specs

- `pure-inet-worker-profile`: exact admission, pure vocabulary, bounded execution, results, recovery, ownership, and qualification.

## Reviewed Source

The baseline is Lattice [`abf1614a19c87a496712c8078fe10af5a7938966`](https://github.com/OnixResearch/lattice/tree/abf1614a19c87a496712c8078fe10af5a7938966).

- [Operation-pack execution and result publication](https://github.com/OnixResearch/lattice/blob/abf1614a19c87a496712c8078fe10af5a7938966/src/operation_pack/shell/execution.rs).
- [Exact pack/runtime/lock contracts](https://github.com/OnixResearch/lattice/blob/abf1614a19c87a496712c8078fe10af5a7938966/src/operation_pack/model.rs).
- [Existing bounded process and clear-environment routes](https://github.com/OnixResearch/lattice/blob/abf1614a19c87a496712c8078fe10af5a7938966/src/process/shell.rs).
- [Authorized product-process dispatch](https://github.com/OnixResearch/lattice/blob/abf1614a19c87a496712c8078fe10af5a7938966/src/engine/effect/runtime.rs).
- [Operation-data identity, retention, and publication outcomes](https://github.com/OnixResearch/lattice/blob/abf1614a19c87a496712c8078fe10af5a7938966/src/operation_data/model.rs).

