# OIDC Dependency Review

Basalt uses two immutable dependencies for the OIDC exchange.

## Selected sources

Bounded HTTP supplies HTTPS discovery and JWKS retrieval.

- Radicle RID: `rad:z4ARxEt93T6gJTrTZpUyKHMAemPiv`
- adapter: `https://seed.radicle.garden/z4ARxEt93T6gJTrTZpUyKHMAemPiv.git`
- reviewed object: `5abc2b93e0f288aba251a0750ce2d716eb9757fc`
- license: `MIT OR Apache-2.0`

`jsonwebtoken` supplies compact JWT parsing, JWK RSA components, and RS256
signature verification through RustCrypto.

- version: `11.0.0`
- upstream tag object: `900010fb1be550ec2cfa8c32038eb154f9ef56d7`
- crates.io checksum: `881733cbc631fc9e472e24447ce32a64bedf2da498d6d8570b08edc87de71f65`
- license: `MIT`
- selected backend: `rust_crypto`

Cargo disables the default PEM feature. Basalt uses public RSA modulus and
exponent values from an admitted JWK. Basalt fixes the accepted algorithm to
RS256 before signature validation.

## Candidate review

The review used three correlated, serial passes. No subagent performed an
independent review.

| Family | Mechanism | Result |
|---|---|---|
| `jsonwebtoken` | Stable JWT, JWS, and JWK types with a RustCrypto provider | Selected |
| `jwt-compact` | Typed algorithm API and pure-Rust RSA | Rejected because the reviewed release is beta |
| `josekit` | Broad JOSE implementation through OpenSSL | Rejected because it expands the native and algorithm boundary |

The selected crate does not own OIDC policy. Basalt disables its ambient time
validation and applies issuer, audience, `exp`, `nbf`, `iat`, claim, and
capability policy with explicit time in the pure core.

## Adversarial audit

The tests reject these cases:

- algorithm switching and unsupported token types;
- embedded JWK, JWK URL, X.509, critical, and unknown header fields;
- bad signatures and wrong public keys;
- encryption-use, wrong-algorithm, malformed, duplicate, excessive, and unknown JWKs;
- wrong issuer, audience, time, claim type, repository, workflow, commit, and reference;
- broad audience, operation, project, resource, and lifetime requests;
- DNS, TLS, redirect, timeout, body-limit, parse, cache, rotation, and outage failures;
- replay duplicates and unavailable replay storage;
- raw bearer, private-key, provider-credential, and token-digest leakage.

## Claim boundary

Source identity and test success do not prove cryptographic correctness,
provider honesty, network truth, remote availability, deployment trust,
replay-store durability, or release eligibility.
