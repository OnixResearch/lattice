# Design: Steel runtime adapter fixture

## Approach

Basalt will keep the core Steel contract as DTO-only and add a test adapter seam that can be satisfied by a fake or minimal Steel runtime fixture. The fixture should exercise the same request/response shape a downstream `steel-lang` adapter would use while preventing the adapter from making authorization decisions independently.

## Boundary rules

- Rust constructs and validates `SteelEvaluationRequest`.
- The adapter receives only declared DTO data, source metadata, and capability context.
- The adapter returns `SteelEvaluationReceipt` with structured evidence.
- Rust validates receipt hash, authority evidence, source binding, decision shape, and effect class before trusting the result.

## Verification

Tests must include one happy path and several fail-closed paths: unsupported host capability, malformed receipt, missing authority evidence, mismatched request hash, and source-hash drift.
