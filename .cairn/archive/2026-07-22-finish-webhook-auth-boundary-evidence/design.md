# Design: finish webhook authentication boundary evidence

## Goal

Complete exactly the two reviewed webhook authentication rows when direct tests BLAKE3-bind valid admission plus raw persistence omission and failed-auth response/history/raw-store omission with safe trigger and credential-reference context, while selected markers remain absent and the 329-item missing set is unchanged.

## Approach registry

### Existing response/history assertions

Rejected because they omit raw persistence checks and do not provide credential-reference context.

### Raw reference identity

Rejected because provider/key strings can disclose operator-chosen naming information on durable and public surfaces.

### Domain-separated reference fingerprint

Selected. Hash the opaque reference identity with a fixed BLAKE3 domain and append only the fingerprint to authentication diagnostics. Unrelated diagnostic and record shapes remain unchanged.

## Functional core and shell

A pure fingerprint function maps one `SecretRef` to deterministic context. Diagnostic construction and record projection remain deterministic cores. Integration tests own temporary stores and workflow execution.

## Adversarial audit

Tests inspect response bytes, loaded records, serialized JSON, and raw redb bytes for the resolved secret, invalid token, authorization header, payload secret, provider, and key. They require trigger identity plus the exact fingerprint and reject classification, evidence, marker, missing-set, dangling, or successful-coverage drift.

## Claim boundaries

A local fingerprint is context, not confidentiality for guessable references or proof of external authentication truth. This does not prove complete history, remote authenticity, arbitrary secret confidentiality, release readiness, or production readiness.
