# Ucan Token Flow Specification

## Purpose

Defines the `ucan-token-flow` capability.

## Requirements

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

### Requirement: Raw UCAN enforcement adapter
r[basalt.ucan_token_flow.verify_ucan_enforcement.contract] Basalt MUST provide a fail-closed library adapter for raw compact UCAN enforcement invocations.

#### Scenario: Adapter boundaries are explicit
r[basalt.ucan_token_flow.verify_ucan_enforcement.contract.boundaries]
- GIVEN a caller supplies a raw compact UCAN, loaded Basalt policy, key resolver, proof store, revocation checker, replay admission, caveat policies, verification time, and limits
- WHEN the adapter verifies an enforcement request
- THEN the adapter MUST use only those supplied boundaries and MUST NOT use ambient network, clock, trust, or storage.

#### Scenario: Reviewed source remains pinned locally
r[basalt.ucan_token_flow.verify_ucan_enforcement.contract.source_boundary]
- GIVEN reviewed integrated UCAN commit `277ab768eb532ac0c715a3bbff2a7dc37a702f11` is remotely published
- WHEN Basalt builds the adapter
- THEN Basalt MUST use a checked-in source snapshot with a deterministic import, revision, BLAKE3 manifest, preserved licenses, source parity, and offline validation.

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

### Requirement: Closed OnixOS target action contract [r[basalt.ucan_token_flow.onixos_target_actions.contract]]

Basalt MUST provide closed typed actions for candidate activation, generation persistence, rollback, state-sync publish, state-sync delete, and state-sync administration.

#### Scenario: Exact target action binds [r[basalt.ucan_token_flow.onixos_target_actions.contract.exact]]

- GIVEN a Basalt request has the reviewed contract and ability for one target action
- WHEN the action binder evaluates the request
- THEN it MUST produce the operation profile for that exact action
- AND the profile MUST bind the action, contract, resource, and ability

#### Scenario: Target action is substituted [r[basalt.ucan_token_flow.onixos_target_actions.contract.substitution]]

- GIVEN a request or holder signature was prepared for another target action, contract, resource, or ability
- WHEN the adapter evaluates the invocation
- THEN it MUST deny before replay admission
- AND it MUST NOT create live authority

### Requirement: Reviewed target policy mappings [r[basalt.ucan_token_flow.onixos_target_actions.policy]]

Basalt MUST define one reviewed Nickel policy contract for each closed OnixOS target action.

#### Scenario: State-sync authority is attenuated [r[basalt.ucan_token_flow.onixos_target_actions.policy.attenuation]]

- GIVEN a capability grants state-sync publish
- WHEN delete or administrative authority is requested
- THEN Basalt MUST deny the broader action
- AND publish authority MUST remain unchanged

### Requirement: Identity-bound UCAN source output [r[basalt.ucan_token_flow.onixos_target_actions.source]]

The Basalt flake MUST export the checked-in reviewed UCAN snapshot through `lib.ucanSource`.

#### Scenario: Downstream packages UCAN source [r[basalt.ucan_token_flow.onixos_target_actions.source.downstream]]

- GIVEN a downstream consumer pins one immutable Basalt revision
- WHEN it evaluates `basalt.lib.ucanSource`
- THEN the output MUST refer to that revision's checked-in `vendor/ucan` tree
- AND the existing revision, BLAKE3 manifest, and license checks MUST remain authoritative

### Requirement: Positive and negative target-action evidence [r[basalt.ucan_token_flow.onixos_target_actions.tests]]

Tests MUST cover every exact mapping and action substitution before downstream use.

#### Scenario: Complete mapping matrix runs [r[basalt.ucan_token_flow.onixos_target_actions.tests.matrix]]

- GIVEN all six target action variants
- WHEN the focused test matrix runs
- THEN each exact request MUST bind
- AND every mismatched action, contract, ability, or resource prefix MUST fail closed

### Requirement: Bounded target-action documentation [r[basalt.ucan_token_flow.onixos_target_actions.docs]]

Documentation MUST state the exact mappings, source output, and non-claims.

#### Scenario: Reviewer inspects target authority [r[basalt.ucan_token_flow.onixos_target_actions.docs.non_claims]]

- GIVEN a live result admits one target action
- WHEN a reviewer inspects its documented claim
- THEN the claim MUST exclude effect execution, effect correctness, transport security, replay durability, global revocation freshness, and release eligibility

