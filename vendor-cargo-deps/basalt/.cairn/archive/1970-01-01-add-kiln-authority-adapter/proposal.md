## Why

Kiln's implementation workflow needs scoped authority for exact work, attempt and
publication operations, but Basalt's reviewed action vocabulary has no Kiln mapping.
[Kiln #69](https://github.com/OnixResearch/kiln/issues/69) consumes this adapter through
Kiln's core/shell and provider boundaries. Only a reviewed, typed mapping may represent
a Kiln operation; a generic document or process capability cannot stand in for one.

Kiln already owns work admission, current attempts and fences, independent verification,
budget accounting and external effects. Basalt owns the operation mapping and the verified
adapter. This change adds only that mapping and adapter; it does not add another policy
engine, token format, work owner or effect owner.

## What Changes

- Add a closed `kiln` operation vocabulary with one reviewed Basalt contract, UCAN ability
  and resource prefix per operation: claim acquire/renew, dispatch, candidate submission,
  verification/evidence admission, attempt cancellation, publication, integration and
  native lifecycle completion.
- Add a holder-invocation adapter that frames every request field into a canonical
  `kiln://implementation/v1/` resource: holder, service audience, project, work and
  version, attempt and fence, authority generation, request digest, target and the
  applicable candidate identity.
- Require a non-zero candidate identity for mutation operations and forbid one for
  non-mutation operations, so a submission binding cannot stand in for dispatch or
  cancellation.
- Mint live authority only after signature, proof, holder, trusted-root, policy, time,
  revocation and final replay admission. Return an opaque, non-`Clone`, non-serializable
  session; a serialized or historical value cannot reconstruct it.
- Revalidate signatures, full proof attenuation, time, current revocations, loaded policy
  and deployment generation before each dispatch without spending replay a second time.
- Add reviewed Kiln policy contracts and regenerate the packaged policy artifact.
- Add positive and negative integration fixtures for exact binding, delegation,
  widening, replay, expiry, revocation, stale generation, untrusted roots, candidate
  substitution and operation substitution.

## Impact

- **Library**: New `kiln` module, new `EnforcementAction` variants and public exports.
- **Policy**: Nine Kiln contracts in `contracts/policy.ncl` and the generated artifact.
- **Tests**: New `tests/ucan_enforcement/kiln.rs` rail over the existing signed fixtures.
- **Documentation**: Adapter contract, ownership boundaries and non-claims.
- **Consumers**: Kiln #69 binds its protected paths to these operations; Kiln #70 owns
  durable grant lifecycle, attenuation and currentness through the same operations.
