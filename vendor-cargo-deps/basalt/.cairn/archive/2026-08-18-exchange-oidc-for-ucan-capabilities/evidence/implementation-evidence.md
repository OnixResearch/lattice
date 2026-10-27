# Implementation evidence

## Result

Implementation commit `bf4342d0c6dcb4825ea4796525c9b9ceafce2238` completes the OIDC-to-UCAN exchange.

The pure core verifies one GitHub Actions OIDC token with explicit time and public key inputs. It returns one exact Mantle UCAN issuance plan or a stable rejection.

The imperative shell owns bounded HTTPS, JWKS cache state, one replay mutation, and UCAN issuance through a caller-owned signer.

## Baseline

Before core changes, these commands passed:

- `cargo test --locked --no-default-features`
- `cargo test --locked --manifest-path vendor/ucan/Cargo.toml`

No baseline failure was present.

## Reviewed sources

Bounded HTTP is pinned to Radicle RID `rad:z4ARxEt93T6gJTrTZpUyKHMAemPiv` at object `5abc2b93e0f288aba251a0750ce2d716eb9757fc`.

`jsonwebtoken` is pinned to release `11.0.0`, upstream object `900010fb1be550ec2cfa8c32038eb154f9ef56d7`, and crates.io checksum `881733cbc631fc9e472e24447ce32a64bedf2da498d6d8570b08edc87de71f65`.

Cargo disables default features and selects the RustCrypto backend.

The review considered three correlated mechanism families:

- `jsonwebtoken` with RustCrypto was selected.
- `jwt-compact` was rejected because the reviewed release is beta.
- `josekit` was rejected because OpenSSL and unused JOSE functions expand the boundary.

No subagent performed an independent review.

## Policy and core

The typed Nickel policy names the exact issuer, discovery URL, JWKS URL, audience, RS256 algorithm, GitHub claims, time limits, transport limits, cache limits, replay rules, and Mantle capability vocabulary.

The core rejects malformed tokens, algorithm switching, unsafe headers, wrong key use, invalid signatures, wrong issuer or audience, invalid time claims, wrong claim shapes, policy mismatches, and excessive capabilities.

The core attenuates audience, project, operation, resource, and lifetime. The issued lifetime cannot exceed OIDC validity or the policy maximum.

## Shell and authority boundaries

Bounded HTTP supplies DNS, TCP, TLS, deadline, redirect, body, and response mechanics. The profile allows configured public HTTPS origins only and denies redirects.

An unknown `kid` can cause one refresh. Duplicate, malformed, excessive, unknown, or stale key sets fail closed. Missing cache metadata gives zero freshness.

Replay storage is a caller-owned port. One atomic replay commit occurs after verification and before issuance.

Issuance requires an admitted SecretSpec result, the reviewed declaration, a valid rotation window, and a public signer digest match. A mismatch fails before network or replay effects.

## Evidence and privacy

Receipts contain metadata identities only. They omit raw OIDC tokens, UCAN bearer tokens, private keys, provider credentials, full JWK sets, and token-derived correlation digests.

A positive scan covers durable OIDC output paths. A negative scan proves every prohibited marker is rejected.

## Validation

The following checks passed:

- 11 focused OIDC exchange tests.
- 2 immutable dependency tests.
- 2 positive and negative evidence-scan tests.
- 8 invalid Nickel policy fixtures.
- 7 invalid immutable-source fixtures.
- `cargo fmt --check`.
- Clippy with `-D warnings` for all targets and all features.
- The direct release verifier.
- 222 no-default-feature release tests.
- 235 CLI-feature release tests.
- 2 doctests.
- Bounded OIDC policy and source Nix checks.
- Full `nix flake check -L` with local builders.
- Cairn validation, design gate, and tasks gate.
- Focused traceability for all 6 new OIDC requirements.

The repository-wide default traceability profile reports 48 of 256 requirements.
It reported 42 of 250 before sync. None of the 6 new OIDC identifiers is missing.
The remaining 208 entries are inherited coverage debt outside this change.

## Non-claims

This evidence does not prove provider honesty, network truth, cryptographic implementation correctness, remote availability, replay-store durability, signing-key custody, deployment trust, Mantle execution success, or release eligibility outside the checked gates.
