## ADDED Requirements

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
