# Design: OIDC-to-UCAN capability exchange

## Context

The exchange has two distinct inputs: untrusted identity evidence from an OIDC issuer and a requested UCAN capability. Basalt must verify the first and attenuate the second. A valid OIDC signature alone does not authorize a Mantle operation.

GitHub-specific acquisition remains in Lattice. Basalt supports issuer profiles through policy and starts with a GitHub Actions profile.

## Policy source

A typed Nickel contract defines:

- exact issuer URLs and allowed discovery origins;
- accepted audiences and JOSE algorithms;
- required standard claims and maximum freshness policy;
- repository owner, repository identifier, workflow reference, commit, ref, environment, and event predicates when selected;
- allowed Mantle audience, project mapping, operations, and resource patterns;
- UCAN lifetime and attenuation limits;
- JWKS cache and network bounds by named fields.

Policy export is canonical and identified with BLAKE3. No signing key, provider credential, OIDC token, or secret value is present in Nickel.

## Functional core

The pure verification core accepts the compact token bytes, parsed protected header, candidate public JWK, explicit verification time, issuer policy, and requested capability. It returns either verified identity facts plus an attenuated issuance plan or a stable rejection.

The core checks:

- exact issuer and audience;
- an allowed algorithm and key use;
- signature validity;
- `exp`, `nbf`, and `iat` with named skew and maximum-age policy;
- required string and collection shapes;
- policy-selected repository and workflow claims;
- requested audience, operation, project, and resource scope;
- requested lifetime against both OIDC validity and policy maximum.

Time and fetched-key state are explicit inputs. The core does not read clocks, environment, files, network state, or secret keys.

## Network shell

The shell accepts only configured HTTPS issuers. It fetches discovery metadata and JWKS with named DNS, connect, request, redirect, body, key-count, and cache limits. Redirects cannot leave the configured origin unless policy names the target.

The shell validates media type, parses bounded documents, and passes candidate public keys to the core. Cache freshness follows response metadata within policy caps. Unknown `kid` can trigger one bounded refresh. Stale or unavailable keys fail closed unless policy explicitly allows a still-fresh cached set.

## UCAN issuance

After verification, the shell asks the existing UCAN library to issue an attenuated token. The audience is the selected Mantle service. Capabilities use a versioned Mantle vocabulary for build submission, admitted input upload, store read, status and log read, and owned-attempt cancellation.

The issued token expires no later than the verified OIDC token and the configured UCAN maximum. It carries only the mapped subject and public provenance references needed by enforcement. It does not embed the raw OIDC token.

## SecretSpec issuer key

Key resolution is split into the child change `resolve-secretspec-issuer-key`,
which pins SecretSpec `v0.17.0`, declares the issuer key under an explicit
`basalt-oidc-exchange` scope, and loads it in a bounded imperative shell. This
change depends on that boundary for the UCAN issuer signing key; it does not
reimplement SecretSpec provider access here.

## Replay and request binding

OIDC tokens are short-lived bearer evidence. Basalt binds each exchange to the requested Mantle audience and capability. Policy can require a one-time exchange identifier backed by an external replay store. The pure core creates the replay key from public issuer, subject, audience, and token identifier facts. The shell owns replay-store mutation.

If the issuer supplies no usable token identifier, policy can deny exchange or use a bounded token digest as transient replay input. That digest is sensitive correlation data and is never written to lifecycle evidence.

## Evidence and privacy

Decision evidence contains policy identity, issuer profile, selected public subject fields, requested and granted capability identities, key identifier, outcome, and stable reason codes. It omits compact tokens, bearer capabilities, private keys, provider credentials, full JWK documents, and secret-derived hashes.

## Failure behavior

Malformed token structure, disallowed algorithm, missing or ambiguous key, signature failure, wrong issuer or audience, stale time claims, policy claim mismatch, excessive capability, replay, network bound, provider failure, and issuance failure are separate typed outcomes. None can fall back to a static Mantle token.

## Rollout

The exchange first runs against offline fixtures, then GitHub test repositories, then selected Lattice workflows. Static CI tickets remain a migration fallback only after Mantle credential hardening. Rollback disables the exchange endpoint and does not broaden existing ticket authority.
