# Hardened Wasmtime Runtime Specification

## Purpose

Defines the `hardened-wasmtime-runtime` capability.

## Requirements

### Requirement: Execution profile admission [r[hardened_wasmtime.profile_admission]]

The core MUST admit explicit resource limits (fuel, maximum memory pages) and MUST reject zero or absent bounds with named errors before any execution.

#### Scenario: Admitted limits round-trip
- GIVEN positive fuel and memory-page bounds
- WHEN the limits are admitted
- THEN admission MUST succeed and expose exactly the given values.

#### Scenario: Zero bound fails closed
- GIVEN a zero fuel bound or a zero memory-page bound
- WHEN the limits are admitted
- THEN admission MUST fail naming the violated field.

### Requirement: Normalized outcome model [r[hardened_wasmtime.outcome_model]]

Execution results MUST normalize to one of: completed with return value, trapped with a stable trap class, or fuel exhausted. Distinct runtime failures MUST map to distinct outcomes.

#### Scenario: Completed normalizes
- GIVEN a module that runs to completion under the fuel budget
- WHEN the result is normalized
- THEN the outcome MUST be completed with the returned value.

#### Scenario: Unreachable trap normalizes to its class
- GIVEN a module that executes `unreachable`
- WHEN the result is normalized
- THEN the outcome MUST be trapped with class `unreachable`.

#### Scenario: Fuel exhaustion is distinct from traps
- GIVEN fuel below the module's requirement
- WHEN the result is normalized
- THEN the outcome MUST be fuel-exhausted, not a trap.

### Requirement: Strategy-parity differential through the harness [r[hardened_wasmtime.strategy_parity]]

Every corpus module MUST execute under both the Cranelift and Pulley strategies; both observations MUST pass `admit_independent_backends` and be compared with `compare_pairwise` from the pinned `differential-execution-core`; the case identity MUST be stable per program.

#### Scenario: Corpus agrees across strategies
- GIVEN the bounded corpus
- WHEN each module runs under both strategies and is compared
- THEN every verdict MUST be `Agreement`
- AND each case identity MUST be reproducible across runs.

#### Scenario: Forced divergence names a surface
- GIVEN one mutated observation for a corpus case
- WHEN compared
- THEN the verdict MUST be `Divergence` with a bounded mismatch surface named.

### Requirement: Independence gate fails closed [r[hardened_wasmtime.independence]]

Duplicate backend implementations or a single observation MUST be rejected by `admit_independent_backends` before comparison.

#### Scenario: Duplicate rejected
- GIVEN two observations naming the same strategy implementation
- WHEN admitted
- THEN the gate MUST fail with `DuplicateImplementation`.

#### Scenario: Single rejected
- GIVEN one observation only
- WHEN admitted
- THEN the gate MUST fail with `OneBackend`.

### Requirement: Oracle classification [r[hardened_wasmtime.oracle]]

For a corpus case, both observations MUST classify against an exact expected-value fixture; drift on either side MUST classify as `Mismatch`.

#### Scenario: Exact match on both sides
- GIVEN observations equal to the fixture
- WHEN classified
- THEN both sides MUST be `ExactMatch`.

#### Scenario: Drifted side mismatches
- GIVEN one divergent observation
- WHEN classified
- THEN that side MUST be `Mismatch` and the other `ExactMatch`.

### Requirement: Explicit Wasmtime feature policy
r[onix.wasmtime_runtime.feature_policy] Every runtime profile MUST state the reviewed value of each supported Wasmtime proposal and deterministic code-generation option. The shell MUST enforce those admitted facts when constructing its engine, rather than accepting arbitrary unvalidated configuration.

#### Scenario: Complete profile passes
r[onix.wasmtime_runtime.feature_policy.valid]
- GIVEN a complete supported feature policy and positive runtime bounds
- WHEN engine construction runs
- THEN the engine enforces exactly the admitted feature and resource settings.

#### Scenario: Missing feature fact fails
r[onix.wasmtime_runtime.feature_policy.missing]
- GIVEN a profile that omits one reviewed Wasmtime feature
- WHEN the core admits the profile
- THEN admission fails rather than using a host default

