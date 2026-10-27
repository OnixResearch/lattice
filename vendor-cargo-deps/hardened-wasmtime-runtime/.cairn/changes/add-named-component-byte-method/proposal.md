## Why

Lattice's operation-pack consumer declares an import-free named interface method with `(string, list<u8>) -> list<u8>`, while the published shared shell only invokes a root byte-result carrier. Repairing the consumer must not add a second engine, bypass artifact admission, change its WIT world, or disable component execution. The shared-runtime maintainers own the generic carrier; Lattice retains product policy and effects.

## What Changes

- Add explicit preparation-time named byte-method signature admission and bounded invocation through the existing shared engine/store/limit machinery. r[onix.wasmtime_runtime.named_byte_method]
- Keep existing carriers and scoped outcome semantics; the consumer continues to operate deadlines and cancellation through the injected epoch service. r[onix.wasmtime_runtime.named_byte_method]

## Impact

- **Files**: shell named-method module and public entry points, actual-component regressions, and existing runtime documentation.
- **Testing**: exact named method success, missing/wrong signature, input/output boundaries, fuel/trap and cumulative cleanup, followed by producer workspace and independent Lattice execution checks. No publication, adoption or release readiness is inferred from a local correction.
