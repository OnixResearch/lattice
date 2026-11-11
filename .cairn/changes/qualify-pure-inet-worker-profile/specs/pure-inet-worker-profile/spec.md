# Pure Interaction-Net Worker Profile Specification

## ADDED Requirements

### Requirement: Reuse the admitted process route
r[lattice.pure_inet_profile.route] Lattice MUST expose the first interaction-net computation only as an explicitly selected sequential profile/pack using the existing `bounded_process` operation runtime, typed operation admission, process runner, and result storage. Multicore or GPU/device reduction MUST require a separate change and qualification. It MUST NOT add a net runtime kind, workflow node, internal reducer, scheduler, automatic acquisition, or alternate-runtime fallback under this capability.

#### Scenario: Exact profile is selected
r[lattice.pure_inet_profile.route.scenario.selected]
- GIVEN an admitted exact local pure-net pack and supported producer/runtime cohort
- WHEN the existing operation route dispatches its declared method
- THEN it invokes one bounded selected worker and returns through existing operation/result ownership.

#### Scenario: Profile or producer is unavailable
r[lattice.pure_inet_profile.route.scenario.unavailable]
- GIVEN a missing profile, unqualified producer schema, unavailable executable, or unsupported enforced limit
- WHEN admission evaluates the invocation
- THEN it refuses before dispatch without installing, building, falling back, or changing other workflow behavior.

### Requirement: Complete immutable computation binding
r[lattice.pure_inet_profile.identity] Each invocation MUST bind exact program and input bytes/schemas, resolved data references, producer/executable/lowering identities, pure-extrinsic semantics, profile/effective limits, platform/numeric behavior, output contract, and current enclosing invocation/attempt identities. Host admission MUST remeasure applicable bytes and reject changed or crossed bindings before dispatch or result use. Serializable identities MUST exclude process addresses and physical net allocation order.

#### Scenario: Program bytes change after profile selection
r[lattice.pure_inet_profile.identity.scenario.changed]
- GIVEN an admitted program identity whose located bytes have changed
- WHEN the host remeasures the program before execution
- THEN the invocation is refused and no stale program or cached result is used.

#### Scenario: Result belongs to another computation
r[lattice.pure_inet_profile.identity.scenario.crossed]
- GIVEN a well-formed response naming another input, program, profile, output contract, or attempt
- WHEN the profile output boundary checks its bindings
- THEN the response is rejected and cannot publish a usable result for this invocation.

### Requirement: Closed pure extrinsic vocabulary
r[lattice.pure_inet_profile.purity] The first selected net profile MUST accept exactly two signed-64-bit inputs and one signed-64-bit output, using only finite acyclic `Input`, `Const`, `Add`, `Sub`, and `Mul` programs with wrapping two's-complement i64 arithmetic under the producer-qualified rule/readback contract. JSON-carried i64 values MUST be canonical exact decimal strings, never JSON numbers. It MUST reject cycles, undeclared operators, extra inputs/outputs, out-of-range or noncanonical integers, and net access to mutable host handles, filesystem, subprocesses, network, credentials, environment, clocks, randomness, devices, dynamic native callbacks, or effect dispatch. The process environment MUST be clear, and native process bounds MUST NOT be described as an OS sandbox.

#### Scenario: Unknown or effectful extrinsic is requested
r[lattice.pure_inet_profile.purity.scenario.denied]
- GIVEN a net requesting an unlisted operation, a host handle, or an I/O extrinsic
- WHEN its program/profile contract is admitted
- THEN admission refuses before evaluation and performs no requested effect.

#### Scenario: Pure declared operation is evaluated
r[lattice.pure_inet_profile.purity.scenario.allowed]
- GIVEN a valid net using only the selected profile's declared pure operations
- WHEN evaluation executes on immutable inputs
- THEN its only result is bounded data under the admitted output contract and it gains no additional operation or effect authority.

#### Scenario: Wide or overflowing integer remains exact
r[lattice.pure_inet_profile.purity.scenario.exact_integer]
- GIVEN canonical decimal i64 inputs outside the exact-integer range of a binary64 JSON-number implementation or arithmetic overflowing signed i64
- WHEN the qualified program uses Add, Sub, or Mul
- THEN parsing preserves exact signed values and the canonical output equals the independent wrapping-i64 reference without floating-point conversion.

#### Scenario: Numeric encoding or program shape is unsupported
r[lattice.pure_inet_profile.purity.scenario.invalid_program]
- GIVEN a JSON number, leading-zero/negative-zero/out-of-range integer string, cycle, extra input/output, or operator outside the closed vocabulary
- WHEN the profile validates program and inputs
- THEN it refuses before reduction rather than normalizing an ambiguous value or selecting a broader evaluator.

### Requirement: Enforced process and evaluator limits
r[lattice.pure_inet_profile.bounds] The profile MUST enforce finite process deadlines and independent input/output stream bounds through the existing runner and MUST enforce its declared evaluator work, arena/live-node, memory/allocation, queue, and output-construction ceilings. Unsupported limits MUST deny qualification. Reservations, ceilings, host observations, producer counters, estimates, and unavailable measurements MUST retain distinct meanings and units.

#### Scenario: Evaluation exhausts a bound
r[lattice.pure_inet_profile.bounds.scenario.exhausted]
- GIVEN a real program exceeding a qualified work, arena, memory, output, or wall-time ceiling
- WHEN the selected worker reaches that ceiling
- THEN execution produces a non-success exhaustion/timeout observation and no complete derived result.

