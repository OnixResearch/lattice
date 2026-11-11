# Design: Pure interaction-net computation on the existing process route

## Context

The current operation-pack route admits exact pack contents and locks, selects a declared executable and launch label, validates the typed operation plan, invokes the existing operation shell, and publishes versioned output. The process layer already provides positive deadlines, separate bounded streams, cancellation, and owned process-group teardown. Operation-pack runtimes use a clear environment.

These are appropriate host boundaries for a pure interaction-net worker. They are not proof of sandbox completeness or of the worker's semantics. This design qualifies one reviewed executable and one closed computation profile; it does not add a new execution engine to Lattice.

## Success Contract

An explicitly selected, exact local pack evaluates one immutable program/input pair through the existing admitted process route. The host validates the typed outcome and exact identities, retains bounded evidence, and exposes a derived result only when evaluation and output admission both succeed. All failure and uncertainty states remain distinguishable. Existing workflows are unchanged when the profile is absent.

## Decisions

### 1. Use the existing operation-pack runtime class

Use `bounded_process`, its content-addressed executable artifact, closed launch label, declared platform/capabilities, and existing operation input/output contracts. A small Rust adapter may translate the producer's qualified serialization into that protocol. The first reducer cohort is sequential; multicore or GPU/device reduction requires a separate profile change and qualification. It must not introduce an `inet` runtime kind, a new workflow node, an internal reducer, or another scheduler.

Missing worker artifacts, unsupported producer schemas, unavailable bounds, or an unqualified target produce an admission refusal. Execution never builds, downloads, updates, selects another evaluator, or falls back to an ambient shell command. A developer-selected reference evaluator is a separate fixture run, not a runtime fallback.

