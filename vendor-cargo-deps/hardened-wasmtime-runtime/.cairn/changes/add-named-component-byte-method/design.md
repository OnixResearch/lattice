## Context

The shared engine already admits exact artifact bytes, interface names, complete feature facts, cumulative limits and an injected epoch service. Lattice needs one generic named-interface carrier without moving its WIT package, profile identity, authority, clocks, cancellation worker or operation effects into the producer.

## Decisions

### Decision: Reuse exact preparation and bounded stores

**Choice:** Add `ComponentInvocation::into_byte_method(interface, function)` returning `ByteMethodInvocation` over `(string, list<u8>) -> list<u8>`. Reflect and validate the named interface/function signature before any instantiation; invoke through the same private engine, limiter, fuel, host-transfer ceiling, concurrency permit and typed outcome classifier. Count method UTF-8 bytes together with input before store allocation. Return owned bytes only after Wasmtime's automatic post-return succeeds.

**Rationale:** This is a product-neutral carrier, not a new policy profile or engine. Existing root invocation and retained slicing remain unchanged. Lattice translates its own explicit descriptor and drives the original epoch service; no host clock/thread, imports or fallback is added here.

## Risks / Trade-offs

- Wasmtime 45.0.0 combines invocation and automatic cleanup failure phases; preserve typed failure class/phase without pretending to distinguish an unobserved cleanup stage. Local exact-source import is not upstream publication or production adoption.
