# Proposal: Add a Wasm component operation-pack runtime

## Why

Lattice operation packs support reviewed external executables and explicitly reject unsupported WebAssembly execution. This leaves portable pack logic on the process path even when the logic needs no ambient host authority.

A separate component runtime class can execute deterministic pack methods through the shared hardened runtime. Lattice must retain operation authority, durable attempts, effects, recovery, output admission, and pack identity.

## What Changes

- Add a `wasm-component` runtime descriptor to typed pack manifests and Rust admission. r[lattice.operation_packs_wasm.manifest]
- Bind locks to exact component, WIT world, Mantle bundle, shared-runtime profile, method mapping, resource, and evidence identities. r[lattice.operation_packs_wasm.lock]
- Add a pure runtime admission core and a thin shell over the shared hardened Wasmtime runtime and Mantle verifier. r[lattice.operation_packs_wasm.functional_core] r[lattice.operation_packs_wasm.shell]
- Use an import-free first world. Components return bounded method results or typed effect intents for existing Lattice operation admission and durable effect paths. r[lattice.operation_packs_wasm.world] r[lattice.operation_packs_wasm.effects]
- Persist the attempt before component dispatch and retain timeout, cancellation, trap, resource, invalid-output, and unknown outcomes. r[lattice.operation_packs_wasm.lifecycle]
- Reject fallback to an executable, shell node, interpreter, or another runtime. r[lattice.operation_packs_wasm.fallback]
- Add offline positive and negative pack fixtures and a focused conformance rail. r[lattice.operation_packs_wasm.verification]

## Impact

- **Pack core**: gains typed component descriptors, lock facts, and deterministic admission.
- **Product shell**: gains one non-default component adapter with no ambient WASI.
- **Existing owners**: Lattice keeps credentials, effects, attempts, persistence, recovery, outputs, and operator evidence.
- **Dependencies**: requires completed shared-runtime and Mantle-verifier publications plus a matching Mantle component bundle.

## Out of Scope

- JavaScript, TypeScript, Deno, Node, shared libraries, arbitrary commands, or undeclared callbacks.
- Ambient guest filesystem, network, process, environment, clock, random, credential, or device authority.
- Automatic runtime fallback, registry installation, or production-readiness claims.

## Affected Specs

- `operation-packs-wasm-runtime`: manifest, lock, core/shell split, world, effects, lifecycle, fallback, and verification.
