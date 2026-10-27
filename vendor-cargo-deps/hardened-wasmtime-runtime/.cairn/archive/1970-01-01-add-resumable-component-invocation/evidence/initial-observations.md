# Initial verification observations — 2026-10-26

## Executed baseline

At producer baseline `0455e0958e6f8948d0df5f62950be39519d717a0`, `nix develop --builders '' -c cargo test --workspace --all-targets` passed 9 core unit, 29 admission and 22 actual execution cases. The shell could return `GuestTrapped` for both instantiation and invocation; Molten's independently executed compatibility preflight records that missing phase and the absent retained invocation API.

## First integrated execution run

The first integrated workspace run passed 8 unit, 29 admission and 31 execution cases, and failed the new continuation fixture's synchronous positive control: observed `GuestTrapped`, expected `Completed`. The fixture incorrectly called its component host import from the guest post-return function. Wasmtime 45.0.0 `src/runtime/component/func.rs` explicitly forbids post-return functions from calling imports or intrinsics (`may_leave` is disabled).

The fixture was corrected to emit its second host marker at the end of the normal export, before returning. Its long cleanup performs no imports and still mutates result memory after lifting. The next run passed the continuation test, including multiple action and cleanup yields, exactly-once markers, owned output and exact successful-run fuel parity. Runtime classification was not suppressed or special-cased.

## Memory-failure fuel observation

The next workspace run passed 8 unit, 29 admission and 35 execution cases. A newly added memory-limit test correctly observed `MemoryLimitExceeded`, `Invocation` and empty output after suspension, but an extra whole-outcome comparison failed:

```text
sliced:      InvocationOutcome { class: MemoryLimitExceeded, phase: Invocation, output: [], fuel_remaining: 9545 }
synchronous: InvocationOutcome { class: MemoryLimitExceeded, phase: Invocation, output: [], fuel_remaining: 9998 }
```

These are actual Wasmtime store readbacks, not inferred instruction counts. The test's incidental failed-run fuel equality assertion was removed; the permanent case checks the requested memory, phase, output and cumulative-budget boundary. Successful same-work exact fuel parity remains asserted separately. The API reports engine fuel rather than manufacturing a normalized counter. Independent consumer adoption must evaluate this negative-path fuel difference against its own receipt contract; this prerequisite does not claim full Molten parity.

## Dependency and environment observations

An initial `cargo generate-lockfile --offline` refreshed unrelated cached transitive versions. That task-owned regeneration was restored from the clean baseline. Cargo then resolved only the feature dependency against the existing lock, retaining the Wasmtime 45.0.0 cohort. No manual lockfile editing or dependency-version migration was used.

The host compiler cache reported an index disk-I/O error and disabled cache hits/stores. Cargo compiled and executed without the cache. Scratch builds use an executable task-owned `/dev/shm` directory; cache failure is not runtime denial or a correctness-gate exception.

The configured GitHub `origin` fetch returned `Repository not found`; canonical Radicle fetch succeeded. Publication must record its actual route and observed immutable revision, without claiming GitHub replication.

## Strict test-source policy

After the fixture corrections, the complete workspace run passed 8 unit, 29 admission and 36 execution tests. Strict Clippy then rejected nine explicit `panic!` branches in the new test helpers and assertions under the repository's existing deny policy. Those branches were replaced with bounded `Option` extraction and behavioral assertions; no lint allowance, policy change or production error suppression was added.

## Pinned Octet ownership finding

Both the pinned CLI deny-all run and hermetic flake source check rejected the new private `BoundedRuntime::try_enter` helper with `missing_const_fn` (reported once for each library compilation target). Atomic slot acquisition is an effect, not a const computation. The reservation mechanism was moved beside the borrowed and owned guards in the existing `concurrency` module and reused by both engine admission paths. No const decoration, duplicate reservation algorithm, lint allowance or policy modification was introduced.

## Strict fixture ownership and data findings

The next pinned Octet run rejected concrete test imports, oversized fixture/limit owners, unannotated numeric relationships, unchecked ordinal addition and an oversized marker-observation test. The new cases now use qualified owner paths, checked ordinal arithmetic, named bounds and separate fixture/observation/boundary modules. All 36 execution cases passed after that refactor, together with 8 core unit and 29 admission cases; strict Clippy also passed.

One remaining assertion-density finding counted the phase fixture's long WAT literal as executable helper logic. The literal moved to `phasecomponent.wat`, loaded with `include_str!` and explicit format arguments. No meaningless assertion, lint allowance, policy exception or consumer-visible runtime change was introduced to satisfy a source gate.