#### Scenario: Unsupported feature fails
r[onix.wasmtime_runtime.feature_policy.unsupported]
- GIVEN a denied feature or inconsistent admitted policy identity
- WHEN engine construction runs
- THEN it fails before compilation or execution.

### Requirement: Exact artifact-shape admission
r[onix.wasmtime_runtime.artifact_shape] The runtime MUST validate exact artifact identity, component or module class, world or ABI identity, imports, exports, and runtime profile before instantiation. It MUST bound artifact bytes before compilation and execute the declared artifact class without fallback to another class.

#### Scenario: Exact shape passes
r[onix.wasmtime_runtime.artifact_shape.valid]
- GIVEN an admitted component and exact materialization facts
- WHEN the component is invoked through its declared export
- THEN its bounded result is returned through the component model, not a substituted core-module guest.

#### Scenario: Artifact bound or shape fails
- GIVEN oversized bytes, a mismatched identity/class or drifted imports/exports
- WHEN preparation runs
- THEN it fails before instantiation or consumer host effects.

#### Scenario: Import drift fails
r[onix.wasmtime_runtime.artifact_shape.import]
- GIVEN runtime imports that differ from admitted materialization facts
- WHEN shape validation runs
- THEN preparation is denied before instantiation or consumer host effects

#### Scenario: Export drift fails
r[onix.wasmtime_runtime.artifact_shape.export]
- GIVEN runtime exports that differ from admitted materialization facts
- WHEN shape validation runs
- THEN preparation is denied before instantiation or consumer host effects

### Requirement: Cumulative resource limits
r[onix.wasmtime_runtime.resource_limits] Every invocation MUST enforce positive admitted limits for fuel, stack, memory, tables, instances, input, output, and related runtime allocations. Component memories MUST share the invocation memory allowance, initialization and post-return MUST share its fuel, and bounded owned output MUST be lifted before automatic guest cleanup. Engine clones MUST share the concurrency ceiling.

#### Scenario: Bounded invocation completes
r[onix.wasmtime_runtime.resource_limits.valid]
- GIVEN a guest that stays within every admitted limit
- WHEN the shell invokes it
- THEN the result records success and remaining or consumed bounded resources

#### Scenario: Fuel exhaustion is normalized
r[onix.wasmtime_runtime.resource_limits.fuel]
- GIVEN a guest that consumes its admitted fuel during initialization, execution or cleanup
- WHEN invocation traps
- THEN the outcome class is fuel exhausted

#### Scenario: Memory growth fails closed
r[onix.wasmtime_runtime.resource_limits.memory]
- GIVEN a guest that requests memory beyond its aggregate admitted store limit
- WHEN growth occurs
- THEN the runtime traps or denies growth with the memory-limit class

#### Scenario: Output stays bounded
r[onix.wasmtime_runtime.resource_limits.output]
- GIVEN output beyond its admitted transfer maximum or guest cleanup that overwrites a returned buffer
- WHEN the host receives the result
- THEN oversized output is rejected before host allocation and successful output retains the exact bytes returned before cleanup

### Requirement: Epoch deadline enforcement
r[onix.wasmtime_runtime.epoch_deadline] Every invocation MUST bind a positive finite epoch deadline budget to an injected host epoch service belonging to the exact engine. The runtime MUST NOT start a timer or epoch-driver thread.

#### Scenario: Invocation before deadline completes
r[onix.wasmtime_runtime.epoch_deadline.valid]
- GIVEN a guest that completes before the admitted epoch deadline
- WHEN the shell invokes it
- THEN deadline enforcement does not change the guest result

#### Scenario: Deadline exhaustion is normalized
r[onix.wasmtime_runtime.epoch_deadline.exhausted]
- GIVEN a guest that remains active after its epoch deadline
- WHEN the consumer operates the service and Wasmtime interrupts it
- THEN the outcome class is deadline exhausted

#### Scenario: Missing epoch service fails
r[onix.wasmtime_runtime.epoch_deadline.service]
- GIVEN a missing or cross-engine epoch service or a zero or sentinel deadline
- WHEN execution preflight runs
- THEN invocation is denied