#### Scenario: Actual counter is unavailable
r[lattice.pure_inet_profile.bounds.scenario.unmeasured]
- GIVEN a configured maximum and no valid observation of the corresponding consumed quantity
- WHEN usage evidence is recorded
- THEN it retains the maximum as a limit or reservation and marks actual usage unavailable rather than copying the maximum into a measured field.

### Requirement: Typed results and non-success artifacts
r[lattice.pure_inet_profile.results] Lattice MUST separately retain process disposition, evaluation completeness, output admission, and artifact publication observations. Only an identity-bound complete evaluation satisfying the declared output contract MAY supply a usable derived result. Partial, stuck, invalid, exhausted, failed, cancelled, truncated, and unknown observations MUST NOT satisfy that contract. Permitted diagnostic artifacts MUST use existing bounded storage, sensitivity, retention, and identity rules and remain non-authoritative.

#### Scenario: Exit zero carries partial evaluation
r[lattice.pure_inet_profile.results.scenario.partial]
- GIVEN a worker exits zero with a well-formed envelope reporting incomplete evaluation
- WHEN the profile validates its result
- THEN process completion remains observable but no complete computation result or consumer-success gate is admitted.

#### Scenario: Complete result is independently checked
r[lattice.pure_inet_profile.results.scenario.complete]
- GIVEN a complete response with the exact admitted identities, bounded output, and satisfied schema/comparator obligations
- WHEN host validation and durable publication succeed
- THEN the host exposes the exact derived result/artifact reference without claiming domain truth or whole-campaign completion.

#### Scenario: Output or publication is incomplete
r[lattice.pure_inet_profile.results.scenario.incomplete]
- GIVEN malformed/truncated output, interrupted capture, or an uncertain artifact publication
- WHEN the host classifies the observation
- THEN it retains the appropriate non-success or unknown outcome and does not manufacture a usable result reference.

### Requirement: Cancellation and explicit recomputation
r[lattice.pure_inet_profile.recovery] Lattice MUST preserve existing durable-start-before-dispatch, cancellation, owned teardown, and recovery authority. It MUST reconcile uncertain outcomes before any explicitly admitted fresh recomputation, preserve attempt lineage and consumed budgets, and reconstruct from immutable inputs. The first profile MUST NOT resume raw net pointers, heap/allocator state, process memory, or a VM snapshot.

#### Scenario: Cancellation reaches the real worker
r[lattice.pure_inet_profile.recovery.scenario.cancelled]
- GIVEN the selected production worker is running with an owned descendant
- WHEN current cancellation is admitted
- THEN the host observes termination and teardown before recording confirmed cancellation or reusing resources, and uncertain teardown remains unknown.

#### Scenario: Restart loses the result observation
r[lattice.pure_inet_profile.recovery.scenario.restart]
- GIVEN an owned dispatch may have run but its terminal result is unobserved after restart
- WHEN recovery examines the durable attempt and artifacts
- THEN it preserves uncertainty until reconciliation and never blindly redispatches or restores an address-bearing snapshot.

#### Scenario: Fresh recomputation is authorized
r[lattice.pure_inet_profile.recovery.scenario.recompute]
- GIVEN reconciliation and current policy permit a new computation from retained exact program/input/profile artifacts
- WHEN the owner admits it
- THEN it allocates a fresh invocation with lineage and remaining budget, revalidates authority, and rebuilds the net without asserting that the earlier attempt never executed.

### Requirement: Consumer and campaign ownership remains external
r[lattice.pure_inet_profile.ownership] Pure-net output MUST NOT own canonical storage, query semantics, authorization/erasure barriers, global claims, successor scheduling, acceptance, or publication authority. Mneme MUST revalidate current snapshot/schema/barrier bindings before using a derived result, and the existing Kiln/Lattice owners MUST retain their work, effect, budget, and recovery responsibilities.

#### Scenario: Snapshot is revoked during evaluation
r[lattice.pure_inet_profile.ownership.scenario.stale]
- GIVEN an evaluation finishes for an input whose current authorization, erasure, schema, or projection generation has been superseded
- WHEN the consumer attempts publication or response
- THEN the consumer rejects stale use even if the worker reports complete reduction.

#### Scenario: Output asks to schedule or merge
r[lattice.pure_inet_profile.ownership.scenario.authority]
- GIVEN output contains an instruction or undeclared field requesting another attempt, permission expansion, file edit, or merge
- WHEN the closed result contract is validated
- THEN it is rejected or retained only as inert permitted data and cannot perform the requested operation.

### Requirement: Source-backed profile qualification
r[lattice.pure_inet_profile.verification] Enabling this optional profile MUST require actual selected-executable fixtures through the production admitted route, an independent reference result check, exact cohort identities, positive and negative limit/effect/output/recovery tests, and observed validation evidence. Structural manifests, model-written success, an empty active-pair queue, or borrowed benchmarks MUST NOT establish qualification or default adoption.

#### Scenario: Real worker matches an independent reference
r[lattice.pure_inet_profile.verification.scenario.positive]
- GIVEN the shared two-input acyclic arithmetic workload and an independent Rust wrapping-i64 reference result under the same input/program semantics
- WHEN the exact sequential worker runs through the admitted process route with fresh evaluation state
- THEN semantic outputs agree and the record names measured costs, supported limits, comparator, producer, and host identities.

#### Scenario: Only structural evidence exists
r[lattice.pure_inet_profile.verification.scenario.structural]
- GIVEN only schema checks, profile parsing, mocked worker responses, or an upstream performance claim
- WHEN readiness is evaluated
- THEN the profile remains unqualified and no runtime, confluence, speedup, or production-readiness claim is made.