### Requirement: Target-action validation [r[basalt.ucan_token_flow.onixos_target_actions.validation]]

The change MUST pass focused tests, formatting, Clippy, release verification, downstream compatibility, and canonical Cairn gates.

#### Scenario: Validation completes [r[basalt.ucan_token_flow.onixos_target_actions.validation.complete]]

- GIVEN implementation and evidence are complete
- WHEN the validation rail runs
- THEN every required check MUST pass or report one exact bounded blocker

### Requirement: Target-action lifecycle closeout [r[basalt.ucan_token_flow.onixos_target_actions.archive]]

The change MUST sync accepted requirements and archive only after validation passes.

#### Scenario: Change archives [r[basalt.ucan_token_flow.onixos_target_actions.archive.complete]]

- GIVEN all tasks and validation are complete
- WHEN Cairn sync and archive execute
- THEN the accepted specification MUST contain these requirements
- AND the active change MUST be absent

### Requirement: ucan_token_flow.oidc.secretspec_issuer_key

r[ucan_token_flow.oidc.secretspec_issuer_key]

Basalt SHALL resolve the UCAN issuer signing key through the pinned SecretSpec Rust SDK in an imperative shell under an explicit profile and `basalt-oidc-exchange` scope.

#### Scenario: Issuer key resolves

- GIVEN an allowed metadata-only declaration and provider
- WHEN the exchange service starts or rotates keys
- THEN the imperative shell SHALL obtain the key
- AND the pure verification core SHALL remain independent of provider access

#### Scenario: Issuer key cannot resolve safely

- GIVEN a missing declaration, wrong scope, provider error, timeout, cancellation, or oversized provider output
- WHEN Basalt loads the key
- THEN issuance SHALL remain disabled
- AND Basalt SHALL NOT fall back to a static or ambient key

#### Scenario: Provider process is bounded

- GIVEN a selected SecretSpec provider can launch a child process
- WHEN Basalt resolves the key
- THEN resolution SHALL run inside an owned bounded worker with an explicit timeout and output-size bound

#### Scenario: Writes and caches are disabled

- GIVEN a SecretSpec boundary used for issuer-key resolution
- WHEN Basalt configures the boundary
- THEN write operations and plaintext value caches SHALL be disabled

#### Scenario: Rotation keeps an overlap window

- GIVEN an explicit key identifier is rotated
- WHEN Basalt overlaps the public verification key window
- THEN private material SHALL NOT be exposed

### Requirement: ucan_token_flow.oidc.secretspec_dependency

r[ucan_token_flow.oidc.secretspec_dependency]

Basalt SHALL pin the SecretSpec Rust SDK to a reviewed revision and record its source identity so builds resolve a durable boundary.

#### Scenario: Dependency is pinned

- GIVEN Basalt declares the SecretSpec dependency
- WHEN Cargo and Nix resolve it
- THEN the resolved revision SHALL match the recorded reviewed source identity

### Requirement: ucan_token_flow.oidc.secretspec_fixtures

r[ucan_token_flow.oidc.secretspec_fixtures]

Basalt SHALL maintain positive and negative fixtures for issuer-key resolution from a bounded SecretSpec boundary.

#### Scenario: Fixture suite covers key health

- GIVEN key-load and rotation fixtures run against a bounded SecretSpec declaration
- WHEN resolution is exercised
- THEN valid loads pass and missing, wrong-scope, timeout, oversized, and provider-failure loads fail closed

### Requirement: Admit an exact vendored UCAN source revision
r[ucan_token_flow.vendored_source.exact_revision] The Basalt vendored UCAN importer MUST accept a full reviewed Git object ID for source parity and import operations.

The importer MUST reject moving references, abbreviated object IDs, malformed values, and unavailable commits before it replaces the vendored snapshot.

#### Scenario: Exact reviewed revision is admitted
r[ucan_token_flow.vendored_source.exact_revision.admitted]
- GIVEN an explicit UCAN repository and a full lowercase Git object ID that exists in that repository
- WHEN the operator runs exact source parity or import
- THEN the importer archives that exact revision and records it in the snapshot manifest

#### Scenario: Moving or malformed revision is rejected
r[ucan_token_flow.vendored_source.exact_revision.rejected]
- GIVEN a branch name, abbreviated object ID, uppercase object ID, malformed value, or unavailable commit
- WHEN the operator requests exact source parity or import
- THEN the importer fails before it replaces the vendored snapshot
