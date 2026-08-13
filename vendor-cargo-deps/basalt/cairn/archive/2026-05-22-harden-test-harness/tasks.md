# Tasks

- [x] [serial] Add a flake release check that delegates to the canonical Rust release verifier. r[crate_consumability.harden_test_harness.flake_release_check]
- [x] [serial] Emit JSON and Markdown release-verification receipts with step status and elapsed time. r[crate_consumability.harden_test_harness.release_receipts]
- [x] [serial] Run integration tests through nextest and keep doctests explicit in the release rail. r[crate_consumability.harden_test_harness.nextest_rail]
- [x] [serial] Add stronger enforcement regression tests for boundary, fail-closed, and multiple-capability cases. r[crate_consumability.harden_test_harness.enforcement_invariants]
- [x] [serial] Add CLI negative smoke tests for denied receipts, malformed policy, and bad capability syntax. r[crate_consumability.harden_test_harness.cli_negative_smokes]
- [x] [serial] Add public DTO/API guardrails and package-content assertions. r[crate_consumability.harden_test_harness.api_and_package_guards]
- [x] [serial] Pin Rust toolchain selection for reproducible release verification. r[crate_consumability.harden_test_harness.toolchain_pin]
- [x] [serial] Run Cairn gates plus updated Rust/Nix verification before archive. r[crate_consumability.harden_test_harness.release_receipts]
