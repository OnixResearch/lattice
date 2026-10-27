# Named byte-method carrier

## ADDED Requirements

### Requirement: Named component byte methods reuse bounded runtime admission

r[onix.wasmtime_runtime.named_byte_method] The shell MUST admit an explicitly named interface function with synchronous `(string, list<u8>) -> list<u8>` shape before instantiation, using the same exact artifact/interface admission, cumulative limits, fuel, host-transfer ceiling, concurrency ownership, injected epoch service and normalized failures as existing carriers. The method string and byte input MUST share the input byte limit. The carrier MUST install no imports, product policy, ambient clock or alternate engine.

#### Scenario: Exact method completes with owned output

- GIVEN an admitted component exposing the exact named interface and synchronous carrier
- WHEN the consumer invokes it under positive limits
- THEN the shell returns the exact owned output only after automatic cleanup succeeds and releases the store and concurrency permit.

#### Scenario: Missing or incompatible method is refused before initialization

- GIVEN a missing interface/function, wrong parameter/result type or asynchronous function
- WHEN carrier admission runs
- THEN the shell refuses without instantiating guest code.

#### Scenario: Cumulative limits and failures remain enforced

- GIVEN an admitted named carrier and an over-bound input/output, fuel-exhausting or trapping guest
- WHEN execution is attempted
- THEN the shell retains the typed refusal/failure, emits no successful output and releases invocation ownership without resetting cumulative budgets.
