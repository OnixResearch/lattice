# Crate Consumability Delta

## ADDED Requirements

### Requirement: The crate MUST define how applications obtain the policy JSON consumed by the Rust API.

r[crate_consumability.policy_artifact_consumption.artifact_contract] The crate MUST define how applications obtain the policy JSON consumed by the Rust API.

#### Scenario: Consumer policy setup

r[crate_consumability.policy_artifact_consumption.artifact_contract.consumer_policy_setup]

- GIVEN a downstream app integrates the crate

- WHEN it needs a policy input

- THEN the docs state whether to include, generate, or supply policy JSON and how to verify freshness

### Requirement: The release rail MUST catch drift between Nickel policy source and generated policy artifacts when generated artifacts are used.

r[crate_consumability.policy_artifact_consumption.freshness_gate] The release rail MUST catch drift between Nickel policy source and generated policy artifacts when generated artifacts are used.

#### Scenario: Policy export check

r[crate_consumability.policy_artifact_consumption.freshness_gate.policy_export_check]

- GIVEN policy source changes

- WHEN the freshness check runs

- THEN stale generated JSON or package artifacts cause verification failure
