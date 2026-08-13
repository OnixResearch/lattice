# Ucan Token Flow Specification Delta

## ADDED Requirements

### Requirement: Raw UCAN enforcement adapter
r[basalt.ucan_token_flow.verify_ucan_enforcement.contract] Basalt MUST provide a fail-closed library adapter for raw compact UCAN enforcement invocations.

#### Scenario: Adapter boundaries are explicit
r[basalt.ucan_token_flow.verify_ucan_enforcement.contract.boundaries]
- GIVEN a caller supplies a raw compact UCAN, loaded Basalt policy, key resolver, proof store, revocation checker, replay admission, caveat policies, verification time, and limits
- WHEN the adapter verifies an enforcement request
- THEN the adapter MUST use only those supplied boundaries and MUST NOT use ambient network, clock, trust, or storage.

#### Scenario: Reviewed source is not remotely available
r[basalt.ucan_token_flow.verify_ucan_enforcement.contract.source_boundary]
- GIVEN reviewed UCAN commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf` is not remotely fetchable
- WHEN Basalt builds the adapter
- THEN Basalt MUST use a checked-in source snapshot with a deterministic import, revision, BLAKE3 manifest, preserved licenses, and offline validation.

### Requirement: Strong pinned UCAN verification
r[basalt.ucan_token_flow.verify_ucan_enforcement.verification] The adapter MUST use the strongest pinned UCAN shell path that supports caller key, proof, revocation, replay, caveat, time, and limit boundaries.

#### Scenario: All verification stages pass
r[basalt.ucan_token_flow.verify_ucan_enforcement.verification.allow]
- GIVEN a compact UCAN has a valid signature, valid proof chain, admitted issuer and holder, valid time, satisfied caveats, exact scope, clear revocation, and admitted replay
- WHEN the adapter verifies the invocation
- THEN the adapter MUST produce finite verified facts for the exact request.

#### Scenario: Bearer ownership is bounded
r[basalt.ucan_token_flow.verify_ucan_enforcement.verification.secret_ownership]
- GIVEN the adapter receives raw compact bearer bytes
- WHEN UCAN parses and verifies the credential
- THEN the verification path MUST borrow those bytes or erase every owned bearer allocation on drop.

#### Scenario: Holder proves possession
r[basalt.ucan_token_flow.verify_ucan_enforcement.verification.holder_pop]
- GIVEN a token names an audience and a request claims that holder
- WHEN the adapter verifies the invocation
- THEN UCAN MUST verify a holder signature over the canonical exact request binding.

#### Scenario: A verification stage fails
r[basalt.ucan_token_flow.verify_ucan_enforcement.verification.deny]
- GIVEN any required verification stage fails or a required backend cannot answer
- WHEN the adapter verifies the invocation
- THEN the adapter MUST fail closed and MUST NOT create an authority decision from caller assertions.

#### Scenario: Replay is the final commit
r[basalt.ucan_token_flow.verify_ucan_enforcement.verification.replay_final]
- GIVEN low-level verification and non-consuming authorization pass
- WHEN Basalt prepares authority and UCAN performs fresh holder verification
- THEN Basalt MUST complete all fallible policy work and allocations before replay, and successful replay MUST be followed only by direct moves into live authority.

### Requirement: Existing authority core remains the decision boundary
r[basalt.ucan_token_flow.verify_ucan_enforcement.authority] The adapter MUST route only internally produced verified facts through `adapt_enforcement_request` and `evaluate_authority`.

#### Scenario: Exact request is admitted
r[basalt.ucan_token_flow.verify_ucan_enforcement.authority.allow]
- GIVEN UCAN verification admits the exact resource, ability, action, holder, caveats, revocation state, and replay state
- WHEN Basalt policy admits the same contract, resource, and ability
- THEN the authority decision MUST allow only that exact request.

#### Scenario: Basalt policy denies before replay
r[basalt.ucan_token_flow.verify_ucan_enforcement.authority.policy_before_replay]
- GIVEN UCAN preflight authorization passes and Basalt policy denies the exact request
- WHEN the adapter evaluates prospective authority
- THEN the adapter MUST deny before holder replay and MUST make zero replay admission calls.

#### Scenario: Caller capability fields are present
r[basalt.ucan_token_flow.verify_ucan_enforcement.authority.unverified_fields]
- GIVEN a caller supplies unverified capability fields or validity booleans
- WHEN the adapter evaluates the request
- THEN those fields MUST NOT create authority.

#### Scenario: Operation maps to ability
r[basalt.ucan_token_flow.verify_ucan_enforcement.authority.operation_mapping]
- GIVEN a typed normalized operation and a reviewed action-to-ability mapping
- WHEN the adapter binds the UCAN invocation to the Basalt request
- THEN the mapped operation identity MUST match the exact resource, ability, and action binding.

### Requirement: Live non-forgeable admission capability
r[basalt.ucan_token_flow.verify_ucan_enforcement.live_capability] A successful result MUST carry a live in-process capability with private fields and borrowed exact request access.

#### Scenario: Persisted evidence is replayed as authority
r[basalt.ucan_token_flow.verify_ucan_enforcement.live_capability.persistence]
- GIVEN a caller persists a decision receipt or copies preverified fields
- WHEN the caller attempts to recreate admission
- THEN Basalt MUST provide no public constructor or deserialization path that recreates the live capability.

### Requirement: Deterministic redacted receipt
r[basalt.ucan_token_flow.verify_ucan_enforcement.receipt] Each adapter outcome MUST include a deterministic BLAKE3 receipt with no credential or proof bytes.
The receipt digest MUST use its own domain and big-endian `u64` length prefixes for each variable field.
The framed fields MUST include the schema, reason, UCAN revision, and each non-claim.

#### Scenario: Receipt is persisted
r[basalt.ucan_token_flow.verify_ucan_enforcement.receipt.redaction]
- GIVEN an adapter outcome includes compact credential and proof material during evaluation
- WHEN the caller serializes the receipt
- THEN the receipt MUST omit credential bytes, proof bytes, keys, caveat payloads, and exact identity or scope text.

#### Scenario: Oversized credential is rejected
r[basalt.ucan_token_flow.verify_ucan_enforcement.receipt.oversized]
- GIVEN a raw compact credential exceeds the configured byte limit
- WHEN the adapter rejects it before parsing
- THEN the receipt MUST omit credential identity unless the complete input was hashed within a separate explicit bound.

#### Scenario: Identical evaluation repeats
r[basalt.ucan_token_flow.verify_ucan_enforcement.receipt.identity]
- GIVEN all explicit inputs and backend outcomes are identical
- WHEN the adapter repeats the evaluation
- THEN the receipt BLAKE3 identity MUST be identical.

### Requirement: Positive and negative adapter evidence
r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.positive] Tests MUST admit valid direct and delegated invocations through the complete adapter path.

#### Scenario: Valid invocation reaches Basalt authority
r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.positive.allow]
- GIVEN a valid exact invocation and matching Basalt policy
- WHEN the focused test runs
- THEN the live result MUST allow the borrowed exact request.

r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.negative] Tests MUST cover malformed and oversized credentials, signature, proof, issuer, holder, time, caveat, scope, revocation, replay, limit, and policy failures.

#### Scenario: Negative matrix runs
r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.negative.matrix]
- GIVEN each required failure fixture
- WHEN the focused test runs
- THEN each fixture MUST fail closed with its stable typed reason.

r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.receipt] Tests MUST cover receipt redaction, deterministic identity, and persisted-receipt non-authority.

#### Scenario: Receipt evidence cannot recreate authority
r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.receipt.non_authority]
- GIVEN a serialized receipt from an allowed invocation
- WHEN code inspects the public API
- THEN the receipt MUST remain evidence only and MUST NOT construct a live allowed result.

r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.no_decision] Unverifiable input MUST produce no `AuthorityDecision`.

#### Scenario: UCAN input is unverifiable
r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.no_decision.error]
- GIVEN compact input or a required verification backend cannot establish a required fact
- WHEN the adapter returns an error
- THEN the error MUST expose a redacted receipt and MUST expose no authority decision.

### Requirement: Bounded claim documentation
r[basalt.ucan_token_flow.verify_ucan_enforcement.docs] Documentation MUST state that the result proves bounded evaluation only.

#### Scenario: Reviewer inspects non-claims
r[basalt.ucan_token_flow.verify_ucan_enforcement.docs.non_claims]
- GIVEN a reviewer inspects the adapter API or receipt
- WHEN the reviewer reads the supported claim
- THEN the text MUST exclude key discovery, global revocation freshness, replay durability, transport security, policy truth, and effect correctness.

### Requirement: Adapter validation evidence
r[basalt.ucan_token_flow.verify_ucan_enforcement.validation] The change MUST pass focused tests, formatting, Clippy, workspace tests, release checks, and canonical-policy Cairn gates.

#### Scenario: Validation rail completes
r[basalt.ucan_token_flow.verify_ucan_enforcement.validation.complete]
- GIVEN the adapter implementation and tests are complete
- WHEN the validation rail runs
- THEN each required command MUST pass or record an exact bounded blocker.

### Requirement: Native lifecycle closeout
r[basalt.ucan_token_flow.verify_ucan_enforcement.archive] The change MUST sync accepted requirements and archive only after validation passes.

#### Scenario: Change is archived
r[basalt.ucan_token_flow.verify_ucan_enforcement.archive.complete]
- GIVEN implementation tasks and validation are complete
- WHEN Cairn sync and archive execute
- THEN the accepted specification MUST contain these requirements and the active change MUST be absent.
