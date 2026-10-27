## Why

Molten's `adopt-shared-wasm-component-runtime` compatibility preflight on 2026-10-26 found two missing product-neutral execution contracts in published runtime revision `0455e0958e6f8948d0df5f62950be39519d717a0`: no retained fuel-sliced invocation, and no typed distinction between instantiation and invocation failures. Actual start-function and export traps both normalize to `GuestTrapped`. A consumer cannot recover their phase from fuel or diagnostic text.

## What Changes

- Add host-driven resumable execution of synchronous component exports using Wasmtime's fuel-based async yielding. Retain one future, store and cumulative resource budget; dropping the continuation cancels and tears it down.
- Report every yielded poll with its ordinal and configured quantum; report exact Wasmtime remaining fuel on completion. Do not misrepresent configured quanta as exact per-poll instruction consumption.
- Add typed admission, linking, instantiation and invocation phases to synchronous module/component and sliced component outcomes.
- Version the shell contract without admitting the guest Component Model async proposal, WASI or ambient effects.

## Impact

- **Files**: runtime outcome model, shell store/component/module paths, new slicing module, explicit Wasmtime host-async feature, execution regressions and README.
- **Testing**: actual positive/negative components; workspace tests, strict Clippy, formatting, deny-all Octet, native Cairn validation/gates and Nix flake checks.
- **Current consumer**: Molten's generation-fenced scheduler and staged runtime receipts. Its 2026-10-26 compatibility preflight owns the frozen consumer corpus.
- **Maintenance owner**: hardened-runtime maintainers own execution and typed observations; Molten maintains product scheduling, continuation identities, authority and receipts.
- **Durable capability**: a reusable bounded continuation API with repeatable real-component resource, cancellation and phase regressions.
- **Adoption path**: publish an immutable upstream implementation; Molten independently pins and validates its complete parity corpus before any runtime cutover. This prerequisite does not claim consumer adoption, materialization trust, whole-runtime correctness, host isolation or release eligibility.
