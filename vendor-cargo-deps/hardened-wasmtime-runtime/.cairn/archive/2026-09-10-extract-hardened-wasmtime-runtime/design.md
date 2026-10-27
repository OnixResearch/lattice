## Context

Aspen configures Wasmtime deterministically, disables unsupported features, consumes fuel, applies store limits, and checks component imports and exports against materialization facts.

The extracted component must add explicit deadlines and output bounds. It must permit reviewed product adapters without installing a universal effect ABI.

## Decisions

### Decision: Separate profile admission from execution

**Choice:** A pure core validates runtime profiles, artifact facts, import descriptors, limits, and normalized outcomes. A thin shell owns Wasmtime engine, store, linker, epoch, and invocation effects.

**Rationale:** Runtime policy needs deterministic tests without starting an engine.

### Decision: Every Wasmtime feature is explicit

**Choice:** The profile records the state of every reviewed proposal and deterministic code-generation option. Unknown or omitted feature facts deny admission.

**Rationale:** Wasmtime defaults can change across versions.

### Decision: Use cumulative independent limits

**Choice:** Fuel, stack, memory, tables, instances, epoch deadline, input, and output limits all apply. Reaching one limit produces its normalized class.

**Rationale:** No single limit bounds all compute and allocation behavior.

### Decision: Epoch service is injected

**Choice:** The shell accepts a host-owned epoch service and binds a deadline budget to the invocation. The core receives only explicit epoch observations.

**Rationale:** The reusable component must not read ambient clocks or create global timer threads.

### Decision: Imports are deny-by-default

**Choice:** The base linker contains no WASI or product functions. A consumer adapter supplies an exact import set bound to operation, capability, schema, and adapter identities.

**Rationale:** Host functions are authority and cannot follow from module names alone.

### Decision: Bound guest exchange

**Choice:** Inputs are bounded before invocation. Outputs are bounded before allocation or acceptance. Core-Wasm memory adapters validate pointer, length, overflow, and memory range.

**Rationale:** Store memory limits do not bound host-side vectors or invalid guest pointers.

### Decision: Normalize failures without erasing detail

**Choice:** Stable classes separate profile, artifact, compile, instantiate, import, fuel, deadline, memory, guest, host, output, and trap failures. Bounded implementation details remain optional diagnostics.

**Rationale:** Consumers need cross-backend comparison without unstable Wasmtime strings as identity.

### Decision: Require a second consumer

**Choice:** Aspen remains the source pilot. Stable publication requires another product adapter with a different world and no Aspen type dependency.

**Rationale:** A single consumer cannot prove the boundary is product-neutral.

## Risks / Trade-offs

- Wasmtime API and feature changes can require a new runtime profile version.
- Epoch interruption depends on a correctly operated host epoch service.
- Product import adapters remain security-critical.
- A passing invocation does not prove guest correctness, deterministic host effects, or semantic equivalence with another backend.
