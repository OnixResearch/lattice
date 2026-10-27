## Context

The runtime extraction already names fuel, memory, table, instance, stack, deadlines, input/output bounds, exact imports, and normalized outcomes. This follow-up specifies an optional final-artifact content profile and its evidence limits.

## Decisions

### Inspect facts before executing code

The shell parses the final Wasm bytes with a pinned parser and supplies bounded facts to pure admission logic. The core performs no file access, compilation, instantiation, clocks, or Wasmtime calls.

The strict content profile rejects imports, start functions, shared memory, atomics, memory growth, indirect/reference calls, and direct recursion. It requires a declared memory maximum and static ABI metadata. The profile records permitted instruction features explicitly. General runtime profiles remain available for workloads outside this subset.

Static getters must have an admitted constant shape. Metadata must resolve from the initial data image without executing guest logic. Reject overlapping input/data regions, incomplete export pairs, wrong signatures, unsupported memory shapes, and overflowing ranges.

### Use layered verdicts

Record core validation, structural admission, static-check findings, and runtime outcomes separately. An unsupported static proof shape is not a semantic defect. A structural pass does not prove termination, correctness, or acceptable latency.

Optional loop-bound and output-size checker adapters are deferred unless the Site pilot demonstrates demand. Before admission, audit them against adversarial artifacts and an independent checker or small verified rule set. A count bound never permits the host to omit output-pointer checks. Do not promote imported Qip checkers to Trellis or Octet proof authority.

### Enforce runtime limits independently

Consume the extraction's runtime policy instead of duplicating it. Limit compilation, instantiation, guest execution, copies, output retention, and owned teardown as supported by explicit contracts. Record unsupported total-deadline claims as limitations rather than inferred guarantees.

The host validates returned output ranges and lengths with checked arithmetic before reading memory. Recoverable rejection yields no output. Traps invalidate the instance. Configuration and scratch-state reuse require explicit recovery tests.

### Bind artifact and profile evidence

Bind the final artifact identity, ABI version, profile, parser/checker versions, runtime version, declared limits, and observed outcome using stack BLAKE3 contracts. Admission applies to those bytes only. A compiler flag, source annotation, or prior version cannot satisfy current admission.

## Alternatives and risks

A general Wasmtime profile remains the correct choice for valid modules outside the strict subset. Qip's default CLI policy is less restrictive than its strict artifact tier. Its compliance introduction and hard-limit table conflict, so neither prose statement establishes implementation behavior.

The static parser and optional checker are part of the trusted computing base. Bound their allocations and execution too. Native translation does not preserve the Wasm sandbox automatically and remains outside this change.

## Validation

Use final Wasm fixtures for valid static metadata and bounded execution. Negative fixtures cover forged getter functions, start effects, imports, input/data overlap, recursion, indirect calls, memory growth, malformed sections, invalid pointers, overflow, excessive output, fuel exhaustion, and deadline expiry.

Run the extraction baseline before changes and repeat focused tests afterward. Run workspace positive/negative tests, Clippy with warnings denied, strict pinned Octet, Nix checks, Cairn validation and gates, and real Site integration. No stable publication before the extraction's consumer requirements pass.
