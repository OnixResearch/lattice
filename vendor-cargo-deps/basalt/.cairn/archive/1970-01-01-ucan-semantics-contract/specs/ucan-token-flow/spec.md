# Ucan Token Flow Delta

## ADDED Requirements

### Requirement: UCAN semantics fixture contract
r[ucan_semantics_contract.contract] Basalt MUST define a reviewed fixture contract for the UCAN authorization semantics consumed by Basalt policy enforcement.

#### Scenario: Valid semantics fixture passes
r[ucan_semantics_contract.positive]
- GIVEN a fixture declares policy contract ID, requested resource, requested ability, UCAN capability facts, caveat outcome, proof traversal status, replay status, expected enforcement result, and required non-claims
- WHEN the fixture contract is validated
- THEN validation MUST pass and preserve the expected Basalt reason class.

#### Scenario: Malformed semantics fixture fails closed
r[ucan_semantics_contract.negative]
- GIVEN a fixture omits the requested operation, weakens non-claims, or declares incompatible UCAN and Basalt result fields
- WHEN the fixture contract is validated
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Runtime adapter fixture replay
r[ucan_semantics_contract.runtime_adapter] Basalt MUST replay exported UCAN semantics fixtures through the Rust enforcement boundary without reimplementing UCAN authorization semantics in Nickel.

#### Scenario: Allow fixture reaches allowed receipt
r[ucan_semantics_contract.runtime_adapter.allow]
- GIVEN a fixture where Basalt policy permits the operation and UCAN capability facts authorize the same concrete operation
- WHEN the Rust fixture replay runs
- THEN Basalt MUST emit an allowed receipt with the expected reason.

#### Scenario: Negative fixture reaches denied receipt
r[ucan_semantics_contract.runtime_adapter.deny]
- GIVEN a fixture with over-delegation, wildcard request misuse, missing caveat policy, stale proof semantics, or replay denial
- WHEN the Rust fixture replay runs
- THEN Basalt MUST emit a denied or error receipt with the expected fail-closed reason.

### Requirement: UCAN dependency boundary
r[ucan_semantics_contract.boundary] Basalt documentation MUST state which UCAN semantic manifest, Git revision, or exported contract identity was validated and which claims remain outside Basalt.

#### Scenario: Dependency identity is visible
r[ucan_semantics_contract.boundary.identity]
- GIVEN a Basalt release or downstream compatibility receipt is produced
- WHEN a reviewer inspects the UCAN dependency section
- THEN the receipt SHOULD name the UCAN semantic manifest digest when available or the pinned UCAN revision when no manifest exists.

#### Scenario: Non-claims are preserved
r[ucan_semantics_contract.boundary.non_claims]
- GIVEN a Basalt enforcement compatibility report passes
- WHEN the supported claim is rendered
- THEN it MUST state that Basalt does not prove key-resolution truth, revocation backend freshness, cryptographic soundness beyond UCAN's boundary, consumer replay-store durability, or application-wide authorization completeness.

### Requirement: Validation evidence
r[ucan_semantics_contract.validation] The change MUST include contract validation, focused Rust fixture replay, and Cairn validation evidence before archive.

#### Scenario: Positive and negative evidence is saved
r[ucan_semantics_contract.validation.fixtures]
- GIVEN the semantics fixture matrix includes positive and negative cases
- WHEN the validation rail runs
- THEN positive fixtures MUST pass, negative fixtures MUST fail closed, and the saved receipt MUST bind fixture and policy identities.
