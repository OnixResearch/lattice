# Crate Consumability Specification Delta

## ADDED Requirements

### Requirement: License consumption contract
r[crate_consumability.license_consumption.contract] Basalt MUST support a receipt contract that records downstream consumer, Basalt artifact identity, license expression, review action, and non-claim boundary for license-sensitive adoption.

#### Scenario: Acknowledged license consumption passes
r[crate_consumability.license_consumption.fixtures.positive]
- GIVEN a consumer receipt names Basalt artifact identity, AGPL license expression, reviewer action, and required non-claims
- WHEN license-consumption validation runs
- THEN validation MUST pass and preserve the consumer identity.

#### Scenario: Missing license review fails
r[crate_consumability.license_consumption.fixtures.negative]
- GIVEN a consumer receipt omits review action, uses wrong artifact identity, has stale license metadata, or weakens non-claims
- WHEN validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: License validation boundary
r[crate_consumability.license_consumption.validation] Basalt MUST validate license-consumption receipt shape without granting legal advice or alternate license terms.

#### Scenario: Non-claim is visible
r[crate_consumability.license_consumption.docs]
- GIVEN a license-consumption receipt validates
- WHEN the supported claim is rendered
- THEN it MUST state that the receipt records review metadata only and is not legal advice or a license exception.

### Requirement: Final validation
r[crate_consumability.license_consumption.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers license receipt health
r[crate_consumability.license_consumption.final_validation.fixtures]
- GIVEN valid and invalid license-consumption fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
