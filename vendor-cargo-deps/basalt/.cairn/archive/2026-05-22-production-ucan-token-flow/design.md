# Design: production UCAN token flow fixture

## Approach

Basalt will add a test-only production-shaped UCAN flow that uses deterministic test identities and the pinned UCAN crate to build or verify representative tokens. The fixture should remain local and secret-free while matching the data shape a production consumer would pass after UCAN verification.

## Boundaries

- UCAN token verification belongs to the UCAN dependency or a thin adapter fixture, not Basalt's policy core.
- Basalt receives validated resource/ability grants and enforces policy intersections.
- Test identities and tokens must be deterministic, documented as fixtures, and safe to commit.
- Invalid/expired/wrong-audience tokens must fail before or at the grant translation boundary, never by creating an allow receipt.

## Verification

The release rail should include the fixture on the full direct verifier path. If sandboxed Nix cannot fetch private UCAN dependencies, the receipt must explicitly defer the Cargo-dependent step rather than claim coverage.
