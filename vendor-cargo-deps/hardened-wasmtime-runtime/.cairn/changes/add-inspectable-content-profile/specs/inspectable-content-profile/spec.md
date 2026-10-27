# Inspectable content profile

## ADDED Requirements

### Requirement: Inspect metadata before guest execution

r[wasmtime.content_profile.static_admission]

When the strict content profile admits a final Wasm artifact, it MUST validate core structure, explicit feature policy, static ABI metadata, and memory regions without executing guest code. It MUST reject imports, start functions, growth, shared memory, atomics, indirect calls, and recursive direct calls.

#### Scenario: Static transform metadata

GIVEN a core-valid module with admitted constant metadata and disjoint input/data regions
WHEN strict profile admission examines the final bytes
THEN it admits the structural facts without instantiating the module.

#### Scenario: Executable getter

GIVEN a metadata getter with a call or loop
WHEN strict profile admission examines its body
THEN it rejects that metadata without executing the getter.

### Requirement: Separate static findings from runtime guarantees

r[wasmtime.content_profile.claims]

When the runtime reports artifact evidence, it MUST distinguish core validity, structural admission, optional static findings, and execution observations. An output-count finding MUST NOT replace output-range validation or imply semantic correctness.

#### Scenario: Unsupported proof shape

GIVEN a structurally valid module outside an optional checker's recognized proof shape
WHEN the checker reports its result
THEN the evidence records an unestablished static property rather than a proven semantic defect.

#### Scenario: Bounded count with invalid pointer

GIVEN an output size within capacity and a pointer outside memory
WHEN the host validates the returned result
THEN it rejects the result without reading the output range.

### Requirement: Preserve explicit resource and recovery policy

r[wasmtime.content_profile.execution]

When a content artifact executes, the runtime MUST enforce the admitted extraction limits, distinguish rejection from traps and exhaustion, and discard trapped instances. Instance reuse MUST satisfy explicit configuration and recovery contracts.

#### Scenario: Recoverable rejection

GIVEN an admitted reusable instance and an expected input rejection
WHEN the guest returns rejection normally
THEN the host reads no output and permits reuse only under the declared recovery contract.

#### Scenario: Exhausted guest

GIVEN a guest that exceeds its execution budget
WHEN the runtime observes exhaustion
THEN it reports that class separately from accepted empty output and invalidates the affected execution state.

### Requirement: Bind profile and consumer evidence

r[wasmtime.content_profile.identity]

Before consumer adoption, the runtime MUST bind final artifact, profile, ABI, checker, runtime, and limit identities to positive and negative execution evidence. Shared contracts MUST use published immutable revisions.

#### Scenario: Exact admitted artifact

GIVEN matching final bytes and profile bindings
WHEN the consumer supplies its integration evidence
THEN admission retains the bounded scope of those observations.

#### Scenario: Changed artifact

GIVEN an admission record for another artifact identity
WHEN the host receives changed bytes
THEN it requires fresh admission rather than inheriting the previous verdict.
