# Tasks

- [x] [serial] Add deterministic property-style generators or fixture tables for enforcement allow/deny/error invariants. r[test_hardening.property_fuzz_test_harness.enforcement_properties]
- [x] [serial] Add generated negative cases for resource prefix siblings, id mismatches, malformed policies, and unsupported schemas. r[test_hardening.property_fuzz_test_harness.enforcement_properties.generated_boundaries]
- [x] [serial] Add generated Steel DTO descriptor/receipt validation and hash-stability cases. r[test_hardening.property_fuzz_test_harness.steel_dto_properties]
- [x] [serial] Wire the bounded property tests into the standard release verifier without non-reproducible fuzzing assumptions. r[test_hardening.property_fuzz_test_harness.reproducible_gate]
- [x] [serial] Run Cargo tests, clippy, Nix release checks, Cairn validate/gates, and `git diff --check` before archive. r[test_hardening.property_fuzz_test_harness.reproducible_gate.release_rail]
