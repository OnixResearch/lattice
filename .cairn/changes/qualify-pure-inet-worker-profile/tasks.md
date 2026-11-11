# Tasks

## Phase 1: Contract and producer qualification

- [ ] [serial] 1.1 Record the current operation-pack/process/data/authority baseline and preserve existing behavior when the profile is not selected. r[lattice.pure_inet_profile.route]
- [ ] [serial] 1.2 Consume Noble #37's qualified immutable pure-program serialization, bounded evaluator entry point, executable, build/toolchain, and exact compatibility evidence; leave unavailable producer contracts visibly blocked and invent no CLI flags. r[lattice.pure_inet_profile.identity] r[lattice.pure_inet_profile.bounds]
- [ ] [serial] 1.3 Define the Lattice envelope around the shared producer contract with complete program/input/rule/runtime/extrinsic/profile/limit/output/attempt bindings, without another net format or global work identity. r[lattice.pure_inet_profile.identity] r[lattice.pure_inet_profile.ownership]
- [ ] [serial] 1.4 Bind the two-input, one-output wrapping-i64 acyclic Input/Const/Add/Sub/Mul vocabulary and exact canonical decimal strings; reject JSON numbers, unsupported operations, ambient effects, native registration, and undeclared extensions. r[lattice.pure_inet_profile.purity]

## Phase 2: Existing-route integration

- [ ] [serial] 2.1 Author an opt-in exact `bounded_process` operation pack and thin Rust protocol adapter using the existing operation admission and process runner; add no runtime class, node kind, scheduler, or fallback. r[lattice.pure_inet_profile.route]
- [ ] [serial] 2.2 Revalidate actual serialized bytes and effective resource/authority bindings before dispatch and validate returned identities before output admission. r[lattice.pure_inet_profile.identity]
- [ ] [serial] 2.3 Enforce producer work/arena/memory ceilings alongside existing process deadline/stream limits, retaining reservations and measured or unavailable usage separately. r[lattice.pure_inet_profile.bounds]
- [ ] [serial] 2.4 Normalize complete, partial, invalid, exhausted, failed, cancelled, and unknown evaluation observations without equating process completion with result usability. r[lattice.pure_inet_profile.results]
- [ ] [serial] 2.5 Publish permitted result/diagnostic artifacts through existing operation-data storage with exact producer, retention, sensitivity, and disposition bindings. r[lattice.pure_inet_profile.results]
- [ ] [serial] 2.6 Reuse durable start, cancellation, teardown, and recovery owners; admit only explicit fresh recomputation from immutable inputs after reconciliation. r[lattice.pure_inet_profile.recovery]

## Phase 3: Real fixtures and review

- [ ] [parallel] 3.1 Add a real selected-sequential-worker positive fixture and independent Rust wrapping-i64 comparator, covering overflow, shared expressions and wide exact integers; compare fresh sequential evaluations by semantic output. r[lattice.pure_inet_profile.verification]
- [ ] [parallel] 3.2 Add crossed identity, changed bytes, cycles, wrong arity, unsupported extrinsic, noncanonical/out-of-range integer and JSON-number rejection, malformed output, output overflow, and incomplete-result fixtures. r[lattice.pure_inet_profile.identity] r[lattice.pure_inet_profile.purity] r[lattice.pure_inet_profile.results]
- [ ] [parallel] 3.3 Add work/arena/deadline exhaustion and cancellation fixtures proving actual process/descendant teardown; retain unknown teardown and incomplete telemetry honestly. r[lattice.pure_inet_profile.bounds] r[lattice.pure_inet_profile.recovery]
- [ ] [parallel] 3.4 Add capture/publication crash and restart fixtures; reject raw pointer/heap resume and preserve budget/attempt lineage for any authorized recomputation. r[lattice.pure_inet_profile.recovery]
- [ ] [parallel] 3.5 Add a consumer fixture rejecting an otherwise valid result after its snapshot/schema/authorization/erasure binding is superseded. r[lattice.pure_inet_profile.ownership]
- [ ] [serial] 3.6 Record measured end-to-end cost, exact compatibility identities, comparator results, failure scope, and keep/revise/stop recommendation without default promotion. r[lattice.pure_inet_profile.verification]
- [ ] [serial] 3.7 Document exact opt-in invocation after the producer CLI/protocol exists, limits, result semantics, cancellation/recompute, dependencies and non-claims; link [Noble #37](https://github.com/OnixResearch/noble-lang/issues/37) and [Mneme #5](https://github.com/OnixResearch/mneme/issues/5)'s add-optional-inet-compute. r[lattice.pure_inet_profile.route] r[lattice.pure_inet_profile.ownership]
- [ ] [serial] 3.8 Run real profile fixtures, existing focused process/pack/authority/recovery regressions, pinned Cairn validation/gates, scripts/check.rs, Octet, and relevant Nix checks; retain failures and leave unsupported claims open. r[lattice.pure_inet_profile.verification]

## Evidence Boundary

All tasks are intentionally open. This change specifies a future qualification path. Source inspection and structural checks of these Markdown/JSON artifacts do not execute the worker, prove purity, or establish runtime conformance.

