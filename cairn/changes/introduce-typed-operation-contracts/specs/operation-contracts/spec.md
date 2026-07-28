# Operation Contracts

## ADDED Requirements

### Requirement: One typed catalog defines reusable operations

r[lattice.operations.catalog] Lattice MUST use typed Nickel as the authored source for versioned operation types and MUST revalidate generated or decoded catalog values in Rust before discovery, planning, or execution.

#### Scenario: Valid operation type is discovered

r[lattice.operations.catalog.valid]
- GIVEN an operation type has one canonical ID, exact revision, named methods, argument and output contracts, preflight declarations, capability classes, runtime bindings, positive bounds, and required non-claims
- WHEN catalog validation and discovery run
- THEN Lattice MUST return deterministic safe metadata and a domain-separated BLAKE3 type identity
- AND discovery MUST NOT load the executor, resolve credentials, or perform an operation effect.

#### Scenario: Catalog entry is malformed or stale

r[lattice.operations.catalog.invalid]
- GIVEN a catalog contains duplicate IDs, unknown fields, unsupported schema versions, stale generated data, empty methods, unsafe bounds, missing non-claims, or conflicting runtime declarations
- WHEN Rust catalog validation runs
- THEN Lattice MUST reject the affected catalog with ordered bounded diagnostics
- AND it MUST NOT register, display as usable, plan, promote, or execute the invalid operation.

### Requirement: Operation instances remain data-only

r[lattice.operations.instances] Lattice MUST bind each operation instance to one exact operation-type revision and MUST limit instance data to admitted target facts, tags, method defaults, and opaque credential references.

#### Scenario: Valid instance binds one type

r[lattice.operations.instances.valid]
- GIVEN an instance names an admitted type revision and supplies contract-valid target facts, safe tags, defaults, and credential references
- WHEN instance validation runs
- THEN Lattice MUST return one deterministic instance identity bound to the exact type revision
- AND display aliases and source paths MUST remain outside operation authority identity.

#### Scenario: Instance attempts to change behavior

r[lattice.operations.instances.override_denied]
- GIVEN an instance adds a method, changes a schema, widens a capability, selects an undeclared runtime, injects a command, embeds plaintext credentials, or removes a required non-claim
- WHEN instance validation runs
- THEN Lattice MUST reject the instance before store, credential, process, network, filesystem, or workflow effects
- AND it MUST NOT reinterpret the override as ordinary target data.

### Requirement: Invocation admission is pure and fail-closed

r[lattice.operations.preflight] Lattice MUST plan each operation invocation through pure deterministic logic over admitted type, instance, method, argument, runtime, capability, credential-reference, and bound facts before any effect.

#### Scenario: Exact invocation is admitted

r[lattice.operations.preflight.admitted]
- GIVEN the type, instance, method, arguments, runtime binding, capability policy, credential-reference facts, and bounds agree
- WHEN invocation admission runs
- THEN Lattice MUST emit one canonical invocation plan with exact input and output identities
- AND planning MUST perform no Nickel evaluation, file read, credential resolution, process, network, clock, persistence, logging, or UI effect.

#### Scenario: Preflight fact is missing or contradictory

r[lattice.operations.preflight.denied]
- GIVEN a method is absent, arguments or outputs violate their contracts, capability policy denies, runtime binding is unavailable, credential use is invalid, identity is stale, or a bound is exceeded
- WHEN invocation admission runs
- THEN Lattice MUST return deterministic blockers and no dispatchable invocation plan
- AND no fallback method, runtime, provider, credential, target, or output contract MAY be selected.

### Requirement: Direct and workflow execution share one operation plan

r[lattice.operations.direct_run] Lattice MUST use the same canonical invocation-plan shape for direct operation runs and operation-backed workflow nodes while retaining runtime, credential, budget, persistence, recovery, and effect authority.

#### Scenario: Direct read-only invocation succeeds

r[lattice.operations.direct_run.success]
- GIVEN an operator selects an admitted instance and method with contract-valid arguments and available bounded runtime services
- WHEN the direct operation command runs
- THEN Lattice MUST reserve a run identity, dispatch the exact plan, validate typed outputs, and persist the bounded outcome
- AND the resulting plan and output contracts MUST match an equivalent promoted workflow node.

#### Scenario: Executor output violates the contract

r[lattice.operations.direct_run.output_rejected]
- GIVEN an executor returns malformed, truncated, mistyped, oversized, contradictory, or credential-bearing output
- WHEN Lattice validates the operation result
- THEN the invocation MUST fail before downstream composition or success reporting
- AND diagnostics MUST omit raw secret material and unsupported external-system claims.

### Requirement: One-off invocation promotion is preview-first

r[lattice.operations.promotion] Lattice MUST provide deterministic preview-first promotion from one exact admitted invocation into reviewed Nickel workflow source and MUST require explicit execution for publication.

#### Scenario: Exact invocation produces a workflow preview

r[lattice.operations.promotion.preview]
- GIVEN a persisted invocation retains current type, instance, method, argument, runtime, credential-reference, and output-contract identities
- WHEN promotion planning runs
- THEN Lattice MUST emit deterministic Nickel content, destination facts, content BLAKE3, and a safe preview without writing
- AND the generated workflow MUST use the same canonical invocation-plan shape as the direct run.

#### Scenario: Promotion source is stale or unsafe

r[lattice.operations.promotion.denied]
- GIVEN the type revision, instance, method, output contract, credential reference, or source invocation changed, the destination exists, a value is unrepresentable, or plaintext secret material is present
- WHEN promotion planning or execution runs
- THEN Lattice MUST fail before publication and preserve the destination
- AND it MUST NOT silently refresh identities, replace files, inline secrets, or weaken the operation contract.

### Requirement: The first external operation pilot is read-only Cairn

r[lattice.operations.pilot] Lattice MUST prove the operation boundary with a read-only Cairn adapter that uses an explicit executable, capability-confined repository root, named limits, and structured output validation.

#### Scenario: Cairn inspection method completes

r[lattice.operations.pilot.valid]
- GIVEN an admitted Cairn operation instance references a safe repository root and an available reviewed executable
- WHEN a read-only status or lifecycle-validation method runs
- THEN Lattice MUST invoke the exact bounded process plan and validate its structured output before returning typed operation data
- AND process success alone MUST NOT establish lifecycle completion, source correctness, or release readiness.

#### Scenario: Cairn pilot requests unsafe behavior

r[lattice.operations.pilot.denied]
- GIVEN a request selects a mutation command, arbitrary extra arguments, ambient repository discovery, unsafe root, missing executable, timeout, cancellation, truncated output, malformed output, or unsupported method
- WHEN Cairn operation admission or execution runs
- THEN Lattice MUST fail before the prohibited mutation or downstream use
- AND it MUST NOT fall back to a shell node, another executable, another root, or unvalidated text output.
