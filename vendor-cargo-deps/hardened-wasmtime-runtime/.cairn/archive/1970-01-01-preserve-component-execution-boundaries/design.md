## Context

The v3 retained future combines linking, asynchronous instantiation and invocation. Pending polls expose configured quanta but no phase; successful instantiation is not externally observable before invocation starts. Runtime memory pages also supply both Wasmtime's per-memory limit and one cumulative allocation budget.

## Success Contract

An actual component can finish bounded instantiation and report a typed checkpoint without calling its selected export. The consumer can cancel there or resume the same retained store. Instantiation yields and callback yields remain distinct, total fuel and concurrency are never reset, and only terminal cleanup exposes output. Separately admitted per-memory and aggregate page bounds each refuse their own over-bound guest.

## Decisions

### One explicit instantiation checkpoint

`SliceBoundary` carries typed `InvocationPhase`. `SliceStep::Instantiated(InstantiationCheckpoint)` reports exact remaining fuel once, before invocation. The same pinned future pauses at that point; advancing it later starts the selected export. `Completed` remains the only output-bearing terminal result. Consumers migrate explicitly to `hwr.shell.v4` and handle the checkpoint; no deprecated alias, diagnostic parsing, guest-cost inference, duplicated engine or restarted invocation is introduced.

A small shared phase/checkpoint control may bridge the suspended future's exclusive store borrow. It must not allocate per poll, retain an unbounded ledger, read a clock, or expose store/future internals. The consumer owns draining or scheduling instantiation and operating the exact-engine epoch service.

### Independent memory ceilings

`RuntimeBounds.max_memory_pages` remains the per-memory ceiling; new `max_total_memory_pages` is the cumulative invocation allocation ceiling. Both are explicit positive bounded integers and are validated before engine construction. Wasmtime's per-memory limiter uses the former, and cumulative allocation accounting uses the latter. A total cap smaller than a per-memory cap is valid and stricter, not silently clamped.

Existing frozen profile-document translation sets both fields to its previous cap to preserve that compatibility input's behavior. Molten's direct adapter can retain its own per-memory ceiling and derive the implicit aggregate ceiling with checked arithmetic from admitted memory count. No product DTO or policy enters the producer.

## Functional Core and Imperative Shell

The core validates supplied resource facts without I/O. The shell owns engine construction, bounded store effects, typed checkpoint observations, retained future lifetime and cleanup. Public counts use fixed-width integers; host-width conversions remain checked at the shell boundary.

## Risks / Trade-offs

- The new slice variant and bound field intentionally require explicit caller migration; the shell contract changes to v4.
- Cooperative fuel/epoch bounds do not guarantee hard wall time or interrupt blocking synchronous imports.
- Cancellation releases owned resources but does not roll back imports or promise guest post-return cleanup.
- A published producer enables, but does not establish, complete product parity or consumer adoption.
