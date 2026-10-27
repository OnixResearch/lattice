# Tasks

## 1. Baseline and policy contract

- [x] [serial] 1.1 Run existing Basalt and UCAN token-flow tests before core changes. r[ucan_token_flow.oidc.pure_verification]
- [x] [serial] 1.2 Add a typed Nickel OIDC exchange contract with named issuer, audience, algorithm, claim, time, network, cache, capability, and replay limits. r[ucan_token_flow.oidc.versioned_policy]
- [x] [serial] 1.3 Export and validate the canonical policy with no secret values. r[ucan_token_flow.oidc.versioned_policy] r[ucan_token_flow.oidc.redacted_evidence]
- [x] [serial] 1.4 Define the versioned Mantle capability vocabulary and subject mapping. r[ucan_token_flow.oidc.attenuated_ucan]

## 2. Build the verification core

- [x] [serial] 2.1 Add bounded OIDC header, claim, JWK, identity, request, issuance-plan, and rejection types. r[ucan_token_flow.oidc.pure_verification]
- [x] [serial] 2.2 Verify algorithm, key use, signature, issuer, audience, `exp`, `nbf`, `iat`, and policy-selected claims with explicit time input. r[ucan_token_flow.oidc.pure_verification]
- [x] [serial] 2.3 Attenuate audience, operation, project, resource, and lifetime to policy. r[ucan_token_flow.oidc.attenuated_ucan]
- [x] [serial] 2.4 Add positive GitHub fixtures and negative malformed, algorithm-confusion, key, signature, issuer, audience, time, type, claim, and excessive-capability fixtures. r[ucan_token_flow.oidc.pure_verification]
- [x] [serial] 2.5 Add property tests for deterministic decisions and monotonic attenuation. r[ucan_token_flow.oidc.pure_verification] r[ucan_token_flow.oidc.attenuated_ucan]

## 3. Build bounded discovery and JWKS shell

- [x] [serial] 3.1 Add configured HTTPS discovery and JWKS retrieval with named transport and document bounds. r[ucan_token_flow.oidc.bounded_jwks]
- [x] [serial] 3.2 Constrain redirects and validate discovery issuer identity. r[ucan_token_flow.oidc.bounded_jwks]
- [x] [serial] 3.3 Add bounded cache freshness, key rotation, and one-refresh behavior. r[ucan_token_flow.oidc.bounded_jwks]
- [x] [serial] 3.4 Add negative fixtures for DNS, TLS, redirect, timeout, oversized body, malformed JSON, excessive keys, unknown key, stale cache, and unavailable issuer. r[ucan_token_flow.oidc.bounded_jwks]

## 4. Integrate UCAN and replay policy

- [x] [serial] 4.1 Issue UCANs through the existing library from accepted issuance plans. r[ucan_token_flow.oidc.attenuated_ucan]
- [x] [serial] 4.2 Ensure issued lifetime cannot exceed OIDC validity or policy maximum. r[ucan_token_flow.oidc.attenuated_ucan]
- [x] [serial] 4.3 Add replay-key planning in the core and replay-store mutation in the shell. r[ucan_token_flow.oidc.replay_boundary]
- [x] [serial] 4.4 Add positive exchange and verification tests plus negative replay, wrong audience, wrong resource, and expired-capability tests. r[ucan_token_flow.oidc.attenuated_ucan] r[ucan_token_flow.oidc.replay_boundary]

## 5. Evidence, rollout, and validation

- [x] [serial] 5.1 Emit redacted exchange evidence with policy, public subject, capability, key, and outcome identities. r[ucan_token_flow.oidc.redacted_evidence]
- [x] [serial] 5.2 Add scans that reject raw OIDC tokens, UCAN bearer tokens, private keys, provider credentials, and sensitive token digests in durable output. r[ucan_token_flow.oidc.redacted_evidence]
- [x] [serial] 5.3 Document GitHub claim policy, rotation, revocation, replay, incident response, and rollback. r[ucan_token_flow.oidc.replay_boundary]
- [x] [serial] 5.4 Run Rust format, focused tests, workspace tests, clippy, Nickel policy validation, release verification, and relevant Nix checks. r[ucan_token_flow.oidc.pure_verification]
- [x] [serial] 5.5 Run Cairn validation, requirement coverage, design gate, and tasks gate. r[ucan_token_flow.oidc.versioned_policy]

## Split

The SecretSpec issuer-key shell was split into the child change
`resolve-secretspec-issuer-key` and its `secretspec_issuer_key` requirement.
This change owns only the OIDC core, JWKS retrieval and shell, UCAN attenuation,
replay, and redacted-evidence work.

## Resolved dependencies

Bounded HTTP is pinned to Radicle object
`5abc2b93e0f288aba251a0750ce2d716eb9757fc`.

The JWT, JWS, and JWK boundary uses `jsonwebtoken` `11.0.0` with the RustCrypto
backend. Its reviewed upstream object is
`900010fb1be550ec2cfa8c32038eb154f9ef56d7`.
