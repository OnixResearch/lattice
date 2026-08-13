# Change: Resolve the OIDC issuer key with SecretSpec

## Why

The OIDC-to-UCAN exchange needs a UCAN issuer signing key. Basalt must resolve
that key from a secure secret provider instead of a committed `.env`, a static
token, or an ambient default. SecretSpec (`cachix/secretspec`, crate
`secretspec`, Apache-2.0) separates secret declaration from secret storage and
ships a typed Rust SDK.

This change is split out of the OIDC exchange so the SecretSpec issuer-key shell
can be pinned, reviewed, and land independently of the OIDC/JWKS verification
core. The exchange change keeps the OIDC core, JWKS retrieval, attenuation,
replay, and evidence work.

## What Changes

- Pin SecretSpec `v0.17.0` and its reviewed source identity through Cargo and Nix.
- Declare the UCAN issuer signing key as metadata under an explicit
  `basalt-oidc-exchange` scope and profile.
- Resolve the key only in the imperative service shell, in an owned bounded
  worker when the provider can launch a process.
- Disable SecretSpec write operations and plaintext value caches.
- Support explicit key identifiers and an overlap window for public
  verification keys without exposing private material.
- Add positive key-load and rotation fixtures and negative missing, wrong-scope,
  timeout, oversized-output, and provider-failure fixtures.

## Non-Goals

- Verify OIDC tokens, signatures, or claims. The OIDC exchange change owns that.
- Issue or attenuate UCAN capabilities. The OIDC exchange change owns that.
- Store secrets in plaintext or in repository-controlled files.
- Fall back to a static or ambient signing key.

## Dependencies

- SecretSpec `v0.17.0`, upstream tag commit
  `a8794e46ec9664a0e1a3869cc3105d0853937e48`.
- The UCAN issuance interfaces at the immutable validated revision already in
  the vendored source boundary.

## Impact

- **Affected specs:** `ucan-token-flow`
- **Affected code:** SecretSpec dependency pin, a metadata declaration, a
  bounded issuer-key loader in the imperative shell, and fixtures.
