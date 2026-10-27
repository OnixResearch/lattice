# hardened-wasmtime-runtime

`hardened-wasmtime-runtime` owns product-neutral Wasmtime profile admission, restricted linking, cumulative limits, bounded guest exchange, and normalized outcomes.

Consumers retain product imports, effect meaning, artifact trust, authorization, storage, and execution policy.

## Status

This repository is experimental. The runtime core admits explicit profile,
artifact-shape, import, resource and deadline facts. The shell executes core
modules and actual components under the exact Wasmtime 45.0.0 cohort.

The archived `complete-runtime-extraction-boundary` correction replaces unsafe
raw-configuration and optional-deadline entrypoints. Its evidence records
passing producer gates and independently pinned Molten and Site consumer runs.
Neither pilot is production-runtime adoption or release qualification.

The `hwr.shell.v4` contract adds typed instantiation yields, a successful
instantiation checkpoint before export execution, and independent positive
per-memory and aggregate memory ceilings. This corrects the executed Molten
scheduling/allocation counterexamples without changing its product profile.
Publication is a consumer prerequisite, not Molten runtime adoption.

## Boundary and maintenance

The hardened-runtime maintainers own engine configuration, exact byte/name
admission, bounded guest exchange and normalized failures. Consumers own trusted
materialization facts, worlds/schema identities, capability and effect meaning,
output validation, retention and canonical product receipts. Identity or a
successful invocation grants none of those authorities.

- Construct `BoundedRuntime` from complete admitted feature facts and positive
  bounds; construction revalidates both. `gc-null` supports reference types;
  guest GC, WASI, threads and component async remain unavailable.
- Supply exact artifact bytes and consumer-trusted facts, admitted descriptors,
  a positive finite epoch budget and this engine's `EpochService` before
  preparation. The consumer must operate the service; the runtime owns no clock
  or timer thread. Budgets are cooperative epoch bounds, not hard wall time.
- Core modules use the `memory` / `run(ptr,len)->length` carrier. Components
  use a selected root `(list<u8>) -> result<list<u8>, string>` export, or
  `ComponentInvocation::into_byte_method(interface, function)` binds an
  import-free synchronous named `(string, list<u8>) -> list<u8>` method.
  Named signature admission occurs before instantiation; method UTF-8 and
  input bytes share the input ceiling. `ByteMethodInvocation::run` uses the
  same cumulative store budgets, concurrency and injected epoch service.
  These carriers define no product effects. Bind only admitted imports.
- Each memory obeys `max_memory_pages`; all memories additionally share the
  independent `max_total_memory_pages` invocation cap. Both are positive and
  admitted without clamping. The frozen Molten document reader retains its old
  cumulative ceiling by mapping that document's memory bound to both fields.
  Initialization, execution and automatic post-return share fuel. Engine clones
  share the concurrency bound, and permits release on success and failure.
- Component results lift to owned bytes **before** automatic guest cleanup.
  `Store::set_hostcall_fuel` enforces the guest-to-host transfer ceiling before
  allocation, including error payloads and host imports. No lazy result view
  survives cleanup. Core pointer arithmetic and memory access are checked.

Wasmtime 45.0.0 keeps its host-transfer-exhaustion error private. The shell
recognizes only its exact root diagnostic using allocation-free formatting;
typed guest/host failures take precedence, and every unknown error maps to
`runtime-failed`, never success. A Wasmtime pin change requires rerunning actual
oversized-output and cleanup regressions; do not repin wording-only tests.

### Retained component invocations

Wrap a prepared `ComponentInvocation` in `Arc`, then call
`start_sliced(SliceOptions { export, input, quantum_fuel }, host, binder)`.
Input and export are owned; the input is moved, not copied. The positive quantum
must not exceed the admitted total fuel. The binder is consumer-owned,
synchronous, `Send + 'static`; this API installs no ambient async effects.

