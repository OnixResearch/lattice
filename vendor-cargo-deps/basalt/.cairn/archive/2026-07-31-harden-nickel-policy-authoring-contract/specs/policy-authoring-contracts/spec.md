# Policy Authoring Contract Specification

## ADDED Requirements

### Requirement: Basalt policy source uses shared hardened primitives

r[basalt.policy_authoring_contracts.shared_primitives] Basalt MUST provide shared Nickel contracts for exact policy schema, normalized ids, supported backend ids, unique non-empty collections, map-entry identity alignment, and deterministic validator issue classes.

#### Scenario: Equivalent identity fields use different rules

- GIVEN contract ids and backend ids are validated by separate policy fragments
- WHEN source-contract validation runs
- THEN both MUST use the shared identity primitive and produce the same issue class for malformed ids.

### Requirement: The whole Nickel policy is coherent before export

r[basalt.policy_authoring_contracts.whole_policy] The Nickel policy contract MUST require the exact supported schema, at least one contract, matching map keys and embedded ids, unique contract/backend identities, supported backend metadata, and non-empty unique resource-prefix and ability allowlists.

#### Scenario: Source policy has ambiguous identity or empty authority

- GIVEN a policy has a key/id mismatch, duplicate identity, unknown backend, no contracts, or an empty resource/ability allowlist
- WHEN Nickel evaluates the whole policy
- THEN export MUST fail before generated JSON is updated.

### Requirement: Resource and ability contracts remain lexical

r[basalt.policy_authoring_contracts.lexical_values] Nickel MUST validate resource-prefix and ability values against bounded normalized lexical preconditions shared with Rust, but MUST NOT implement UCAN wildcard containment, attenuation, caveat, proof, replay, token, or authorization semantics.

#### Scenario: Lexically malformed ability is authored

- GIVEN an ability is empty, contains control or whitespace ambiguity, exceeds its declared bound, or has malformed path segments
- WHEN Nickel validates the policy
- THEN export MUST fail with a lexical issue
- AND the failure MUST NOT be described as a UCAN authorization decision.

### Requirement: Source and runtime identity checks are layered

r[basalt.policy_authoring_contracts.layered_identity] Nickel MUST reject contract/backend key-id disagreement in reviewed source, and Rust MUST independently reject duplicate, missing, mismatched, or unsupported identities in every externally supplied policy projection.

#### Scenario: Consumer bypasses Nickel

- GIVEN external JSON has a contract or backend key that disagrees with its embedded id
- WHEN `parse_policy_json` and runtime identity-map admission run
- THEN Rust MUST reject the artifact before enforcement regardless of whether any Nickel source exists.

### Requirement: Generated policy meaning matches the Rust consumer

r[basalt.policy_authoring_contracts.projection_parity] Basalt MUST retain exact Nickel export freshness and MUST additionally compare the generated artifact's normalized Rust DTO and identity-map meaning against the supported contract, without evaluating Nickel at runtime.

#### Scenario: Fresh projection is DTO-incompatible

- GIVEN a freshly exported policy uses a contract-accepted field or backend combination the Rust consumer cannot admit
- WHEN projection parity validation runs
- THEN validation MUST fail before packaging with contract/DTO mismatch evidence.

### Requirement: Authoring and external entry paths have fixtures

r[basalt.policy_authoring_contracts.fixtures] Basalt MUST maintain positive and negative Nickel fixtures plus corresponding external-JSON failure fixtures for schema, backend, identity, collection, lexical, freshness, and DTO parity behavior.

#### Scenario: New source invariant has no bypass test

- GIVEN the Nickel policy gains a new authoring invariant
- WHEN fixture coverage is checked
- THEN the suite MUST include a valid source case, an invalid source case, and an external JSON case proving Rust still fails closed when Nickel is bypassed.

### Requirement: Nickel source contracts do not own authorization

r[basalt.policy_authoring_contracts.runtime_boundary] Nickel MUST own reviewed policy shape and checked projection inputs, while Rust/UCAN MUST retain token verification, capability and wildcard semantics, caveat/proof/replay decisions, backend execution, enforcement, and receipt authority.

#### Scenario: Policy source passes all contracts

- GIVEN a policy source and projection pass contract and parity validation
- WHEN the result is summarized
- THEN it MUST NOT claim that any request is authorized, any token is valid, any backend is truthful, or any runtime receipt is correct.
