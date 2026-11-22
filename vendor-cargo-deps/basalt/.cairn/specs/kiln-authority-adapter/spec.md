# Kiln Authority Adapter Specification

## Purpose

Defines the `kiln-authority-adapter` capability.

## Requirements

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

r[basalt.kiln_authority.binding] The adapter MUST frame the deployment audience,
project, work, work version, attempt, fence, authority generation, payload digest, target
and applicable candidate identity into one canonical resource that both the UCAN grant and
the holder signature cover. The frozen `kiln://implementation/v1/` framing MUST also frame
the holder first and MUST keep its bytes unchanged. Attenuable framings MUST bind the
holder through UCAN audience equality and the holder signature instead of a resource
segment. Every string field MUST be bounded, ASCII and free of control bytes, and
delimiter or Unicode normalization MUST NOT alias two different requests.

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

r[basalt.kiln_authority.policy] Every Kiln implementation and source operation MUST
require its own reviewed contract with its distinct ability. Each implementation contract
MUST admit the `kiln://implementation/v1/` and `kiln://implementation/v2/` prefixes, and
each source contract MUST admit `kiln://source/v1/`. If a policy lacks the selected
contract, or the contract does not admit the selected framing prefix and ability,
admission MUST deny. An unadmitted policy MUST NOT create live authority.

#### Scenario: Missing contract denies

r[basalt.kiln_authority.policy.scenario.missing]
- GIVEN a policy without the selected Kiln contract
- WHEN a request for that operation is verified
- THEN the adapter MUST deny before replay

#### Scenario: Packaged policy carries every reviewed contract

r[basalt.kiln_authority.policy.scenario.packaged]
- GIVEN the packaged policy artifact
- WHEN it is parsed
- THEN it MUST contain each implementation contract with both implementation prefixes and its ability
- AND it MUST contain each source contract with the source prefix and its ability

### Requirement: Delegation cannot widen Kiln authority

r[basalt.kiln_authority.delegation] A verified parent MUST be able to delegate a narrower
Kiln resource and ability to an admitted recipient. A child MUST NOT widen the resource,
substitute another operation or obtain publication, integration, verification or lifecycle
authority from claim, dispatch or submission authority. Attenuable families MUST also
apply the explicit delegation chain rule, and a v1 chain that carries a covering grant
wider than the v1 family MUST require explicit `kiln/delegate` from every delegator.

#### Scenario: Exact delegated operation is admitted

r[basalt.kiln_authority.delegation.scenario.exact]
- GIVEN a trusted root granting a Kiln prefix to a delegate
- WHEN the delegate issues an exact operation resource to the holder
- THEN the holder's invocation MUST be admitted and revalidated

#### Scenario: Widening denies

r[basalt.kiln_authority.delegation.scenario.widen]
- GIVEN a parent grant for dispatch or for one scope
- WHEN a child requests lifecycle completion, publication, or a wider scope
- THEN verification MUST deny before replay

### Requirement: Attenuable implementation framing

r[basalt.kiln_authority.attenuable] Basalt MUST offer a `kiln://implementation/v2/`
framing without a holder segment, ordered audience, generation, project, work, work
version, attempt, fence, target, payload digest and candidate. Every segment MUST end on
`/`, so that a byte-prefix grant stopping after a segment expresses a deployment, project,
work or attempt ceiling for any holder. Basalt MUST expose a typed prefix builder that
denies a later segment without every earlier one. The frozen v1 framing MUST remain
verifiable until the operator sets `LegacyFraming::Refuse`, and a v1 grant MUST NOT cover
a v2 request.

#### Scenario: One ceiling serves different holders

r[basalt.kiln_authority.attenuable.scenario.cross_holder]
- GIVEN a delegator holding a project ceiling and the explicit delegate ability
- WHEN it delegates an exact request to one holder and an attempt ceiling to another
- THEN both holders' invocations MUST be admitted and revalidated
- AND the attempt ceiling MUST NOT admit a sibling attempt

#### Scenario: Frozen v1 bytes and chains remain verifiable

r[basalt.kiln_authority.attenuable.scenario.v1_stable]
- GIVEN a v1 request and a v1 chain without any delegate capability
- WHEN it is framed and verified with a delegation depth of zero
- THEN its resource MUST equal the recorded pre-v2 bytes
- AND the chain MUST be admitted while the same request framed as v2 denies

