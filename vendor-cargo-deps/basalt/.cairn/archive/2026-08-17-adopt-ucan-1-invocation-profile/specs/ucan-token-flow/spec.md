# Final UCAN 1.0 Invocation Adoption Delta

## ADDED Requirements

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
