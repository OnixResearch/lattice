# Crate Consumability Delta

## ADDED Requirements

### Requirement: The crate MUST include integration tests that use only the public consumer API.

r[crate_consumability.consumer_integration_tests.public_surface_tests] The crate MUST include integration tests that use only the public consumer API.

#### Scenario: External style test

r[crate_consumability.consumer_integration_tests.public_surface_tests.external_style_test]

- GIVEN integration tests run

- WHEN they import the crate like a downstream app

- THEN policy parsing, request construction, enforcement, and receipt inspection work through public APIs

### Requirement: The integration suite MUST prove fail-closed behavior for common consumer mistakes.

r[crate_consumability.consumer_integration_tests.negative_coverage] The integration suite MUST prove fail-closed behavior for common consumer mistakes.

#### Scenario: Consumer misconfigurations

r[crate_consumability.consumer_integration_tests.negative_coverage.consumer_misconfigurations]

- GIVEN tests exercise malformed policy, unsupported schema, wrong resource, wrong ability, and missing capability

- WHEN enforcement runs

- THEN the crate returns deterministic errors or denied receipts without panics
