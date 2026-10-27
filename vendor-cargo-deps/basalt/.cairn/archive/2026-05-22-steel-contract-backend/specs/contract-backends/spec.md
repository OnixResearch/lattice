# Contract Backends Specification Delta

## Purpose

Defines the `contract-backends` capability for UCAN contract language backends, including Nickel compatibility, Steel Scheme support, and higher-order callable contract descriptors.

## Requirements

### Requirement: The crate MUST expose a backend-neutral UCAN contract envelope for Nickel and Steel contracts.

r[contract_backends.steel_contract_backend.shared_envelope] The crate MUST expose a backend-neutral UCAN contract envelope that identifies the contract backend, contract id/version, normalized source hash, input schema, output schema, and receipt schema version.

#### Scenario: Nickel and Steel backend identity

r[contract_backends.steel_contract_backend.shared_envelope.backend_identity]

- GIVEN a consumer loads a UCAN contract envelope
- WHEN the envelope names a supported `nickel` or `steel` backend
- THEN Rust validation preserves the backend identity and source binding before evaluating or trusting backend-specific evidence

#### Scenario: Unsupported backend denial

r[contract_backends.steel_contract_backend.shared_envelope.unsupported_backend]

- GIVEN a consumer loads a UCAN contract envelope with an unsupported backend id
- WHEN Rust validation runs
- THEN the crate returns a deterministic denied or error receipt without attempting backend evaluation

### Requirement: Steel contract evaluation MUST use bounded DTO inputs and structured receipts only.

r[contract_backends.steel_contract_backend.steel_dto_boundary] Steel contract evaluation MUST use bounded DTO inputs and structured receipts only, with Rust retaining authority enforcement and receipt validation.

#### Scenario: Steel predicate request

r[contract_backends.steel_contract_backend.steel_dto_boundary.predicate_request]

- GIVEN a Steel-backed contract is selected for a UCAN caveat or capability predicate
- WHEN the evaluation request is built
- THEN the request contains only bounded contract DTO fields, declared input data, source/hash metadata, and declared capability context

#### Scenario: No ambient Steel authority

r[contract_backends.steel_contract_backend.steel_dto_boundary.no_ambient_authority]

- GIVEN Steel contract code is evaluated through the crate's backend adapter
- WHEN the code requests filesystem, process, network, credential, mutation, or undeclared host capability access
- THEN the adapter denies the request or omits the host function so Rust emits a deterministic denied/error receipt

### Requirement: Higher-order contract descriptors MUST make callable/predicate values explicit and hash-bound.

r[contract_backends.steel_contract_backend.higher_order_descriptors] Higher-order contract descriptors MUST make callable or predicate values explicit with stable callable id, arity, argument contracts, return contract, required UCAN capabilities, determinism/effect class, source/content hash, and redaction class.

#### Scenario: Callable descriptor accepted

r[contract_backends.steel_contract_backend.higher_order_descriptors.callable_descriptor]

- GIVEN a Steel contract exports a predicate or callable used by a UCAN caveat
- WHEN Rust validates the higher-order descriptor
- THEN arity, argument contracts, return contract, capability requirements, determinism/effect class, source hash, and redaction class are present before invocation is allowed

#### Scenario: Opaque callable rejected

r[contract_backends.steel_contract_backend.higher_order_descriptors.opaque_callable_rejected]

- GIVEN a Steel contract attempts to pass an opaque closure or unhashable callable across the UCAN boundary
- WHEN Rust validates the higher-order descriptor
- THEN validation fails closed with a deterministic issue code and without invoking the callable

### Requirement: Steel backend receipts MUST be deterministic and authority-bound.

r[contract_backends.steel_contract_backend.receipts] Steel backend receipts MUST bind backend id, contract id/version, normalized source hash, request hash, decision, caveat evidence, declared effect class, and required capability checks.

#### Scenario: Receipt hash stability

r[contract_backends.steel_contract_backend.receipts.hash_stability]

- GIVEN the same Steel contract request is evaluated with the same source, policy, capabilities, and input DTO
- WHEN receipt hashing runs
- THEN the receipt hash is stable across repeated runs and excludes display-only or secret material

#### Scenario: Missing authority evidence rejected

r[contract_backends.steel_contract_backend.receipts.missing_authority_evidence]

- GIVEN a Steel backend receipt claims an allow decision for an effectful or capability-bearing callable
- WHEN required UCAN capability evidence is absent or mismatched
- THEN Rust validation rejects the receipt and returns a deterministic denied/error result

### Requirement: Nickel compatibility MUST remain intact when Steel support is added.

r[contract_backends.steel_contract_backend.nickel_compatibility] Nickel compatibility MUST remain intact when Steel support is added.

#### Scenario: Existing Nickel policies still validate

r[contract_backends.steel_contract_backend.nickel_compatibility.existing_nickel_policies]

- GIVEN an existing Nickel-authored policy artifact is consumed by the crate
- WHEN Steel backend support is present but not selected
- THEN existing Nickel validation, generated artifact freshness, and allow/deny receipt behavior are preserved
