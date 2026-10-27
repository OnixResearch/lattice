# Runtime extraction boundary correction

## MODIFIED Requirements

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

### Requirement: Reviewed source inventory
r[onix.wasmtime_runtime.source_inventory] The extraction MUST record exact Aspen source, Wasmtime version, copied fixtures, semantic changes, licenses, and retained consumer authority. A correction MUST identify its published baseline and newly owned mechanisms, preserving historical evidence rather than presenting it as a fresh run.

#### Scenario: Inventory is complete
r[onix.wasmtime_runtime.source_inventory.complete]
- GIVEN the corrected source tree and its historical extraction inventory
- WHEN provenance validation runs
- THEN imported mechanisms remain attributed and correction evidence names its baseline, exact cohort, semantic changes, owners and observed commands

