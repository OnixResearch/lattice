## Context

The existing shell prepares exact admitted components and executes them synchronously under Wasmtime 45.0.0. Molten needs fuel-quantum suspension of a synchronous WIT export, not the guest component-async proposal. Its published compatibility evidence also demonstrates that a normalized class alone loses failure phase. Baseline producer execution passed 9 core unit, 29 admission and 22 actual execution cases before source edits.

## Decisions

### Decision: Keep one owned future across host-driven polls

**Choice:** `Arc<ComponentInvocation>::start_sliced(SliceOptions, host, binder)` admits a positive quantum no larger than total fuel and bounded owned input, then constructs a `ResumableInvocation`. `run_slice` polls once with a no-op waker. Pending returns a `SliceBoundary { ordinal, quantum_fuel }`; ready returns the single terminal outcome. Polling after completion returns `AlreadyCompleted`. No background executor, clocks or timer threads are installed. The binder remains synchronous and consumer-owned; imports that require external async wakeups are not installed by this API.

**Rationale:** Owning prepared state and input avoids consumer self-referential objects and redundant input copies. One pinned future preserves guest state; restarting after a trap does not. A concurrency permit is reserved before returning the handle, including before its first poll, held across execution and suspension, and released when the future completes or is dropped. The ordinary synchronous path retains its borrowed permit without allocating.

### Decision: Distinguish quantum configuration from fuel measurement

**Choice:** Yield observations contain the exact poll ordinal and configured fuel interval, not an invented per-poll remaining-fuel number. Wasmtime's borrowed suspended store cannot be read through its live future using the safe API. Exact cumulative remaining fuel comes from the store after the final invocation including automatic cleanup. No unbounded slice ledger is retained by the runtime.

**Rationale:** Wasmtime fuel checks can batch instructions; configured quantum is not a promise of exact instruction consumption. Consumers own their ledgers and must reconcile with the terminal measurement. Total fuel, memory, transfer and epoch limits remain cumulative, without resets between polls. Epoch deadlines are not extended on resume.

### Decision: Carry typed phases independently of failure class

**Choice:** Add `InvocationPhase::{Admission, Linking, Instantiation, Invocation}` to the pure outcome vocabulary and shell outcomes. Store phase changes before the corresponding operation. Admission precedes binding; instantiation traps remain `GuestTrapped` with `Instantiation`, while export calls, lifting and automatic post-return use `Invocation`. Successful results have `Invocation`. Bump `SHELL_SCHEMA_ID` to `hwr.shell.v3`.

**Rationale:** This preserves normalized classes while retaining the information required by staged receipts. No diagnostic matching or fuel-based phase inference is admitted. Wasmtime 45 performs post-return inside the typed call; the contract does not fabricate a distinct cleanup phase.

### Decision: Enable host async, not guest async

**Choice:** Add only Wasmtime's `async` feature, keeping `component-model-async` and threads unavailable. Admission feature facts and synchronous profiles remain unchanged. Wasmtime remains exactly 45.0.0; let Cargo resolve the enabled fiber dependency against the existing lock without refreshing unrelated versions.

**Rationale:** Fuel-based host yielding is independent of the guest language feature. The runtime core remains I/O-free and consumers retain execution policy and scheduling.

## Risks / Trade-offs

- Cancellation destroys the suspended store; it does not undo already executed consumer imports or promise to run guest cleanup. Consumers retain uncertainty and transactional effect policy.
- A binder or synchronous host import is consumer code, not metered Wasm. It must return; the runtime cannot preempt it or claim a hard wall-clock bound.
- Arc ownership and one boxed future are needed for a movable retained invocation; the runtime does not copy the owned input or allocate a slice history.
- The new shell result field is an intentional source/wire contract change. Existing in-repository constructors and consumers must migrate; external consumers must pin the new schema and rerun their parity corpus.
- GitHub's configured origin fetch returned repository-not-found. Radicle fetch succeeded at the recorded baseline; publication will use the available canonical Radicle route and report its exact revision, not invent GitHub replication.
