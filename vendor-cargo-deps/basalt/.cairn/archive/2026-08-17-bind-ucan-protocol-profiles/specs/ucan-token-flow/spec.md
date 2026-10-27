# UCAN Protocol Profile Binding Delta

## ADDED Requirements

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
