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

### Requirement: UCAN dependency cohorts include protocol profile

r[basalt.ucan_token_flow.protocol_profiles.cohort] Basalt MUST bind every reviewed UCAN dependency cohort to an exact protocol-profile ID, UCAN source revision, UCAN support-catalog BLAKE3 identity, and Basalt policy identity.

#### Scenario: Current vendored cohort is inspected

- GIVEN Basalt uses the reviewed compact UCAN snapshot
- WHEN the dependency cohort is rendered
- THEN it MUST name `onix-compact-ed25519-v1`
- AND it MUST NOT claim final UCAN `1.0.0` compatibility

#### Scenario: Cohort fields disagree

- GIVEN a protocol profile, source revision, support catalog, or policy identity does not match a reviewed cohort
- WHEN cohort admission runs
- THEN admission MUST fail before token parsing or authority evaluation

### Requirement: Protocol profile selection is explicit

r[basalt.ucan_token_flow.protocol_profiles.selection] Basalt UCAN enforcement inputs and semantics fixtures MUST carry a closed admitted protocol-profile value. Basalt MUST NOT infer a profile from untrusted credential bytes.

#### Scenario: Supported profile is supplied

- GIVEN an enforcement input selects a profile admitted by the current policy and source cohort
- WHEN adapter selection runs
- THEN Basalt MUST select only the adapter for that exact profile

#### Scenario: Profile is absent or unknown

- GIVEN the profile is missing, unknown, or unsupported
- WHEN adapter selection runs
- THEN Basalt MUST deny before token parsing
- AND replay admission MUST receive zero calls

### Requirement: Compact enforcement stays profile-bound

r[basalt.ucan_token_flow.protocol_profiles.compact] The current raw compact enforcement adapter MUST accept only the local compact protocol cohort and MUST preserve its existing policy, holder, operation, replay, and live-capability behavior.

#### Scenario: Compact cohort matches

- GIVEN a valid holder-signed compact Invocation and matching Basalt policy select the local compact cohort
- WHEN raw enforcement runs
- THEN the existing adapter MAY produce live authority for the exact request

#### Scenario: Final profile reaches compact adapter

- GIVEN an input selects final UCAN `1.0.0` or supplies final-profile bytes to the compact cohort
- WHEN raw enforcement runs
- THEN the compact adapter MUST reject the request
- AND it MUST NOT auto-dispatch another decoder

### Requirement: Verified UCAN facts enter through an application contract

r[basalt.ucan_token_flow.protocol_profiles.adapter_contract] Basalt MUST define an application-owned verified UCAN authority input that binds the protocol profile, source cohort, subject or holder reference, operation mapping inputs, request binding, public token identities, and bounded non-claims. UCAN wire and vendor codec types MUST NOT enter the Basalt authority core.

#### Scenario: Current compact result is mapped

- GIVEN the compact UCAN adapter returns verified authority for an exact request
- WHEN the adapter projects it into the Basalt input contract
- THEN the contract MUST preserve the admitted profile and request bindings
- AND only bounded application authority facts MAY enter the Basalt core

#### Scenario: Caller supplies assertions without UCAN proof

- GIVEN a caller supplies profile, operation, policy, or validity fields without the required UCAN verified value
- WHEN mapping runs
- THEN Basalt MUST deny
- AND caller assertions MUST NOT create authority

### Requirement: Receipts bind protocol cohorts

r[basalt.ucan_token_flow.protocol_profiles.receipt] Basalt UCAN enforcement receipts MUST include and hash the receipt schema, protocol-profile ID, vendored UCAN revision, UCAN support-catalog identity, Basalt policy identity, operation profile, bounded token and request identities, reason, and non-claims.

#### Scenario: Identical profile evaluation repeats

- GIVEN all explicit profile, cohort, policy, request, and backend facts are identical
- WHEN receipt generation repeats
- THEN the receipt BLAKE3 identity MUST be identical

#### Scenario: Profile is substituted

- GIVEN all fields are equal except the protocol profile
- WHEN receipt generation runs
- THEN the receipt identity MUST differ
- AND the substituted receipt MUST NOT recreate live authority

### Requirement: Profile tests include cross-profile failures

r[basalt.ucan_token_flow.protocol_profiles.tests] Tests MUST cover accepted compact and future final-profile cohorts plus missing, unknown, stale, unsupported, source-mismatch, policy-mismatch, adapter-substitution, command-mapping, and lossy-conversion failures.

#### Scenario: Cross-profile matrix runs

- GIVEN credentials, verified values, adapters, source cohorts, and policies from both profiles
- WHEN every mismatched combination is tested
- THEN each combination MUST fail closed
- AND denied cases MUST make zero state-consuming replay calls

### Requirement: Basalt retains its authority boundary