The returned `ResumableInvocation::run_slice` polls the same future once.
`Yielded` carries its typed `Instantiation` or `Invocation` phase, a one-based
yield ordinal and configured quantum, **not** a measured per-poll fuel count.
Exactly one `Instantiated` checkpoint reports remaining cumulative fuel after
successful instantiation and before the selected export can execute. The next
poll resumes the same store; the checkpoint is not a callback fuel yield and
does not increment the yield ordinal. Failed instantiation emits no checkpoint.
`Completed` returns the single bounded output after automatic cleanup and exact
Wasmtime remaining-fuel readback. Subsequent polls return `AlreadyCompleted`.
Fuel, aggregate memory, transfer limits and the injected epoch deadline are not
reset between polls or at the checkpoint. No internal slice ledger is retained.

Concurrency is reserved before returning the handle, including before its first
poll. Completion or dropping the handle releases it and destroys the store.
Cancellation does not undo host imports or promise guest post-return cleanup.
The consumer owns scheduling and epoch advancement; synchronous host callbacks
must return. Fuel slices are not hard wall-clock bounds.

`InvocationOutcome.phase` independently records `Admission`, `Linking`,
`Instantiation` or `Invocation` for both synchronous carriers and sliced calls.
Export selection, lifting and automatic post-return belong to `Invocation`;
the API does not fabricate a distinct cleanup phase. Successful calls report
`Invocation`. Consumers must explicitly migrate exhaustive matches and supply
both memory bounds for the v4 contract. Cancellation is also valid at the
instantiation checkpoint: it releases ownership without executing the export.

Host async support enables Wasmtime fuel yielding of synchronous exports; guest
component async and threads remain denied and compiled out. Successful same-work
fuel/output parity is covered by actual-component regressions. Failed runs can
have different engine fuel readbacks between synchronous and sliced calls;
consumers must evaluate that observation against their own receipt contract.
See the [continuation regressions](crates/hardened-wasmtime-shell/tests/cases/slicing.rs),
[cumulative limits](crates/hardened-wasmtime-shell/tests/cases/slicelimits.rs) and
[input/memory boundaries](crates/hardened-wasmtime-shell/tests/cases/slicebounds.rs).
The [checkpoint regressions](crates/hardened-wasmtime-shell/tests/cases/checkpoints.rs)
exercise start work, callback fencing, exact successful fuel, failed starts and
checkpoint cancellation. Actual allocation cases distinguish per-memory from
aggregate denial rather than inferring allocation from admission alone.


The frozen Molten profile-document reader remains a compatibility input;
independent consumers supply their own materialization world and product policy.
This boundary does not prove sandboxing, arbitrary guest correctness, behavioral
equivalence, source trust, whole-system safety or release eligibility.

## Repeatable checks

Run from this repository:

```sh
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo fmt --all --check
nix run .#octet-deny-all -- --workspace -- --all-targets
nix flake check -L
```

Execution regressions are Cargo-discoverable under
`crates/hardened-wasmtime-shell/tests/execution.rs`; admission regressions are
under `crates/hardened-wasmtime-runtime/tests/admission.rs`. They cover actual
module/component execution, imports, fuel, aggregate memory, pointers, epoch
interruption, concurrency, bounded output and post-return cleanup. Correction
evidence lives in
[the native Cairn archive](.cairn/archive/1970-01-01-complete-runtime-extraction-boundary/verification.md).

## License

MIT. See `LICENSE`.

## References

- [Qip](https://github.com/royalicing/qip) — static content-profile reference for `.cairn/changes/add-inspectable-content-profile/`, after the runtime extraction. No checker or runtime dependency is adopted. An immutable reference pin and independent checker audit remain prerequisites.

- [Aspen](https://github.com/OnixResearch/aspen) — hardened component-runtime source and first pilot.
- [Wasmtime](https://github.com/bytecodealliance/wasmtime) — runtime API and security boundary dependency.