### Requirement: Restricted imports
r[onix.wasmtime_runtime.restricted_imports] The base runtime MUST expose no WASI or product host import and MUST link only exact imports supplied by an admitted consumer adapter.

#### Scenario: Exact adapter import passes
r[onix.wasmtime_runtime.restricted_imports.valid]
- GIVEN an exact import descriptor bound to the selected world, operation, schema, capability, and adapter identities
- WHEN the linker is built
- THEN only that admitted import is available

#### Scenario: Undeclared import fails
r[onix.wasmtime_runtime.restricted_imports.undeclared]
- GIVEN a guest that imports a function absent from the admitted descriptor set
- WHEN instantiation runs
- THEN instantiation is denied

#### Scenario: Capability mismatch fails
r[onix.wasmtime_runtime.restricted_imports.capability]
- GIVEN an import descriptor with a capability observation for another request or operation
- WHEN linker admission runs
- THEN the import is denied

#### Scenario: WASI is absent by default
r[onix.wasmtime_runtime.restricted_imports.wasi]
- GIVEN a guest that imports a WASI interface under the base profile
- WHEN instantiation runs
- THEN instantiation is denied

### Requirement: Bounded guest-memory exchange
r[onix.wasmtime_runtime.guest_memory] Core-Wasm adapters MUST validate pointer, length, arithmetic overflow, memory range, input limit, and output limit before host or guest memory access.

#### Scenario: Valid range passes
r[onix.wasmtime_runtime.guest_memory.valid]
- GIVEN a pointer and length within the admitted guest memory and transfer limit
- WHEN the adapter accesses the range
- THEN the exact range is used

#### Scenario: Overflow fails
r[onix.wasmtime_runtime.guest_memory.overflow]
- GIVEN a pointer and length whose sum overflows the address type
- WHEN the adapter validates the range
- THEN access is denied before memory use

#### Scenario: Out-of-range pointer fails
r[onix.wasmtime_runtime.guest_memory.range]
- GIVEN a pointer and length outside current guest memory
- WHEN the adapter validates the range
- THEN access is denied

### Requirement: Normalized runtime outcomes
r[onix.wasmtime_runtime.normalized_outcomes] The runtime MUST return stable classes for profile, artifact, compile, instantiate, import, fuel, deadline, memory, guest, host, output, and trap outcomes.

#### Scenario: Stable trap class is independent of text
r[onix.wasmtime_runtime.normalized_outcomes.trap]
- GIVEN two Wasmtime errors with different display text and the same admitted trap class
- WHEN the shell normalizes both
- THEN both outcomes use the same stable class

#### Scenario: Host denial stays distinct
r[onix.wasmtime_runtime.normalized_outcomes.host]
- GIVEN an admitted import whose host adapter denies the operation
- WHEN invocation returns
- THEN the outcome is host denial rather than guest trap or fuel exhaustion

#### Scenario: Unknown error fails closed
r[onix.wasmtime_runtime.normalized_outcomes.unknown]
- GIVEN a Wasmtime error that the selected profile cannot classify
- WHEN normalization runs
- THEN the outcome is unknown runtime failure and not success

### Requirement: Pure runtime policy core
r[onix.wasmtime_runtime.functional_core] Runtime profile, artifact, import, and outcome decisions MUST be pure over explicit values.

#### Scenario: Core needs no engine
r[onix.wasmtime_runtime.functional_core.explicit]
- GIVEN complete profile and observation values
- WHEN the core admits or classifies them
- THEN it does not create an engine, store, linker, timer, file, process, or network operation

### Requirement: Portable runtime fixtures
r[onix.wasmtime_runtime.fixture_compatibility] The repository MUST publish exact positive and negative fixtures for profiles, artifacts, limits, imports, memory exchange, and outcomes. Regression tests MUST be Cargo-discoverable and component cases MUST execute actual Component Model artifacts rather than substitute core modules.

#### Scenario: Positive fixture agrees
r[onix.wasmtime_runtime.fixture_compatibility.positive]
- GIVEN a published successful invocation fixture
- WHEN an independent consumer runs it
- THEN the admitted profile and normalized outcome equal the fixture

