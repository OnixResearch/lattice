## ADDED Requirements

### Requirement: Explicit Wasmtime feature policy
r[onix.wasmtime_runtime.feature_policy] Every runtime profile MUST state the reviewed value of each supported Wasmtime proposal and deterministic code-generation option.

#### Scenario: Complete profile passes
r[onix.wasmtime_runtime.feature_policy.valid]
- GIVEN a complete supported feature policy
- WHEN the core admits the profile
- THEN it returns a deterministic runtime profile identity

#### Scenario: Missing feature fact fails
r[onix.wasmtime_runtime.feature_policy.missing]
- GIVEN a profile that omits one reviewed Wasmtime feature
- WHEN the core admits the profile
- THEN admission fails rather than using a host default

#### Scenario: Unsupported feature fails
r[onix.wasmtime_runtime.feature_policy.unsupported]
- GIVEN a profile that enables a denied or unknown Wasmtime feature
- WHEN the core admits the profile
- THEN admission fails before engine creation

### Requirement: Exact artifact-shape admission
r[onix.wasmtime_runtime.artifact_shape] The runtime MUST validate exact artifact identity, component or module class, world or ABI identity, imports, exports, and runtime profile before instantiation.

#### Scenario: Exact shape passes
r[onix.wasmtime_runtime.artifact_shape.valid]
- GIVEN component bytes and materialization facts that agree on every required identity and shape field
- WHEN runtime admission runs
- THEN the artifact can continue to compilation

#### Scenario: Import drift fails
r[onix.wasmtime_runtime.artifact_shape.import]
- GIVEN runtime imports that differ from admitted materialization facts
- WHEN shape validation runs
- THEN compilation or instantiation is denied

#### Scenario: Export drift fails
r[onix.wasmtime_runtime.artifact_shape.export]
- GIVEN runtime exports that differ from admitted materialization facts
- WHEN shape validation runs
- THEN compilation or instantiation is denied

### Requirement: Cumulative resource limits
r[onix.wasmtime_runtime.resource_limits] Every invocation MUST enforce positive admitted limits for fuel, stack, memory, tables, instances, input, output, and related runtime allocations.

#### Scenario: Bounded invocation completes
r[onix.wasmtime_runtime.resource_limits.valid]
- GIVEN a guest that stays within every admitted limit
- WHEN the shell invokes it
- THEN the result records success and remaining or consumed bounded resources

#### Scenario: Fuel exhaustion is normalized
r[onix.wasmtime_runtime.resource_limits.fuel]
- GIVEN a guest that consumes its admitted fuel
- WHEN invocation traps
- THEN the outcome class is fuel exhausted

#### Scenario: Memory growth fails closed
r[onix.wasmtime_runtime.resource_limits.memory]
- GIVEN a guest that requests memory beyond its admitted store limit
- WHEN growth occurs
- THEN the runtime traps or denies growth with the memory-limit class

#### Scenario: Output stays bounded
r[onix.wasmtime_runtime.resource_limits.output]
- GIVEN a guest that returns output beyond the admitted maximum
- WHEN the host receives the result
- THEN the runtime rejects or truncates according to the explicit profile without unbounded allocation

### Requirement: Epoch deadline enforcement
r[onix.wasmtime_runtime.epoch_deadline] Every invocation MUST bind an explicit epoch deadline budget to an injected host epoch service.

#### Scenario: Invocation before deadline completes
r[onix.wasmtime_runtime.epoch_deadline.valid]
- GIVEN a guest that completes before the admitted epoch deadline
- WHEN the shell invokes it
- THEN deadline enforcement does not change the guest result

#### Scenario: Deadline exhaustion is normalized
r[onix.wasmtime_runtime.epoch_deadline.exhausted]
- GIVEN a guest that remains active after its epoch deadline
- WHEN Wasmtime interrupts it
- THEN the outcome class is deadline exhausted

#### Scenario: Missing epoch service fails
r[onix.wasmtime_runtime.epoch_deadline.service]
- GIVEN a profile that requires epoch interruption without a current host epoch service
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
r[onix.wasmtime_runtime.fixture_compatibility] The repository MUST publish exact positive and negative fixtures for profiles, artifacts, limits, imports, memory exchange, and outcomes.

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
r[onix.wasmtime_runtime.consumer_pilots] Aspen and one independent product consumer MUST pass pilots before stable publication.

#### Scenario: Two worlds pass
r[onix.wasmtime_runtime.consumer_pilots.valid]
- GIVEN an Aspen world and one different Kamacite, Animus, or runtime world
- WHEN both use the extracted runtime
- THEN both preserve their product imports and DTOs in adapters

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
r[onix.wasmtime_runtime.source_inventory] The extraction MUST record exact Aspen source, Wasmtime version, copied fixtures, semantic changes, licenses, and retained consumer authority.

#### Scenario: Inventory is complete
r[onix.wasmtime_runtime.source_inventory.complete]
- GIVEN the extracted source tree
- WHEN provenance validation runs
- THEN every imported mechanism maps to one reviewed Aspen source artifact
