# Resumable component execution delta

## ADDED Requirements

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