r[basalt.ucan_token_flow.protocol_profiles.docs] Documentation MUST state that Basalt owns application policy and action mapping while UCAN owns protocol wire and authority semantics. Profile matching MUST NOT prove key trust, revocation freshness, replay durability, policy completeness, transport security, or effect correctness.

#### Scenario: Operator reads profile support

- GIVEN an allowed Basalt result names a UCAN protocol profile
- WHEN the operator reads its claim
- THEN the claim MUST remain limited to the exact reviewed cohort and request
- AND all external authority and evidence boundaries MUST remain visible

### Requirement: Profile integration passes release checks

r[basalt.ucan_token_flow.protocol_profiles.validation] The change MUST pass policy freshness, vendored source checks, positive and negative adapter tests, receipt checks, formatting, Clippy, release verification, Cairn gates, and the repository Nix check.

#### Scenario: Validation rail completes

- GIVEN profile policy, source cohort, adapters, fixtures, receipts, and documentation are current
- WHEN the validation rail runs
- THEN every required check MUST pass or report one exact bounded blocker

### Requirement: Final-profile adoption requires archived upstream evidence

r[basalt.ucan_token_flow.v1_profile.upstream] Basalt MUST admit final UCAN `1.0.0` only from a reviewed vendored source cohort that binds archived UCAN wire and authorization evidence, the exact profile ID, source revision, support-catalog BLAKE3, source manifest, and licenses.

#### Scenario: Upstream evidence is complete

- GIVEN exact archived upstream receipts and the vendored snapshot establish every required final-profile surface
- WHEN cohort admission runs
- THEN Basalt MAY admit the final profile for adapter implementation

#### Scenario: Upstream evidence is incomplete

- GIVEN any required receipt is missing, stale, failed, unsupported, untested, mismatched, or malformed
- WHEN cohort admission runs
- THEN final-profile adoption MUST remain blocked

### Requirement: Final Invocation facts enter through a separate adapter

r[basalt.ucan_token_flow.v1_profile.adapter] The final-profile adapter MUST consume UCAN's live verified Invocation value and MUST project only bounded application authority facts into Basalt's application-owned input. It MUST NOT parse wire bytes or duplicate UCAN semantics.

#### Scenario: Verified Invocation is mapped

- GIVEN UCAN returns live verified final-profile authority for an exact Invocation
- WHEN the adapter maps it
- THEN profile, cohort, subject, executor, command, request, policy, proof, revocation, replay, and public token identity facts MUST remain bound

#### Scenario: Caller supplies assertions

- GIVEN a caller supplies profile, command, policy, proof, revocation, replay, or validity assertions without the UCAN live value
- WHEN the adapter evaluates them
- THEN it MUST deny
- AND assertions MUST NOT create Basalt authority

### Requirement: Final command mapping is closed policy

r[basalt.ucan_token_flow.v1_profile.mapping] Basalt Nickel policy MUST define closed mappings from admitted final-profile commands and complete subject or resource facts to exact actions, contracts, abilities, and operation profiles. Unknown, ambiguous, duplicate, incomplete, or lossy mappings MUST deny.

#### Scenario: Exact command mapping passes

- GIVEN a verified command and complete authority facts match one reviewed mapping
- WHEN the pure mapping core evaluates them
- THEN it MUST return the exact configured `EnforcementAction`

#### Scenario: Resource meaning is incomplete

- GIVEN an action requires an external resource owner fact that is absent or not verified
- WHEN mapping runs
- THEN it MUST deny before prospective authority is created

### Requirement: Final-profile authority retains ordering

r[basalt.ucan_token_flow.v1_profile.authority] Basalt MUST complete all fallible profile mapping, policy evaluation, authority construction, allocation, and receipt preparation before the composed final replay commit. It MUST NOT call a second replay boundary after UCAN state consumption.

#### Scenario: Basalt policy denies

- GIVEN UCAN preflight facts pass but Basalt mapping or policy denies
- WHEN enforcement runs
- THEN final replay MUST receive zero calls
- AND no live Basalt capability MUST exist

#### Scenario: Replay accepts

- GIVEN UCAN and Basalt preflight facts pass and final replay accepts
- WHEN live authority is constructed
- THEN only infallible moves MUST occur afterward

### Requirement: Final-profile tests cover substitutions

r[basalt.ucan_token_flow.v1_profile.tests] Tests MUST cover valid direct and delegated final Invocations plus unknown mapping, incomplete fact, lossy conversion, caller assertion, source mismatch, profile mismatch, policy denial, and compact/final adapter substitution.

#### Scenario: Cross-profile matrix runs

- GIVEN compact and final credentials, live values, adapters, policies, and source cohorts
- WHEN mismatched combinations are evaluated
- THEN every mismatch MUST fail closed
- AND denied cases MUST create no live authority

