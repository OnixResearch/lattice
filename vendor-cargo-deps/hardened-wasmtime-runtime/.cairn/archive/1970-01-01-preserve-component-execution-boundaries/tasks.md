## Phase 1: Mechanism and regressions

- [x] [serial] Preserve the existing executed consumer counterexamples and define the v4 checkpoint/resource contract. r[onix.wasmtime_runtime.instantiation_checkpoint] r[onix.wasmtime_runtime.independent_memory_limits]
- [x] [parallel] Implement explicit positive per-memory and aggregate bounds, migrate all producer callers and add positive/negative actual allocation cases. r[onix.wasmtime_runtime.independent_memory_limits]
- [x] [parallel] Expose typed yielded phases and a single successful instantiation checkpoint before invocation, retaining cumulative limits and ownership. r[onix.wasmtime_runtime.instantiation_checkpoint]
- [x] [serial] Exercise start-work scheduling, no callback before checkpoint, instantiation failure, checkpoint cancellation, fuel/cleanup and no duplicate checkpoint/completion. r[onix.wasmtime_runtime.instantiation_checkpoint]

## Phase 2: Evidence and publication

- [x] [serial] Update public contract/source inventory, run workspace tests, strict Clippy, formatting, deny-all Octet, native Cairn and host Nix checks, and record exact evidence/non-claims. r[onix.wasmtime_runtime.instantiation_checkpoint] r[onix.wasmtime_runtime.independent_memory_limits]
- [x] [serial] Review the completed producer correction and record its immutable-publication handoff and required independent Molten verification without claiming adoption. r[onix.wasmtime_runtime.instantiation_checkpoint] r[onix.wasmtime_runtime.independent_memory_limits]