[Noble #37](https://github.com/OnixResearch/noble-lang/issues/37) owns the future producer's exact net-byte serialization, rules, and readback contract. This profile consumes that contract after qualification; it does not define a competing net format or presume a producer CLI. The consumer is [Mneme #5](https://github.com/OnixResearch/mneme/issues/5), with change `add-optional-inet-compute`. An external IVM at revision prefix `cf0c3258` is comparison-only and requires its full revision and measured artifact identities before use in evidence.

### 2. Bind the complete computation

The closed invocation envelope binds:

- program bytes and schema identity;
- immutable input/snapshot bytes, input schema, and any exact resolved data references;
- producer revision and worker executable/runtime identities;
- compiler or lowering identity if the selected artifact was compiled;
- pure extrinsic vocabulary and versioned operation semantics;
- profile, effective limits, platform, numeric behavior, and output-contract identity;
- enclosing workflow/run/invocation and attempt/fence identities when supplied by the current owner.

Locators locate already admitted bytes; they are not identities or authority. The host remeasures selected bytes through existing pack/data access before dispatch and checks the result's binding before admitting output. A changed source, schema, ruleset, extrinsic profile, executable, or contract requires a new invocation/evidence binding. No cache is introduced in this slice; any later cache must use these complete bindings.

Canonical serialized identities must not include process pointers, allocator addresses, thread IDs, or physical node allocation order. Output comparison uses the producer's declared semantic canonicalization, not raw in-memory net layout.

### 3. Keep net operations pure and closed

The first profile accepts exactly two signed 64-bit inputs and one signed 64-bit output. Its program is finite and acyclic, with only `Input(0)`, `Input(1)`, `Const(i64)`, `Add`, `Sub`, and `Mul`; shared references are permitted only under the producer's qualified finite representation. Addition, subtraction, and multiplication use wrapping two's-complement `i64` semantics, equivalent to modulo-2^64 arithmetic followed by signed interpretation. There is no division, floating point, recursion, arbitrary net loading, or user-defined rule/host callback in this first profile.

When the shared versioned artifact protocol uses JSON, every `i64` constant, input, and result is an exact canonical decimal string: `0` or a signed nonzero decimal without a leading plus, leading zeroes, or negative zero, within the signed 64-bit range. JSON number values are rejected at this boundary, including values that happen to fit exactly. A separately versioned exact typed encoding may replace this representation only through a coordinated producer/consumer contract change. Host code must not parse through a floating-point number on the way to `i64`.

The first profile exposes only the producer-qualified pure arithmetic implementation needed by this closed vocabulary. Program/net identity, rule revision, runtime identity, and input/output digests remain explicit even for this small profile. An unknown symbol, cycle, extra input/output, out-of-range value, extra callback, runtime registration request, unsupported operation, or undeclared extension fails before evaluation.

Do not expose filesystem, subprocess, network, environment, credentials, clock, randomness, device, or dynamically loaded native callbacks to the net. Pass authorized immutable data as input. Output is data; it cannot create another operation, lease, approval, or effect.

The executable itself is a reviewed native dependency and can perform its bounded host protocol I/O. A clear environment and allowlisted net operations do not establish OS sandboxing. Reuse a separately qualified isolation provider if required by the admitted trust policy; otherwise reject the unsupported policy rather than claim an isolation guarantee.

### 4. Bound the process and evaluator separately

Reuse process admission for stdin/stdout/stderr limits, finite wall deadline, cancellation, output-truncation rejection, and teardown. The producer profile must additionally enforce its declared reduction/work, live-node or arena, allocation/memory, queue, and output construction ceilings. Never advertise a bound that the selected executable cannot enforce.

Host-measured elapsed time and process termination are separate from producer-reported reductions or peak allocations. Retain units, scope, observation provenance, and unavailable/incomplete measurements. Profile ceilings and budget reservations are not measurements; this change does not replace the ledger or add model-token accounting.

### 5. Separate process, evaluation, and publication outcomes

Normalize one bounded, closed typed response. Keep process disposition, evaluator disposition, result admission, and durable artifact publication as separate facts.

| Evaluation observation | Consumer treatment |
| --- | --- |
| Complete and valid output for the admitted binding | Eligible as a derived computation result, subject to consumer snapshot checks |
| Resource/fuel exhaustion or explicitly partial evaluation | Incomplete; optional bounded diagnostic artifact, never a passing result |
| Invalid program, stuck/unsupported result shape, malformed output, or identity mismatch | Refused/failed; no usable result |
| Observed cancellation or timeout | Non-success with actual termination/cleanup observation |
| Lost process, failed teardown, interrupted capture, or uncertain publication | Unknown or incomplete according to the existing owner; no inferred success |

An exit-zero invocation may successfully transport an incomplete evaluation. It must not therefore satisfy the pure-result contract. The profile output adapter or required validation stage gates parent/result usability on the evaluator's checked complete disposition. Do not relabel a process-level `Completed` record as proof of normal form or Mneme correctness.

Publish admitted results and permitted diagnostics through existing operation-data/artifact storage, retaining content identities, producing invocation, sensitivity, and retention policy. Partial artifacts are explicitly non-authoritative and cannot be selected as completed result data. Byte identity does not prove truth; ordinary operator reports omit payloads and private paths.

### 6. Recover by reconciliation or fresh recomputation

Persist dispatch intent and the existing operation `Started` transition before process execution. Cancellation must reach the actual process and descendants; receipt of a request or sending a signal is insufficient proof of termination. Preserve unknown cleanup rather than releasing/reusing resources prematurely.

After loss of observation, reconcile the owned attempt and any published artifacts through the current durable owner. Do not reopen an old invocation or silently rerun it. If policy admits recomputation after reconciliation, allocate a new invocation/attempt with retained lineage and consumed budget, revalidate current authority, and reconstruct evaluation from the exact immutable program/input/profile. Recompute is not evidence that the old attempt failed or never executed.

The first profile accepts no raw VM-memory snapshot, pointer graph, allocator state, or thread checkpoint. Portable semantic checkpoints remain separate work under Lattice #12 and would require a versioned producer contract and qualification.

### 7. Preserve consumer and campaign ownership

Mneme retains canonical storage, projection registration, provenance, schema semantics, and current authorization/erasure checks before publication or response. An old snapshot result cannot cross a newer barrier merely because reduction completed. Incremental deletion and retraction remain consumer algorithms; fresh net evaluation is the reference behavior for this pilot.

Kiln retains work identity, global claims/fences, later implementation attempts, acceptance, and publication. Lattice retains its graph, budget, authority, process, output, and recovery owners. This profile neither schedules another task nor verifies that the entire Mneme implementation is complete.

## Functional Core and Imperative Shell

- **Core:** closed profile/input/output validation, identity comparison, supported extrinsic/limit checks, and normalized outcome decisions.
- **Shell:** exact artifact reads, existing admitted process launch, cancellation/teardown observations, and existing durable result publication.
- **Producer:** existing interaction-net engine, serialization, reduction semantics, bounds, and pure extrinsics.
- **Consumer:** authorized snapshot construction and domain-specific result validation/publication.

## Qualification

Use actual source-backed checks through the selected pack and production process route. Compare the shared two-input wrapping-`i64` arithmetic workload against an independent Rust reference using `wrapping_add`, `wrapping_sub`, and `wrapping_mul`, using the qualified sequential producer cohort. Cover minimum/maximum `i64`, negative values, overflow in each operator, shared subexpressions, and values outside JavaScript's exact integer range. Do not use a worker-authored `passed` field as the oracle.

Required negatives cover crossed identities, changed bytes, unsupported/impure extrinsics, cyclic programs, noncanonical or out-of-range integer strings, JSON numeric values, wrong input/output arity, failed/nonzero process, malformed/truncated/oversized output, resource exhaustion, cancellation with a descendant, interrupted capture/publication, and stale consumer snapshot. A restart fixture must retain uncertainty and require explicit fresh recomputation rather than resume addresses.

Record end-to-end encoding, process/startup, reduction, readback, memory/allocation observations and comparator outcomes separately. No performance threshold, speedup claim, default promotion, or broader confluence guarantee is inferred from one cohort.

Use the pinned development shell and [repository development instructions](../../../README.md#development). The following existing regression targets and validation commands are part of the implementation closeout, not evidence already produced by this design:

```sh
nix develop
cargo test --locked --test test_local_process --test test_operation_pack_core --test test_operation_pack_shell --test test_choregraph_effect_authority --test test_orchestration_shell
"${LATTICE_CAIRN_BIN:-cairn}" validate --root . --strict
scripts/check.rs
octet-precommit-check
nix flake check -L --option secret-key-files ''
```

Add real selected-worker fixtures before recording their commands or results. Use the pinned Cairn CLI for proposal/design/tasks gates and later lifecycle closeout. Structural document validation is not execution qualification, and missing dependencies or unavailable native tools remain explicit blockers.

## Non-Claims

This profile does not establish universal net termination, arbitrary extension confluence, a verified compiler, secure erasure, OS sandbox completeness, exactly-once external effects, live VM migration, or correctness of Mneme's domain logic. No runtime or validation task is complete merely because these planning artifacts exist.

