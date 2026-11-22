# Kiln Authority Adapter

## ADDED Requirements

### Requirement: Closed Kiln operation mapping

r[basalt.kiln_authority.actions] Basalt MUST expose a closed Kiln operation vocabulary in
which each supported operation maps to exactly one reviewed contract, one UCAN ability and
one resource prefix. Unknown operations, generic document or process authority, and an
action from another downstream consumer MUST NOT authorize a Kiln operation.

#### Scenario: Every supported operation is distinct

r[basalt.kiln_authority.actions.scenario.distinct]
- GIVEN the closed claim, renewal, dispatch, submission, verification, cancellation, publication, integration and lifecycle operations
- WHEN their contract identifiers, abilities and prefixes are inspected
- THEN each operation MUST have exactly one reviewed mapping
- AND no operation MUST share a contract or ability with another operation

#### Scenario: One operation cannot stand in for another

r[basalt.kiln_authority.actions.scenario.substitution]
- GIVEN a live session admitted for one Kiln operation
- WHEN the session is checked against a different operation, target or payload digest
- THEN the check MUST deny
- AND the denial MUST occur before any protected effect

### Requirement: Exact canonical request binding

r[basalt.kiln_authority.binding] The adapter MUST frame the holder, deployment audience,
project, work, work version, attempt, fence, authority generation, payload digest, target
and applicable candidate identity into one canonical resource that both the UCAN grant and
the holder signature cover. Every string field MUST be bounded, ASCII and free of control
bytes, and delimiter or Unicode normalization MUST NOT alias two different requests.

#### Scenario: Substituted binding invalidates the invocation

r[basalt.kiln_authority.binding.scenario.substitution]
- GIVEN a signed invocation for one exact Kiln request
- WHEN the holder, audience, project, work, work version, attempt, fence, generation, digest, target or candidate changes
- THEN verification MUST deny
- AND no replay boundary MUST be consumed

#### Scenario: Candidate identity is required exactly where a mutation binds one

r[basalt.kiln_authority.binding.scenario.candidate]
- GIVEN submission, verification, publication, integration and lifecycle operations bind a candidate identity
- WHEN a candidate identity is zero for one of them, or non-zero for claim, dispatch or cancellation
- THEN admission MUST deny before replay

### Requirement: Live authority only after verified ingress

r[basalt.kiln_authority.ingress] The adapter MUST mint live authority only after signature,
proof attenuation, holder, trusted-root, policy, time, revocation and final replay checks
succeed. The resulting session MUST be opaque, MUST NOT be cloneable or serializable, and
MUST have no construction path outside the adapter. Policy denial MUST complete before any
consuming replay call.

#### Scenario: Unverified input mints no authority

r[basalt.kiln_authority.ingress.scenario.denied]
- GIVEN malformed, unverifiable, expired, revoked, wrong-holder or untrusted-root input
- WHEN verification runs
- THEN it MUST return denial
- AND it MUST NOT return a session or commit replay for a request that failed earlier checks

#### Scenario: Historical values cannot recreate authority

r[basalt.kiln_authority.ingress.scenario.opaque]
- GIVEN a session, a redacted receipt and a serialized request
- WHEN they are retained, logged or copied
- THEN none of them MUST construct or restore live authority

### Requirement: Current revalidation without replay consumption

r[basalt.kiln_authority.revalidation] Before each protected dispatch the adapter MUST
revalidate signatures, the full proof chain, time, current revocations, the loaded policy
and the deployment generation. Revalidation MUST NOT consume holder-invocation replay a
second time and MUST NOT refresh, extend or reissue authority.

#### Scenario: Stale or revoked authority denies later dispatch

r[basalt.kiln_authority.revalidation.scenario.stale]
- GIVEN a live session
- WHEN the token has expired, a proof is revoked, the deployment generation moved or a trusted root is removed
- THEN revalidation MUST deny
- AND the original replay admission MUST remain the only one recorded

#### Scenario: Revalidation does not replace final admission

r[basalt.kiln_authority.revalidation.scenario.nonconsuming]
- GIVEN a session whose invocation already passed final replay admission
- WHEN revalidation succeeds for a later effect
- THEN it MUST NOT record another replay admission

### Requirement: Reviewed policy contracts gate every operation

r[basalt.kiln_authority.policy] Every Kiln operation MUST require its own reviewed contract
with the Kiln resource prefix and its distinct ability. A policy that lacks the selected
contract, or supplies a different prefix or ability, MUST deny; an unadmitted policy MUST
NOT create live authority.

#### Scenario: Missing contract denies

r[basalt.kiln_authority.policy.scenario.missing]
- GIVEN a policy without the selected Kiln contract
- WHEN a request for that operation is verified
- THEN the adapter MUST deny before replay

#### Scenario: Packaged policy carries every reviewed contract

r[basalt.kiln_authority.policy.scenario.packaged]
- GIVEN the packaged policy artifact
- WHEN it is parsed
- THEN it MUST contain each Kiln contract with the Kiln prefix and its ability

### Requirement: Delegation cannot widen Kiln authority

r[basalt.kiln_authority.delegation] A verified parent MUST be able to delegate a narrower
Kiln resource and ability to an admitted recipient. A child MUST NOT widen the resource,
substitute another operation or obtain publication, integration, verification or lifecycle
authority from claim, dispatch or submission authority.

#### Scenario: Exact delegated operation is admitted

r[basalt.kiln_authority.delegation.scenario.exact]
- GIVEN a trusted root granting a Kiln prefix to a delegate
- WHEN the delegate issues an exact operation resource to the holder
- THEN the holder's invocation MUST be admitted and revalidated

#### Scenario: Widening denies

r[basalt.kiln_authority.delegation.scenario.widen]
- GIVEN a parent grant for dispatch
- WHEN a child requests lifecycle completion or publication
- THEN verification MUST deny before replay