#### Scenario: Negative fixture agrees
r[onix.wasmtime_runtime.fixture_compatibility.negative]
- GIVEN an unknown-feature, bad-import, invalid-pointer, exhausted, oversized, or trap fixture
- WHEN an independent consumer runs it
- THEN the stable denial or outcome equals the fixture

### Requirement: Independent consumer pilots
r[onix.wasmtime_runtime.consumer_pilots] Molten and one independent product consumer MUST pass actual component pilots before stable publication, selecting an immutable corrected producer revision without a delivered sibling-path override. Pilots MUST retain product world, admission, effect and output meaning in consumer adapters.

#### Scenario: Two worlds pass
r[onix.wasmtime_runtime.consumer_pilots.valid]
- GIVEN a Molten actor world and a different independent product world
- WHEN both execute components through the same corrected extracted runtime revision
- THEN both preserve their product imports and DTOs in adapters and observe exact product outputs and denials

#### Scenario: Aspen type leakage blocks stability
r[onix.wasmtime_runtime.consumer_pilots.leak]
- GIVEN a public shared API that requires an Aspen or Molten product type
- WHEN extraction readiness is evaluated
- THEN stable publication is denied

### Requirement: Immutable runtime source
r[onix.wasmtime_runtime.source_revision] Cargo and Nix consumers MUST select the same immutable reviewed runtime revision and Wasmtime dependency set.

#### Scenario: Source identities agree
r[onix.wasmtime_runtime.source_revision.agree]
- GIVEN Cargo and Nix declarations
- WHEN source validation runs
- THEN both select the same full runtime and Wasmtime source identities

### Requirement: Reviewed source inventory
r[onix.wasmtime_runtime.source_inventory] The extraction MUST record exact Aspen source, Wasmtime version, copied fixtures, semantic changes, licenses, and retained consumer authority. A correction MUST identify its published baseline and newly owned mechanisms, preserving historical evidence rather than presenting it as a fresh run.

#### Scenario: Inventory is complete
r[onix.wasmtime_runtime.source_inventory.complete]
- GIVEN the corrected source tree and its historical extraction inventory
- WHEN provenance validation runs
- THEN imported mechanisms remain attributed and correction evidence names its baseline, exact cohort, semantic changes, owners and observed commands

### Requirement: Retained fuel-sliced component invocation
r[onix.wasmtime_runtime.resumable_execution] The shell MUST allow a consumer to poll one retained invocation of a synchronous component export across positive admitted fuel quanta, preserving the same guest state and cumulative fuel budget. Every yielded poll MUST expose its ordinal and configured quantum, and the single terminal outcome MUST expose exact remaining fuel after automatic cleanup. Configured quanta MUST NOT be claimed as exact per-poll fuel measurements. A terminal invocation MUST NOT execute or publish output again.

#### Scenario: A continuation resumes the same work
- GIVEN an admitted component whose turn exceeds one quantum
- WHEN the consumer alternates polling it with independent work
- THEN the component continues from its suspended state and returns the same bytes and exact terminal remaining fuel as its unsliced execution

#### Scenario: Completed invocations cannot run again
- GIVEN an invocation has returned its terminal outcome
- WHEN the consumer polls it again
- THEN the shell returns an already-completed error without invoking guest code or returning the output again

### Requirement: Resource limits survive suspension and cancellation
r[onix.wasmtime_runtime.slice_limits] A sliced invocation MUST retain admitted total fuel, aggregate memory, transfer bounds, epoch deadline and concurrency ownership across yields without resetting them. Zero or over-budget quanta MUST deny before execution. Cancellation by dropping the invocation MUST destroy its store and release runtime concurrency ownership; it MUST NOT claim rollback of consumer effects or execution of guest cleanup.

#### Scenario: Total fuel exhaustion is terminal
- GIVEN a guest whose work exceeds admitted total fuel
- WHEN the consumer repeatedly resumes it
- THEN execution terminates with FuelExhausted and empty output without replenishing fuel

#### Scenario: Suspension retains concurrency and cancellation releases it
- GIVEN a suspended invocation occupies the sole engine permit
- WHEN another invocation requests execution
- THEN it is denied for concurrency, and after cancellation a fresh invocation can complete