#### Scenario: Wide grants cannot escape through v1

r[basalt.kiln_authority.attenuable.scenario.v1_wide]
- GIVEN a delegator holding dispatch over `kiln://` without `kiln/delegate`
- WHEN it issues a `kiln://implementation/v1/` grant used with v1 framing
- THEN verification MUST deny before replay
- AND the same chain MUST be admitted when the delegator also holds `kiln/delegate` over `kiln://`

#### Scenario: Operators can close the v1 window

r[basalt.kiln_authority.attenuable.scenario.v1_refused]
- GIVEN a verifier configured with `LegacyFraming::Refuse`
- WHEN a v1 request is framed, a v1 session minted under `Accept` is revalidated, or a v2 session's leaf also carries a v1 grant
- THEN the v1 request and the revalidation MUST deny
- AND the v1 grant MUST NOT appear in the session's effective capabilities

### Requirement: Explicit delegation chain rule

r[basalt.kiln_authority.delegation.chain] For attenuable Kiln families the adapter MUST
admit a delegated chain only when three rules hold. First, every intermediate delegator
holds, from a parent proof, an uncaveated capability whose ability is exactly the family's
delegate ability (`kiln/delegate` or `kiln/source/delegate`). That capability's resource
must be a byte prefix of each grant the delegator issued that covers the request. Second,
every grant in the chain that covers the request, including root-issued grants, lies
inside the family, pins the audience and generation, and stops on a segment boundary.
Third, no proof path contains more delegation hops than the operator-configured maximum
depth. A token with several proofs passes the delegate rule when any of its parents holds
the delegate, and every one of its proof paths is checked for depth. Wildcard abilities,
caveated delegate capabilities and the other family's delegate ability MUST NOT satisfy
the rule. Only an operator-trusted root MUST issue a token without proofs. This is the
compact-profile counterpart of UCAN's rule that the root Delegation issuer is the chain
subject: a delegator that holds the delegate ability never becomes a root. Revalidation
MUST apply the same rule with the verifier's current settings.

#### Scenario: Scoped delegation is admitted

r[basalt.kiln_authority.delegation.chain.scenario.scoped]
- GIVEN a trusted root granting a scope plus the family delegate ability to a delegator
- WHEN the delegator issues a narrower scope to a holder
- THEN the holder's invocation MUST be admitted and revalidated

#### Scenario: Missing delegate ability denies

r[basalt.kiln_authority.delegation.chain.scenario.missing_delegate]
- GIVEN a delegator without the explicit family delegate ability over the delegated prefix
- WHEN it issues a grant that UCAN attenuation alone would accept
- THEN verification MUST deny before replay

#### Scenario: Excess depth denies

r[basalt.kiln_authority.delegation.chain.scenario.depth]
- GIVEN a chain with more delegation hops than the configured maximum
- WHEN verification runs
- THEN it MUST deny before replay
- AND the same chain MUST be admitted when the maximum allows its depth
- AND a session admitted at a larger maximum MUST fail revalidation at a smaller one

#### Scenario: Grants cut inside a segment deny

r[basalt.kiln_authority.delegation.chain.scenario.aligned]
- GIVEN a covering grant whose resource ends inside a framed segment
- WHEN it appears anywhere in the chain
- THEN verification MUST deny before replay

#### Scenario: Root substitution denies

r[basalt.kiln_authority.delegation.chain.scenario.root]
- GIVEN an attenuable chain whose proof-free root token is issued by an untrusted issuer, or by a delegator that holds the delegate ability
- WHEN verification runs
- THEN it MUST deny before replay

#### Scenario: Grants that do not pin the generation deny

r[basalt.kiln_authority.delegation.chain.scenario.generation]
- GIVEN an audience-wide grant, a family-wide grant or a `kiln://` grant
- WHEN it covers a request for a later generation
- THEN verification MUST deny before replay
- AND a grant pinned at that generation MUST be admitted

#### Scenario: Caveated delegate capability denies

r[basalt.kiln_authority.delegation.chain.scenario.caveated]
- GIVEN a delegator whose only delegate capability carries a caveat
- WHEN it issues a covering grant
- THEN verification MUST deny before replay

#### Scenario: Multi-proof tokens check every path

