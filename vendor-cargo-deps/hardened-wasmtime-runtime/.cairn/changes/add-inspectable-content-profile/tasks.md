## Prerequisites

- [ ] [serial] r[wasmtime.content_profile.identity] Register the local dependency on `extract-hardened-wasmtime-runtime` through Cairn, then verify that foundation evidence passes before implementation.
- [ ] [serial] r[wasmtime.content_profile.identity] Pin the reviewed Qip source, inspect notices, reconcile documentation drift, and record the extraction test baseline.

## Admission and execution

- [ ] [serial] r[wasmtime.content_profile.static_admission] Define the typed Nickel profile and implement pure admission over bounded parsed artifact facts, with valid and forged-metadata fixtures.
- [ ] [serial] r[wasmtime.content_profile.static_admission] Add final-artifact controls for imports, start functions, growth, shared memory, atomics, recursion, indirect calls, malformed sections, and overlapping ranges.
- [ ] [serial] r[wasmtime.content_profile.execution] Connect the profile to existing extraction limits and add pointer, overflow, output, rejection, trap-disposal, fuel, deadline, and reuse controls.
- [ ] [serial] r[wasmtime.content_profile.claims] Separate structural, optional static, and runtime verdicts. Defer loop/output checker adoption unless Site demand and an independent audit justify it.

## Evidence

- [ ] [serial] r[wasmtime.content_profile.identity] Bind artifact, profile, parser/checker, runtime, ABI, limits, and consumer observations in versioned identity records.
- [ ] [serial] r[wasmtime.content_profile.identity] Run real Site consumer integration against a published immutable contract under separate publication authorization.
- [ ] [serial] r[wasmtime.content_profile.execution] Run focused before/after tests, workspace positive/negative tests, Clippy, strict pinned Octet, Nix checks, and Cairn validation and gates without promoting scoped evidence to universal safety.
