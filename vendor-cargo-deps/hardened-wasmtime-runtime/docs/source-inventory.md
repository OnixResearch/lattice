# Aspen source inventory for the hardened Wasmtime extraction

Recorded 2026-09-09 for `extract-hardened-wasmtime-runtime` task
`r[onix.wasmtime_runtime.source_inventory]`. Everything here was inspected
in the local Aspen checkout; paths are relative to the Aspen repository
root and name the exact reviewed sources.

## Reviewed Aspen source modules

| Module | Path | Lines | Role |
| --- | --- | ---: | --- |
| Component runtime profile contracts | `docs/wasm-component-runtime/contracts.ncl` | typed Nickel | `molten.wasm-component-profile.v1`: feature policy, resource limits, toolchain pin, WIT identity, non-claims |
| Generated runtime profile | `docs/wasm-component-runtime/generated/profile.json` | exported JSON | the checked profile Rust consumes (`profile_id = molten.wasm.component.v1`) |
| Profile template | `docs/wasm-component-runtime/profile.ncl`, `profile-template.ncl` | Nickel | deterministic profile rendering |
| Negative profile fixtures | `docs/wasm-component-runtime/fixtures/negative/` | 4 Nickel | ambient WASI, stale cohort, nondeterministic growth, incomplete cohort |
| Hyperlight embedded runtime | `.pi/worktrees/aspen-wasm-plugin` branch `aspen-wasm-plugin`, `crates/aspen-wasm-plugin/src/` | 4,287 | product-specific hyperlight micro-VM path (`wasmtime =36.0.3` embedded); out of scope for the portable-component extraction, inventoried for its precompile and manifest-limit patterns |

## Wasmtime revision and toolchain

| Tool | Version (per generated profile) |
| --- | --- |
| wasmtime | 45.0.0 |
| wasmtime-wasi | 45.0.0 |
| wasmparser | 0.240.0 |
| wit-bindgen | 0.46.0 |
| wasm-tools | 1.240.0 |
| WASI packages | 0.2.6 |

The extraction pins `wasmtime =45.0.0` with an explicit feature set; the
hyperlight path's `=36.0.3` embedded runtime stays product-owned and is
not extracted.

## Feature settings (complete explicit policy, `molten.wasm.component.v1`)

Enabled: `bulk_memory`, `component_model`, `multi_value`, `reference_types`,
`simd`, plus determinism controls `nan_canonicalization`,
`relaxed_simd_deterministic`, `fuel_interruption`,
`memory_growth = fixed`, `table_growth = fixed`,
`host_inputs = recorded-effects-only`.

Disabled (denied if enabled): `component_async`, `custom_page_sizes`,
`exceptions`, `extended_const`, `function_references`, `gc`, `memory64`,
`multi_memory`, `relaxed_simd`, `tail_call`, `threads`,
`wide_arithmetic`. `allowed_wasi_interfaces` is empty and
`allowed_imports` is empty: WASI and ambient imports are denied.

## Resource limits (per admitted profile)

fuel 100000; max component bytes 4 MiB; max WIT bytes 64 KiB; memory
16 MiB; tables 4 × 1024 elements; instances 16; memories 4; stack
512 KiB; host-call bytes 64 KiB; result bytes 64 KiB; concurrency 1;
imports ≤ 128; exports ≤ 128.

## World and artifact identity

WIT package `molten:component-runtime@1.0.0`, world `actor`, WIT source
identity `blake3:83be349bb27975ada30dbe60817c5404df7862babdf02ae229b399886e76d5e8`.
`runtime_strategy = portable-component` (Cranelift/Winch strategies come
from the existing `Strategy` admission in `crates/hardened-wasmtime-runtime`).

## Fixtures

Aspen's four negative profile fixtures are frozen into this repository
under `fixtures/aspen/` (task
`r[onix.wasmtime_runtime.fixture_compatibility]`), with a manifest naming
the compile, invocation, limit, and trap fixture classes the extraction
must add in Phase 3.

## Non-claims

This inventory records reviewed sources and facts. It does not prove the
extraction's correctness, sandboxing strength, behavioral equivalence with
the Aspen runtime, or release eligibility.

## Executable boundary correction (2026-10-26)

The native `complete-runtime-extraction-boundary` change corrects the published
`8d88e00` extraction. Its historical archive and the inventory above are not fresh
execution evidence. The correction retains the existing MIT license, frozen
Aspen profile fixtures and exact Wasmtime `45.0.0` dependency, and adds explicit
`gc-null` support for admitted reference types without admitting guest GC.

New repository-owned shell code enforces engine policy, exact artifact bytes
and names, aggregate store limits, consumer-installed imports and engine-bound
epoch budgets. Component results become bounded owned bytes before Wasmtime's
automatic post-return cleanup. The executable regression entrypoints are
`crates/hardened-wasmtime-runtime/tests/admission.rs` and
`crates/hardened-wasmtime-shell/tests/execution.rs`; source files alone do not
establish that a run passed.

The hardened-runtime maintainers own this mechanism and its cohort regressions.
Molten owns its actor pilot, policy, materialization trust and canonical runtime
decisions; Site owns its independent renderer world, preview admission and output
sink. Neither consumer pilot is production adoption, effect authorization,
source-trust evidence, sandboxing proof or release eligibility. Exact observed
commands and consumer source pins belong in the correction's evidence, not in
the historical extraction receipt.

## Resumable execution prerequisite (2026-10-26)

Molten's `adopt-shared-wasm-component-runtime` compatibility preflight at product
baseline `d2f21b1b4c90163de6034d37b0cbd03e820adced` identified missing resumable
fuel execution and failure-phase observations in the published synchronous shell.
The `add-resumable-component-invocation` change implements those product-neutral
contracts in `hwr.shell.v3`, retaining the same exact Wasmtime 45.0.0 version.
Only host async support is enabled; guest component async remains denied.

The shell owns one retained future/store, cumulative limits, cancellation and
typed phases. Molten retains scheduler selection, continuation identity,
generation fences, canonical receipts and production adoption. Exact terminal
fuel is the engine's readback, not an independent instruction count; the recorded
memory-trap difference across synchronous and sliced execution must remain
visible to consumer parity review. No new product dependency or policy is
introduced here, and no sibling path becomes a durable runtime dependency.

## Instantiation and independent-memory correction (2026-10-26)

Molten's executed frozen corpus at `ce5873f2` found that v3 exposes start-function
fuel suspensions to the callback scheduler and applies one memory bound to two
different meanings. The v4 producer adds a typed yield phase and one successful
instantiation checkpoint before export execution. It retains the same future,
store, cumulative fuel/epoch/transfer/memory limits and cancellation ownership.

Resource admission now requires independent positive `max_memory_pages` and
`max_total_memory_pages`. Wasmtime's per-memory limiter and the invocation-wide
remaining-byte counter enforce them separately. The frozen profile reader maps
its existing document bound to both, retaining that reader's cumulative policy;
consumers with different policy must supply their own reviewed exact mapping.

The hardened-runtime maintainers own this mechanism and actual-component
regressions. Molten owns its unchanged profiles, initialization-draining adapter,
callback ledger, canonical receipts, authority and eventual adoption. No new
product effect ABI, host clock, local-engine fallback or consumer cutover is
introduced. Independent consumer proof must cite an immutable published revision.

