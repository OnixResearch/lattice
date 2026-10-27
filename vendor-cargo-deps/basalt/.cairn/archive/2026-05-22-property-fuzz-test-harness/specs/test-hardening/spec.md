# Test Hardening Specification

## Purpose

Defines the `test-hardening` capability for expanding Basalt's deterministic regression coverage.

## Requirements

### Requirement: The crate MUST include deterministic property-style tests for enforcement invariants.

r[test_hardening.property_fuzz_test_harness.enforcement_properties] The crate MUST include deterministic property-style tests for enforcement invariants across resources, abilities, contract ids, and capability grants.

#### Scenario: Generated enforcement boundaries

r[test_hardening.property_fuzz_test_harness.enforcement_properties.generated_boundaries]

- GIVEN generated enforcement requests and capability grants
- WHEN the property test harness evaluates allowed, denied, malformed, and mismatched cases
- THEN Basalt returns deterministic allow, deny, or error outcomes without panics and without prefix-sibling authorization leaks

### Requirement: The Steel descriptor and receipt DTOs MUST have generated negative and hash-stability coverage.

r[test_hardening.property_fuzz_test_harness.steel_dto_properties] The Steel descriptor and receipt DTOs MUST have generated negative and hash-stability coverage for authority-relevant fields.

#### Scenario: Generated Steel DTO cases

r[test_hardening.property_fuzz_test_harness.steel_dto_properties.generated_steel_cases]

- GIVEN generated callable descriptors, evaluation requests, and receipts
- WHEN required hash, schema, arity, effect, or capability fields are missing or changed
- THEN validation fails closed or hash outputs change deterministically according to the DTO contract

### Requirement: Property and fuzz-style coverage MUST be reproducible in the standard release rail.

r[test_hardening.property_fuzz_test_harness.reproducible_gate] Property and fuzz-style coverage MUST be reproducible in the standard release rail.

#### Scenario: Bounded release execution

r[test_hardening.property_fuzz_test_harness.reproducible_gate.release_rail]

- GIVEN maintainers run the documented release verifier
- WHEN property-style tests execute
- THEN the cases are bounded, deterministic, and included in normal Cargo/Nix verification without requiring live services or ambient randomness
