# Mantle live operation authority

## ADDED Requirements

### Requirement: Exact signed operation binding

r[basalt.mantle_authority.binding] Basalt MUST distinguish every reviewed Mantle operation and bind subject, account, project, endpoint, audience, policy artifact, generation, target and normalized request digest to actual token and holder-signature verification. Only explicitly admitted root issuers MAY grant authority. A request or persisted receipt MUST NOT construct a live session.

#### Scenario: Exact holder invocation

- GIVEN an explicitly trusted issuer, matching policy and valid current holder-signed request
- WHEN initial admission verifies the complete context
- THEN Basalt SHALL return an opaque session only after one final replay commit.

#### Scenario: Substituted or widened authority

- GIVEN a changed operation/context, untrusted issuer or wider requested privilege
- WHEN admission or exact session use is attempted
- THEN Basalt SHALL deny without exposing live authority or spending replay for a rejected pre-admission binding.

### Requirement: Current non-consuming revalidation

r[basalt.mantle_authority.revalidation] Each dispatch MUST revalidate current holder authorization, proof attenuation, time, revocations, selected caveats and loaded policy. Revalidation MUST reject times before initial admission and MUST NOT consume replay again. Consumers SHALL retain monotonic time across observations.

#### Scenario: Current admitted session

- GIVEN an admitted session and unchanged current authority
- WHEN its exact request is revalidated repeatedly
- THEN it SHALL remain usable and the replay commit count SHALL remain one.

#### Scenario: Expired revoked stale or earlier authority

- GIVEN expired or revoked authority, changed generation/policy or time before admission
- WHEN revalidation runs
- THEN it SHALL deny without exposing a new session or committing replay.

### Requirement: Bounded owner and evidence boundary

r[basalt.mantle_authority.boundary] Credentials MUST remain in the zeroizing token owner, sessions MUST NOT be serialized or cloned, and public errors/Debug MUST be redacted. Default and minimal feature contracts SHALL preserve signature, revocation and replay checks. Consumers retain effects, currentness services, durable replay, lifecycle and release authority.

#### Scenario: Persisted observation

- GIVEN a saved request, receipt or redacted projection
- WHEN a consumer attempts to reuse it
- THEN it MUST NOT construct live authority or prove any effect, store publication or release eligibility.
