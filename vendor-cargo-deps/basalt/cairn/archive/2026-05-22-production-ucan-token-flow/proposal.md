# Proposal: production UCAN token flow fixture

## Summary

Add a production-shaped UCAN token flow fixture that exercises Basalt against minted/delegated UCAN tokens or verified UCAN library equivalents, rather than only hand-built capability DTOs.

## Problem

Current tests cover Basalt's capability DTO and enforcement behavior. They do not prove that realistic UCAN token creation/delegation/verification data maps into the same allow/deny receipts a production consumer would use.

## Scope

- Add deterministic fixtures for UCAN token issuance/delegation/attenuation/verification using the pinned UCAN dependency.
- Translate verified token capabilities into Basalt `CapabilityGrant` inputs through a narrow adapter or fixture helper.
- Cover allow, attenuation-deny, malformed/invalid token, wrong audience/issuer, expiry/not-before where supported, and caveat/resource mismatch behavior.

## Non-goals

- No live KMS, wallet, DID resolver, or network service dependency.
- No secret keys committed to the repository; use deterministic test keys only.
- No expansion of Basalt into a UCAN runtime/orchestrator.
