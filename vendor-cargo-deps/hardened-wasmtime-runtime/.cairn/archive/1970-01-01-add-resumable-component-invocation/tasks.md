# Tasks: Add resumable component invocation

## Contract and baseline

- [x] [serial] Review Molten's executable compatibility handoff and record producer baseline: 9 unit, 29 admission and 22 execution tests passed. r[onix.wasmtime_runtime.resumable_execution]
- [x] [serial] Define and gate the native proposal, design and positive/negative acceptance scenarios before integrating source. r[onix.wasmtime_runtime.failure_phase]

## Implementation

- [x] [parallel] Carry typed failure phases through real module and component execution, with start-trap and export-trap regressions. r[onix.wasmtime_runtime.failure_phase]
- [x] [parallel] Implement a retained host-polled future with explicit quanta, terminal fuel, cancellation and no guest-async admission. r[onix.wasmtime_runtime.resumable_execution]
- [x] [parallel] Add real-component continuation, cumulative-limit, cleanup, cancellation, concurrency and deadline regressions. r[onix.wasmtime_runtime.slice_limits]

## Validation and handoff

- [x] [serial] Exercise the public slicing API through an actual component and retain observed phase, output and fuel evidence. r[onix.wasmtime_runtime.resumable_execution]
- [x] [serial] Run workspace tests, formatting, strict Clippy, deny-all Octet, strict Cairn validation/gates and relevant Nix checks, preserving failures and bounded claims. r[onix.wasmtime_runtime.slice_limits]
- [x] [serial] Update current documentation, review and synchronize accepted requirements, archive only completed work, and publish an immutable upstream revision for independent Molten adoption. r[onix.wasmtime_runtime.failure_phase]
