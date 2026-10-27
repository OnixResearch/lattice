# Contract Backends Specification Delta

## Purpose

Extends the `contract-backends` capability with a fixture-backed Steel runtime adapter seam.

## Requirements

### Requirement: Basalt MUST prove the Steel DTO boundary against a runtime-adapter-shaped fixture.

r[contract_backends.steel_runtime_adapter_fixture.adapter_fixture] Basalt MUST prove the Steel DTO boundary against a runtime-adapter-shaped fixture while keeping Rust responsible for authority and receipt validation.

#### Scenario: Adapter-shaped evaluation

r[contract_backends.steel_runtime_adapter_fixture.adapter_fixture.evaluation]

- GIVEN a Steel-backed contract request is prepared by Rust
- WHEN an adapter-shaped fixture evaluates the request and returns a structured receipt
- THEN Rust validates the request hash, source hash, decision, evidence, and capability binding before accepting the result

### Requirement: The Steel adapter fixture MUST deny or omit ambient host authority.

r[contract_backends.steel_runtime_adapter_fixture.no_ambient_authority] The Steel adapter fixture MUST deny or omit filesystem, process, network, credential, mutation, and undeclared host capability authority.

#### Scenario: Host authority blocked

r[contract_backends.steel_runtime_adapter_fixture.no_ambient_authority.blocked_host_action]

- GIVEN Steel code or a fixture requests an undeclared host action
- WHEN the adapter boundary processes the request
- THEN the result is a deterministic denied/error receipt and no host side effect is performed

### Requirement: Runtime-adapter receipts MUST fail closed on malformed or drifted evidence.

r[contract_backends.steel_runtime_adapter_fixture.receipt_validation] Runtime-adapter receipts MUST fail closed when request hashes, source hashes, capability evidence, effect class, or output schemas do not match the validated request.

#### Scenario: Drifted receipt rejected

r[contract_backends.steel_runtime_adapter_fixture.receipt_validation.drift_rejected]

- GIVEN an adapter returns a receipt with mismatched request/source hashes, missing authority evidence, or malformed output
- WHEN Rust validates the receipt
- THEN Basalt rejects it deterministically without treating the adapter decision as authoritative