#### Scenario: Deadline is not extended on resume
- GIVEN an invocation is suspended under its injected engine epoch service
- WHEN the consumer advances the admitted deadline and resumes it
- THEN the invocation terminates with DeadlineExhausted and empty output

#### Scenario: Cleanup and transfer limits remain enforced
- GIVEN a component with fuel-consuming cleanup or an over-bound output
- WHEN it executes through the sliced carrier
- THEN no output becomes visible before terminal cleanup and the same fuel and transfer failures remain enforced

### Requirement: Failure phase is independent of normalized class
r[onix.wasmtime_runtime.failure_phase] Every synchronous module/component and sliced component outcome MUST identify its typed Admission, Linking, Instantiation or Invocation phase independently of normalized failure class. A successful instantiation followed by an export trap MUST remain distinguishable from a start-function trap without diagnostic parsing or fuel inference. Successful bounded calls MUST report Invocation phase.

#### Scenario: Two guest traps retain distinct phases
- GIVEN one component traps in its start function and another traps in its selected export
- WHEN both execute under the same admitted runtime
- THEN both normalize to GuestTrapped but report Instantiation and Invocation respectively

#### Scenario: Admission and binding do not claim instantiation
- GIVEN an invocation has invalid input or its consumer binder refuses
- WHEN the shell returns a failure outcome
- THEN its phase is Admission or Linking respectively and it does not claim successful guest instantiation

### Requirement: Successful instantiation is a separate checkpoint
r[onix.wasmtime_runtime.instantiation_checkpoint] The v4 sliced component shell MUST expose the typed phase of each yielded poll and one successful instantiation checkpoint before the selected export executes. That checkpoint MUST report exact remaining fuel and MUST retain the same store, future, deadline, limits and concurrency ownership. A later poll resumes invocation without restarting instantiation or resetting fuel. Instantiation failure MUST terminate without a successful checkpoint. Only terminal completion may expose owned output after cleanup; repeated polls MUST NOT repeat a checkpoint or execution.

#### Scenario: Start work does not execute the callback before the checkpoint
- GIVEN an admitted component with fuel-consuming start work and an observable selected export
- WHEN its consumer polls until successful instantiation
- THEN start yields identify Instantiation, the checkpoint reports measured remaining fuel, and the selected export has not executed
- AND later polling invokes that export once and retains exact total fuel accounting

#### Scenario: Failed instantiation has no successful checkpoint
- GIVEN an admitted component whose start function traps or exhausts cumulative fuel
- WHEN its retained invocation is polled
- THEN it terminates in Instantiation phase without successful instantiation, invocation output or a fabricated checkpoint

#### Scenario: Cancellation at the checkpoint releases ownership
- GIVEN a successful checkpoint retains the sole runtime concurrency permit
- WHEN its consumer drops the retained invocation
- THEN a new admitted invocation can reserve the permit and the cancelled export has not executed
- AND no rollback of consumer effects or guest cleanup is claimed

### Requirement: Per-memory and total memory bounds are independent
r[onix.wasmtime_runtime.independent_memory_limits] Runtime admission MUST require explicit positive per-memory and aggregate invocation page bounds. The shell MUST independently enforce each bound across module/component initialization and execution, retaining cumulative aggregate accounting across sliced yields. It MUST NOT substitute, multiply, clamp, reset or silently infer either supplied bound. The compatibility profile-document adapter MUST explicitly preserve its previous aggregate cap.

#### Scenario: Two individually admitted memories fit the total budget
- GIVEN two fixed memories each within the admitted per-memory bound and a total within the aggregate bound
- WHEN the actual component instantiates and executes
- THEN both memories are admitted and invocation completes under the same cumulative fuel and output rules

#### Scenario: Either memory bound denies independently
- GIVEN one memory exceeds its per-memory cap while fitting the aggregate cap, or individually valid memories exceed the aggregate cap
- WHEN the actual component instantiates
- THEN the respective allocation is refused without successful invocation or output

#### Scenario: Missing zero bounds fail admission
- GIVEN either supplied memory page bound is zero
- WHEN complete runtime bounds are admitted
- THEN admission refuses the request before engine construction
