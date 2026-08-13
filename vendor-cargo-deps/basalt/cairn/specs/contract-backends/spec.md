# Contract Backends Specification

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

### Requirement: Policy-boundary audit profiles

r[basalt.ast_grep_policy_audits.profile]

Basalt MUST define ast-grep audit profiles for syntax-level policy-boundary checks with declared scan scope, release posture, severity, and non-claims.

#### Scenario: Policy audit is advisory

- GIVEN a Basalt ast-grep policy-boundary rule is configured as advisory
- WHEN the release verifier runs the scan
- THEN findings MUST be reported as candidate boundary evidence and MUST NOT block release by default.

### Requirement: Receipt identity

r[basalt.ast_grep_policy_audits.identity]

Basalt MUST bind ast-grep tool identity, rule bundle BLAKE3 identity, scan scope, release verifier run identity, findings summary, and non-claims into audit receipts.

#### Scenario: Receipt references stale rule bundle

- GIVEN a release receipt references an ast-grep rule bundle identity
- WHEN the rule bundle changes before review
- THEN Basalt MUST require a fresh scan receipt before treating the audit as current.

### Requirement: Positive and negative fixtures

r[basalt.ast_grep_policy_audits.fixtures]

Basalt MUST require positive and negative fixtures before an ast-grep policy-boundary rule can become release-blocking.

#### Scenario: Blocking rule lacks negative fixture

- GIVEN a policy-boundary rule is proposed for blocking posture without a negative fixture
- WHEN Basalt validates the rule pack
- THEN validation MUST fail closed and keep the rule advisory.

### Requirement: Release receipt reporting

r[basalt.ast_grep_policy_audits.release_receipts]

Basalt SHOULD report ast-grep audit outcomes in release verification receipts as structural candidate evidence with remediation guidance and explicit non-claims.

#### Scenario: Enforcement bypass candidate is reported

- GIVEN ast-grep reports a candidate direct bypass around the enforcement helper
- WHEN the release receipt is emitted
- THEN the receipt SHOULD identify the rule, span, severity, and remediation note while preserving that authorization was not proven or disproven.

### Requirement: Shell-owned scan execution

r[basalt.ast_grep_policy_audits.shell_boundary]

Basalt MUST keep ast-grep process execution, filesystem scans, and raw output capture in release verifier or CLI shell code.

#### Scenario: Core enforcement runs

- GIVEN core enforcement evaluates a UCAN-backed request
- WHEN policy validation executes
- THEN it MUST NOT depend on ast-grep state, filesystem scans, process execution, or ambient scan results.

### Requirement: Audit validation evidence

r[basalt.ast_grep_policy_audits.validation]

Basalt MUST validate ast-grep policy audits with rule tests, fixture scans, release verifier receipt checks, Cairn gates, and focused release rails.

#### Scenario: Audit change is reviewed

- GIVEN ast-grep policy-boundary audit behavior changes
- WHEN validation evidence is assembled
- THEN evidence MUST include positive fixtures, negative fixtures, release receipt output, lifecycle gates, and focused Basalt verifier results.
