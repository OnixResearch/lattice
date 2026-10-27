# Crate Consumability Delta

## ADDED Requirements

### Requirement: The crate MUST document the supported public API that downstream applications should call.

r[crate_consumability.stabilize_public_api.documented_api] The crate MUST document the supported public API that downstream applications should call.

#### Scenario: Docs example

r[crate_consumability.stabilize_public_api.documented_api.docs_example]

- GIVEN a downstream developer opens the crate docs or README

- WHEN they follow the library example

- THEN they can parse policy JSON, construct a request, enforce it, and inspect the receipt without using private knowledge

### Requirement: The crate MUST have tests that lock intended public allow/deny/error behavior.

r[crate_consumability.stabilize_public_api.api_regression_tests] The crate MUST have tests that lock intended public allow/deny/error behavior.

#### Scenario: Public behavior tests

r[crate_consumability.stabilize_public_api.api_regression_tests.public_behavior_tests]

- GIVEN public API tests run

- WHEN positive and negative enforcement scenarios execute

- THEN expected receipts and errors remain stable
