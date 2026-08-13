# Change: Exchange verified OIDC identity for UCAN capabilities

## Why

GitHub Actions and other CI systems can issue short-lived OIDC identity tokens. Mantle needs narrow build and store authority, not a broad static bearer token. Basalt is the correct policy boundary for verifying OIDC claims and mapping accepted identity facts to explicit UCAN capabilities.

This avoids adding Biscuit or another authorization system. It also keeps issuer trust, claim policy, capability attenuation, and evidence separate from Lattice transport orchestration and Mantle execution.

## What Changes

- Add a versioned Nickel policy for trusted OIDC issuers, audiences, algorithms, claim predicates, freshness, and repository or workflow identity.
- Add a bounded shell for HTTPS discovery and JWKS retrieval. Keep claim and policy evaluation in a pure core with explicit time input.
- Map verified claims to a bounded Basalt subject and requested Mantle capability set.
- Issue short-lived UCAN capabilities whose audience, lifetime, operations, project scope, and resource scope cannot exceed policy or the verified OIDC token.
- Depend on the child `resolve-secretspec-issuer-key` change for the bounded UCAN issuer-key loader through the pinned SecretSpec boundary.
- Emit redacted decision evidence without raw OIDC tokens, key material, provider credentials, or secret-derived hashes.
- Add positive GitHub OIDC fixtures and negative issuer, audience, algorithm, signature, freshness, claim, replay, and capability fixtures.

## Non-Goals

- OIDC token acquisition, which belongs to Lattice or another client shell.
- Mantle execution, scheduling, store, or cache policy.
- GitHub Check Run management.
- Ambient trust in any token from a configured issuer.
- A general identity provider or account database.
- Biscuit token support.

## Dependencies

- The existing UCAN issuance and verification interfaces at an immutable validated revision.
- The child `resolve-secretspec-issuer-key` change, which pins SecretSpec `v0.17.0` and provides the bounded issuer-key loader.
- Deployment trust roots and service credentials declared by OnixOS.

## Split

The SecretSpec issuer-key shell lives in the child change
`resolve-secretspec-issuer-key`. This change owns only OIDC verification, JWKS
retrieval, UCAN attenuation, replay, and redacted evidence.

## Impact

- **Affected specs:** `ucan-token-flow`
- **Affected code:** Nickel policy, OIDC/JWK types, verification core, bounded fetch shell, UCAN exchange service, receipts, and fixtures
