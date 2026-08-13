# UCAN Token Flow Specification

## Purpose

Defines the `ucan-token-flow` capability for production-shaped UCAN token fixtures feeding Basalt enforcement.

## Requirements

### Requirement: Basalt MUST have a deterministic production-shaped UCAN token fixture for grant translation.

r[ucan_token_flow.production_ucan_token_flow.deterministic_fixture] Basalt MUST have a deterministic production-shaped UCAN token fixture that verifies or models issued UCAN tokens and translates authorized capabilities into Basalt grants.

#### Scenario: Verified token grants

r[ucan_token_flow.production_ucan_token_flow.deterministic_fixture.verified_grants]

- GIVEN deterministic test identities and UCAN token fixtures
- WHEN the fixture verifies issuance, delegation, and attenuation data
- THEN only verified resource/ability grants are translated into Basalt enforcement inputs

### Requirement: UCAN token flow tests MUST cover allow and fail-closed production cases.

r[ucan_token_flow.production_ucan_token_flow.fail_closed_cases] UCAN token flow tests MUST cover allow and fail-closed production cases including invalid tokens, wrong audience or issuer, attenuation mismatch, and temporal/caveat constraints where supported by the UCAN dependency.

#### Scenario: Invalid token denied

r[ucan_token_flow.production_ucan_token_flow.fail_closed_cases.invalid_or_mismatched]

- GIVEN a token is malformed, invalid, wrong-audience, wrong-issuer, expired, not-yet-valid, over-attenuated, or mismatched to the requested resource/ability
- WHEN the fixture attempts to translate it into Basalt grants
- THEN Basalt receives no unauthorized grant and enforcement cannot produce an allow receipt from that token

### Requirement: The UCAN token fixture MUST be local, deterministic, and secret-free.

r[ucan_token_flow.production_ucan_token_flow.secret_free] The UCAN token fixture MUST be local, deterministic, and secret-free with no committed production credentials, network dependencies, wallet access, or KMS dependency.

#### Scenario: Safe fixture material

r[ucan_token_flow.production_ucan_token_flow.secret_free.safe_material]

- GIVEN the repository is reviewed for secrets and reproducibility
- WHEN UCAN token fixtures and test identities are inspected
- THEN all key material is deterministic test-only material and no production secret or network lookup is required

### Requirement: Release verification MUST expose UCAN token-flow coverage or an explicit sandbox deferral.

r[ucan_token_flow.production_ucan_token_flow.release_coverage] Release verification MUST expose UCAN token-flow coverage or an explicit sandbox deferral when private dependency access prevents the fixture from running in sandboxed Nix.

#### Scenario: Token flow receipt

r[ucan_token_flow.production_ucan_token_flow.release_coverage.receipt]

- GIVEN the release verifier runs
- WHEN the UCAN token-flow fixture executes or is deferred by private-dependency sandbox constraints
- THEN the verifier receipt records the status, command, bounded output, and reason without overstating coverage
