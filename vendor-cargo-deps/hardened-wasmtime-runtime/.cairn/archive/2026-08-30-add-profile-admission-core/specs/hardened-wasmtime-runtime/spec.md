# Spec: Profile admission core and strategy-parity rail

## ADDED Requirements

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
