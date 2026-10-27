# Tasks

- [x] [serial] Add a Steel adapter-shaped fixture or trait boundary that consumes validated DTO requests and returns structured receipts. r[contract_backends.steel_runtime_adapter_fixture.adapter_fixture]
- [x] [serial] Add a happy-path fixture showing Rust validates adapter request/source/receipt evidence before accept. r[contract_backends.steel_runtime_adapter_fixture.adapter_fixture.evaluation]
- [x] [serial] Add denial fixtures for filesystem/process/network/credential/mutation/undeclared host authority attempts. r[contract_backends.steel_runtime_adapter_fixture.no_ambient_authority]
- [x] [serial] Add malformed/drifted receipt tests for request hash, source hash, authority evidence, effect class, and output schema mismatches. r[contract_backends.steel_runtime_adapter_fixture.receipt_validation]
- [x] [serial] Verify with no-default/CLI feature tests, clippy, Nix release checks, Cairn validate/gates, and `git diff --check` before archive. r[contract_backends.steel_runtime_adapter_fixture.receipt_validation.drift_rejected]
