## Design

### Module and surface

A new `basalt::kiln` module mirrors the reviewed Mantle adapter. It exposes:

- `Operation`, a closed enum of nine Kiln operations.
- `Request`, the untrusted request. It binds operation, holder `subject`, deployment
  `audience`, `project`, `work`, `work_version`, `attempt`, `fence`, `generation`,
  `request_digest`, `target` and `candidate`.
- `Invocation`, the credential-bearing input with no `Debug` or serde surface.
- `Verifier`, operator-loaded policy, artifact reference, generation, audience and
  trusted roots.
- `Session`, live authority minted only after final replay admission. It is not
  `Clone`, not serializable and has no constructor outside the module.
- `Denied`, the single stable denial.

`EnforcementAction` gains `Kiln*` variants whose `contract_id`, `ability` and
`resource_prefix` are reviewed constants. `as_str` is the contract id for these
variants, as it already is for Mantle.

### Operations and abilities

| Operation | Contract | Ability |
| --- | --- | --- |
| `ClaimAcquire` | `kiln-claim-acquire` | `kiln/claim/acquire` |
| `ClaimRenew` | `kiln-claim-renew` | `kiln/claim/renew` |
| `Dispatch` | `kiln-dispatch` | `kiln/dispatch` |
| `CandidateSubmit` | `kiln-candidate-submit` | `kiln/candidate/submit` |
| `VerificationAdmit` | `kiln-verification-admit` | `kiln/verification/admit` |
| `AttemptCancel` | `kiln-attempt-cancel` | `kiln/attempt/cancel` |
| `PublicationPublish` | `kiln-publication-publish` | `kiln/publication/publish` |
| `IntegrationLand` | `kiln-integration-land` | `kiln/integration/land` |
| `LifecycleComplete` | `kiln-lifecycle-complete` | `kiln/lifecycle/complete` |

No operation implies another. A dispatch grant cannot admit candidate submission,
verification, publication, integration or lifecycle completion.

### Canonical binding

`Verifier::enforcement` builds one resource:

```text
kiln://implementation/v1/<len>/<hex>/.../*
```

Every string field is length-framed and hex-encoded before it reaches the resource, so
delimiter and Unicode-normalization ambiguity cannot alias one request to another. The
numeric `work_version`, `fence` and `generation` follow the framed strings in order,
then the request digest and candidate digest as fixed-width hex.

Admission rules:

- `generation` MUST equal the verifier's non-zero generation.
- `audience` MUST equal the configured deployment audience.
- Each string field MUST be non-empty, at most 512 bytes, ASCII, untrimmed-equal and free
  of control bytes.
- The canonical resource MUST stay at or below 4096 bytes.
- `candidate` MUST be non-zero exactly when the operation requires a candidate.
  `CandidateSubmit`, `VerificationAdmit`, `PublicationPublish`, `IntegrationLand` and
  `LifecycleComplete` require one; claim, dispatch and cancellation forbid one.
- A malformed or refused binding returns `Denied` before replay.

The caller's `request_digest` is the digest of the canonical payload it intends to
execute. Basalt binds it; the receiving owner recomputes it.

### Ingress and revalidation

`Verifier::verify` follows the Mantle sequence: build the enforcement request, parse the
compact token, verify signature/proofs/time/revocations, check holder identity before
replay, walk roots to a configured trusted root, verify the exact UCAN enforcement
request, and mint `Session` only after final replay admission.

`Verifier::revalidate` re-verifies the token and every proof, checks root trust, audience,
caveat evaluation against the exact resource and ability, and reloads the reviewed policy.
It performs no second replay commit. Possession of a `Session` proves its invocation
already passed final admission; revalidation proves current authority before a new effect.

### Policy

Nine new contracts join `contracts/policy.ncl`, each with the
`kiln://implementation/v1/` prefix and its distinct ability. `contracts/generated/policy.json`
is regenerated; the packaged artifact is asserted equal to the tracked file. A policy
without the selected contract denies because `verify` requires the contract to exist.

### Ownership and non-claims

Basalt verifies authority only. Kiln retains current work/fence/policy facts, claims,
budgets, verification acceptance, publication/integration and lifecycle effects.
Lattice retains execution admission and confinement. UCAN retains proof, holder and
attenuation semantics. Callers retain keys, proof loading, revocation currentness,
monotonic time and durable replay.

A `Session`, a redacted receipt, a policy artifact or a serialized request is not a grant
and cannot construct live authority. Revalidation does not refresh, extend or reissue
authority, and no decision here proves effect completion, independent acceptance or
release eligibility.

### Verification

Positive and negative fixtures use the existing signed UCAN test harness and the packaged
policy. Coverage: every operation distinct; substitution of each bound field denied before
replay; expired, revoked, stale-generation and untrusted-root sessions denied; delegation
to an exact operation; widening to publication/lifecycle denied; candidate presence rule;
revalidation not consuming replay; root revocation denying revalidation.
