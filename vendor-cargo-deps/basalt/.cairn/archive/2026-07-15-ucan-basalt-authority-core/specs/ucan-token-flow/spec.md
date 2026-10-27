# Ucan Token Flow Delta

## ADDED Requirements

### Requirement: UCAN/Basalt authority core contract
r[basalt.ucan_token_flow.ucan_basalt_authority_core.contract] Basalt MUST define pure authority core contracts for request identity, contract ID, resource, ability, verified UCAN grant refs, UCAN verification receipt refs, Basalt policy refs, caveat/replay dispositions, deny diagnostics, and bounded non-claims.

#### Scenario: Matching authority input admits request
r[basalt.ucan_token_flow.ucan_basalt_authority_core.fixtures.positive]
- GIVEN an authority input names a Basalt contract, resource, ability, request ref, verified UCAN grant refs, matching UCAN verification receipt refs, and matching Basalt policy refs
- WHEN the authority core evaluates the input
- THEN the decision MUST allow only the exact bounded request described by the input.

#### Scenario: Invalid authority input denies request
r[basalt.ucan_token_flow.ucan_basalt_authority_core.fixtures.negative]
- GIVEN an authority input has missing verified grants, wrong resource, wrong ability, stale verification refs, caveat denial, replay denial, missing policy refs, malformed request refs, or authority overclaims
- WHEN the authority core evaluates the input
- THEN the decision MUST deny with deterministic diagnostics naming the failed binding.

### Requirement: UCAN/Basalt authority validation is pure
r[basalt.ucan_token_flow.ucan_basalt_authority_core.validation] Basalt MUST implement authority decision validation as pure deterministic logic over in-memory request facts, verified UCAN fact refs, policy refs, caveat dispositions, and replay dispositions.

#### Scenario: Shell owns verification inputs
r[basalt.ucan_token_flow.ucan_basalt_authority_core.validation.shell]
- GIVEN a caller parses tokens, resolves keys, loads policy JSON, reads clocks, or consults revocation/replay backends
- WHEN the authority core evaluates a request
- THEN those side effects MUST remain outside the authority core.

### Requirement: UCAN verification remains external input
r[basalt.ucan_token_flow.ucan_basalt_authority_core.ucan_boundary] The authority core MUST consume verified UCAN facts and MUST NOT parse compact tokens, verify signatures, resolve DIDs, read clocks, fetch proofs, consult revocation stores, or evaluate replay stores.

#### Scenario: Raw token is insufficient
r[basalt.ucan_token_flow.ucan_basalt_authority_core.ucan_boundary.raw_token]
- GIVEN a request carries only a raw token or content reference without verified grant refs and UCAN verification receipt refs
- WHEN the authority core evaluates the request
- THEN the decision MUST deny before treating token possession as current authority.

### Requirement: Nickel remains policy review boundary
r[basalt.ucan_token_flow.ucan_basalt_authority_core.policy_boundary] Basalt SHOULD keep human-authored contract policy in Nickel while the authority core consumes deterministic Rust policy values.

#### Scenario: Core does not evaluate Nickel
r[basalt.ucan_token_flow.ucan_basalt_authority_core.policy_boundary.no_runtime_nickel]
- GIVEN a caller loads Basalt policy from a Nickel-exported runtime artifact
- WHEN it invokes the authority core
- THEN Nickel evaluation and filesystem reads MUST occur outside the authority core.

### Requirement: Compatibility adapters preserve existing Basalt requests
r[basalt.ucan_token_flow.ucan_basalt_authority_core.adapters] Basalt SHOULD provide adapters for existing Basalt enforcement requests and UCAN verification receipt references without changing public request fields.

#### Scenario: Existing enforcement request adapts
r[basalt.ucan_token_flow.ucan_basalt_authority_core.adapters.request]
- GIVEN an existing Basalt enforcement request uses current public contract, resource, ability, and capability fields
- WHEN an adapter validates the request
- THEN the adapter SHOULD preserve those fields while routing reusable authority checks through the shared core.

### Requirement: Authority core remains bounded
r[basalt.ucan_token_flow.ucan_basalt_authority_core.docs] Passing authority-core validation MUST NOT grant transport authority, storage authority, persistence authority, replay-store truth, key-resolution trust, downstream business authority, release eligibility, or whole-system safety.

#### Scenario: Boundary is visible
r[basalt.ucan_token_flow.ucan_basalt_authority_core.final_validation]
- GIVEN an authority-core decision allows a request
- WHEN an operator reads the supported claim
- THEN the report MUST state that the decision applies only to the exact contract/resource/ability/request binding supplied as verified input.
