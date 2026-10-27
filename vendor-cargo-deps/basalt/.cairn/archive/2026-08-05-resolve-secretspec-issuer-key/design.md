## Design

Split the SecretSpec issuer-key resolution out of the OIDC exchange so the two
independent surfaces can be pinned, reviewed, and landed separately.

## Dependency pin

Pin SecretSpec `v0.17.0` through Cargo (`secretspec = "0.17.0"`) and Nix. Record
the reviewed source identity from upstream tag commit
`a8794e46ec9664a0e1a3869cc3105d0853937e48` so builds resolve a durable,
reviewed boundary rather than a moving head.

SecretSpec is Apache-2.0. The pin does not host secrets in the repository.

## Metadata declaration

A metadata-only declaration names the UCAN issuer signing key under the
`basalt-oidc-exchange` scope and an explicit profile. The declaration carries no
key value.

## Imperative shell boundary

Key resolution runs only in the repository/service shell. It reads declared
metadata and asks the pinned SecretSpec SDK to load the value from the selected
provider.

If the selected provider can launch a child process, resolution runs inside an
owned bounded worker with an explicit timeout and output-size bound. The pure
verification core never reads clocks, environment, files, network state, or
secret keys.

## Disable writes and plaintext caches

Write operations are disabled. Plaintext value caching is disabled. The loader
requests only the value bound to the declared scope and key identifier.

## Rotation

The shell supports an explicit key identifier. Rotation selects a new identifier
and overlaps the public verification key window without exposing private
material. A missing, wrong-scope, timeout, oversized, or provider-failure
resolution disables issuance and never falls back to a static key.

## Claim boundary

This change proves only that a reviewed SecretSpec boundary can resolve a
declared issuer key. It does not verify OIDC tokens, issue UCAN capabilities, or
prove that any provider is a correct secret authority.
