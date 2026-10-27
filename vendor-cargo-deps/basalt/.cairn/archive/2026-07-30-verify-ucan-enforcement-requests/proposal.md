## Why

Basalt accepts verified UCAN fact references through `authority_core`, but it does not verify raw compact invocations. Consumers can assemble preverified DTOs without one reusable shell that proves how those facts were produced.

Basalt needs a fail-closed adapter over the pinned UCAN shell API. The adapter must admit only live in-process authority after cryptographic, proof, policy, replay, and exact-binding checks.

## What Changes

- Add a reusable adapter for raw compact UCAN invocation bytes supplied through the library API.
- Inject the key resolver, proof store, revocation checker, replay admission, caveat policies, verification time, and explicit limits.
- Convert only successful UCAN checks into finite facts for `adapt_enforcement_request` and `evaluate_authority`.
- Return a live borrowed admission capability or a typed denial.
- Return a deterministic redacted BLAKE3 receipt that cannot recreate authority.
- Reject malformed, oversized, unverifiable, revoked, replayed, mismatched, or policy-denied requests without ambient effects.
- Document bounded claims and explicit non-claims.

## Impact

- **Library**: Add a UCAN enforcement adapter module and public exports.
- **Tests**: Add positive and negative integration tests for every required failure class.
- **Documentation**: Add the adapter contract, lifecycle evidence, and release-rail coverage.
- **Source**: Build against the reviewed vendored UCAN snapshot from commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf`.
- **Lifecycle**: Sync and archive only after the vendored source check, adapter tests, release rail, and Cairn gates pass.