### Requirement: Final-profile receipts are redacted evidence

r[basalt.ucan_token_flow.v1_profile.receipt] Final-profile receipts MUST bind the Basalt schema, exact UCAN profile and source cohort, support catalog, mapping, policy, operation, bounded token and request identities, reason, and non-claims with BLAKE3. They MUST omit raw credentials, proofs, arguments, metadata, policies, keys, and private identity text.

#### Scenario: Receipt is persisted

- GIVEN final-profile enforcement succeeds or fails
- WHEN the receipt is serialized
- THEN it MUST contain no bearer or request payload
- AND it MUST provide no path to recreate live authority

### Requirement: Final-profile documentation keeps non-claims

r[basalt.ucan_token_flow.v1_profile.docs] Documentation MUST state that UCAN owns wire and authority verification while Basalt owns application mapping and policy. An allow MUST NOT prove key trust, external resource ownership, revocation freshness, replay durability, transport security, policy completeness, effect execution, or effect correctness.

#### Scenario: Operator reads an allow

- GIVEN final-profile Basalt enforcement allows one exact request
- WHEN the operator reads its supported claim
- THEN the exact cohort and mapping MUST be visible
- AND retained caller boundaries MUST remain explicit

### Requirement: Final-profile adoption passes release checks

r[basalt.ucan_token_flow.v1_profile.validation] The change MUST pass upstream evidence checks, vendored source checks, Nickel freshness, positive and negative adapter tests, receipt tests, formatting, Clippy, release verification, Cairn gates, and the repository Nix check.

#### Scenario: Validation completes

- GIVEN source, evidence, policy, adapters, fixtures, receipts, and documentation are current
- WHEN the validation rail runs
- THEN every required check MUST pass or report one exact bounded blocker

### Requirement: ucan_token_flow.oidc.versioned_policy

r[ucan_token_flow.oidc.versioned_policy]

Basalt SHALL evaluate OIDC exchange under a typed, versioned Nickel policy that names trusted issuers, audiences, algorithms, claim predicates, freshness, network bounds, cache bounds, Mantle capabilities, and UCAN lifetime limits.

#### Scenario: Policy export is valid

- **GIVEN** an accepted OIDC exchange policy
- **WHEN** Basalt exports it for runtime use
- **THEN** the export SHALL be canonical and BLAKE3-identified
- **AND** it SHALL contain no private key, provider credential, OIDC token, or secret value

#### Scenario: Required policy field is absent

- **GIVEN** a policy without an issuer, audience, algorithm, freshness, claim, capability, or network bound required by its profile
- **WHEN** the contract validates it
- **THEN** validation SHALL fail before service startup

### Requirement: ucan_token_flow.oidc.pure_verification

r[ucan_token_flow.oidc.pure_verification]

Basalt SHALL verify OIDC structure, algorithm, key use, signature, issuer, audience, time claims, selected identity claims, and requested capability in a pure core with explicit time and public-key inputs.

#### Scenario: GitHub token satisfies policy

- **GIVEN** a correctly signed token from the configured issuer with the required audience, fresh time claims, accepted repository and workflow claims, and an allowed capability request
- **WHEN** the core verifies it
- **THEN** it SHALL return bounded identity facts and an attenuated issuance plan

#### Scenario: One verification condition fails

- **GIVEN** a malformed token, disallowed algorithm, wrong key use, bad signature, wrong issuer or audience, stale time claim, wrong claim type, claim mismatch, or excessive request
- **WHEN** the core verifies it
- **THEN** it SHALL return a stable typed rejection
- **AND** it SHALL NOT produce an issuance plan

### Requirement: ucan_token_flow.oidc.bounded_jwks

r[ucan_token_flow.oidc.bounded_jwks]

Basalt SHALL retrieve OIDC discovery and JWKS documents only from configured HTTPS origins under named transport, redirect, document, key-count, freshness, and refresh bounds.

#### Scenario: Unknown key rotates safely

- **GIVEN** a token with an unknown `kid` and a configured issuer
- **WHEN** the current key set is still eligible for bounded refresh
- **THEN** the shell MAY fetch one fresh bounded key set
- **AND** the core SHALL verify only a policy-allowed candidate key

#### Scenario: Discovery or key retrieval is unsafe

- **GIVEN** an unapproved redirect, issuer mismatch, TLS failure, timeout, oversized document, malformed document, excessive key set, or stale unavailable cache
- **WHEN** the shell retrieves verification material
- **THEN** exchange SHALL fail closed
- **AND** it SHALL NOT use an ambient or unconfigured key

### Requirement: ucan_token_flow.oidc.attenuated_ucan

r[ucan_token_flow.oidc.attenuated_ucan]

