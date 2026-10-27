## Why

Aspen has a strong Wasmtime component runtime with explicit feature policy, fuel, store limits, and artifact-shape checks. The runtime remains product-specific and its first cohort rejects all imports.

OnixResearch needs a focused hardened Wasmtime component that can serve more than one product without owning product effect ABIs.

## What Changes

- Extract a `hardened-wasmtime-runtime` core and shell from reviewed Aspen code.
- Admit one explicit Wasmtime feature policy and reject host-default drift.
- Enforce fuel, memory, table, instance, stack, epoch-deadline, input, and output limits.
- Admit only exact restricted imports supplied by a product adapter. Deny WASI and undeclared imports by default.
- Validate artifact hashes, component shape, world identity, imports, exports, and runtime profile before instantiation.
- Normalize compile, instantiate, trap, fuel, deadline, memory, guest, host, and output failures.
- Keep product effect semantics, capabilities, artifact fetching, clocks, and policy outside the component.
- Require Aspen and a second real consumer pilot before stable publication.

## Impact

- **Source**: Aspen Wasmtime component runtime and admission profiles
- **New component**: focused `hardened-wasmtime-runtime`
- **Consumers**: Aspen and one Kamacite, Animus, or later runtime adapter
- **Compatibility**: product WIT worlds, effect ABIs, and artifact formats remain consumer-owned