r[basalt.kiln_authority.delegation.chain.scenario.multi_proof]
- GIVEN a delegator whose operation grant and delegate capability come from different parents, one of them a hop longer
- WHEN the configured maximum covers the longer path
- THEN the chain MUST be admitted
- AND it MUST deny when the maximum covers only the shorter path or when the delegate-carrying parent is absent

### Requirement: Checked effective capabilities

r[basalt.kiln_authority.effective_scope] A session's effective capabilities MUST contain
every leaf grant that covers the admitted request. They MAY contain any other leaf grant
only when that grant starts with the session's scope prefix and passes the same chain rule
with its own resource. The scope prefix is the v2 deployment prefix (family, verifier
audience, verifier generation) for implementation sessions, and the source repository
and context prefix for source sessions. v1 sessions MUST expose covering grants only. An
admitted request MUST NOT be denied because no other grant qualified. The set MUST be
computed before replay is consumed. Per-grant chain checks MUST share one verified proof
graph, so proof-verification work does not grow with the number of leaf grants.

#### Scenario: Unchecked leaf grants are omitted

r[basalt.kiln_authority.effective_scope.scenario.unchecked]
- GIVEN a root granting a delegator repository-wide discover plus the delegate ability over `a` only
- WHEN the delegator grants a holder discover over `a` and over `secret`, and the holder is admitted on `a`
- THEN the session's effective capabilities MUST NOT include the `secret` grant
- AND a request on `secret` MUST deny

#### Scenario: Grants outside the current deployment and scope are omitted

r[basalt.kiln_authority.effective_scope.scenario.deployment]
- GIVEN a leaf that, beside the admitted grant, carries chain-valid grants pinned to an earlier generation, a foreign audience, another repository or another source context
- WHEN the session is admitted after the generation bump
- THEN none of those grants MUST appear in its effective capabilities
- AND a sibling grant under the same scope prefix MUST appear

#### Scenario: Proof verification does not grow with grant count

r[basalt.kiln_authority.effective_scope.scenario.memoized]
- GIVEN leaves with 1 and with 64 grants over the same delegated chain
- WHEN each is admitted
- THEN both admissions MUST perform the same number of key resolutions
- AND a forged holder signature MUST deny before any chain walk without consuming replay

### Requirement: Closed Kiln source operations

r[basalt.kiln_authority.source_actions] Basalt MUST expose the closed `kiln://source/v1/`
operations discover, read, history, prepare, modify, create, delegate and result. Each
MUST map to exactly one reviewed contract and one `kiln/source/*` ability. Read, modify
and create MUST name exactly one entry. Unknown operations MUST NOT deserialize and MUST
NOT authorize anything.

#### Scenario: Source operations are distinct and exact

r[basalt.kiln_authority.source_actions.scenario.distinct]
- GIVEN a signed source session for one operation, path, scope and digest
- WHEN another operation, path, scope or digest is checked or substituted
- THEN the check MUST deny
- AND no two source operations MUST share a contract or ability

#### Scenario: Unknown source operations deny

r[basalt.kiln_authority.source_actions.scenario.unknown]
- GIVEN a source request naming an operation outside the closed vocabulary
- WHEN it is deserialized or a policy lacks the selected source contract
- THEN admission MUST deny

### Requirement: Component-framed source path scope

r[basalt.kiln_authority.source_path] Source path scope MUST be framed per normalized
component, each length-prefixed and hex-encoded. It MUST be followed by a terminal marker
that distinguishes an exact entry (`e/`) from a subtree (`s/`). A component MUST be
non-empty, bounded, free of separators and control characters, and never `.` or `..`. A
grant for one path MUST NOT cover a sibling whose name shares a byte prefix, and an entry
grant MUST NOT cover descendants or the subtree.

#### Scenario: Paths never alias

r[basalt.kiln_authority.source_path.scenario.alias]
- GIVEN subtree, entry and subtree-node grants for `a/b`
- WHEN requests for `a/bc`, `ab`, `a/b/c` and `a/b` are framed
- THEN the subtree grant MUST cover only `a/b` and its descendants
- AND the entry grant MUST cover only the `a/b` entry
- AND an operator-context grant MUST NOT cover an attempt-context request

#### Scenario: Attempt-context grants bind attempt and fence

r[basalt.kiln_authority.source_path.scenario.attempt_context]
- GIVEN an attempt-context source grant for one attempt and fence
- WHEN requests with that attempt, with no attempt, and with another fence are verified
- THEN only the first MUST be admitted