Basalt SHALL issue a UCAN only after successful OIDC verification, and the UCAN audience, operations, project, resource scope, and lifetime SHALL NOT exceed policy or verified OIDC validity.

#### Scenario: Allowed build capability is exchanged

- **GIVEN** verified identity facts and a request for an allowed Mantle build resource
- **WHEN** Basalt issues the UCAN
- **THEN** the token SHALL target the configured Mantle audience
- **AND** it SHALL contain only the granted capability scope
- **AND** it SHALL expire no later than both applicable limits

#### Scenario: Caller requests broader authority

- **GIVEN** a request for an unapproved operation, project, resource, audience, or lifetime
- **WHEN** attenuation evaluates it
- **THEN** exchange SHALL reject or reduce it only as the explicit policy defines
- **AND** it SHALL never silently grant the broader request

### Requirement: ucan_token_flow.oidc.replay_boundary

r[ucan_token_flow.oidc.replay_boundary]

Basalt SHALL bind an exchange to the requested Mantle audience and capability and SHALL apply the configured replay policy before issuance.

#### Scenario: One-time exchange repeats

- **GIVEN** a policy that requires one-time exchange and a replay key already committed for the same public token facts and request
- **WHEN** the caller repeats exchange
- **THEN** Basalt SHALL reject the replay
- **AND** it SHALL NOT issue another UCAN

#### Scenario: Replay storage is unavailable

- **GIVEN** a policy that requires replay prevention and an unavailable replay store
- **WHEN** exchange reaches replay admission
- **THEN** exchange SHALL fail closed

### Requirement: ucan_token_flow.oidc.redacted_evidence

r[ucan_token_flow.oidc.redacted_evidence]

Basalt SHALL record bounded exchange decisions without raw OIDC tokens, UCAN bearer tokens, private keys, provider credentials, full JWK sets, or secret-derived hashes.

#### Scenario: Exchange succeeds or fails

- **GIVEN** any terminal exchange outcome
- **WHEN** Basalt emits evidence
- **THEN** evidence MAY include policy, issuer profile, selected public subject, requested and granted capability, key identifier, and reason code
- **AND** it SHALL omit bearer and secret material

### Requirement: Explicit release scratch storage
r[basalt.ucan_token_flow.process_release_scratch] The release verifier MAY accept `RELEASE_VERIFY_SCRATCH_DIR` as an absolute normalized UTF-8 non-root path. It MUST reject invalid configuration before checks. It SHALL relocate only its five owned scratch paths, preserve distinct target directories, and record actual commands and Cargo targets. The option MUST NOT skip checks or confer filesystem custody. Unset configuration SHALL preserve existing paths.

#### Scenario: Home-backed verification
- GIVEN an explicit valid scratch directory with available storage
- WHEN the full verifier runs
- THEN bootstrap outputs and executions use the same relocated paths and all existing checks remain selected

#### Scenario: Invalid configuration
- GIVEN an empty, relative, root, control-bearing or non-normalized scratch directory
- WHEN the verifier starts
- THEN it rejects before starting checks

### Requirement: Exact process execution grant
r[basalt.ucan_token_flow.process_execution] Basalt MUST bind process execution to a distinct closed action, contract, ability and exact normalized context resource. It SHALL preserve holder-signature, revocation and final replay checks. A document or deployment grant SHALL NOT substitute for execution authority.

#### Scenario: Exact signed invocation
- GIVEN an explicit execution policy and a valid holder-signed execution request
- WHEN current supplied verification services allow that exact request
- THEN Basalt returns private live authority after exactly one replay commit and runs no process

#### Scenario: Wrong purpose or changed context
- GIVEN document authority or a signature for another execution context
- WHEN execution admission runs
- THEN Basalt denies before replay and exposes no live authority

#### Scenario: Malformed execution resource
- GIVEN a missing, uppercase, truncated, oversized or non-hex context reference
- WHEN the pure action binder runs
- THEN it denies without token parsing or effects

#### Scenario: Revoked or repeated invocation
- GIVEN revoked credentials or a replayed holder invocation
- WHEN current verification runs
- THEN it denies without returning a live grant

### Requirement: Explicit policy activation
r[basalt.ucan_token_flow.process_policy] Process execution SHALL require a separately selected policy. The packaged default policy and existing operation identities MUST remain unchanged. Receipts SHALL NOT authenticate native inputs or confer effect, custody, durability or promotion guarantees.

#### Scenario: Default policy remains restrictive
- GIVEN otherwise valid process authority and the packaged default policy
- WHEN admission runs
- THEN it denies before replay because execution policy was not selected

#### Scenario: Receipt is persisted
- GIVEN a successful execution-purpose verification receipt
- WHEN it is stored or read
- THEN it cannot construct live authority or prove that any process ran
