## Context

Mantle already selects closed operation profiles but needs a public owner API for
holder-authenticated gateway requests and current revalidation. The API must not
turn a serialized request, a receipt, a registered holder key or a previous clock
observation into reusable authority. Existing UCAN signature/proof/caveat,
revocation and replay ports remain the authority mechanisms.

## Decisions

### Decision: Exact operation and request binding

Use distinct build-submit, input-upload, store-read, project-declare, status,
log, event, result, usage, owned-attempt-cancel, publish and admin profiles.
Length-frame and hex-encode every bounded resource field, including subject,
account, project, endpoint, deployment audience, policy artifact, target,
generation and normalized request digest. The issuer and holder use the same
canonicalizer. No operation implies another grant.

### Decision: Opaque live session and one consuming replay boundary

Verify trusted root issuers, actual token/proof signatures, holder identity,
current time/revocation/caveat policy and exact action before final replay.
Return a non-cloneable, non-deserializable session retaining UCAN's zeroizing
token owner. Require the exact operation/target/digest at use and independently
revalidate live holder authorization and loaded policy without spending replay
again. Reject time before original admission; consumers retain monotonic time
across later observations.

### Decision: Preserve feature and privacy boundaries

Keep the existing minimal/default feature contracts. Durable adapters are exported
only with their feature, while signature/revocation/replay checks remain required.
Public errors expose only `gateway-authority-denied`; Debug does not expose tokens
or request payloads. Persisted projections remain observations, never sessions.

### Decision: Explicit native lifecycle policy cohort

Adopt Cairn's unmodified `cairn-policy/generated/cairn-policy.json` from
`OnixResearch/cairn` commit `d2c1dd7867350b787cfffe0e311fd20115238339`, Git blob
`b8bd7044eb1dd2a871ea0233c7903ce2b01dd47f`. The prior copied projection omits
the `outcome-machine` workflow profile required by the current owner tool.
This is an exact owner projection import, not a hand-edited generated policy or
a validation bypass. Refresh from that repository's Nickel source and export
workflow; do not use a sibling checkout as a durable runtime dependency.
The pinned policy supplies lifecycle decisions only, not SDK authority or release
acceptance. Preserve the existing AGPL distribution boundary and the explicit
`spec-driven` change metadata.

## Risks and verification

Consumers still own resolver correctness, revocation freshness, durable replay,
normalization and every effect. Exercise all distinct signed operations, context
substitution, untrusted roots, widening, expiry/revocation/generation changes,
rollback of time and repeated admission, including unchanged replay counts on
revalidation. Run the repository release verifier and native lifecycle checks;
neither test passage nor lifecycle validation grants release authority.
